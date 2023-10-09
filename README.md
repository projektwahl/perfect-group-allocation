# perfect-group-allocation

```
# TODO we should get a valid tls certificate from our domain as otherwise the browsers don't like you
sudo apt install lego
HETZNER_API_KEY=xxx lego --cert.timeout 180 --email Moritz.Hedtke@t-online.de --dns hetzner --domains h3.selfmade4u.de --accept-tos run
```

if using wsl in vscode, add port forward in vscode

https://h3.selfmade4u.de/

https://github.com/tokio-rs/axum/tree/main/examples

https://docs.rs/hyper/1.0.0-rc.4/hyper/index.html
https://hyper.rs/guides/1/
https://github.com/hyperium/hyper/blob/master/examples/README.md
https://blog.cloudflare.com/speeding-up-https-and-http-3-negotiation-with-dns/

https://docs.quic.tech/quiche/h3/index.html


https://interop.seemann.io/
ngtcp2 and picoquic best impls
https://interop.seemann.io/?client=ngtcp2,picoquic

network.http.http3.alt-svc-mapping-for-testing
h3.selfmade4u.de;h3=":443"

// quinn doesnt to http3 but only quic
gh repo clone quinn-rs/quinn
cd quinn
cargo run --release --example client -- --url https://localhost:443/

gh repo clone cloudflare/quiche
cd quiche
cargo run --bin quiche-client -- --dump-json https://h3.selfmade4u.de/
cargo run --bin quiche-client -- --dump-json https://127.0.0.1/
cargo run --bin quiche-client -- --dump-json https://[::1]/

# dns only 127.0.0.1 as chrome won't resolve link local addresses
cargo run --bin quiche-client -- --dump-json https://192.168.2.126/
cargo run --bin quiche-client -- --dump-json https://[fe80::acab:ec2e:86c3:1517]/

https://github.com/aws/s2n-quic/blob/main/quic/s2n-quic-qns/src/server/h3.rs

cargo build --release --bin server && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server && ./target/release/server
cargo build --release --bin server-http3 && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server-http3 && ./target/release/server-http3

cargo build --release --bin server-http3-s2n && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server-http3-s2n && ./target/release/server-http3-s2n


cargo build --bin server && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/debug/server && ./target/debug/server
cargo build --bin server-http3 && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/debug/server-http3 && ./target/debug/server-http3



chromium-browser --enable-quic --origin-to-force-quic-on=localhost:443
```bash

openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -keyout example.com.key.pem -out example.com.cert.pem -subj "/CN=example.com" -addext "subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1"
openssl rsa -inform pem -in example.com.key.pem -outform der -out example.com.key.der
openssl x509 -outform der -in example.com.cert.pem -out example.com.cert.der

cargo run --bin server -- --cert example.com.crt --key example.com.key.der
```