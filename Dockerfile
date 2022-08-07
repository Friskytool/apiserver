FROM ekidd/rust-musl-builder:latest as planner

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl

WORKDIR /backend
RUN sudo chown -R rust:rust .
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
RUN pwd && ls

FROM ekidd/rust-musl-builder:latest as cacher
WORKDIR /backend
RUN sudo chown -R rust:rust .
RUN cargo install cargo-chef
COPY --from=planner /backend/recipe.json recipe.json
RUN cargo chef cook --release --target=x86_64-unknown-linux-musl --recipe-path recipe.json

FROM ekidd/rust-musl-builder:latest as builder
WORKDIR /backend
RUN sudo chown -R rust:rust .
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /backend/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --target=x86_64-unknown-linux-musl --bin backend

FROM debian:bullseye as runtime
WORKDIR /backend
RUN apk add --no-cache ca-certificates
COPY --from=builder /backend/target/x86_64-unknown-linux-musl/release/backend /usr/local/bin
COPY --from=builder /backend/static /backend/static
EXPOSE 8000
ENV ROCKET_ADDRESS="0.0.0.0"
ENV ROCKET_PORT="8000"
CMD ["/usr/local/bin/backend"]