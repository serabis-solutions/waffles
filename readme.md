#Waffles
Generally you wnat to use rust stable, but rustfmt and clippy both requre nightly.

## Run with clippy
```
rustup run nightly cargo run --features clippy
```

## Run rustfmt
```
rustup run nightly cargo install rustfmt-nightly
rustup run nightly cargo fmt
```
