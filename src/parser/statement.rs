use crate::lexer::Token;
use super::{Parser, symbol_table::{Symbol, Class}};

impl<'a> Parser<'a> {
    // Parse a statement
    pub fn parse_statement(&mut self) -> Result<(), String> {
        if let Some(token) = self.lexer.peek_token() {
            match token {
                Token::Int | Token::CharType => {
                    if self.second_pass {
                        // Skip type
                        self.lexer.next_token();

                        // Skip identifier
                        if let Some(Token::Id(_)) = self.lexer.peek_token() {
                            self.lexer.next_token();

                            // Skip initialization if present
                            if let Some(Token::Assign) = self.lexer.peek_token() {
                                self.lexer.next_token(); // Skip =
                                if let Some(Token::Num(_)) = self.lexer.peek_token() {
                                    self.lexer.next_token(); // Skip number
                                }
                            }

                            // Skip semicolon
                            if let Some(Token::Semi) = self.lexer.peek_token() {
                                self.lexer.next_token();
                            }
                        }
                        Ok(())
                    } else {
                        self.parse_local_declaration()
                    }
                },
                Token::If => self.parse_if_statement(),
                Token::While => self.parse_while_statement(),
                Token::Return => self.parse_return_statement(),
                Token::OpenBrace => self.parse_compound_statement(),
                _ => self.parse_expression_statement(),
            }
        } else {
            Err("Unexpected end of input while parsing statement".to_string())
        }
    }

    // Parse if statement: if (expression) statement [else statement]
    pub fn parse_if_statement(&mut self) -> Result<(), String> {
        // Consume 'if'
        self.lexer.next_token();

        // Expect '('
        if let Some(Token::OpenParen) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected '(' after 'if'".to_string());
        }

        // Parse condition
        self.parse_expression()?;

