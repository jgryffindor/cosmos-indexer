# Use the official Rust image as the base image
FROM rust:slim-bookworm
 
# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    clang \
    llvm-dev \
    libssl-dev \
    pkg-config \
    gcc \
    libc-dev \
    make \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

EXPOSE 9000

# Copy the Cargo.toml and Cargo.lock files to the container
COPY ./indexer . 

# # Build the Rust application with the specified features
RUN cargo build --release

RUN ln -s /app/target/release/cosmos-indexer /usr/local/bin/cosmos-indexer
