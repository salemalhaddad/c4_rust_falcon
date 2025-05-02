pub mod symbol_table;
pub mod types;
pub mod declaration;
pub mod expression;
pub mod statement;

use crate::lexer::{Lexer, Token};
use symbol_table::{Class, SymbolTable};
use types::Type;

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub symbol_table: SymbolTable,
    pub current_id: Option<String>,
    pub current_class: Option<Class>,
    pub current_type: Option<Type>,
    pub current_value: i64,
    pub arg_count: usize,
    pub local_offset: i32,
    pub line: usize,
    pub second_pass: bool,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        let mut lexer = Lexer::new(src);
        lexer.next_token(); // Initialize with first token
        
        Self {
            lexer,
            symbol_table: SymbolTable::new(),
            current_id: None,
            current_class: None,
            current_type: None,
            current_value: 0,
            arg_count: 0,
            local_offset: 0,
            line: 1,
            second_pass: false,
        }
    }

    // Add a string to the data segment and return its address
    pub fn add_string(&mut self, s: &str) -> usize {
        // Create a code generator to store the string
        let mut code_gen = crate::codegen::CodeGenerator::new();
        let addr = code_gen.store_string(s);
        
        println!("DEBUG: Stored string '{}' at address {}", s, addr);
        addr
    }
    
    pub fn parse(&mut self) -> Result<(Vec<i32>, Vec<u8>), String> {
        // Initialize symbol table with built-in types and functions
        self.symbol_table.init_builtins();
        
        println!("DEBUG: First pass - building symbol table");
        // First pass: Parse all declarations to build the symbol table
        let mut main_symbol = None;
        
        // Store the current position to reset after first pass
        let initial_pos = self.lexer.pos;
        
        // Parse declarations at the global scope
        while let Some(token) = self.lexer.peek_token() {
            if token == Token::Eof {
                break;
            }
            
            // Parse the next global declaration
            self.parse_global_declaration()?;
            
            // Check if we found main
            if let Some(ref id) = self.current_id {
                if id == "main" && matches!(self.current_class, Some(symbol_table::Class::Function)) {
                    println!("DEBUG: Found main function in first pass");
                    // Store the main symbol for later
                    if let Some(symbol) = self.symbol_table.lookup("main") {
                        println!("DEBUG: Main symbol found in symbol table: {:?}", symbol);
                        main_symbol = Some(symbol.clone());
                    } else {
                        println!("DEBUG: Main symbol NOT found in symbol table!");
                    }
                }
            }
        }
        
        println!("DEBUG: Symbol table after first pass: {:?}", self.symbol_table);
        println!("DEBUG: All symbols after first pass:");
        for (name, symbol) in self.symbol_table.all_symbols() {
            println!("DEBUG: symbol: '{}' class: {:?}", name, symbol.class);
        }
        
        // Check if we found main after the first pass
        if main_symbol.is_none() {
            // Try to look it up directly in the symbol table
            if let Some(symbol) = self.symbol_table.lookup("main") {
                println!("DEBUG: Found main function in symbol table after first pass");
                main_symbol = Some(symbol.clone());
            }
        }
        
        // Save the symbol table state after the first pass
        let saved_symbol_table = self.symbol_table.clone();
        println!("DEBUG: All symbols before second pass:");
        for (name, symbol) in self.symbol_table.all_symbols() {
            println!("DEBUG: symbol: '{}' class: {:?}", name, symbol.class);
        }
        
        // Reset lexer position for second pass
        self.lexer.pos = initial_pos;
        self.lexer.next_token(); // Get the first token again

        // Print the first 10 tokens for debug
        println!("DEBUG: First 10 tokens after lexer reset for second pass:");
        let mut preview_pos = self.lexer.pos;
        for i in 0..10 {
            let token = self.lexer.peek_token();
            println!("DEBUG: token[{}]: {:?}", i, token);
            if token == Some(Token::Eof) { break; }
            self.lexer.next_token();
        }
        // Reset lexer again for actual codegen
        self.lexer.pos = initial_pos;
        self.lexer.next_token();
        
        println!("DEBUG: Second pass - generating code");
        // Create code generator
        let mut code_gen = crate::codegen::CodeGenerator::new();
        
        // Restore the symbol table and set second pass flag
        self.symbol_table = saved_symbol_table;
        self.second_pass = true;
        
        // Walk through all declarations
        while let Some(token) = self.lexer.peek_token() {
            if token == Token::Eof {
                break;
            }
            
            // Parse the declaration
            self.parse_global_declaration()?;
            
            // If it's a function, generate code for it
            if let Some(id) = &self.current_id {
                if let Some(Class::Function) = self.current_class {
                    println!("DEBUG: Emitting function `{}` at addr {}", id, code_gen.text_offset);
                    
                    // Get the symbol for this function and clone it
                    let sym = self.symbol_table.lookup(id)
                        .ok_or_else(|| format!("Function {} not found in symbol table", id))?
                        .clone();
                    
                    // Generate code for the function
                    code_gen.gen_function(self, &sym)?;
                }
            }
        }
        
        // Ensure program exits cleanly
        code_gen.emit(crate::codegen::Opcode::EXIT);
        
        println!("DEBUG: Generated {} instructions", code_gen.text.len());
        println!("DEBUG: Generated {} bytes of data", code_gen.data.len());
        
        // Print out the generated instructions for debugging
        println!("DEBUG: Generated instructions:");
        for (i, instr) in code_gen.text.iter().enumerate() {
            println!("DEBUG:   [{}]: {}", i, instr);
        }
        
        // Return both the code and data segments
        Ok((code_gen.text, code_gen.data))
    }
}
