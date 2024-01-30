use std::path::Path;

use bytes::{Buf, Bytes};
use futures_util::Future;
use h3::error::ErrorLevel;
use http::Response;
use http_body::{Body, Frame};
use http_body_util::BodyExt as _;
use hyper::service::Service as _;
use perfect_group_allocation_config::Config;
use tracing::{error, info, trace};

use crate::error::AppError;
use crate::{setup_server, Svc, CERT_PATH, KEY_PATH, PORT};

pub struct H3Body<S: h3::quic::RecvStream + 'static>(h3::server::RequestStream<S, Bytes>);

impl<S: h3::quic::RecvStream + 'static> Body for H3Body<S> {
    type Error = h3::Error;

    type Data = impl Buf + Send + 'static;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let recv_data = self.0.recv_data();
        let recv_data = std::pin::pin!(recv_data);
        recv_data
            .poll(cx)
            .map(|v| v.transpose().map(|v| v.map(Frame::data)))
    }
}

async fn handle_connection<C: h3::quic::Connection<Bytes> + Send, MyRecvStream: h3::quic::RecvStream + 'static>(
service: Svc::<<H3Body<MyRecvStream> as Body>::Data>, // the service needs to use the same impl buf that recvstream decided to use
        mut connection: h3::server::Connection<C, Bytes>,
    ) where
// RequestBodyBuf needs to == H3Body::Data

// the RecvStream uses impl Buf
        C::BidiStream: h3::quic::BidiStream<Bytes, RecvStream = MyRecvStream> + Send + 'static,
        <C as h3::quic::Connection<bytes::Bytes>>::SendStream: Send,
        <C as h3::quic::Connection<bytes::Bytes>>::RecvStream: Send,
        <<C as h3::quic::Connection<bytes::Bytes>>::BidiStream as h3::quic::BidiStream<bytes::Bytes>>::RecvStream: std::marker::Send,
        <<C as h3::quic::Connection<bytes::Bytes>>::BidiStream as h3::quic::BidiStream<bytes::Bytes>>::SendStream: std::marker::Send
    {
    loop {
        match connection.accept().await {
            Ok(Some((req, stream))) => {
                trace!("new request: {:#?}", req);

                let service = service.clone();

                tokio::spawn(async move {
                    let (mut response_body, request_body) = stream.split();
                    let request = req.map(|()| H3Body(request_body));
                    let response = service
                        .call(request.map(|body| body.map_err(AppError::from)))
                        .await;
                    if let Ok(response) = response {
                        let response: Response<_> = response;
                        let (parts, body) = response.into_parts();

                        response_body
                            .send_response(Response::from_parts(parts, ()))
                            .await
                            .unwrap();

                        let mut body = std::pin::pin!(body);
                        while let Some(value) = body.frame().await {
                            let value = value.unwrap();
                            if value.is_data() {
                                response_body
                                    .send_data(value.into_data().unwrap())
                                    .await
                                    .unwrap();
                            } else if value.is_trailers() {
                                response_body
                                    .send_trailers(value.into_trailers().unwrap())
                                    .await
                                    .unwrap();
                                return;
                            }
                        }
                        response_body.finish().await.unwrap();
                    }
                });
            }

            // indicating no more streams to be received
            Ok(None) => {
                break;
            }

            Err(err) => {
                error!("error on accept {}", err);
                match err.get_error_level() {
                    ErrorLevel::ConnectionError => break,
                    ErrorLevel::StreamError => continue,
                }
            }
        }
    }
}

type TestS2n = <H3Body<s2n_quic_h3::RecvStream> as Body>::Data;

pub fn run_http3_server_s2n(
    config: Config,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let service = setup_server::<TestS2n>(config)?;

    // if the error says "no error" it failed to load the certificates
    // quic start error: Permission denied (os error 13) means it does not have sufficient permissions to listen on the specified port
    let mut server = s2n_quic::Server::builder()
        .with_tls((Path::new(CERT_PATH), Path::new(KEY_PATH)))?
        .with_io(format!("0.0.0.0:{PORT}").as_str())?
        .start()?;

    info!("listening on localhost:{PORT}");

    // https://github.com/aws/s2n-quic/blob/main/quic/s2n-quic-qns/src/server/h3.rs

    Ok(async move {
        while let Some(connection) = server.accept().await {
            let service = service.clone();

            // spawn a new task for the connection
            tokio::spawn(async move {
                let connection =
                    h3::server::Connection::new(s2n_quic_h3::Connection::new(connection))
                        .await
                        .unwrap();
                handle_connection(service, connection).await;
            });
        }
        Ok(())
    })
}

/*
// depends on old version of tokio-rustls
static ALPN: &[u8] = b"h3";

type TestQuinn = <H3Body<h3_quinn::RecvStream> as Body>::Data;

#[allow(clippy::needless_pass_by_value)]
pub fn run_http3_server_quinn(
    database_url: String,
    certs: Vec<Certificate>,
    key: PrivateKey,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let service = setup_server::<TestQuinn>(&database_url)?;

    let listen = std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), PORT));

    let mut tls_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    tls_config.max_early_data_size = u32::MAX;
    tls_config.alpn_protocols = vec![ALPN.into()];

    let server_config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));
    let endpoint = quinn::Endpoint::server(server_config, listen)?;

    info!("listening on {}", listen);

    Ok(async move {
        // handle incoming connections and requests

        while let Some(new_conn) = endpoint.accept().await {
            trace_span!("New connection being attempted");

            let service = service.clone();

            tokio::spawn(async move {
                match new_conn.await {
                    Ok(conn) => {
                        info!("new connection established");

                        let connection: h3::server::Connection<h3_quinn::Connection, Bytes> =
                            h3::server::Connection::new(h3_quinn::Connection::new(conn))
                                .await
                                .unwrap();
                        handle_connection(service, connection).await;
                    }
                    Err(err) => {
                        error!("accepting connection failed: {:?}", err);
                    }
                }
            });
        }

        // shut down gracefully
        // wait for connections to be closed before exiting
        endpoint.wait_idle().await;

        Ok(())
    })
}
*/
