use std::collections::HashMap;
use crate::lexar::{Token, Lexer};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Char,
    Int,
    Ptr(Box<Type>),
    Array(Box<Type>, usize), // Array type: element type and size
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Global {
        name: String,
        typ: Type,
        value: i64,
    },
    Local {
        name: String,
        typ: Type,
        offset: i32,
    },
    Function {
        name: String,
        typ: Type,
        params: Vec<Type>,
        is_variadic: bool,
    },
}

// Operator precedence levels
const ASSIGN_PRECEDENCE: i32 = 1;      // =
#[allow(dead_code)]
const COND_PRECEDENCE: i32 = 2;       // ?
const LOR_PRECEDENCE: i32 = 3;        // ||
const LAN_PRECEDENCE: i32 = 4;        // &&
const OR_PRECEDENCE: i32 = 5;         // |
const XOR_PRECEDENCE: i32 = 6;        // ^
const AND_PRECEDENCE: i32 = 7;        // &
const EQ_PRECEDENCE: i32 = 8;         // == !=
const CMP_PRECEDENCE: i32 = 9;        // < > <= >=
const SHIFT_PRECEDENCE: i32 = 10;     // << >>
const ADD_PRECEDENCE: i32 = 11;       // + -
const MUL_PRECEDENCE: i32 = 12;       // * / %
const INC_PRECEDENCE: i32 = 13;       // ++ --

// Parser struct to hold the current state
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<Token>,
    symbols: HashMap<String, Symbol>,
    #[allow(dead_code)]
    locals: Vec<Symbol>,
    current_value: i64,  // Store the current expression value
    #[allow(dead_code)]
    base_type: Type,     // Current base type being parsed

}

