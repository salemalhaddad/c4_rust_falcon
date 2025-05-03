use crate::lexer::Token;
use super::{Parser, symbol_table::{Symbol, Class}, types::Type};

impl<'a> Parser<'a> {
    pub fn parse_global_declaration(&mut self) -> Result<(), String> {
        println!("DEBUG: Parsing global declaration, current token: {:?}", self.lexer.peek_token());

        // In the second pass, we might start with an identifier
        if let Some(Token::Id(id)) = self.lexer.peek_token() {
            println!("DEBUG: Found identifier: {}", id);
            self.current_id = Some(id.clone());
            self.lexer.next_token(); // Consume identifier

            // If this is a known function, set its type from the symbol table
            if let Some(symbol) = self.symbol_table.lookup(&id) {
                self.current_type = Some(symbol.typ.clone());
                self.current_class = Some(symbol.class.clone());

                // Function declaration/definition
                if let Some(Token::OpenParen) = self.lexer.peek_token() {
                    self.parse_function_declaration()?;
                } else {
                    // Global variable declaration
                    self.parse_global_variable()?;
                }
                return Ok(());
            }
        }

        // If we didn't find an identifier or it wasn't in the symbol table,
        // parse as a new declaration
        self.parse_type()?;
		
        println!("DEBUG: After parse_type, current token: {:?}", self.lexer.peek_token());

        // Parse declarator
        if let Some(Token::Id(id)) = self.lexer.peek_token() {
            println!("DEBUG: Found identifier: {}", id);
            self.current_id = Some(id.clone());
            self.lexer.next_token(); // Consume identifier

            // Create symbol for function or variable
            let symbol = Symbol {
                name: id.clone(),
                class: Class::Function, // We'll update this if it's not a function
                typ: self.current_type.clone().unwrap(),
                val: 0,
                offset: 0,
            };

            // Add to symbol table
            self.symbol_table.add_symbol(symbol)?;
        } else {
            println!("DEBUG: Expected identifier but found: {:?}", self.lexer.peek_token());
            return Err("Expected identifier in declaration".to_string());
        }

        // Function declaration/definition
        if let Some(Token::OpenParen) = self.lexer.peek_token() {
            self.parse_function_declaration()?;
        } else {
            // Global variable declaration
            self.parse_global_variable()?;
        }

        Ok(())
    }

    pub fn parse_type(&mut self) -> Result<(), String> {
        println!("DEBUG: Parsing type, current token: {:?}", self.lexer.peek_token());
        if let Some(token) = self.lexer.peek_token() {
            match token {
                Token::Int => {
                    println!("DEBUG: Found Int type");
                    self.current_type = Some(Type::Int);
                    self.lexer.next_token();
                }
                Token::CharType => {
                    println!("DEBUG: Found Char type");
                    self.current_type = Some(Type::Char);
                    self.lexer.next_token();
                }
                _ => {
                    println!("DEBUG: Expected type specifier but found: {:?}", token);
                    return Err(format!("Expected type specifier, found: {:?}", token));
                }
            }

            // Handle pointer types
            while let Some(Token::Mul) = self.lexer.peek_token() {
                println!("DEBUG: Found pointer type");
                if let Some(typ) = self.current_type.take() {
                    self.current_type = Some(Type::Ptr(Box::new(typ)));
                }
                self.lexer.next_token();
            }

            println!("DEBUG: Finished parsing type, current token: {:?}", self.lexer.peek_token());
            Ok(())
        } else {
            println!("DEBUG: Unexpected end of input while parsing type");
            Err("Unexpected end of input while parsing type".to_string())
        }
    }

    fn parse_global_variable(&mut self) -> Result<(), String> {
        // Create symbol for global variable
        let symbol = Symbol {
            name: self.current_id.clone().unwrap(),
            class: Class::Global,
            typ: self.current_type.clone().unwrap(),
            val: 0, // Will be set to the address in data section
            offset: 0,
        };

        // Add to symbol table
        self.symbol_table.add_symbol(symbol)?;

        // Handle initialization if present
        if let Some(Token::Assign) = self.lexer.peek_token() {
            self.lexer.next_token(); // Consume '='

            // Parse initializer expression
            // TODO: Implement expression parsing

            // For now, just skip until semicolon
            while let Some(token) = self.lexer.peek_token() {
                if token == Token::Semi {
                    break;
                }
                self.lexer.next_token();
            }
        }

        // Expect semicolon
        if let Some(Token::Semi) = self.lexer.peek_token() {
            self.lexer.next_token();
            Ok(())
        } else {
            Err("Expected ';' after variable declaration".to_string())
        }
    }

    fn parse_function_declaration(&mut self) -> Result<(), String> {
        println!("DEBUG: Parsing function declaration, current token: {:?}", self.lexer.peek_token());
        // Consume '('
        if let Some(Token::OpenParen) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected '(' in function declaration".to_string());
        }

        // Parse parameter list
        self.parse_parameter_list()?;

        // Consume ')'
        if let Some(Token::CloseParen) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected ')' after parameter list".to_string());
        }

        // Function definition (has a body)
        println!("DEBUG: Checking for function body, current token: {:?}", self.lexer.peek_token());
        if let Some(Token::OpenBrace) = self.lexer.peek_token() {
            // Enter new scope for function body
            self.symbol_table.enter_scope();

            // Reset local offset for function parameters and local variables
            self.local_offset = 0;

            // Parse statements in the function body
            println!("DEBUG: Parsing function body statements");

            // Parse the compound statement
            self.parse_compound_statement()?;

            // Exit function scope
            self.symbol_table.exit_scope();
        }
        // Function declaration (no body, just semicolon)
        else if let Some(Token::Semi) = self.lexer.peek_token() {
            self.lexer.next_token();
        } else {
            return Err("Expected '{' or ';' after function declaration".to_string());
        }

        Ok(())
    }

    fn parse_parameter_list(&mut self) -> Result<(), String> {
        // Parse parameters until we hit ')'
        while let Some(token) = self.lexer.peek_token() {
            if token == Token::CloseParen {
                break;
            }

            // Parse parameter type
            if let Some(Token::Id(id)) = self.lexer.peek_token() {
                // In second pass, just consume the identifier
                self.lexer.next_token();
            } else {
                // In first pass, parse the type
                self.parse_type()?;

                // Parse parameter name
                if let Some(Token::Id(id)) = self.lexer.peek_token() {
                    let param_name = id.clone();
                    self.lexer.next_token();

                    // Only add to symbol table in first pass
                    if !self.second_pass {
                        // Create symbol for parameter
                        let symbol = Symbol {
                            name: param_name,
                            class: Class::Local,
                            typ: self.current_type.clone().unwrap(),
                            val: 0,
                            offset: self.local_offset,
                        };

                        // Add parameter to symbol table
                        self.symbol_table.add_symbol(symbol)?;

                        // Update local offset for next parameter
                        self.local_offset += self.current_type.as_ref().unwrap().size();
                    }
                } else {
                    return Err("Expected parameter name".to_string());
                }
            }

            // Check for comma
            if let Some(Token::Comma) = self.lexer.peek_token() {
                self.lexer.next_token();
            } else if let Some(Token::CloseParen) = self.lexer.peek_token() {
                break;
            } else {
                return Err("Expected ',' or ')' in parameter list".to_string());
            }
        }

        Ok(())
    }

    // These functions are already defined above, so we don't need to redefine them
}
