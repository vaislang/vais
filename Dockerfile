# Multi-stage Docker build for Vais compiler
# Stage 1: Builder - compile the Rust project with LLVM 17
FROM rust:1.82-slim AS builder

# Install LLVM 17 development libraries and build dependencies
RUN apt-get update && \
    apt-get install -y wget gnupg lsb-release software-properties-common && \
    wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - && \
    echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-17 main" > /etc/apt/sources.list.d/llvm.list && \
    apt-get update && \
    apt-get install -y \
        llvm-17-dev \
        libpolly-17-dev \
        clang-17 \
        build-essential \
        pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Set LLVM environment variable for inkwell
ENV LLVM_SYS_170_PREFIX=/usr/lib/llvm-17

# Set working directory
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY std ./std
COPY examples ./examples
COPY benches ./benches

# Build the vaisc compiler in release mode
# Exclude Python and Node.js bindings as they require additional toolchains
RUN cargo build --release -p vaisc

# Stage 2: Runtime - minimal image with only runtime dependencies
FROM debian:bookworm-slim

# Install LLVM 17 runtime libraries and clang for compilation
RUN apt-get update && \
    apt-get install -y wget gnupg lsb-release software-properties-common && \
    wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - && \
    echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-17 main" > /etc/apt/sources.list.d/llvm.list && \
    apt-get update && \
    apt-get install -y \
        llvm-17 \
        clang-17 && \
    rm -rf /var/lib/apt/lists/*

# Create symlinks for LLVM tools (optional, for convenience)
RUN ln -s /usr/lib/llvm-17/bin/clang /usr/local/bin/clang && \
    ln -s /usr/lib/llvm-17/bin/llc /usr/local/bin/llc && \
    ln -s /usr/lib/llvm-17/bin/opt /usr/local/bin/opt

# Set LLVM environment variable
ENV LLVM_SYS_170_PREFIX=/usr/lib/llvm-17

# Copy the compiled vaisc binary from builder
COPY --from=builder /build/target/release/vaisc /usr/local/bin/vaisc

# Copy the standard library
COPY --from=builder /build/std /usr/local/lib/vais/std

# Set the standard library path environment variable
ENV VAIS_STD_PATH=/usr/local/lib/vais/std

# Set vaisc as the entrypoint
ENTRYPOINT ["vaisc"]

# Default command: show help
CMD ["--help"]
