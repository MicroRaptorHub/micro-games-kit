# List the just recipe list
list:
    just --list

format:
    cargo fmt

build:
    cargo build

run:
    cargo run

clippy:
    cargo clippy

test:
    cargo test

checks:
    just format
    just build
    just clippy
    just test

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

list-outdated:
  cargo outdated -R -w

update:
    cargo update --aggressive

example NAME="hello_world":
    cargo run --all-features --example {{NAME}}

publish:
    cargo publish --no-verify
