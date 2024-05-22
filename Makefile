lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-features --benches --examples --tests -- -D warnings

fmt:
    cargo fmt --all

test:
    cargo nextest run --workspace --all-features
