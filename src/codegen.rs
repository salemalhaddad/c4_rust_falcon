use crate::parser::{Parser, symbol_table::{Symbol, Class}};
// VM instruction set
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum Opcode {
    // Control flow
    LEA = 1,  // Load effective address
    IMM,      // Load immediate value
    JMP,      // Jump
    JSR,      // Jump to subroutine
    BZ,       // Branch if zero
    BNZ,      // Branch if not zero
    ENT,      // Enter function
    ADJ,      // Adjust stack
    LEV,      // Leave function
    
    // Memory access
    LI,       // Load int
    LC,       // Load char
    SI,       // Store int
    SC,       // Store char
    PSH,      // Push value onto stack
    
    // Arithmetic and logic
    OR,       // Bitwise OR
    XOR,      // Bitwise XOR
    AND,      // Bitwise AND
    EQ,       // Equal
    NE,       // Not equal
    LT,       // Less than
    GT,       // Greater than
    LE,       // Less than or equal
    GE,       // Greater than or equal
    SHL,      // Shift left
    SHR,      // Shift right
    ADD,      // Add
    SUB,      // Subtract
    MUL,      // Multiply
    DIV,      // Divide
    MOD,      // Modulo
    
    // System calls
    OPEN,     // Open file
    READ,     // Read from file
    CLOS,     // Close file
    PRTF,     // Printf
    MALC,     // Malloc
    FREE,     // Free
    MSET,     // Memset
    MCMP,     // Memcmp
    EXIT,     // Exit
}
pub struct CodeGenerator {
    pub text: Vec<i32>,        // Code segment
    pub data: Vec<u8>,         // Data segment
    pub text_offset: usize,    // Current offset in code segment
    pub data_offset: usize,    // Current offset in data segment
}
impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            data: Vec::new(),
            text_offset: 0,
            data_offset: 0,
        }
    }
    
    // Emit an instruction
    pub fn emit(&mut self, op: Opcode) {
        self.text.push(op as i32);
        self.text_offset += 1;
    }
    
    // Emit an instruction with an immediate value
    pub fn emit_imm(&mut self, op: Opcode, val: i32) {
        self.emit(op);
        self.text.push(val);
        self.text_offset += 1;
    }
    
    // Allocate space in the data segment
    pub fn allocate_data(&mut self, size: usize) -> usize {
        let offset = self.data_offset;
        self.data_offset += size;
        self.data.resize(self.data_offset, 0);
        offset
    }
    
    // Store a string in the data segment and return its address
    pub fn store_string(&mut self, s: &str) -> usize {
        let addr = self.data.len();
        
        println!("DEBUG: Storing string '{}' at address {}", s, addr);
        println!("DEBUG: String bytes: {:?}", s.as_bytes());
        
        // Add the string to the data segment
        for byte in s.as_bytes() {
            self.data.push(*byte);
        }
        
        // Add null terminator
        self.data.push(0);
        
        println!("DEBUG: Data segment size after storing string: {}", self.data.len());
        println!("DEBUG: First 10 bytes of data segment: {:?}", &self.data[0..std::cmp::min(10, self.data.len())]);
        
        addr
    }
    
    // Generate code for a function
    pub fn gen_function(&mut self, parser: &mut Parser, _symbol: &Symbol) -> Result<(), String> {
        // Record the function's entry point
        let entry_point = self.text_offset;
        
        // Emit ENT and reserve its slot for locals in one go
        self.emit_imm(Opcode::ENT, 0); // Placeholder for local variable space
        
        // Parse function body
        parser.parse_compound_statement()?;
        
        // Emit function epilogue
        self.emit(Opcode::LEV);
        
        // Update the local variable space
        self.text[entry_point + 1] = parser.local_offset as i32;
        
        Ok(())
    }
    
    // Generate code for an expression
    pub fn gen_expression(&mut self, parser: &mut Parser) -> Result<(), String> {
        println!("CODEGEN DEBUG: Entering gen_expression, current token: {:?}", parser.lexer.peek_token());
        parser.parse_expression()?;
        println!("DEBUG: Expression result: value={}, class={:?}", parser.current_value, parser.current_class);
        println!("CODEGEN DEBUG: Before codegen match, current_class={:?}, current_id={:?}", parser.current_class, parser.current_id);
        match parser.current_class {
            Some(crate::parser::symbol_table::Class::Global) => {
                // Global variable
                self.emit_imm(Opcode::IMM, parser.current_value as i32);
                self.emit(Opcode::LI);
                self.emit(Opcode::PSH);
            },
            Some(crate::parser::symbol_table::Class::Local) => {
                // Local variable
                self.emit_imm(Opcode::LEA, parser.current_value as i32);
                self.emit(Opcode::LI);
                self.emit(Opcode::PSH);
            },
            Some(crate::parser::symbol_table::Class::Function) => {
                // Function call: arguments already pushed on stack
                // Look up the *true* entry point for this function
                let func_name = parser.current_id.as_ref().expect("current_id should be set for Function");
                let sym = parser.symbol_table.lookup(func_name).ok_or_else(|| format!("Unknown function `{}`", func_name))?;
                // Emit the real address
                self.emit_imm(Opcode::IMM, sym.val as i32);
                self.emit(Opcode::JSR);
                // Now pop the arguments off the stack:
                let arg_count = parser.arg_count as i32;
                if arg_count > 0 {
                    self.emit_imm(Opcode::ADJ, arg_count);
                }
            },
            Some(crate::parser::symbol_table::Class::Sys) => {
                // System call: arguments must already be pushed on stack (by expression parser)
                match parser.current_id.as_deref() {
                    Some("printf") => self.emit(Opcode::PRTF),
                    Some("open") => self.emit(Opcode::OPEN),
                    Some("read") => self.emit(Opcode::READ),
                    Some("close") => self.emit(Opcode::CLOS),
                    Some("malloc") => self.emit(Opcode::MALC),
                    Some("free") => self.emit(Opcode::FREE),
                    Some("memset") => self.emit(Opcode::MSET),
                    Some("memcmp") => self.emit(Opcode::MCMP),
                    Some("exit") => self.emit(Opcode::EXIT),
                    _ => println!("DEBUG: Unknown system function: {:?}", parser.current_id),
                }
                let arg_count = parser.arg_count as i32;
                if arg_count > 0 {
                    self.emit_imm(Opcode::ADJ, arg_count);
                }
            },
            None => {
                // Literal or result
                self.emit_imm(Opcode::IMM, parser.current_value as i32);
                self.emit(Opcode::PSH);
            },
        }

        Ok(())
    }
    
    // Generate code for a statement
    pub fn gen_statement(&mut self, parser: &mut Parser) -> Result<(), String> {
        println!("DEBUG: [gen_statement] Entered gen_statement, current token: {:?}", parser.lexer.peek_token());
        let result = match parser.lexer.peek_token() {
            Some(crate::lexer::Token::If) => {
                println!("DEBUG: [gen_statement] Detected IF statement");
                self.gen_if_statement(parser)
            },
            Some(crate::lexer::Token::While) => {
                println!("DEBUG: [gen_statement] Detected WHILE statement");
                self.gen_while_statement(parser)
            },
            Some(crate::lexer::Token::Return) => {
                println!("DEBUG: [gen_statement] Detected RETURN statement");
                self.gen_return_statement(parser)
            },
            Some(crate::lexer::Token::OpenBrace) => {
                println!("DEBUG: [gen_statement] Detected COMPOUND statement");
                self.gen_compound_statement(parser)
            },
            _ => {
                println!("DEBUG: [gen_statement] Detected EXPRESSION statement");
                self.gen_expression_statement(parser)
            },
        };
        println!("DEBUG: [gen_statement] Exiting gen_statement, current token: {:?}", parser.lexer.peek_token());
        result
    }
    
    // Generate code for if statement
    fn gen_if_statement(&mut self, parser: &mut Parser) -> Result<(), String> {
        // Consume 'if'
        parser.lexer.next_token();
        
        // Expect '('
        if let Some(crate::lexer::Token::OpenParen) = parser.lexer.peek_token() {
            parser.lexer.next_token();
        } else {
            return Err("Expected '(' after 'if'".to_string());
        }
        
        // Generate code for condition
        self.gen_expression(parser)?;
        
        // Expect ')'
        if let Some(crate::lexer::Token::CloseParen) = parser.lexer.peek_token() {
            parser.lexer.next_token();
        } else {
            return Err("Expected ')' after if condition".to_string());
        }
        
        // Emit branch if zero (condition is false)
        self.emit(Opcode::BZ);
        let else_jump = self.text_offset;
        self.emit_imm(Opcode::IMM, 0); // Placeholder for else jump address
        
        // Generate code for then-branch
        self.gen_statement(parser)?;
        
        // Check for else-branch
        if let Some(crate::lexer::Token::Else) = parser.lexer.peek_token() {
            parser.lexer.next_token();
            
            // Emit jump to skip else-branch
            self.emit(Opcode::JMP);
            let end_jump = self.text_offset;
            self.emit_imm(Opcode::IMM, 0); // Placeholder for end jump address
            
            // Update else jump address
            self.text[else_jump] = self.text_offset as i32;
            
            // Generate code for else-branch
            self.gen_statement(parser)?;
            
            // Update end jump address
            self.text[end_jump] = self.text_offset as i32;
        } else {
            // No else-branch, update else jump address to current position
            self.text[else_jump] = self.text_offset as i32;
        }
        
        Ok(())
    }
    
    // Generate code for while statement
    fn gen_while_statement(&mut self, parser: &mut Parser) -> Result<(), String> {
        // Consume 'while'
        parser.lexer.next_token();
        
        // Record start of loop for condition
        let loop_start = self.text_offset;
        
        // Expect '('
        if let Some(crate::lexer::Token::OpenParen) = parser.lexer.peek_token() {
            parser.lexer.next_token();
        } else {
            return Err("Expected '(' after 'while'".to_string());
        }
        
        // Generate code for condition
        self.gen_expression(parser)?;
        
        // Expect ')'
        if let Some(crate::lexer::Token::CloseParen) = parser.lexer.peek_token() {
            parser.lexer.next_token();
        } else {
            return Err("Expected ')' after while condition".to_string());
        }
        
        // Emit branch if zero (condition is false)
        self.emit(Opcode::BZ);
        let end_jump = self.text_offset;
        self.emit_imm(Opcode::IMM, 0); // Placeholder for end jump address
        
        // Generate code for loop body
        self.gen_statement(parser)?;
        
        // Emit jump back to condition
        self.emit_imm(Opcode::JMP, loop_start as i32);
        
        // Update end jump address
        self.text[end_jump] = self.text_offset as i32;
        
        Ok(())
    }
    fn gen_return_statement(&mut self, parser: &mut Parser) -> Result<(), String> {
        println!("CODEGEN DEBUG: Entering gen_return_statement, current token: {:?}", parser.lexer.peek_token());
        // Consume 'return'
        parser.lexer.next_token();
        println!("CODEGEN DEBUG: After consuming 'return', current token: {:?}", parser.lexer.peek_token());
        // Always emit IMM for the return value (default 0 if none)
        if parser.lexer.peek_token() != Some(crate::lexer::Token::Semi) {
            println!("CODEGEN DEBUG: Generating code for return expression");
            self.gen_expression(parser)?;
        } else {
            println!("CODEGEN DEBUG: No return value, emitting IMM 0");
            self.emit_imm(Opcode::IMM, 0);
        }
        println!("CODEGEN DEBUG: After generating return expression, current token: {:?}", parser.lexer.peek_token());
        // Expect ';'
        if let Some(crate::lexer::Token::Semi) = parser.lexer.peek_token() {
            println!("CODEGEN DEBUG: Found semicolon after return, consuming it");
            parser.lexer.next_token();
        } else {
            println!("CODEGEN DEBUG: Expected semicolon after return but found: {:?}", parser.lexer.peek_token());
            return Err("Expected ';' after return statement".to_string());
        }
        println!("CODEGEN DEBUG: Emitting LEV for function epilogue");
        self.emit(Opcode::LEV);
        println!("CODEGEN DEBUG: Exiting gen_return_statement");
        Ok(())
    }
    pub fn gen_compound_statement(&mut self, parser: &mut Parser) -> Result<(), String> {
        let entry_token = parser.lexer.peek_token();
        println!("DEBUG: [gen_compound_statement] ENTER: token = {:?}", entry_token);
        if entry_token != Some(crate::lexer::Token::OpenBrace) {
            println!("ERROR: gen_compound_statement called but token is not OpenBrace! Token: {:?}", entry_token);
            return Err(format!("Expected '{{' at start of compound statement, got {:?}", entry_token));
        }
        parser.lexer.next_token(); // Consume '{'
        println!("DEBUG: Entering gen_compound_statement");
        // Consume '{'
        parser.lexer.next_token();
        // Enter a new scope
        parser.symbol_table.enter_scope();
        println!("DEBUG: Entered a new scope in gen_compound_statement");
        // Reset local offset for this scope
        let saved_local_offset = parser.local_offset;
        parser.local_offset = 0;
        // Generate code for declarations and statements
        let mut stmt_count = 0;
        while let Some(token) = parser.lexer.peek_token() {
            println!("DEBUG: [gen_compound_statement] Statement #{}: token BEFORE = {:?}", stmt_count, token);
            if token == crate::lexer::Token::CloseBrace {
                break;
            }
            // Local variable declaration
            if matches!(token, crate::lexer::Token::Int | crate::lexer::Token::CharType) {
                println!("DEBUG: Found local variable declaration in gen_compound_statement");
                parser.parse_local_declaration()?;
                if let Some(ref var_name) = parser.current_id {
                    println!("DEBUG: Processed local variable '{}' with offset {}", var_name, parser.local_offset - parser.current_type.as_ref().unwrap().size());
                    let size = parser.current_type.as_ref().unwrap().size() as i32;
                    self.emit_imm(Opcode::ADJ, -size);
                    println!("DEBUG: Allocated {} bytes on stack for local variable '{}'", size, var_name);
                    if parser.current_value != 0 {
                        println!("DEBUG: Initializing local variable '{}' with value {}", var_name, parser.current_value);
                        self.emit_imm(Opcode::IMM, parser.current_value as i32);
                        self.emit(Opcode::SI);
                    }
                }
            } else {
                println!("DEBUG: [gen_compound_statement] Entering gen_statement for statement #{}", stmt_count);
                self.gen_statement(parser)?;
                println!("DEBUG: [gen_compound_statement] Exited gen_statement for statement #{}", stmt_count);
            }
            let after_token = parser.lexer.peek_token();
            println!("DEBUG: [gen_compound_statement] Statement #{}: token AFTER = {:?}", stmt_count, after_token);
            stmt_count += 1;
        }
        parser.local_offset = saved_local_offset;
        parser.symbol_table.exit_scope();
        println!("DEBUG: Exited scope in gen_compound_statement");
        if let Some(crate::lexer::Token::CloseBrace) = parser.lexer.peek_token() {
            parser.lexer.next_token();
            println!("DEBUG: Consumed closing brace in gen_compound_statement");
            Ok(())
        } else {
            Err("Expected '}' at end of compound statement".to_string())
        }
    }
    fn gen_expression_statement(&mut self, parser: &mut Parser) -> Result<(), String> {
        println!("CODEGEN DEBUG: Entering gen_expression_statement");
        
        // Empty statement (just a semicolon)
        if let Some(crate::lexer::Token::Semi) = parser.lexer.peek_token() {
            parser.lexer.next_token();
            return Ok(());
        }
        
        // Generate code for the full expression
        self.gen_expression(parser)?;
        
        // Expect ';'
        if let Some(crate::lexer::Token::Semi) = parser.lexer.peek_token() {
            parser.lexer.next_token();
            // Only ADJ if not a function or system call (result unused)
            match parser.current_class {
                Some(crate::parser::symbol_table::Class::Function) | Some(crate::parser::symbol_table::Class::Sys) => {},
                _ => self.emit_imm(Opcode::ADJ, 1),
            }
            Ok(())
        } else {
            Err("Expected ';' after expression statement".to_string())
        }
    }
}