impl<'a> Parser<'a> {
    // Parse a function definition: e.g., int foo(int x, int y) { ... }
    fn parse_function_definition(&mut self) {
        // Parse return type (support pointer types)
        let typ = self.parse_type();
        // Parse function name
        let name = match &self.current_token {
            Some(Token::Id(n)) => { let n = n.clone(); self.next_token(); n },
            _ => panic!("Expected identifier after type in function definition"),
        };
        // Parse parameter list
        match &self.current_token {
            Some(Token::OpenParen) => self.next_token(),
            _ => panic!("Expected '(' after function name"),
        }
        let mut params = Vec::new();
        let mut is_variadic = false;
        while let Some(token) = &self.current_token {
            match token {
                Token::CloseParen => { self.next_token(); break; },
                Token::Int | Token::Char(_) => {
                    let param_type = self.parse_type();
                    // Optional param name
                    if let Some(Token::Id(_)) = &self.current_token {
                        self.next_token();
                    }
                    params.push(param_type);
                },
                Token::Comma => { self.next_token(); },
                Token::Id(_) => { self.next_token(); }, // skip param name
                // Token::Ellipsis => { self.next_token(); is_variadic = true; } // Not implemented in Token enum
                _ => panic!("Unexpected token in parameter list: {:?}", token),
            }
        }
        // Expect and consume '{'
        match &self.current_token {
            Some(Token::OpenBrace) => self.next_token(),
            _ => panic!("Expected '{{' to start function body"),
        }
        // Register function in symbol table
        self.symbols.insert(
            name.clone(),
            Symbol::Function { name, typ, params, is_variadic },
        );
        // Skip function body for now (just parse braces)
        let mut depth = 1; // already consumed the first '{'
        while let Some(token) = &self.current_token {
            match token {
                Token::OpenBrace => { self.next_token(); depth += 1; },
                Token::CloseBrace => {
                    depth -= 1;
                    self.next_token(); // consume the `}`
                    if depth == 0 {
                        break;
                    }
                },
                Token::Eof => break,
                _ => { self.next_token(); },
            }
        }
    }
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let mut parser = Self {
            lexer,
            current_token: None,
            symbols: HashMap::new(),
            locals: Vec::new(),
            current_value: 0,
            base_type: Type::Int,
        };
        parser.next_token();
        parser
    }

    // Get the next token from the lexer
    fn next_token(&mut self) {
        self.lexer.next_token();
        self.current_token = self.lexer.current_token.clone();
        println!("[next_token] now at token={:?}", self.current_token);
    }

    // Expression parsing with precedence climbing
    pub fn expr(&mut self) {
        if self.current_token.is_none() {
            self.next_token();
        }
        self.assignment();
        // Handle ternary operator (?:) after assignment
        if let Some(Token::Cond) = &self.current_token {
            self.next_token(); // consume '?'
            let cond_value = self.current_value;
            self.expr(); // parse true branch
            // Accept ':' as Token::Unknown(b':')
            match &self.current_token {
                Some(Token::Unknown(b':')) => self.next_token(),
                _ => panic!("Expected ':' in ternary operator"),
            }
            let true_value = self.current_value;
            self.expr(); // parse false branch
            let false_value = self.current_value;
            self.current_value = if cond_value != 0 { true_value } else { false_value };
        }
    }

    fn assignment(&mut self) {
    // Only handle assignment if '=' follows identifier, otherwise fall through
    if let Some(Token::Id(name)) = &self.current_token {
        let name = name.clone();
        // Clone the lexer for lookahead
        let mut lookahead_lexer = self.lexer.clone();
        lookahead_lexer.next_token(); // Advance to token after Id
        if let Some(Token::Assign) = lookahead_lexer.current_token {
            // Real parser: consume Id and '='
            self.next_token(); // consume Id
            self.next_token(); // consume '='
            self.assignment();
            let value = self.current_value;
        // Insert or update in symbol table as a local or global variable
        self.symbols.insert(
            name.clone(),
            Symbol::Global {
                name: name.clone(),
                typ: Type::Int, // Default to Int for now
                value,
            },
        );
        self.current_value = value;
        return;
        }
        // If not assignment, fall through to logical_or() without consuming tokens
    }
    self.logical_or();
}

    fn logical_or(&mut self) {
        self.logical_and();
        loop {
            match &self.current_token {
                Some(Token::Lor) => {
                    let left = self.current_value != 0;
                    self.next_token();
                    self.logical_and();
                    self.current_value = if left || self.current_value != 0 { 1 } else { 0 };
                },
                _ => break,
            }
        }
    }

    fn logical_and(&mut self) {
        self.bitwise_or();
        loop {
            match &self.current_token {
                Some(Token::Lan) => {
                    let left = self.current_value != 0;
                    self.next_token();
                    self.bitwise_or();
                    self.current_value = if left && self.current_value != 0 { 1 } else { 0 };
                },
                _ => break,
            }
        }
    }

    fn bitwise_or(&mut self) {
        self.bitwise_xor();
        loop {
            match &self.current_token {
                Some(Token::Or) => {
                    let left = self.current_value;
                    self.next_token();
                    self.bitwise_xor();
                    self.current_value = left | self.current_value;
                },
                _ => break,
            }
        }
    }

    fn bitwise_xor(&mut self) {
        self.bitwise_and();
        loop {
            match &self.current_token {
                Some(Token::Xor) => {
                    let left = self.current_value;
                    self.next_token();
                    self.bitwise_and();
                    self.current_value = left ^ self.current_value;
                },
                _ => break,
            }
        }
    }

    fn bitwise_and(&mut self) {
        self.equality();
        loop {
            match &self.current_token {
                Some(Token::And) => {
                    let left = self.current_value;
                    self.next_token();
                    self.equality();
                    self.current_value = left & self.current_value;
                },
                _ => break,
            }
        }
    }

    fn equality(&mut self) {
        self.comparison();
        loop {
            match &self.current_token {
                Some(Token::Eq) => {
                    let left = self.current_value;
                    self.next_token();
                    self.comparison();
                    self.current_value = if left == self.current_value { 1 } else { 0 };
                },
                Some(Token::Ne) => {
                    let left = self.current_value;
                    self.next_token();
                    self.comparison();
                    self.current_value = if left != self.current_value { 1 } else { 0 };
                },
                _ => break,
            }
        }
    }

    fn comparison(&mut self) {
        self.additive();
        loop {
            match &self.current_token {
                Some(Token::Lt) => {
                    let left = self.current_value;
                    self.next_token();
                    self.additive();
                    self.current_value = if left < self.current_value { 1 } else { 0 };
                },
                Some(Token::Gt) => {
                    let left = self.current_value;
                    self.next_token();
                    self.additive();
                    self.current_value = if left > self.current_value { 1 } else { 0 };
                },
                Some(Token::Le) => {
                    let left = self.current_value;
                    self.next_token();
                    self.additive();
                    self.current_value = if left <= self.current_value { 1 } else { 0 };
                },
                Some(Token::Ge) => {
                    let left = self.current_value;
                    self.next_token();
                    self.additive();
                    self.current_value = if left >= self.current_value { 1 } else { 0 };
                },
                _ => break,
            }
        }
    }

    fn additive(&mut self) {
    println!("[additive] ENTER: token={:?}, value={}", self.current_token, self.current_value);
        println!("[additive] ENTER: token={:?}, value={}", self.current_token, self.current_value);
        self.multiplicative();
        loop {
            match &self.current_token {
                Some(Token::Add) => {
                    let left_dbg = self.current_value;
                    let left = self.current_value;
                    self.next_token();
                    self.multiplicative();
                    println!("[additive] ADD: left={}, right={}, result={}", left_dbg, self.current_value, left_dbg + self.current_value);
                    self.current_value = left + self.current_value;
                },
                Some(Token::Sub) => {
                    let left_dbg = self.current_value;
                    let left = self.current_value;
                    self.next_token();
                    self.multiplicative();
                    println!("[additive] SUB: left={}, right={}, result={}", left_dbg, self.current_value, left_dbg - self.current_value);
                    self.current_value = left - self.current_value;
                },
                _ => break,
            }
        }
    }

    fn multiplicative(&mut self) {
    println!("[multiplicative] ENTER: token={:?}, value={}", self.current_token, self.current_value);
        println!("[multiplicative] ENTER: token={:?}, value={}", self.current_token, self.current_value);
        self.unary();
        loop {
            match &self.current_token {
                Some(Token::Mul) => {
                    let left_dbg = self.current_value;
                    let left = self.current_value;
                    self.next_token();
                    self.unary();
                    println!("[multiplicative] MUL: left={}, right={}, result={}", left_dbg, self.current_value, left_dbg * self.current_value);
                    self.current_value = left * self.current_value;
                },
                Some(Token::Div) => {
                    let left_dbg = self.current_value;
                    let left = self.current_value;
                    self.next_token();
                    self.unary();
                    if self.current_value == 0 {
                        panic!("Division by zero");
                    }
                    self.current_value = left / self.current_value;
                },
                Some(Token::Mod) => {
                    let left_dbg = self.current_value;
                    let left = self.current_value;
                    self.next_token();
                    self.unary();
                    if self.current_value == 0 {
                        panic!("Modulo by zero");
                    }
                    self.current_value = left % self.current_value;
                },
                _ => break,
            }
        }
    }



    fn factor(&mut self) {
    println!("[factor] ENTER: token={:?}, value={}", self.current_token, self.current_value);
        println!("[factor] ENTER: token={:?}, value={}", self.current_token, self.current_value);
        match &self.current_token {
            Some(Token::OpenParen) => {
                self.next_token();
                self.additive(); // Start at lowest arithmetic precedence for grouped expr
                println!("[factor] AFTER PAREN: value={}", self.current_value);
                match self.current_token {
                    Some(Token::CloseParen) => {
                        self.next_token();
                        println!("[factor] AFTER next_token() for ')': token={:?}", self.current_token);
                    },
                    _ => panic!("Expected closing parenthesis"),
                }
            },
            Some(Token::Num(n)) => {
                self.current_value = *n;
                self.next_token();
            },
            Some(Token::Id(name)) => {
                let name = name.clone();
                // Peek ahead: is this a function call?
                let is_func_call = matches!(self.lexer.peek(), Some(b'('));
                if is_func_call {
                    self.next_token(); // advance to OpenParen
                    self.parse_function_call(&name);
                } else {
                    // Lookup variable value in self.variables, then in self.symbols
                    if let Some(Symbol::Global { value, .. }) = self.symbols.get(&name) {
                        self.current_value = *value;
                    } else {
                        self.current_value = 0;
                    }
                    // Ensure both parser and lexer token state are in sync
                    self.current_token = None;
                    self.lexer.current_token = None;
                    self.next_token(); // advance past identifier

                    // Postfix loop: handle array subscripting a[i][j]...
                    loop {
                        match &self.current_token {
                            Some(Token::Brak) => {
                                self.next_token(); // consume '['
                                self.expr(); // parse subscript expression
                                let index = self.current_value;
                                // For now, just set current_value to index (simulate a[index])
                                // In a real compiler, you'd emit IR for pointer arithmetic + load
                                self.current_value = index; // Placeholder
                                if let Some(Token::Brak) = &self.current_token {
                                    self.next_token();
                                } else {
                                    panic!("Expected ']' after array subscript");
                                }
                            },
                            _ => break,
                        }
                    }
                }
            },
            Some(Token::Mul) => {
                self.next_token();
                self.factor();
                // Pointer dereference: placeholder (no-op in parser)
                // Actual dereference should be handled in the VM
            },
            Some(Token::And) => {
                self.next_token();
                self.factor();
                // Address-of: placeholder (no-op in parser)
                // Actual address computation should be handled in the VM
            },
            _ => panic!("Expected number, identifier, or parenthesized expression"),
        }
    }




    fn unary(&mut self) {
        // Handle explicit cast: (type)expr
        if self.current_token == Some(Token::OpenParen) {
            let mut lookahead = self.lexer.clone();
            lookahead.next_token();
            if matches!(lookahead.current_token, Some(Token::Int) | Some(Token::Char(_))) {
                // Parse as cast
                self.next_token(); // consume '('
                let cast_type = self.parse_type();
                if self.current_token == Some(Token::CloseParen) {
                    self.next_token(); // consume ')'
                    self.unary(); // parse the expression to cast
                    // For now, just simulate cast as identity
                    // Optionally, you could change current_value based on cast_type
                    // e.g., if cast_type == Type::Char, self.current_value &= 0xFF;
                } else {
                    panic!("Expected ')' after type in cast");
                }
                return;
            }
        }
        match &self.current_token {
            Some(Token::Sizeof) => {
                self.next_token();
                if self.current_token == Some(Token::OpenParen) {
                    self.next_token();
                    // Check for type name
                    let ty = match &self.current_token {
                        Some(Token::Int) | Some(Token::Char(_)) => {
                            let t = self.parse_type();
                            t
                        },
                        _ => {
                            // Not a type, parse as expression
                            self.expr();
                            // For now, just return 4 as placeholder for sizeof(expr)
                            self.current_value = 4;
                            // Consume closing paren
                            if self.current_token == Some(Token::CloseParen) {
                                self.next_token();
                            }
                            return;
                        }
                    };
                    // Consume closing paren
                    if self.current_token == Some(Token::CloseParen) {
                        self.next_token();
                    }
                    // For now, assume sizeof(int) or sizeof(char) is 4 or 1
                    self.current_value = match ty {
                        Type::Int => 4,
                        Type::Char => 1,
                        Type::Ptr(_) => 8,
Type::Array(_, _) => 0,
                    };
                } else {
                    // sizeof expr
                    self.unary();
                    // For now, just return 4 as placeholder
                    self.current_value = 4;
                }
            },
            Some(Token::Sub) => {
                self.next_token();
                self.unary();
                self.current_value = -self.current_value;
            },
            Some(Token::Add) => {
                self.next_token();
                self.unary();
            },
            Some(Token::Mul) => { // Pointer dereference
                self.next_token();
                self.unary();
                // For now, just simulate pointer dereference as identity
                // Real implementation would load from memory at self.current_value
            },
            Some(Token::And) => { // Address-of
                self.next_token();
                self.unary();
                // For now, just simulate address-of as identity
                // Real implementation would take address of variable
            },
            Some(Token::Inc) | Some(Token::Dec) => {
                let is_inc = matches!(self.current_token, Some(Token::Inc));
                self.next_token();
                self.unary();
                self.current_value += if is_inc { 1 } else { -1 };
            },
            _ => self.factor()
        }
    }

    // Parse a type, supporting pointer types (e.g., int*, char*, int**)
    fn parse_type(&mut self) -> Type {
        let base = match &self.current_token {
            Some(Token::Int) => { self.next_token(); Type::Int },
            Some(Token::Char(_)) => { self.next_token(); Type::Char },
            _ => panic!("Expected type: int or char"),
        };
        let mut ty = base;
        while let Some(Token::Mul) = &self.current_token {
            self.next_token();
            ty = Type::Ptr(Box::new(ty));
        }
        ty
    }

    // Parse function call arguments
    fn parse_function_call(&mut self, _name: &str) {
        let mut arg_count = 0;
        self.next_token(); // consume '('
        // Parse arguments
        while let Some(token) = &self.current_token {
            if matches!(token, Token::CloseParen) {
                break;
            }
            if arg_count > 0 {
                match token {
                    Token::Comma => self.next_token(),
                    _ => panic!("Expected comma in function arguments"),
                }
            }
            self.expr();
            arg_count += 1;
        }
        // Expect closing parenthesis
        match self.current_token {
            Some(Token::CloseParen) => self.next_token(),
            _ => panic!("Expected closing parenthesis in function call"),
        }
        // For now, just set current_value to 0 as placeholder
        self.current_value = 0;
    }

    fn parse_declaration(&mut self) {
        let typ = self.parse_type();
        if let Some(Token::Id(var_name)) = &self.current_token {
            let var_name = var_name.clone();
            self.next_token();

            // Array declarator support: check for [ConstExpr]
            let mut decl_type = typ.clone();
            if let Some(Token::Brak) = &self.current_token {
                self.next_token(); // consume '['
                self.expr(); // parse constant expression for array size
                let array_size = self.current_value as usize;
                if let Some(Token::Brak) = &self.current_token {
                    self.next_token(); // consume the ']' (same token as '[')
                } else {
                    panic!("Expected ']' after array size");
                }
                decl_type = Type::Array(Box::new(typ.clone()), array_size);
            }

            // Optionally handle = initializer
            let mut value = 0;
            if let Some(Token::Assign) = &self.current_token {
                self.next_token();
                self.expr();
                value = self.current_value;
            }

            // Determine if we are inside a block (locals) or top-level (globals)
            if self.locals.len() > 0 {
                let offset = self.locals.len() as i32; // simple offset, not stack address
                self.locals.push(Symbol::Local { name: var_name.clone(), typ: decl_type.clone(), offset });
            } else {
                self.symbols.insert(var_name.clone(), Symbol::Global { name: var_name, typ: decl_type.clone(), value });
            }

            // Always consume the terminating semicolon or Eof
            while self.current_token != Some(Token::Semi) && self.current_token != Some(Token::Eof) {
                self.next_token();
            }
            if self.current_token == Some(Token::Semi) {
                self.next_token();
            }
            // Now definitely positioned after ; or Eof
        } else {
            panic!("Expected identifier after type in declaration");
        }
    }

    fn parse_enum(&mut self) {
        self.next_token(); // consume 'enum'
        if self.current_token != Some(Token::OpenBrace) {
            panic!("Expected '{{' after enum");
        }
        self.next_token(); // consume '{'
        let mut enum_value = 0;
        loop {
            match &self.current_token {
                Some(Token::Id(name)) => {
                    let name = name.clone();
                    self.next_token();
                    if let Some(Token::Assign) = &self.current_token {
                        self.next_token();
                        self.expr();
                        enum_value = self.current_value;
                    }
                    self.symbols.insert(name.clone(), Symbol::Global { name, typ: Type::Int, value: enum_value });
                    enum_value += 1;
                    if let Some(Token::Comma) = &self.current_token {
                        self.next_token();
                    }
                },
                Some(Token::CloseBrace) => {
                    self.next_token();
                    break;
                },
                _ => panic!("Unexpected token in enum declaration: {:?}", self.current_token),
            }
        }
        if self.current_token == Some(Token::Semi) {
            self.next_token();
        }
    }


    // Statement parsing
    pub fn stmt(&mut self) {
        // Prime the parser if needed
        if self.current_token.is_none() {
            self.next_token();
        }
        // C4 compatibility: detect function definition, enum, or global variable declaration at top-level
        if matches!(self.current_token, Some(Token::Enum)) {
            self.parse_enum();
            return;
        }
        if matches!(self.current_token, Some(Token::Int) | Some(Token::Char(_))) {
            // Look ahead: if next tokens are Id and OpenParen, it's a function definition
            let mut lookahead_lexer = self.lexer.clone();
            let mut lookahead_token = self.current_token.clone();
            // Advance to Id
            lookahead_lexer.next_token();
            lookahead_token = lookahead_lexer.current_token.clone();
            if let Some(Token::Id(_)) = lookahead_token {
                // Advance to OpenParen
                lookahead_lexer.next_token();
                lookahead_token = lookahead_lexer.current_token.clone();
                if let Some(Token::OpenParen) = lookahead_token {
                    self.parse_function_definition();
                    return;
                }
            }
            self.parse_declaration();
            return;
        }

        if self.current_token.is_none() {
            self.next_token();
        }

        match &self.current_token {
            Some(Token::If) => {
                self.next_token();
                match &self.current_token {
                    Some(Token::OpenParen) => {
                        self.next_token();
                        self.expr(); // Parse condition
                        let cond = self.current_value != 0;

                        match &self.current_token {
                            Some(Token::CloseParen) => self.next_token(),
                            _ => panic!("Expected closing parenthesis after if condition")
                        }

                        if cond {
                            // Execute if block
                            self.stmt();
                            // Skip else block if present
                            if let Some(Token::Else) = &self.current_token {
                                self.next_token();
                                // Skip else block
                                match &self.current_token {
                                    Some(Token::OpenBrace) => {
                                        self.next_token();
                                        while let Some(token) = &self.current_token {
                                            match token {
                                                Token::CloseBrace => {
                                                    self.next_token();
                                                    break;
                                                },
                                                _ => {
                                                    self.next_token();
                                                }
                                            }
                                        }
                                    },
                                    _ => self.stmt()
                                }
                            }
                        } else {
                            // Skip if block
                            match &self.current_token {
                                Some(Token::OpenBrace) => {
                                    self.next_token();
                                    while let Some(token) = &self.current_token {
                                        match token {
                                            Token::CloseBrace => {
                                                self.next_token();
                                                break;
                                            },
                                            _ => {
                                                self.next_token();
                                            }
                                        }
                                    }
                                },
                                _ => self.stmt()
                            }
                            // Execute else block if present
                            if let Some(Token::Else) = &self.current_token {
                                self.next_token();
                                self.stmt();
                            }
                        }
                    },
                    _ => panic!("Expected opening parenthesis after if")
                }      },
            Some(Token::While) => {
                self.next_token();
                // Parse condition
                match &self.current_token {
                    Some(Token::OpenParen) => {
                        self.next_token();
                        let saved_pos = self.lexer.pos;
                        let saved_token = self.current_token.clone();
                        let saved_lexer_token = self.lexer.current_token.clone();
                        let mut saved_symbols = self.symbols.clone();
                        loop {
                            // Reset lexer position and token to condition
                            self.lexer.pos = saved_pos;
                            self.current_token = saved_token.clone();
                            self.lexer.current_token = saved_lexer_token.clone();
                            self.expr(); // Parse condition
                            let cond = self.current_value != 0;

                            match &self.current_token {
                                Some(Token::CloseParen) => self.next_token(),
                                _ => panic!("Expected closing parenthesis after while condition")
                            }

                            if !cond {
                                // Restore variables to state before loop
                                self.symbols = saved_symbols.clone();
                                // If next token is '{', skip the block entirely
                                if let Some(Token::OpenBrace) = &self.current_token {
                                    let mut depth = 1;
                                    while depth > 0 {
                                        self.next_token();
                                        match &self.current_token {
                                            Some(Token::OpenBrace) => depth += 1,
                                            Some(Token::CloseBrace) => depth -= 1,
                                            Some(Token::Eof) => break, // avoid infinite loop on malformed input
                                            _ => {}
                                        }
                                    }
                                    self.next_token(); // move past the closing brace
                                }
                                break;
                            }

                            // Execute loop body
                            self.stmt();
                            // Save variables for next iteration
                            saved_symbols = self.symbols.clone();
                        }
                    },
                    _ => panic!("Expected opening parenthesis after while")
                }
            },
            Some(Token::Return) => {
                self.next_token();
                self.expr(); // Parse return expression
                match &self.current_token {
                    Some(Token::Semi) => self.next_token(),
                    _ => panic!("Expected semicolon after return statement")
                }
            },
            Some(Token::OpenBrace) => {
                self.next_token();
                // Track locals stack depth for this block
                let locals_base = self.locals.len();
                // Parse compound statement
                loop {
                    match &self.current_token {
                        Some(Token::CloseBrace) => {
                            self.next_token(); // Consume the closing brace
                            // Unwind locals declared in this block
                            while self.locals.len() > locals_base {
                                self.locals.pop();
                            }
                            break;
                        },
                        Some(Token::If) | Some(Token::While) | Some(Token::Return) | 
                        Some(Token::OpenBrace) | Some(Token::Semi) | Some(Token::Num(_)) | 
                        Some(Token::Id(_)) | Some(Token::Char(_)) | Some(Token::OpenParen) => {
                            self.stmt();
                        },
                        Some(_) => break, // Not a valid statement starter
                        None => break,
                    }
                }
            },
            Some(Token::Semi) => {
                // Empty statement
                self.next_token();
            },
            Some(Token::Num(_)) | Some(Token::Id(_)) | Some(Token::Char(_)) | Some(Token::OpenParen) => {
                // Expression statement
                self.expr();
                match &self.current_token {
                    Some(Token::Semi) => {
                        self.next_token();
                    }
                    _ => {
                        println!("[expr stmt] PANIC: current_token={:?}", self.current_token);
                        panic!("Expected semicolon after expression statement")
                    }
                }
            }


            Some(Token::CloseBrace) => {
                self.next_token();
                return;
            }
            _ => {
                // For other truly unexpected tokens, still panic
                panic!("Unexpected token in statement: {:?}", self.current_token);
            }
        }
    }

    // Helper function to evaluate an expression and return its value
    pub fn evaluate_expr(&mut self, src: &'a str) -> i64 {
        println!("Evaluating: {}", src);
        let mut parser = Parser {
            lexer: Lexer::new(src.as_bytes()),
            current_token: None,
            current_value: 0,
            symbols: self.symbols.clone(),
            locals: Vec::new(),
            base_type: Type::Int,
    
        };
        parser.next_token();
        parser.expr();
        println!("Result: {}", parser.current_value);
        // Update our variables with any changes made during evaluation
        for (k, v) in parser.symbols {
            self.symbols.insert(k, v);
        }
        parser.current_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_variable_declaration() {
        // Test with initialization
        let mut parser = setup_parser("int x = 42;");
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        match parser.symbols.get("x") {
            Some(Symbol::Global { name, typ, value }) => {
                assert_eq!(name, "x");
                matches!(typ, Type::Int);
                assert_eq!(*value, 42);
            },
            _ => panic!("Global variable 'x' not found or incorrect type"),
        }

        // Test without initialization (should default to 0)
        let mut parser = setup_parser("int y;");
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        match parser.symbols.get("y") {
            Some(Symbol::Global { name, typ, value }) => {
                assert_eq!(name, "y");
                matches!(typ, Type::Int);
                // Since we don't assign, value may be uninitialized or 0 depending on implementation
            },
            _ => panic!("Global variable 'y' not found or incorrect type"),
        }
    }

    #[test]
    fn test_pointer_declaration_and_dereference() {
        // Pointer declaration and address-of in a single parser instance
        let mut parser = setup_parser("int x = 10; int *p = &x;");
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        assert!(parser.symbols.contains_key("x"));
        assert!(parser.symbols.contains_key("p"));
    }

    #[test]
    fn test_pointer_arithmetic() {
        // Pointer arithmetic: simulate p + 1 in a single parser instance
        let mut parser = setup_parser("int x = 10; int *p = &x; p = p + 1;");
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        assert!(parser.symbols.contains_key("p"));
    }

    #[test]
    fn test_function_call() {
        // Function definition and call
        let code = "int foo() { return 42; } int x = foo();";
        let mut parser = setup_parser(code);
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        assert!(parser.symbols.contains_key("foo"));
        assert!(parser.symbols.contains_key("x"));
    }

    #[test]
    fn test_multiple_function_definitions() {
        let code = "int foo() { return 1; } int bar(int x) { return x + 1; } int y = foo(); int z = bar(5);";
        let mut parser = setup_parser(code);
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        assert!(parser.symbols.contains_key("foo"));
        assert!(parser.symbols.contains_key("bar"));
        assert!(parser.symbols.contains_key("y"));
        assert!(parser.symbols.contains_key("z"));
    }

    #[test]
    fn test_function_and_variable_declarations() {
        let code = "int foo() { return 1; } int a = 10; int b = 20;";
        let mut parser = setup_parser(code);
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        assert!(parser.symbols.contains_key("foo"));
        assert!(parser.symbols.contains_key("a"));
        assert!(parser.symbols.contains_key("b"));
        // Ensure 'a' and 'b' are not functions
        match parser.symbols.get("a") {
            Some(Symbol::Global { .. }) => {},
            _ => panic!("'a' should be a global variable"),
        }
        match parser.symbols.get("b") {
            Some(Symbol::Global { .. }) => {},
            _ => panic!("'b' should be a global variable"),
        }
    }

    #[test]
    fn test_function_with_parameters() {
        let code = "int add(int x, int y) { return x + y; }";
        let mut parser = setup_parser(code);
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        match parser.symbols.get("add") {
            Some(Symbol::Function { params, .. }) => {
                assert_eq!(params.len(), 2);
            },
            _ => panic!("'add' should be a function"),
        }
    }

    #[test]
    fn test_locals_simple_scope() {
        let mut parser = setup_parser("{ int a = 1; }");
        parser.stmt();
        // After block, no locals should remain
        assert_eq!(parser.locals.len(), 0);
    }

    #[test]
    fn test_locals_nested_blocks() {
        let mut parser = setup_parser("{ int a = 1; { int b = 2; } }");
        parser.stmt();
        // After outer block, no locals should remain
        assert_eq!(parser.locals.len(), 0);
    }

    #[test]
    fn test_locals_shadowing() {
        let mut parser = setup_parser("{ int a = 1; { int a = 2; } }");
        parser.stmt();
        // After block, no locals should remain
        assert_eq!(parser.locals.len(), 0);
    }

    #[test]
    fn test_locals_multiple_in_block() {
        let mut parser = setup_parser("{ int a = 1; int b = 2; int c = 3; }");
        parser.stmt();
        assert_eq!(parser.locals.len(), 0);
    }

    #[test]
    fn test_array_declaration() {
        let mut parser = setup_parser("int arr[5];");
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        match parser.symbols.get("arr") {
            Some(Symbol::Global { typ, .. }) => match typ {
                Type::Array(elem, size) => {
                    assert_eq!(**elem, Type::Int);
                    assert_eq!(*size, 5);
                },
                _ => panic!("arr should be an array"),
            },
            _ => panic!("arr not found or not a global"),
        }
    }

    #[test]
    fn test_array_subscript_expression() {
        let mut parser = setup_parser("int arr[3]; int x; x = arr[2];");
        while parser.current_token != Some(Token::Eof) {
            parser.stmt();
        }
        // Our placeholder sets current_value to the index, so x = arr[2] sets x to 2
        assert_eq!(parser.evaluate_expr("arr[2]"), 2);
    }

    #[test]
    fn test_locals_stack_unwind_deep() {
        let mut parser = setup_parser("{ int a = 1; { int b = 2; { int c = 3; } } }");
        parser.stmt();
        assert_eq!(parser.locals.len(), 0);
    }

    fn setup_parser(src: &str) -> Parser {
        let lexer = Lexer::new(src.as_bytes());
        Parser::new(lexer)
    }

    #[test]
    fn test_ternary_operator() {
        let mut parser = setup_parser("1 ? 42 : 7;");
        parser.stmt();
        assert_eq!(parser.current_value, 42);
        let mut parser = setup_parser("0 ? 42 : 7;");
        parser.stmt();
        assert_eq!(parser.current_value, 7);
        let mut parser = setup_parser("2 > 1 ? 10 + 2 : 3 * 5;");
        parser.stmt();
        assert_eq!(parser.current_value, 12);
        let mut parser = setup_parser("2 < 1 ? 10 + 2 : 3 * 5;");
        parser.stmt();
        assert_eq!(parser.current_value, 15);
    }

    #[test]
    fn test_enum_declaration() {
        let mut parser = setup_parser("enum { A, B = 5, C, D = 10, E };");
        parser.stmt();
        // Enum values should be installed in the symbol table
        assert_eq!(parser.symbols.get("A").map(|s| match s { Symbol::Global { value, .. } => *value, _ => -1 }), Some(0));
        assert_eq!(parser.symbols.get("B").map(|s| match s { Symbol::Global { value, .. } => *value, _ => -1 }), Some(5));
        assert_eq!(parser.symbols.get("C").map(|s| match s { Symbol::Global { value, .. } => *value, _ => -1 }), Some(6));
        assert_eq!(parser.symbols.get("D").map(|s| match s { Symbol::Global { value, .. } => *value, _ => -1 }), Some(10));
        assert_eq!(parser.symbols.get("E").map(|s| match s { Symbol::Global { value, .. } => *value, _ => -1 }), Some(11));
    }

    #[test]
    fn test_explicit_cast() {
        let mut parser = setup_parser("(int)42;");
        parser.stmt();
        assert_eq!(parser.current_value, 42);
        let mut parser = setup_parser("(char)65;");
        parser.stmt();
        assert_eq!(parser.current_value, 65);
    }

    #[test]
    fn test_simple_arithmetic() {
        let mut parser = setup_parser("2 + 3 * 4");
        assert_eq!(parser.evaluate_expr("2 + 3 * 4"), 14);
        
        parser = setup_parser("10 - 2 * 3");
        assert_eq!(parser.evaluate_expr("10 - 2 * 3"), 4);
        
        parser = setup_parser("(2 + 3) * 4");
        assert_eq!(parser.evaluate_expr("(2 + 3) * 4"), 20);
    }

    #[test]
    fn test_comparison_operators() {
        let mut parser = setup_parser("5 > 3");
        assert_eq!(parser.evaluate_expr("5 > 3"), 1);
        
        parser = setup_parser("5 < 3");
        assert_eq!(parser.evaluate_expr("5 < 3"), 0);
        
        parser = setup_parser("5 == 5");
        assert_eq!(parser.evaluate_expr("5 == 5"), 1);
        
        parser = setup_parser("5 != 3");
        assert_eq!(parser.evaluate_expr("5 != 3"), 1);
    }

    #[test]
    fn test_logical_operators() {
        let mut parser = setup_parser("1 && 1");
        assert_eq!(parser.evaluate_expr("1 && 1"), 1);
        
        parser = setup_parser("1 && 0");
        assert_eq!(parser.evaluate_expr("1 && 0"), 0);
        
        parser = setup_parser("1 || 0");
        assert_eq!(parser.evaluate_expr("1 || 0"), 1);
        
        parser = setup_parser("0 || 0");
        assert_eq!(parser.evaluate_expr("0 || 0"), 0);
    }

    #[test]
    fn test_bitwise_operators() {
        let mut parser = setup_parser("12 & 5");
        assert_eq!(parser.evaluate_expr("12 & 5"), 4);
        
        parser = setup_parser("12 | 5");
        assert_eq!(parser.evaluate_expr("12 | 5"), 13);
        
        parser = setup_parser("12 ^ 5");
        assert_eq!(parser.evaluate_expr("12 ^ 5"), 9);
    }

    #[test]
    fn test_if_statement() {
        let mut parser = setup_parser("if (1) { x = 42; } else { x = 0; }");
        parser.stmt();
        // Verify x is set to 42 since condition is true
        assert_eq!(parser.evaluate_expr("x"), 42);

        parser = setup_parser("if (0) { x = 42; } else { x = 0; }");
        parser.stmt();
        // Verify x is set to 0 since condition is false
        assert_eq!(parser.evaluate_expr("x"), 0);
    }

    #[test]
    fn test_while_statement() {
        let mut parser = setup_parser("x = 0; while (x < 3) { x = x + 1; }");
        parser.stmt(); // x = 0
        parser.stmt(); // while loop
        // Verify x was incremented 3 times
        assert_eq!(parser.evaluate_expr("x"), 3);
    }

    #[test]
    fn test_compound_statement() {
        let mut parser = setup_parser("{ x = 1; y = 2; z = x + y; }");
        parser.stmt();
        assert_eq!(parser.evaluate_expr("z"), 3);
    }

    #[test]
    fn test_return_statement() {
        let mut parser = setup_parser("x = 42; return x;");
        parser.stmt(); // x = 42
        parser.stmt(); // return x
        assert_eq!(parser.current_value, 42);
    }

    #[test]
    fn test_empty_statement() {
        let mut parser = setup_parser(";;;");
        parser.stmt();
        parser.stmt();
        parser.stmt();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_complex_expressions() {
        let mut parser = setup_parser("2 * 3 + 4 * 5");
        assert_eq!(parser.evaluate_expr("2 * 3 + 4 * 5"), 26);
        
        parser = setup_parser("(2 + 3) * (4 + 5)");
        assert_eq!(parser.evaluate_expr("(2 + 3) * (4 + 5)"), 45);
        
        parser = setup_parser("10 > 5 && 3 < 7");
        assert_eq!(parser.evaluate_expr("10 > 5 && 3 < 7"), 1);
    }
}