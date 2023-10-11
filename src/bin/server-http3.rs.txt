use std::{
    io::BufReader,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::{Bytes, BytesMut};
use http::{Request, StatusCode};
use rustls::{version::TLS13, Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, ec_private_keys};
use structopt::StructOpt;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{error, info, trace_span};

use h3::{error::ErrorLevel, quic::BidiStream, server::RequestStream};
use h3_quinn::quinn;

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
struct Opt {
    #[structopt(
        name = "dir",
        short,
        long,
        help = "Root directory of the files to serve. \
                If omitted, server will respond OK."
    )]
    pub root: Option<PathBuf>,

    #[structopt(
        short,
        long,
        default_value = "[::]:443", // needs to be < 1024
        help = "What address:port to listen for new connections"
    )]
    pub listen: SocketAddr,
}

static ALPN: &[u8] = b"h3";

fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> ServerConfig {
    let mut key_reader = BufReader::new(std::fs::File::open(key).unwrap());
    let mut cert_reader = BufReader::new(std::fs::File::open(cert).unwrap());

    let key = PrivateKey(ec_private_keys(&mut key_reader).unwrap().remove(0));
    let certs = certs(&mut cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let config = ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("bad certificate/key");

    config
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    // process cli arguments

    let opt = Opt::from_args();

    let root = if let Some(root) = opt.root {
        if !root.is_dir() {
            return Err(format!("{}: is not a readable directory", root.display()).into());
        } else {
            info!("serving {}", root.display());
            Arc::new(Some(root))
        }
    } else {
        Arc::new(None)
    };

    // create quinn server endpoint and bind UDP socket

    let mut tls_config = rustls_server_config(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".lego/certificates/h3.selfmade4u.de.key"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".lego/certificates/h3.selfmade4u.de.crt"),
    );

    tls_config.max_early_data_size = u32::MAX;
    tls_config.alpn_protocols = vec![ALPN.into()];

    let server_config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));
    let endpoint = quinn::Endpoint::server(server_config, opt.listen)?;

    info!("listening on {}", opt.listen);

    // handle incoming connections and requests

    while let Some(new_conn) = endpoint.accept().await {
        trace_span!("New connection being attempted");

        let root = root.clone();

        tokio::spawn(async move {
            match new_conn.await {
                Ok(conn) => {
                    info!("new connection established");

                    let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                        .await
                        .unwrap();

                    loop {
                        match h3_conn.accept().await {
                            Ok(Some((req, stream))) => {
                                info!("new request: {:#?}", req);

                                let root = root.clone();

                                tokio::spawn(async {
                                    if let Err(e) = handle_request(req, stream, root).await {
                                        error!("handling request failed: {}", e);
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
}

async fn handle_request<T>(
    req: Request<()>,
    mut stream: RequestStream<T, Bytes>,
    serve_root: Arc<Option<PathBuf>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: BidiStream<Bytes>,
{
    let (status, to_serve) = match serve_root.as_deref() {
        None => (StatusCode::OK, None),
        Some(_) if req.uri().path().contains("..") => (StatusCode::NOT_FOUND, None),
        Some(root) => {
            let to_serve = root.join(req.uri().path().strip_prefix('/').unwrap_or(""));
            match File::open(&to_serve).await {
                Ok(file) => (StatusCode::OK, Some(file)),
                Err(e) => {
                    error!("failed to open: \"{}\": {}", to_serve.to_string_lossy(), e);
                    (StatusCode::NOT_FOUND, None)
                }
            }
        }
    };

    let resp = http::Response::builder()
        .status(status)
        .header("alt-svc", r#"h3=":443"; ma=86400"#)
        .body(())
        .unwrap();

    match stream.send_response(resp).await {
        Ok(_) => {
            info!("successfully respond to connection");
        }
        Err(err) => {
            error!("unable to send response to connection peer: {:?}", err);
        }
    }

    if let Some(mut file) = to_serve {
        loop {
            let mut buf = BytesMut::with_capacity(4096 * 10);
            if file.read_buf(&mut buf).await? == 0 {
                break;
            }
            stream.send_data(buf.freeze()).await?;
        }
    }

    Ok(stream.finish().await?)
}
