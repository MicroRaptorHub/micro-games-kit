# List the just recipe list
list:
    just --list

format:
    cargo fmt

build:
    cargo build

run NAME="top-down":
    cd ./templates/{{NAME}} && just run

clippy:
    cargo clippy

test:
    cargo test

checks:
    just format
    just build
    just clippy
    just test
    cd ./templates/fresh-start && just checks
    cd ./templates/top-down && just checks

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

package-templates:
    powershell Compress-Archive -Force "./templates/fresh-start/*" ./target/fresh-start-template.zip
    powershell Compress-Archive -Force "./templates/top-down/*" ./target/top-down-template.zip
