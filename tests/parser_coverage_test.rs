use c4_rust::parser::Parser;
use c4_rust::parser::symbol_table::{Symbol, Class, SymbolTable};
use c4_rust::parser::types::Type;

// Helper to initialize parser with first token
fn parser_with_first_token(src: &str) -> Parser {
    Parser::new(src.as_bytes())
}

#[test]
fn test_invalid_type_specifier() {
    let mut parser = parser_with_first_token("float x;");
    let result = parser.parse_global_declaration();
    assert!(result.is_err());
}

#[test]
fn test_missing_semicolon() {
    let mut parser = parser_with_first_token("int x");
    let result = parser.parse_global_declaration();
    assert!(result.is_err());
}

#[test]
fn test_missing_identifier_after_type() {
    let mut parser = parser_with_first_token("int ;");
    let result = parser.parse_global_declaration();
    assert!(result.is_err());
}

#[test]
fn test_duplicate_symbol_declaration() {
    let mut parser = parser_with_first_token("int x;");
    let result1 = parser.parse_global_declaration();
    
    // Reset parser to parse another declaration
    let mut parser = parser_with_first_token("int x;");
    let result1 = parser.parse_global_declaration();
    let result2 = parser.parse_global_declaration();
    assert!(result1.is_ok());
    assert!(result2.is_err());
}

#[test]
fn test_variable_initialization() {
    let mut parser = parser_with_first_token("int x = 5;");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    let symbol = parser.symbol_table.lookup("x");
    assert!(symbol.is_some());
    assert_eq!(symbol.unwrap().class, Class::Global);
}

#[test]
fn test_pointer_initialization() {
    let mut parser = parser_with_first_token("int *p = 0;");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    let symbol = parser.symbol_table.lookup("p");
    assert!(symbol.is_some());
    match &symbol.unwrap().typ {
        Type::Ptr(inner) => assert!(matches!(**inner, Type::Int)),
        _ => panic!("Expected pointer type"),
    }
}

#[test]
fn test_function_declaration_no_params() {
    let mut parser = parser_with_first_token("int foo();");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    let symbol = parser.symbol_table.lookup("foo");
    assert!(symbol.is_some());
    assert_eq!(symbol.unwrap().class, Class::Function);
}

#[test]
fn test_function_with_pointer_param() {
    let mut parser = parser_with_first_token("int bar(int *p) { return 1; }");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    let symbol = parser.symbol_table.lookup("bar");
    assert!(symbol.is_some());
    assert_eq!(symbol.unwrap().class, Class::Function);
}

#[test]
fn test_empty_compound_statement() {
    let source = "{}";
    let mut parser = Parser::new(source.as_bytes());
    let result = parser.parse_compound_statement(true);
    assert!(result.is_ok());
}

#[test]
fn test_nested_compound_statements() {
    let source = "{ int x; { int y; } }";
    let mut parser = Parser::new(source.as_bytes());
    let result = parser.parse_compound_statement(true);
    assert!(result.is_ok());
}

#[test]
fn test_if_without_else() {
    let source = "if (1) { int x; }";
    let mut parser = Parser::new(source.as_bytes());
    let result = parser.parse_statement();
    assert!(result.is_ok());
}

#[test]
fn test_while_with_empty_body() {
    let source = "while (1) {}";
    let mut parser = Parser::new(source.as_bytes());
    let result = parser.parse_statement();
    assert!(result.is_ok());
}

