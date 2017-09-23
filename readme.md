generally use rust stable, but rustfmt and clippy both requre nightly

#run with clippy
rustup run nightly cargo run --features clippy

#run rustfmt
rustup run nightly cargo install rustfmt-nightly
rustup run nightly cargo fmt