        // Expect ')'
        if let Some(Token::CloseParen) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected ')' after if condition".to_string());
        }

        // Parse then-branch
        self.parse_statement()?;

        // Parse else-branch if present
        if let Some(Token::Else) = self.lexer.peek_token() {
            self.lexer.next_token();
            self.parse_statement()?;
        }

        Ok(())
    }

    // Parse while statement: while (expression) statement
    pub fn parse_while_statement(&mut self) -> Result<(), String> {
        // Consume 'while'
        self.lexer.next_token();

        // Expect '('
        if let Some(Token::OpenParen) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected '(' after 'while'".to_string());
        }

        // Parse condition
        self.parse_expression()?;

        // Expect ')'
        if let Some(Token::CloseParen) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected ')' after while condition".to_string());
        }

        // Parse body
        self.parse_statement()?;

        Ok(())
    }

    // Parse return statement: return [expression];
    pub fn parse_return_statement(&mut self) -> Result<(), String> {
        println!("DEBUG: Entering parse_return_statement, current token: {:?}", self.lexer.peek_token());
        // Consume 'return'
        self.lexer.next_token();
        println!("DEBUG: After consuming 'return', current token: {:?}", self.lexer.peek_token());

        // Parse return expression (if any)
        if self.lexer.peek_token() != Some(Token::Semi) {
            println!("DEBUG: Parsing return expression");
            self.current_class = None; // Reset class before parsing return expression
            // Special case for numeric literals
            if let Some(Token::Num(n)) = self.lexer.peek_token() {
                println!("DEBUG: Found numeric literal in return: {}", n);
                self.current_value = n;
                self.current_type = Some(super::types::Type::Int);
                self.lexer.next_token();
            } else {
                self.parse_expression()?;
            }
        }

        println!("DEBUG: After parsing return expression, current token: {:?}", self.lexer.peek_token());

        // Expect ';'
        if let Some(Token::Semi) = self.lexer.peek_token() {
            println!("DEBUG: Found semicolon after return, consuming it");
            self.lexer.next_token();
            println!("DEBUG: After return statement, next token: {:?}", self.lexer.peek_token());
            Ok(())
        } else {
            println!("DEBUG: Expected semicolon after return but found: {:?}", self.lexer.peek_token());
            Err(format!("Expected ';' after return statement, found: {:?}", self.lexer.peek_token()))
        }
    }

    // Parse compound statement: { [statement]* }
    pub fn parse_compound_statement(&mut self) -> Result<(), String> {
        println!("DEBUG: Entering parse_compound_statement, current token: {:?}", self.lexer.peek_token());

        // Expect '{'
        if let Some(Token::OpenBrace) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected '{' at start of compound statement".to_string());
        }

        // Enter new scope
        println!("DEBUG: Entered a new scope");
        self.symbol_table.enter_scope();

        // Parse statements
        println!("DEBUG: Parsing statements in compound statement");
        while let Some(token) = self.lexer.peek_token() {
            if token == Token::CloseBrace {
                break;
            }

            println!("DEBUG: Processing token in compound statement: {:?}", token);

            match token {
                Token::Int | Token::CharType => {
                    println!("DEBUG: Parsing local declaration");
                    if self.second_pass {
                        // Skip type
                        self.lexer.next_token();

                        // Skip identifier
                        if let Some(Token::Id(_)) = self.lexer.peek_token() {
                            self.lexer.next_token();

                            // Skip initialization if present
                            if let Some(Token::Assign) = self.lexer.peek_token() {
                                self.lexer.next_token(); // Skip =
                                if let Some(Token::Num(_)) = self.lexer.peek_token() {
                                    self.lexer.next_token(); // Skip number
                                }
                            }

                            // Skip semicolon
                            if let Some(Token::Semi) = self.lexer.peek_token() {
                                self.lexer.next_token();
                            }
                        }
                    } else {
                        self.parse_local_declaration()?;
                    }
                },
                Token::If => self.parse_if_statement()?,
                Token::While => self.parse_while_statement()?,
                Token::Return => {
                    println!("DEBUG: Parsing return statement");
                    self.parse_return_statement()?
                },
                Token::OpenBrace => self.parse_compound_statement()?,
                _ => {
                    println!("DEBUG: Parsing expression statement with token: {:?}", token);
                    self.parse_expression_statement()?
                },
            }
        }

        // Expect '}'
        if let Some(Token::CloseBrace) = self.lexer.peek_token() {
            println!("DEBUG: Found closing brace, exiting compound statement");
            self.lexer.next_token();
        } else {
            return Err("Expected '}' at end of compound statement".to_string());
        }

        // Exit scope
        println!("DEBUG: Exited scope");
        self.symbol_table.exit_scope();

        println!("DEBUG: Consumed closing brace, next token: {:?}", self.lexer.peek_token());
        Ok(())
    }

    // Parse expression statement: [expression];
    pub fn parse_expression_statement(&mut self) -> Result<(), String> {
        println!("DEBUG: Entering parse_expression_statement, current token: {:?}", self.lexer.peek_token());

        // Empty statement (just a semicolon)
        if let Some(Token::Semi) = self.lexer.peek_token() {
            println!("DEBUG: Empty statement (just a semicolon)");
            self.lexer.next_token();
            return Ok(());
        }

        // Parse expression
        println!("DEBUG: Parsing expression in statement");
        match self.parse_expression() {
            Ok(_) => {
                println!("DEBUG: After parsing expression, current token: {:?}", self.lexer.peek_token());

                // Expect ';'
                match self.lexer.peek_token() {
                    Some(Token::Semi) => {
                        println!("DEBUG: Found semicolon, consuming it");
                        self.lexer.next_token();
                        println!("DEBUG: After semicolon, next token: {:?}", self.lexer.peek_token());
                        Ok(())
                    },
                    Some(other) => {
                        println!("DEBUG: Expected semicolon but found: {:?}", other);
                        Err(format!("Expected ';' after expression statement, found: {:?}", other))
                    },
                    None => {
                        println!("DEBUG: Unexpected end of input after expression");
                        Err("Unexpected end of input after expression".to_string())
                    }
                }
            },
            Err(e) => {
                println!("DEBUG: Error parsing expression: {}", e);
                Err(e)
            }
        }
    }

    // Parse a local variable declaration
    pub fn parse_local_declaration(&mut self) -> Result<(), String> {
        println!("DEBUG: Entering parse_local_declaration, current token: {:?}", self.lexer.peek_token());

        // Parse type specifier
        self.parse_type()?; // Using the public method from declaration.rs
        println!("DEBUG: After parse_type, current token: {:?}", self.lexer.peek_token());

        // Parse declarator
        if let Some(Token::Id(id)) = self.lexer.peek_token() {
            let var_name = id.clone();
            println!("DEBUG: Found local variable name: {}", var_name);
            self.current_id = Some(var_name.clone()); // Set current_id for code generation
            self.lexer.next_token();

            // Create symbol for local variable
            let symbol = Symbol {
                name: var_name.clone(),
                class: Class::Local,
                typ: self.current_type.clone().unwrap(),
                val: 0,
                offset: self.local_offset,
            };

            // Update local offset for next variable
            self.local_offset += self.current_type.as_ref().unwrap().size();

            // Add to symbol table
            println!("DEBUG: Adding local variable '{}' to symbol table", var_name);
            self.symbol_table.add_symbol(symbol)?;

            // Handle initialization if present
            if let Some(Token::Assign) = self.lexer.peek_token() {
                println!("DEBUG: Found initialization for local variable");
                self.lexer.next_token(); // Consume '='

                // Parse initializer expression
                if let Some(Token::Num(n)) = self.lexer.peek_token() {
                    println!("DEBUG: Initializing with numeric literal: {}", n);
                    self.current_value = n;
                    self.lexer.next_token();

                    // Store the value in the symbol table
                    self.symbol_table.update_symbol(&var_name, |symbol| {
                        symbol.val = n;
                    })?;
                } else {
                    println!("DEBUG: Initializing with expression");
                    self.parse_expression()?;
                }
            }

            // Expect semicolon
            if let Some(Token::Semi) = self.lexer.peek_token() {
                println!("DEBUG: Found semicolon after local declaration");
                self.lexer.next_token();
                Ok(())
            } else {
                println!("DEBUG: Expected semicolon after local declaration but found: {:?}", self.lexer.peek_token());
                Err("Expected ';' after variable declaration".to_string())
            }
        } else {
            println!("DEBUG: Expected identifier in local declaration but found: {:?}", self.lexer.peek_token());
            Err("Expected identifier in local declaration".to_string())
        }
    }
}
