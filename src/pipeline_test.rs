//! SCAFFOLD: Integration test between lexer and VM (before parser is ready)
//! This file demonstrates how the lexer and VM will work together,
//! manually constructing an AST until the real parser is complete.

mod lexar;
mod vm;

use lexar::{Lexer, Token};
use vm::{AstNode, run_vm};

fn main() {
    println!("=== SCAFFOLD TEST ===\n");
    
    // Test input: "x = 2 + 3;"
    println!("Input: x = 2 + 3;");
    
    // 1. Lexer test
    let src = "x = 2 + 3;";
    let mut lexer = Lexer::new(src.as_bytes());
    let mut tokens = Vec::new();
    lexer.next_token(); // Get the first token
    while let Some(token) = lexer.current_token.take() {
        if token == Token::Eof { break; }
        tokens.push(token);
        lexer.next_token();
    }
    println!("\nTokens: {:?}", tokens);

    // 2. Mock AST (temporary until parser is ready)
    println!("\nConstructing mock AST: (x = (2 + 3))");
    let ast = AstNode::Assign {
        left: Box::new(AstNode::Id("x".to_string())),
        right: Box::new(AstNode::Add(
            Box::new(AstNode::Num(2)),
            Box::new(AstNode::Num(3)),
        )),
    };

    // 3. VM execution
    println!("\nExecuting in VM:");
    match run_vm(&ast) {
        Ok(val) => println!("Result: {:?}", val),
        Err(e) => println!("Error: {:?}", e),
    }
    
    println!("\n=== END SCAFFOLD TEST ===");
}
