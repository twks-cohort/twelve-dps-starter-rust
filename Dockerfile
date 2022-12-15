# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
FROM docker.io/rust:1.58.1-buster AS production-build
WORKDIR /usr/src

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo new basic-bot
WORKDIR /usr/src/basic-bot
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Copy the source and build the application.
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

# Copy the statically-linked binary into a scratch container.
FROM scratch
WORKDIR /
COPY --from=production-build /usr/local/cargo/bin/basic-bot .
USER 1000
CMD ["./basic-bot"]
