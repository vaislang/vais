#!/bin/bash -eu
# OSS-Fuzz build script for Vais compiler
# https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/

cd $SRC/vais

# Build fuzz targets
cd fuzz

# List of fuzz targets
FUZZ_TARGETS=(
    fuzz_lexer
    fuzz_parser
    fuzz_type_checker
    fuzz_codegen
    fuzz_full_pipeline
)

# Build each fuzz target
for target in "${FUZZ_TARGETS[@]}"; do
    echo "Building fuzz target: $target"

    cargo +nightly fuzz build $target --release

    # Copy binary to output directory
    cp $SRC/vais/fuzz/target/x86_64-unknown-linux-gnu/release/$target $OUT/

    # Copy seed corpus if exists
    if [ -d "$SRC/vais/fuzz/corpus/$target" ]; then
        zip -j $OUT/${target}_seed_corpus.zip $SRC/vais/fuzz/corpus/$target/*
    fi
done

# Copy dictionary
if [ -f "$SRC/vais/fuzz/dictionaries/vais.dict" ]; then
    for target in "${FUZZ_TARGETS[@]}"; do
        cp $SRC/vais/fuzz/dictionaries/vais.dict $OUT/${target}.dict
    done
fi

echo "OSS-Fuzz build completed successfully!"
