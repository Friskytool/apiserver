FROM lukemathwalker/cargo-chef:latest-rust-1.57.0 AS chef
WORKDIR /backend

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /backend/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin backend

FROM debian as runner
WORKDIR /backend
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libleptonica-dev libtesseract-dev clang postgresql\
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /backend/target/release/backend /usr/local/bin
COPY static/ static/
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/backend"]