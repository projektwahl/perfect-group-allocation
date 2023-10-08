# perfect-group-allocation

```
# TODO we should get a valid tls certificate from our domain as otherwise the browsers don't like you
lego --cert.timeout 180 --email Moritz.Hedtke@t-online.de --dns hetzner --domains h3.
selfmade4u.de run
```

https://h3.selfmade4u.de/

https://github.com/tokio-rs/axum/tree/main/examples

https://docs.rs/hyper/1.0.0-rc.4/hyper/index.html
https://hyper.rs/guides/1/
https://github.com/hyperium/hyper/blob/master/examples/README.md
https://blog.cloudflare.com/speeding-up-https-and-http-3-negotiation-with-dns/


network.http.http3.alt-svc-mapping-for-testing
localhost;h3=":443"

 gh repo clone quinn-rs/quinn
 cargo run --example client https://localhost:4433/Cargo.toml

cargo build --release --bin server && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server && ./target/release/server
cargo build --release --bin server-http3 && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server-http3 && ./target/release/server-http3

cargo build --bin server && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/debug/server && ./target/debug/server
cargo build --bin server-http3 && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/debug/server-http3 && ./target/debug/server-http3



chromium-browser --enable-quic --origin-to-force-quic-on=localhost:443
```bash

openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -keyout example.com.key.pem -out example.com.cert.pem -subj "/CN=example.com" -addext "subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1"
openssl rsa -inform pem -in example.com.key.pem -outform der -out example.com.key.der
openssl x509 -outform der -in example.com.cert.pem -out example.com.cert.der

cargo run --bin server -- --cert example.com.crt --key example.com.key.der
```