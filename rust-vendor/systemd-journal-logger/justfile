default:
    just --list

vet:
    # Only consider Linux dependencies, as that's all I care for.
    # Seems to be unofficial, see https://github.com/mozilla/cargo-vet/issues/579, but works
    env CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu cargo vet --locked

test-all: vet
    cargo +stable deny --all-features --locked check
    cargo +stable fmt -- --check
    cargo +stable clippy --all-targets --locked
    # Stable build, test, and docs
    cargo +stable build --locked
    cargo +stable test --locked
    cargo +stable doc
    # MSRV
    cargo +1.68 build --locked --all-targets
    # semver checks
    cargo semver-checks

release *ARGS: test-all
    cargo release {{ARGS}}
