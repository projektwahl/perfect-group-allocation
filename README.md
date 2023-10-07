# perfect-group-allocation

https://docs.rs/hyper/1.0.0-rc.4/hyper/index.html
https://hyper.rs/guides/1/
https://github.com/hyperium/hyper/blob/master/examples/README.md
https://blog.cloudflare.com/speeding-up-https-and-http-3-negotiation-with-dns/

 gh repo clone quinn-rs/quinn
 cargo run --example client https://localhost:4433/Cargo.toml


```bash

openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -keyout example.com.key.pem -out example.com.cert.pem -subj "/CN=example.com" -addext "subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1"
openssl rsa -inform pem -in example.com.key.pem -outform der -out example.com.key.der
openssl x509 -outform der -in example.com.cert.pem -out example.com.cert.der

cargo run --bin server -- --cert example.com.crt --key example.com.key.der
```