build:
    cargo build

test:
    cargo nextest run

lint:
    cargo check
    cargo clippy

check: test lint
