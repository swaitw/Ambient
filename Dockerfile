# Instructions (replace "/path/to/your/project" with path to your project):
# 1. Build the image: docker build -f Dockerfile -t ambient .
# 2. Run the image: docker run -p 8999:8999/tcp -p 9000:9000/udp -v /path/to/your/project:/app/project ambient
# 3. Run `ambient join` locally to connect to the server

FROM rust:1.73-bullseye AS builder
RUN apt-get update && \
    apt-get install -y \
    zip build-essential cmake pkg-config \
    libfontconfig1-dev clang libasound2-dev ninja-build \
    libxcb-xfixes0-dev mesa-vulkan-drivers
ADD . /build
WORKDIR /build
RUN cargo build --release --no-default-features --features production
RUN strip target/release/ambient

FROM rust:1.73-bullseye
RUN apt-get update && \
    apt-get install -y \
    ca-certificates libasound2
RUN rustup toolchain install stable
RUN rustup target add --toolchain stable wasm32-wasi
WORKDIR /app
COPY --from=builder /build/target/release/ambient ./
CMD [ "./ambient", "serve", "--public-host", "localhost", "--release", "project" ]
