# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

RUN rustup default stable

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .

# RUN apt-get update && apt-get install -y wget unzip
# RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v3.18.0/protoc-3.18.0-linux-x86_64.zip
# RUN unzip protoc-3.18.0-linux-x86_64.zip
# RUN mv bin/protoc /usr/local/bin/
# RUN rm -rf include/ bin/ readme.txt protoc-3.18.0-linux-x86_64.zip
RUN cargo build --release --bin delivery 

# We do not need the Rust toolchain to run the binary!
FROM ubuntu:latest AS runtime

# RUN export OPENSSL_DIR=/usr/lib/ssl
# RUN apt-get update && apt-get install -y wget unzip
# RUN wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.0g-2ubuntu4_amd64.deb
# RUN dpkg -i libssl1.1_1.1.0g-2ubuntu4_amd64.deb

WORKDIR /app

COPY --from=builder /app/target/release/delivery /usr/local/bin
COPY ./resources /app/resources
COPY ./configs /app/configs

EXPOSE 5050 
ENTRYPOINT ["/usr/local/bin/delivery"]
