# Multi-stage build for Vais programming language compiler
# Stage 1: Build the compiler
FROM rust:1.83-bookworm AS builder

# Install LLVM and Clang dependencies for compilation
RUN apt-get update && apt-get install -y \
    clang \
    llvm-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy the entire project
COPY . .

# Build the vaisc compiler in release mode
RUN cargo build --release -p vaisc

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
# clang is needed at runtime to compile LLVM IR to native code
RUN apt-get update && apt-get install -y \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled vaisc binary from builder stage
COPY --from=builder /build/target/release/vaisc /usr/local/bin/vaisc

# Copy the standard library
COPY --from=builder /build/std /usr/local/share/vais/std

# Set environment variable for standard library path
ENV VAIS_STD_PATH=/usr/local/share/vais/std

# Set working directory
WORKDIR /workspace

# Set vaisc as the entrypoint
ENTRYPOINT ["vaisc"]

# Default command (show help)
CMD ["--help"]
