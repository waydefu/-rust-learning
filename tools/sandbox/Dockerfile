# The image version matches rust-toolchain.toml. Dependencies are downloaded
# only during image construction; example execution is network-isolated.
FROM rust:1.85.1-bookworm AS builder
WORKDIR /course
COPY rust-toolchain.toml ./
COPY tools/ tools/
COPY examples/ examples/
RUN cargo build --locked --manifest-path tools/site-render/Cargo.toml --bin site-render \
    && CARGO_HOME=/opt/example-cache cargo fetch --locked --manifest-path examples/Cargo.toml \
    && printf '[net]\noffline = true\n' > /opt/example-cache/config.toml

FROM rust:1.85.1-bookworm
WORKDIR /course
COPY rust-toolchain.toml ./
COPY content/ content/
COPY examples/Cargo.toml examples/Cargo.lock examples/
COPY --from=builder /course/tools/target/debug/site-render /usr/local/bin/site-render
COPY --from=builder /opt/example-cache /opt/example-cache
RUN chmod -R a+rX /opt/example-cache /course
ENV HOME=/tmp/home CARGO_HOME=/tmp/cargo RUSTUP_TOOLCHAIN=1.85.1-x86_64-unknown-linux-gnu
USER 10001:10001
CMD ["sh", "-eu", "-c", "mkdir -p \"$HOME\" \"$CARGO_HOME\"; cp -R /opt/example-cache/. \"$CARGO_HOME/\"; test \"$(id -u)\" != 0; site-render check-rust"]
