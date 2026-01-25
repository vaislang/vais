# OSS-Fuzz Dockerfile for Vais compiler
# Based on: https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/

FROM gcr.io/oss-fuzz-base/base-builder-rust

# Install additional dependencies for LLVM
RUN apt-get update && apt-get install -y \
    llvm-17 \
    llvm-17-dev \
    clang-17 \
    && rm -rf /var/lib/apt/lists/*

# Clone the repository
RUN git clone --depth 1 https://github.com/sswoo88/vais.git $SRC/vais

# Set working directory
WORKDIR $SRC/vais

# Copy build script
COPY build.sh $SRC/
