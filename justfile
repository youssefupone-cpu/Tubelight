set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

dev:
    pnpm dev

build:
    pnpm build

test:
    cargo test --workspace
    pnpm test

fmt:
    cargo fmt --all
    pnpm format

lint:
    cargo clippy --workspace --all-targets -- -D warnings
    pnpm lint

update-sidecars:
    ./scripts/update-sidecars.sh
