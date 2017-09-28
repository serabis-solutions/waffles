# Waffles

Generally you want to use rust stable, but rustfmt and clippy both requre nightly.

## Run with clippy
```
rustup run nightly cargo run --features clippy
```

## Run rustfmt
```
rustup run nightly cargo install rustfmt-nightly
rustup run nightly cargo fmt
```

## Create TLS Cert For Testing
Waffles uses p12 format TLS Certificates. You can create one as follows:

```
mkdir cert
openssl req -newkey rsa:2048 -x509 -keyout cert/test_key.pem -out cert/test.csr -days 3650
openssl pkcs12 -export -in cert/test.csr -inkey cert/test_key.pem -out cert/identity.p12 -name "testkey"
```
Currently passwordless certs aren't supported.