# perfect-group-allocation

https://docs.rs/hyper/1.0.0-rc.4/hyper/index.html
https://hyper.rs/guides/1/
https://github.com/hyperium/hyper/blob/master/examples/README.md

```bash
openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -outform der -keyform der -keyout example.com.key -out example.com.crt -subj "/CN=example.com" -addext "subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1"

openssl rsa -inform pem -in example.com.key -outform der -out example.com.key.der

cargo run --bin server -- --cert example.com.crt --key example.com.key.der
```