#[test]
fn test_complex_expression_parsing() {
    let source = "x = a * (b + c) / d - e & f | g ^ h;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add x to symbol table so expression can reference it
    let x_symbol = Symbol {
        name: "x".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(x_symbol).unwrap();
    
    // Add other variables to symbol table
    for var in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result = parser.parse_expression_statement();
    assert!(result.is_ok());
}

#[test]
fn test_nested_if_else_statements() {
    let source = "if (a) { if (b) { x = 1; } else { x = 2; } } else { x = 3; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["a", "b", "x"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result = parser.parse_statement();
    assert!(result.is_ok());
}

#[test]
fn test_nested_while_statements() {
    let source = "while (a) { while (b) { x = x + 1; } }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["a", "b", "x"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result = parser.parse_statement();
    assert!(result.is_ok());
}

#[test]
fn test_conditional_expression() {
    let source = "x = a ? b : c;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["a", "b", "c", "x"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result = parser.parse_expression_statement();
    assert!(result.is_ok());
}

#[test]
fn test_function_call_expression() {
    let source = "result = add(x, y * 2);";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables and function to symbol table
    for var in ["result", "x", "y"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    // Add function to symbol table
    let add_symbol = Symbol {
        name: "add".to_string(),
        class: Class::Function,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(add_symbol).unwrap();
    
    let result = parser.parse_expression_statement();
    assert!(result.is_ok());
}

#[test]
fn test_unary_operators() {
    let source = "x = -y + !z + ~w + *ptr;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["x", "y", "z", "w"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    // Add pointer to symbol table
    let ptr_symbol = Symbol {
        name: "ptr".to_string(),
        class: Class::Global,
        typ: Type::Ptr(Box::new(Type::Int)),
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(ptr_symbol).unwrap();
    
    let result = parser.parse_expression_statement();
    assert!(result.is_ok());
}

#[test]
fn test_complex_function_definition() {
    let source = "int max(int a, int b) { if (a > b) { return a; } else { return b; } }";
    let mut parser = parser_with_first_token(source);
    
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    
    // Check function was added to symbol table
    let max_symbol = parser.symbol_table.lookup("max");
    assert!(max_symbol.is_some());
    assert_eq!(max_symbol.unwrap().class, Class::Function);
}

#[test]
fn test_function_with_multiple_statements() {
    let source = "int sum(int n) { int result; int i; result = 0; i = 1; while (i <= n) { result = result + i; i = i + 1; } return result; }";
    let mut parser = parser_with_first_token(source);
    
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    
    // Check function was added to symbol table
    let sum_symbol = parser.symbol_table.lookup("sum");
    assert!(sum_symbol.is_some());
    assert_eq!(sum_symbol.unwrap().class, Class::Function);
}

#[test]
fn test_nested_pointer_declaration() {
    let mut parser = parser_with_first_token("int **pp;");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    
    let symbol = parser.symbol_table.lookup("pp");
    assert!(symbol.is_some());
    
    // Check it's a pointer to a pointer to int
    match &symbol.unwrap().typ {
        Type::Ptr(inner1) => {
            match &**inner1 {
                Type::Ptr(inner2) => assert!(matches!(**inner2, Type::Int)),
                _ => panic!("Expected pointer to pointer type"),
            }
        },
        _ => panic!("Expected pointer type"),
    }
}

#[test]
fn test_invalid_function_declaration() {
    let mut parser = parser_with_first_token("int foo(int) { return 0; }");
    let result = parser.parse_global_declaration();
    assert!(result.is_err());
}

#[test]
fn test_missing_closing_brace() {
    let source = "{ int x; int y;";
    let mut parser = Parser::new(source.as_bytes());
    let result = parser.parse_compound_statement(true);
    assert!(result.is_err());
}

#[test]
fn test_missing_closing_paren_in_if() {
    let source = "if (x > 0 { return 1; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add x to symbol table
    let x_symbol = Symbol {
        name: "x".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(x_symbol).unwrap();
    
    let result = parser.parse_statement();
    assert!(result.is_err());
}

#[test]
fn test_variable_shadowing() {
    let source = "{ int x; x = 1; { int x; x = 2; } }";
    let mut parser = Parser::new(source.as_bytes());
    let result = parser.parse_compound_statement(true);
    assert!(result.is_ok());
}

#[test]
fn test_parse_expression_statement() {
    let source = "x = 42;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add x to symbol table
    let x_symbol = Symbol {
        name: "x".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(x_symbol).unwrap();
    
    let result = parser.parse_expression_statement();
    assert!(result.is_ok());
}

#[test]
fn test_parse_char_type() {
    let mut parser = parser_with_first_token("char c;");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    
    let symbol = parser.symbol_table.lookup("c");
    assert!(symbol.is_some());
    assert!(matches!(symbol.unwrap().typ, Type::Char));
}

#[test]
fn test_parse_pointer_to_char() {
    let mut parser = parser_with_first_token("char *str;");
    let result = parser.parse_global_declaration();
    assert!(result.is_ok());
    
    let symbol = parser.symbol_table.lookup("str");
    assert!(symbol.is_some());
    match &symbol.unwrap().typ {
        Type::Ptr(inner) => assert!(matches!(**inner, Type::Char)),
        _ => panic!("Expected pointer type"),
    }
}

#[test]
fn test_parse_complex_return() {
    let source = "return a + b * (c - d);";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["a", "b", "c", "d"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result = parser.parse_statement();
    assert!(result.is_ok());
}

#[test]
fn test_parse_empty_statement() {
    let source = ";";
    let mut parser = Parser::new(source.as_bytes());
    let result = parser.parse_statement();
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiple_statements() {
    let source = "x = 1; y = 2; z = x + y;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["x", "y", "z"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result1 = parser.parse_statement();
    let result2 = parser.parse_statement();
    let result3 = parser.parse_statement();
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[test]
fn test_parse_comparison_operators() {
    let source = "if (a == b && c != d || e < f && g > h || i <= j && k >= l) { x = 1; }";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "x"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result = parser.parse_statement();
    assert!(result.is_ok());
}

#[test]
fn test_parse_bitwise_operators() {
    let source = "x = a & b | c ^ d << 2 >> 1;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    for var in ["a", "b", "c", "d", "x"] {
        let symbol = Symbol {
            name: var.to_string(),
            class: Class::Global,
            typ: Type::Int,
            val: 0,
            offset: 0,
        };
        parser.symbol_table.add_symbol(symbol).unwrap();
    }
    
    let result = parser.parse_expression_statement();
    assert!(result.is_ok());
}

#[test]
fn test_parse_address_of_operator() {
    let source = "ptr = &x;";
    let mut parser = Parser::new(source.as_bytes());
    
    // Add variables to symbol table
    let x_symbol = Symbol {
        name: "x".to_string(),
        class: Class::Global,
        typ: Type::Int,
        val: 0,
        offset: 0,
    };
    let ptr_symbol = Symbol {
        name: "ptr".to_string(),
        class: Class::Global,
        typ: Type::Ptr(Box::new(Type::Int)),
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(x_symbol).unwrap();
    parser.symbol_table.add_symbol(ptr_symbol).unwrap();
    
    let result = parser.parse_expression_statement();
    assert!(result.is_ok());
}
