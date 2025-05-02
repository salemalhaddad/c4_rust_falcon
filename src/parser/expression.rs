use crate::lexer::Token;
use super::{Parser, types::Type};

// Operator precedence levels
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Assignment = 1,  // =
    Conditional,     // ?:
    LogicalOr,       // ||
    LogicalAnd,      // &&
    BitwiseOr,       // |
    BitwiseXor,      // ^
    BitwiseAnd,      // &
    Equality,        // ==, !=
    Relational,      // <, >, <=, >=
    Shift,           // <<, >>
    Additive,        // +, -
    Multiplicative,  // *, /, %
    Unary,           // !, ~, -, +, *, &, sizeof
    Postfix,         // (), [], ->, ., ++, --
    Primary,         // literals, identifiers, (expression)
}



impl<'a> Parser<'a> {
    // Entry point for expression parsing
    pub fn parse_expression(&mut self) -> Result<(), String> {

        println!("DEBUG: Entering parse_expression, current token: {:?}", self.lexer.peek_token());
        // Save class before parsing
        println!("DEBUG: [parse_expression] class BEFORE: {:?}", self.current_class);
        let prev_class = self.current_class.clone();
        // For both statements and conditions, stop at ';' or ')'
        self.parse_expr_with_precedence(
            Precedence::Assignment,
            Some(&[Token::Semi, Token::CloseParen, Token::Comma]),
        )?;
        // If class is None but previous was Function or Sys, restore it
        if self.current_class.is_none() {
            if let Some(super::symbol_table::Class::Function) | Some(super::symbol_table::Class::Sys) = prev_class {
                self.current_class = prev_class;
            }
        }
        println!("DEBUG: Finished parse_expression, current token: {:?}", self.lexer.peek_token());
        println!("DEBUG: [parse_expression] current_class at end: {:?}", self.current_class);
        Ok(())
    }
    
    // Precedence climbing algorithm with stop tokens
    fn parse_expr_with_precedence(&mut self, precedence: Precedence, stop_tokens: Option<&[Token]>) -> Result<(), String> {
        println!("DEBUG: [parse_expr_with_precedence] class at start: {:?}", self.current_class);
        // Parse the first operand
        self.parse_primary_expr(stop_tokens)?;
        
        // Save current_class before operator parsing
        let saved_class = self.current_class.clone();
        // Keep processing operators while their precedence is high enough
        while let Some(token) = self.lexer.peek_token() {
            // Stop if we hit a stop token
            if let Some(stops) = stop_tokens {
                if stops.iter().any(|t| t == &token) {
                    break;
                }
            }
            
            // Get the precedence of the next token
            let token_precedence = self.get_token_precedence(&token);
            
            // If the next token is not an operator or has lower precedence, we're done
            if token_precedence < precedence {
                break;
            }
            // If the token is not an operator at all, also break (do NOT error)
            if token_precedence == Precedence::Primary && !matches!(token,
                Token::Add | Token::Sub | Token::Mul | Token::Div | Token::Mod | Token::Assign | Token::Cond | Token::Eq | Token::Ne | Token::Lt | Token::Gt | Token::Le | Token::Ge | Token::Shl | Token::Shr | Token::And | Token::Or | Token::Xor | Token::Lor | Token::Lan) {
                break;
            }
            // Otherwise, handle as operator
            // Consume the operator token
            self.lexer.next_token();
            // Before parsing the right-hand side, save the current_class
            let lhs_class = self.current_class.clone();
            // Generate code for the operator
            match token {
                Token::Add => {
                    // Parse the right-hand side with higher precedence
                    self.parse_expr_with_precedence(Precedence::Multiplicative, stop_tokens)?;
                }
                Token::Sub => {
                    // Parse the right-hand side with higher precedence
                    self.parse_expr_with_precedence(Precedence::Multiplicative, stop_tokens)?;
                }
                Token::Mul => {
                    // Parse the right-hand side with higher precedence
                    self.parse_expr_with_precedence(Precedence::Unary, stop_tokens)?;
                }
                Token::Div => {
                    // Parse the right-hand side with higher precedence
                    self.parse_expr_with_precedence(Precedence::Unary, stop_tokens)?;
                }
                Token::Mod => {
                    // Parse the right-hand side with higher precedence
                    self.parse_expr_with_precedence(Precedence::Unary, stop_tokens)?;
                }
                Token::Assign => {
                    // Parse the right-hand side with precedence just below assignment
                    self.parse_expr_with_precedence(Precedence::Conditional, stop_tokens)?;
                }
                Token::Cond => {
                    // Parse the middle expression (between ? and :)
                    self.parse_expr_with_precedence(Precedence::Assignment, stop_tokens)?;
                    
                    // Expect and consume the colon
                    if let Some(Token::Unknown(b':')) = self.lexer.peek_token() {
                        self.lexer.next_token();
                    } else {
                        return Err("Expected ':' in conditional expression".to_string());
                    }
                    
                    // Parse the right-hand side with precedence just below assignment
                    self.parse_expr_with_precedence(Precedence::Conditional, stop_tokens)?;
                }
                // Handle other operators similarly...
                _ => {
                    // For simplicity, we'll just parse the right-hand side with the current precedence
                    self.parse_expr_with_precedence(Precedence::Assignment, stop_tokens)?;
                }
            }
            // After parsing the right-hand side, only update current_class if it is Function or Sys; otherwise, preserve lhs_class if it was Function or Sys
            // After parsing each operator, restore class if needed
            if self.current_class.is_none() {
                if let Some(super::symbol_table::Class::Function) | Some(super::symbol_table::Class::Sys) = saved_class {
                    self.current_class = saved_class.clone();
                }
            }
        }
        
        // Final check - if we still don't have a class but saved_class was a function/sys, restore it
        if self.current_class.is_none() {
            if let Some(super::symbol_table::Class::Function) | Some(super::symbol_table::Class::Sys) = saved_class {
                self.current_class = saved_class;
            }
        }
        
        Ok(())
    }
    
