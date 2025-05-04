#!/bin/bash

# Function to run and time a compiler
run_compiler() {
    local compiler=$1
    local input=$2
    local output="${input%.c}.out"
    
    # Compile
    echo "Compiling with $compiler..."
    time $compiler $input
    
    # Run
    echo "Running..."
    time ./$output
    
    # Clean up
    rm -f $output
}

# Run benchmarks for hello.c
echo "\n=== Testing hello.c ==="
echo "\nOriginal C4:"
run_compiler "../c4" "../examples/hello.c"

echo "\nRust C4:"
run_compiler "cargo run --release --" "../examples/hello.c"
