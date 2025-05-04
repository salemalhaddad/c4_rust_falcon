use std::time::{Instant};
use std::process::Command;
use std::fs::File;
use std::io::{self, Write};

const TEST_PROGRAMS: [(&str, &str); 3] = [
    ("fib", "int fib(int n) { return n < 2 ? 1 : fib(n-1) + fib(n-2); }\nint main() { return fib(20); }"),
    ("matrix_mult", "int mat[2][2] = {{1,2},{3,4}}; int main() { int sum = 0; for(int i=0;i<2;i++) for(int j=0;j<2;j++) sum += mat[i][j]; return sum; }"),
    ("string_sort", "char str[10] = \"hello\"; int main() { int i = 0; while(str[i]) i++; return i; }"),
];

pub fn benchmark_c4() -> Result<(), io::Error> {
    println!("Starting C4 benchmarks...");
    println!("----------------------------------");

    for (name, program) in TEST_PROGRAMS {
        println!("\nBenchmarking {}...", name);

        // Write program to file
        let mut file = File::create(format!("{}.c", name))?;
        file.write_all(program.as_bytes())?;

        // Compile with C4
        let start = Instant::now();
        Command::new("../c4")
            .arg(format!("{}.c", name))
            .output()?;
        let compile_time = start.elapsed();

        // Run the compiled program
        let start = Instant::now();
        Command::new(format!("{}.out", name))
            .output()?;
        let run_time = start.elapsed();

        println!("Compile time: {:.3?}", compile_time);
        println!("Run time: {:.3?}", run_time);

        // Clean up
        std::fs::remove_file(format!("{}.c", name))?;
        std::fs::remove_file(format!("{}.out", name))?;
    }

    Ok(())
}

pub fn benchmark_rust_c4() -> Result<(), io::Error> {
    println!("\nStarting Rust C4 benchmarks...");
    println!("----------------------------------");

    for (name, program) in TEST_PROGRAMS {
        println!("\nBenchmarking {}...", name);

        // Write program to file
        let mut file = File::create(format!("{}.c", name))?;
        file.write_all(program.as_bytes())?;

        // Compile with Rust C4
        let start = Instant::now();
        Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("--")
            .arg(format!("{}.c", name))
            .output()?;
        let compile_time = start.elapsed();

        // Run the compiled program
        let start = Instant::now();
        Command::new(format!("{}.out", name))
            .output()?;
        let run_time = start.elapsed();

        println!("Compile time: {:.3?}", compile_time);
        println!("Run time: {:.3?}", run_time);

        // Clean up
        std::fs::remove_file(format!("{}.c", name))?;
        std::fs::remove_file(format!("{}.out", name))?;
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    benchmark_c4()?;
    benchmark_rust_c4()?;
    Ok(())
}