    // Parse primary expressions (literals, identifiers, parenthesized expressions)
    fn parse_primary_expr(&mut self, stop_tokens: Option<&[Token]>) -> Result<(), String> {
        println!("DEBUG: Entering parse_primary_expr, current token: {:?}", self.lexer.peek_token());

        // --- new: if this token is one of our stops (e.g. ';'), just return ---
        if let Some(stops) = stop_tokens {
            if let Some(tok) = self.lexer.peek_token() {
                if stops.iter().any(|t| t == &tok) {
                    println!("DEBUG: parse_primary_expr saw stop token: {:?}, ending expr", tok);
                    return Ok(());
                }
            }
        }
        
        if let Some(token) = self.lexer.peek_token() {
            match token.clone() {
                // Handle type tokens during the second pass
                Token::Int | Token::CharType => {
                    // Skip type tokens during expression parsing
                    // This happens during the second pass when we're generating code
                    self.lexer.next_token();
                    // Skip the identifier and any initialization
                    while let Some(t) = self.lexer.peek_token() {
                        if t == Token::Semi {
                            self.lexer.next_token(); // Consume semicolon
                            break;
                        }
                        self.lexer.next_token();
                    }
                    return Ok(());
                },
                // Numeric literal
                Token::Num(val) => {
                    println!("DEBUG: Found numeric literal: {}", val);
                    self.current_value = val;
                    self.current_type = Some(Type::Int);
                    self.lexer.next_token();
                    return Ok(());
                }
                
                // Character literal
                Token::Char(c) => {
                    println!("DEBUG: Found character literal: {}", c);
                    self.current_value = c as i64;
                    self.current_type = Some(Type::Char);
                    self.lexer.next_token();
                    return Ok(());
                }
                
                // String literal
                Token::Str(s) => {
                    println!("DEBUG: Found string literal: {}", s);
                    // Add the string to the data section and set the current value to its index
                    self.current_value = self.add_string(&s) as i64;
                    self.current_type = Some(Type::Ptr(Box::new(Type::Char)));
                    self.lexer.next_token();
                    return Ok(());
                }
                
                // Identifier
                Token::Id(id) => {
                    println!("DEBUG: Found identifier: {}", id);
                    // Look up the identifier in the symbol table
                    if let Some(symbol) = self.symbol_table.lookup(&id) {
                        self.current_id = Some(id.clone());
                        self.current_class = Some(symbol.class.clone());
                        self.current_type = Some(symbol.typ.clone());
                        self.current_value = symbol.val;
                        self.lexer.next_token();
                        let func_class = self.current_class.clone();
                        let func_id = self.current_id.clone();
                        // Check for function call or array indexing
                        let mut did_call = false;
                        if let Some(Token::OpenParen) = self.lexer.peek_token() {
                            self.parse_function_call()?;
                            // Always restore class/id after parsing arguments
                            self.current_class = func_class.clone();
                            self.current_id = func_id.clone();
                            // Explicitly set class to function's class after parsing call
                            self.current_class = func_class;
                        }
                        // Handle postfix operators (e.g., array indexing) after restoration
                        if let Some(Token::Brak) = self.lexer.peek_token() {
                            self.parse_postfix_operators(stop_tokens)?;
                        }
                        return Ok(());
                    } else {
                        return Err(format!("Undefined identifier: {}", id));
                    }
                }
                
                // System function calls
                Token::Printf | Token::Open | Token::Read | Token::Close | 
                Token::Malloc | Token::Free | Token::Memset | Token::Memcmp | Token::Exit => {
                    // Get the function name from the token
                    let func_name = match &token {
                        Token::Printf => "printf",
                        Token::Open => "open",
                        Token::Read => "read",
                        Token::Close => "close",
                        Token::Malloc => "malloc",
                        Token::Free => "free",
                        Token::Memset => "memset",
                        Token::Memcmp => "memcmp",
                        Token::Exit => "exit",
                        _ => unreachable!(),
                    };
                    
                    println!("DEBUG: Found system function: {}", func_name);
                    
                    // Look up the system function in the symbol table
                    if let Some(symbol) = self.symbol_table.lookup(func_name) {
                        self.current_id = Some(func_name.to_string());
                        self.current_class = Some(symbol.class.clone());
                        self.current_type = Some(symbol.typ.clone());
                        self.current_value = symbol.val;
                        self.lexer.next_token();
                        let func_class = self.current_class.clone();
                        let func_id = self.current_id.clone();
                        if let Some(Token::OpenParen) = self.lexer.peek_token() {
                            self.parse_function_call()?;
                            // Always restore class/id after parsing arguments
                            self.current_class = func_class.clone();
                            self.current_id = func_id.clone();
                            // Explicitly set class to function's class after parsing call
                            self.current_class = func_class;
                        } else {
                            return Err(format!("Expected '(' after system function: {}", func_name));
                        }
                        // Handle postfix operators (e.g., array indexing) after restoration
                        if let Some(Token::Brak) = self.lexer.peek_token() {
                            self.parse_postfix_operators(stop_tokens)?;
                        }
                        return Ok(());
                    } else {
                        return Err(format!("System function not found in symbol table: {}", func_name));
                    }
                }
                
                // sizeof operator
                Token::Sizeof => {
                    self.lexer.next_token();
                    // Check if the next token is an open parenthesis
                    if let Some(Token::OpenParen) = self.lexer.peek_token() {
                        self.lexer.next_token();
                        // Parse the type or expression inside sizeof
                        if matches!(self.lexer.peek_token(), Some(Token::Int) | Some(Token::CharType)) {
                            // sizeof a type
                            self.parse_type()?; // Using the public method from declaration.rs
                            // Set the result to the size of the type
                            if let Some(typ) = &self.current_type {
                                self.current_value = typ.size() as i64;
                                self.current_type = Some(Type::Int);
                            }
                        } else {
                            // sizeof an expression
                            self.parse_expr_with_precedence(Precedence::Assignment, stop_tokens)?;
                            // Set the result to the size of the expression's type
                            if let Some(typ) = &self.current_type {
                                self.current_value = typ.size() as i64;
                                self.current_type = Some(Type::Int);
                            }
                        }
                        // Expect closing parenthesis
                        if let Some(Token::CloseParen) = self.lexer.peek_token() {
                            self.lexer.next_token();
                        } else {
                            return Err("Expected ')' after sizeof expression".to_string());
                        }
                    } else {
                        return Err("Expected '(' after sizeof".to_string());
                    }
                    return Ok(());
                }
                
                // Parenthesized expression
                Token::OpenParen => {
                    self.lexer.next_token();
                    // If next token is ')', treat as empty parenthesized expression
                    if let Some(Token::CloseParen) = self.lexer.peek_token() {
                        self.lexer.next_token();
                        self.current_value = 0;
                        self.current_type = Some(Type::Int);
                        return Ok(());
                    }
                    
                    // Parse the expression inside parentheses
                    self.parse_expr_with_precedence(Precedence::Assignment, Some(&[Token::CloseParen]))?;
                    
                    // Expect closing parenthesis
                    if let Some(Token::CloseParen) = self.lexer.peek_token() {
                        self.lexer.next_token();
                    } else {
                        return Err("Expected ')' after expression".to_string());
                    }
                    
                    return Ok(());
                }
                
                // Unary operators
                Token::Add | Token::Sub | Token::Mul | Token::And => {
                    let op = token.clone();
                    self.lexer.next_token();
                    // Parse the operand with unary precedence
                    self.parse_expr_with_precedence(Precedence::Unary, stop_tokens)?;
                    
                    // Handle the unary operator
                    match op {
                        Token::Add => {
                            // Unary + is a no-op
                        }
                        Token::Sub => {
                            // Negate the result
                            self.current_value = -self.current_value;
                        }
                        Token::Mul => {
                            // Dereference a pointer
                            if let Some(Type::Ptr(base_type)) = &self.current_type {
                                self.current_type = Some(*base_type.clone());
                            } else {
                                return Err("Cannot dereference non-pointer type".to_string());
                            }
                        }
                        Token::And => {
                            // Take the address of a variable
                            if let Some(typ) = &self.current_type {
                                self.current_type = Some(Type::Ptr(Box::new(typ.clone())));
                            }
                        }
                        _ => unreachable!(),
                    }
                    return Ok(());
                }
                
                _ => {
                    // Unknown token in expression
                    println!("DEBUG: [parse_primary_expr] current_class at end: {:?}", self.current_class);
                    return Err(format!("Unexpected token in expression: {:?}", token));
                }
            }
        } else {
            return Err("Unexpected end of input in expression".to_string());
        }
    }

