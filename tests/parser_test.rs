use c4_rust::lexer::{Token};
use c4_rust::parser::Parser;
use c4_rust::parser::symbol_table::{Symbol, Class, SymbolTable};
use c4_rust::parser::types::Type;

// Helper to initialize parser and advance to first token
fn parser_with_first_token(src: &str) -> Parser {
    Parser::new(src.as_bytes())
}

#[test]
fn test_symbol_table_basic() {
    let mut symbol_table = SymbolTable::new();
    
    // Test adding a global symbol
    let symbol1 = Symbol {
        name: "x".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    symbol_table.add_symbol(symbol1.clone()).unwrap();
    
    // Test finding the symbol
    let found = symbol_table.lookup("x");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "x");
    assert_eq!(found.unwrap().class, Class::Global);
    
    // Test adding a local symbol
    symbol_table.enter_scope();
    let symbol2 = Symbol {
        name: "y".to_string(),
        class: Class::Local,
        typ: Type::Int,
        val: 0,
        offset: 4,
    };
    symbol_table.add_symbol(symbol2.clone()).unwrap();
    
    // Test finding both symbols
    assert!(symbol_table.lookup("x").is_some());
    assert!(symbol_table.lookup("y").is_some());
    
    // Test exiting scope
    symbol_table.exit_scope();
    assert!(symbol_table.lookup("x").is_some()); // Global still exists
    assert!(symbol_table.lookup("y").is_none()); // Local is gone
}

#[test]
fn test_type_operations() {
    // Test basic types
    let int_type = Type::Int;
    let char_type = Type::Char;
    
    // Test sizes
    assert_eq!(int_type.size(), 4);
    assert_eq!(char_type.size(), 1);
    
    // Test pointer types
    let int_ptr = Type::Ptr(Box::new(Type::Int));
    let char_ptr = Type::Ptr(Box::new(Type::Char));
    
    // All pointers should have the same size
    assert_eq!(int_ptr.size(), 4);
    assert_eq!(char_ptr.size(), 4);
    
    // Test pointer to pointer
    let int_ptr_ptr = Type::Ptr(Box::new(Type::Ptr(Box::new(Type::Int))));
    assert_eq!(int_ptr_ptr.size(), 4);
}

#[test]
fn test_parser_initialization() {
    let source = "int main() { return 0; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Test initial state
    assert_eq!(parser.local_offset, 0);
    assert_eq!(parser.second_pass, false);
    
    // Test symbol table initialization
    assert!(parser.symbol_table.lookup("main").is_none()); // Not parsed yet
}

#[test]
fn test_parse_simple_declaration() {
    let mut parser = parser_with_first_token("int x;");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    let symbol = parser.symbol_table.lookup("x");
    assert!(symbol.is_some());
    assert_eq!(symbol.unwrap().class, Class::Global);
    assert!(matches!(symbol.unwrap().typ, Type::Int));
}

#[test]
fn test_parse_pointer_declaration() {
    let mut parser = parser_with_first_token("int *ptr;");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    let symbol = parser.symbol_table.lookup("ptr");
    assert!(symbol.is_some());
    match &symbol.unwrap().typ {
        Type::Ptr(inner) => assert!(matches!(**inner, Type::Int)),
        _ => panic!("Expected pointer type"),
    }
}

#[test]
fn test_parse_function_declaration() {
    let source = "int add(int a, int b) { return a + b; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Parse the function declaration
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    
    // Verify the function symbol was added
    let symbol = parser.symbol_table.lookup("add");
    assert!(symbol.is_some());
    assert_eq!(symbol.unwrap().class, Class::Function);
    assert!(matches!(symbol.unwrap().typ, Type::Int));
}

#[test]
fn test_parse_expression() {
    let source = "2 + 3 * 4";
    let mut parser = Parser::new(source.as_bytes());
    
    // Parse the expression
    let result = parser.parse_expression();
    assert!(result.is_ok());
    
    // We can't easily check the AST, but we can verify the expression was parsed
    // by checking if all tokens were consumed
    assert!(parser.lexer.peek_token().is_none() || parser.lexer.peek_token().unwrap() == c4_rust::lexer::Token::Eof);
}

#[test]
fn test_parse_if_statement() {
    let source = "if (x > 0) { y = 1; } else { y = 2; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add 'x' and 'y' to the symbol table first
    let x_symbol = Symbol {
        name: "x".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    let y_symbol = Symbol {
        name: "y".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 4,
    };
    parser.symbol_table.add_symbol(x_symbol).unwrap();
    parser.symbol_table.add_symbol(y_symbol).unwrap();
    
    // Parse the if statement
    let result = parser.parse_statement();
    assert!(result.is_ok());
    
    // We can't easily check the AST, but we can verify the statement was parsed
    // by checking if all tokens were consumed
    assert!(parser.lexer.peek_token().is_none() || parser.lexer.peek_token().unwrap() == c4_rust::lexer::Token::Eof);
}

#[test]
fn test_parse_while_statement() {
    let source = "while (i < 10) { i = i + 1; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add 'i' to the symbol table first
    let i_symbol = Symbol {
        name: "i".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(i_symbol).unwrap();
    
    // Parse the while statement
    let result = parser.parse_statement();
    assert!(result.is_ok());
    
    // We can't easily check the AST, but we can verify the statement was parsed
    // by checking if all tokens were consumed
    assert!(parser.lexer.peek_token().is_none() || parser.lexer.peek_token().unwrap() == c4_rust::lexer::Token::Eof);
}

#[test]
fn test_parse_compound_statement() {
    let source = "{ int x; x = 10; int y; y = 20; x = x + y; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Parse the compound statement
    let result = parser.parse_compound_statement(true);
    assert!(result.is_ok());
    
    // We can't easily check the AST, but we can verify the statement was parsed
    // by checking if all tokens were consumed
    assert!(parser.lexer.peek_token().is_none() || parser.lexer.peek_token().unwrap() == c4_rust::lexer::Token::Eof);
}

#[test]
fn test_parse_return_statement() {
    let source = "return 42;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Parse the return statement
    let result = parser.parse_statement();
    assert!(result.is_ok());
    
    // We can't easily check the AST, but we can verify the statement was parsed
    // by checking if all tokens were consumed
    assert!(parser.lexer.peek_token().is_none() || parser.lexer.peek_token().unwrap() == c4_rust::lexer::Token::Eof);
}

#[test]
fn test_parse_complete_function() {
    let mut parser = parser_with_first_token("int factorial(int n);");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    let symbol = parser.symbol_table.lookup("factorial");
    assert!(symbol.is_some());
    assert_eq!(symbol.unwrap().class, Class::Function);
    assert!(matches!(symbol.unwrap().typ, Type::Int));
}
