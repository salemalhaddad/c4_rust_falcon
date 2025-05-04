mod lexer;
mod parser;
mod codegen;
mod vm;

use std::env;
use std::fs;
// No need for std::io import
use std::process;

use parser::Parser;
use vm::VM;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file> [options]", args[0]);
        eprintln!("Options:");
        eprintln!("  -d    Debug mode (print VM instructions)");
        process::exit(1);
    }

    let source_file = &args[1];
    let debug_mode = args.iter().any(|arg| arg == "-d");

    // Read source file
    let source = match fs::read(source_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", source_file, err);
            process::exit(1);
        }
    };

    // Debug: Print the source code
    if debug_mode {
        println!("Source code:\n{}", String::from_utf8_lossy(&source));
    }

    // Debug: Tokenize the source code and print tokens
    if debug_mode {
        println!("\nTokens:");
        let mut lexer = crate::lexer::Lexer::new(&source);
        lexer.next_token();
        while let Some(token) = lexer.peek_token() {
            if token == crate::lexer::Token::Eof {
                println!("  Token::Eof");
                break;
            }
            println!("  {:?}", token);
            lexer.next_token();
        }
        println!();
    }

    // Create parser
    let mut parser = Parser::new(&source);

    // Parse source code and get code and data segments
    let (code, data) = match parser.parse() {
        Ok((code, data)) => (code, data),
        Err(err) => {
            eprintln!("Compilation error: {}", err);
            process::exit(1);
        }
    };

    if debug_mode {
        println!("DEBUG: Generated code size: {} instructions", code.len());
        println!("DEBUG: Generated data size: {} bytes", data.len());
        if !data.is_empty() {
            println!("DEBUG: First 10 bytes of data segment: {:?}", &data[0..std::cmp::min(10, data.len())]);
        }
    }

    // Create VM
    let mut vm = VM::new(
        code,
        data,               // Use the data segment from the code generator
        1024 * 1024,        // 1MB stack
        debug_mode,
    );

    // Run VM
    match vm.run() {
        Ok(exit_code) => {
            if debug_mode {
                println!("Program exited with code: {}", exit_code);
            }
            process::exit(exit_code as i32);
        }
        Err(err) => {
            eprintln!("Runtime error: {}", err);
            process::exit(1);
        }
    }
}

// Function to compile and run C code directly
pub fn compile_and_run(source: &[u8], debug_mode: bool) -> Result<i32, String> {
    // Create parser
    let mut parser = Parser::new(source);

    // Parse source code and get code and data segments
    let (code, data) = parser.parse()?;

    if debug_mode {
        println!("DEBUG: Generated code size: {} instructions", code.len());
        println!("DEBUG: Generated data size: {} bytes", data.len());
        if !data.is_empty() {
            println!("DEBUG: First 10 bytes of data segment: {:?}", &data[0..std::cmp::min(10, data.len())]);
        }
    }

    // Create VM
    let mut vm = VM::new(
        code,
        data,               // Use the data segment from the code generator
        1024 * 1024,        // 1MB stack
        debug_mode,
    );

    // Run VM
    vm.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let source = r#"
            #include <stdio.h>

			int main() {
				printf("Hello, World!\n");
				return 0;
			}

        "#;

        let result = compile_and_run(source.as_bytes(), true);
        if let Err(e) = &result {
            eprintln!("compile_and_run error: {}", e);
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_factorial() {
        let source = r#"
            int factorial(int n) {
                if (n <= 1) return 1;
                return n * factorial(n - 1);
            }

            int main() {
                printf("Factorial of 5: %d\n", factorial(5));
                return 0;
            }
        "#;

        let result = compile_and_run(source.as_bytes(), true);
        if let Err(e) = &result {
            eprintln!("compile_and_run error: {}", e);
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