    // Parse postfix operators (++, --, [])
    fn parse_postfix_operators(&mut self, stop_tokens: Option<&[Token]>) -> Result<(), String> {
        while let Some(token) = self.lexer.peek_token() {
            match token {
                Token::Inc => {
                    self.lexer.next_token();
                    // Handle post-increment
                    // For now, just note that we've seen it
                }
                Token::Dec => {
                    self.lexer.next_token();
                    // Handle post-decrement
                    // For now, just note that we've seen it
                }
                Token::Brak => {
                    self.lexer.next_token();
                    // Parse the index expression
                    self.parse_expr_with_precedence(Precedence::Assignment, stop_tokens)?;
                    // Expect closing bracket
                    if let Some(Token::Unknown(b']')) = self.lexer.peek_token() {
                        self.lexer.next_token();
                    } else {
                        return Err("Expected ']' after array index".to_string());
                    }
                    // Handle array indexing
                    if let Some(Type::Ptr(base_type)) = self.current_type.clone() {
                        self.current_type = Some(*base_type);
                    } else {
                        return Err("Cannot index non-pointer type".to_string());
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    // Parse function call
    pub fn parse_function_call(&mut self) -> Result<(), String> {
        println!("DEBUG: Parsing function call");
        self.lexer.next_token(); // consume '('
        let mut arg_count = 0;
        
        // Special-case empty argument list
        if let Some(Token::CloseParen) = self.lexer.peek_token() {
            println!("DEBUG: No arguments in function call (empty argument list)");
            self.lexer.next_token(); // consume ')'
        } else {
            // Parse comma-separated arguments
            loop {
                // Parse the full expression for this argument
                self.parse_expr_with_precedence(Precedence::Assignment, Some(&[Token::Comma, Token::CloseParen]))?;
                arg_count += 1;
                
                match self.lexer.peek_token() {
                    Some(Token::Comma) => {
                        println!("DEBUG: Found comma, consuming and continuing");
                        self.lexer.next_token(); // consume ','
                    },
                    Some(Token::CloseParen) => {
                        println!("DEBUG: Found closing parenthesis, end of arguments");
                        self.lexer.next_token(); // consume ')'
                        break;
                    },
                    other => {
                        println!("DEBUG: Expected ',' or ')' but found: {:?}", other);
                        return Err(format!("Expected ',' or ')' in function call, found: {:?}", other));
                    }
                }
            }
        }
        
        // Update both argument count trackers
        self.current_value = arg_count;
        self.arg_count = arg_count as usize;
        println!("DEBUG: [parse_function_call] parsed {} args, class at end: {:?}", arg_count, self.current_class);
        Ok(())
    }
    
    // Get the precedence of a token
    fn get_token_precedence(&self, token: &Token) -> Precedence {
        match token {
            Token::Assign => Precedence::Assignment,
            Token::Cond => Precedence::Conditional,
            Token::Lor => Precedence::LogicalOr,
            Token::Lan => Precedence::LogicalAnd,
            Token::Or => Precedence::BitwiseOr,
            Token::Xor => Precedence::BitwiseXor,
            Token::And => Precedence::BitwiseAnd,
            Token::Eq | Token::Ne => Precedence::Equality,
            Token::Lt | Token::Gt | Token::Le | Token::Ge => Precedence::Relational,
            Token::Shl | Token::Shr => Precedence::Shift,
            Token::Add | Token::Sub => Precedence::Additive,
            Token::Mul | Token::Div | Token::Mod => Precedence::Multiplicative,
            Token::Inc | Token::Dec | Token::OpenParen | Token::Brak => Precedence::Postfix,
            _ => Precedence::Primary,
        }
    }
}