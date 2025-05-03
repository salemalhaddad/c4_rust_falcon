use crate::codegen::Opcode;
use std::io::{self, Read, Write};
use std::fs::File;

pub struct VM {
    pub pc: usize,         // program counter
    pub sp: usize,         // stack pointer
    pub bp: usize,         // base pointer
    pub ax: i32,           // accumulator
    pub data: Vec<u8>,     // static/global data
    pub text: Vec<i32>,    // code segment
    pub stack: Vec<i32>,   // execution stack
    pub debug: bool,       // debug mode
}

impl VM {
    pub fn new(text: Vec<i32>, data: Vec<u8>, stack_size: usize, debug: bool) -> Self {
        // For downward-growing stack, sp and bp start at stack_size (one past last valid index)
        let sp = stack_size;
        Self {
            pc: 0,
            sp,
            bp: sp,
            ax: 0,
            data,
            text,
            stack: vec![0; stack_size],
            debug,
        }
    }
    
    // Run the virtual machine
    pub fn run(&mut self) -> Result<i32, String> {
        // Reset state
        self.pc = 0;
        self.sp = self.stack.len();
        self.bp = self.stack.len();

        if self.debug {
            println!("DEBUG: VM starting with stack size: {}", self.stack.len());
            println!("DEBUG: VM starting with sp: {}", self.sp);
            println!("DEBUG: VM starting with bp: {}", self.bp);
            println!("DEBUG: VM code size: {} instructions", self.text.len());
            println!("DEBUG: VM data size: {} bytes", self.data.len());
        }

        // Main execution loop
        while self.pc < self.text.len() {
            // 1) fetch opcode
            let inst = self.text[self.pc];
            self.pc += 1;

            if self.debug {
                println!("PC: {}, OP: {:?}", self.pc - 1, self.get_opcode(inst));
            }

            // 2) dispatch
            // Skip opcode 0 (no-op) and continue with next instruction
            if inst == 0 {
                if self.debug {
                    println!("DEBUG: Skipping opcode 0 (no-op)");
                }
                continue;
            }

            match self.get_opcode(inst) {
                // Load effective address
                Some(Opcode::LEA) => {
                    self.ax = (self.bp as i32) + self.text[self.pc];
                    self.pc += 1;
                }
                
                // Load immediate value
                Some(Opcode::IMM) => {
                    self.ax = self.text[self.pc];
                    self.pc += 1;
                }
                
                // Jump
                Some(Opcode::JMP) => {
                    self.pc = self.text[self.pc] as usize;
                }
                
                // Jump to subroutine
                Some(Opcode::JSR) => {
                    if self.sp == 0 {
                        return Err("JSR: stack overflow".to_string());
                    }
                    self.sp -= 1;
                    if self.sp >= self.stack.len() {
                        return Err(format!("JSR: stack out of bounds: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    self.stack[self.sp] = self.pc as i32 + 1;
                    if self.debug {
                        println!("DEBUG: JSR saving return address {} at sp: {}", self.pc + 1, self.sp);
                        println!("DEBUG: Stack contents around sp: {:?}", &self.stack[self.sp.saturating_sub(5)..std::cmp::min(self.sp + 5, self.stack.len())]);
                    }
                    self.pc = self.text[self.pc] as usize;
                }
                
                // Branch if zero
                Some(Opcode::BZ) => {
                    if self.ax == 0 {
                        self.pc = self.text[self.pc] as usize;
                    } else {
                        self.pc += 1;
                    }
                }
                
                // Branch if not zero
                Some(Opcode::BNZ) => {
                    if self.ax != 0 {
                        self.pc = self.text[self.pc] as usize;
                    } else {
                        self.pc += 1;
                    }
                }
                
                // Enter function
                Some(Opcode::ENT) => {
                    // Check for stack overflow before decrement
                    if self.sp == 0 {
                        return Err("ENT: stack overflow before saving bp".to_string());
                    }
                    self.sp -= 1;
                    if self.sp >= self.stack.len() {
                        return Err(format!("ENT: stack out of bounds after decrement: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    self.stack[self.sp] = self.bp as i32;
                    if self.debug {
                        println!("DEBUG: ENT saving old bp {} at sp: {}", self.bp, self.sp);
                    }
                    self.bp = self.sp;
                    // Adjust for local variables
                    let local_count = self.text[self.pc] as usize;
                    if self.sp < local_count {
                        return Err(format!("ENT: stack underflow when allocating locals: sp={} local_count={}", self.sp, local_count));
                    }
                    self.sp -= local_count;
                    if self.debug {
                        println!("DEBUG: ENT allocated {} locals, new sp: {}", local_count, self.sp);
                        println!("DEBUG: Stack contents around sp: {:?}", &self.stack[self.sp.saturating_sub(5)..std::cmp::min(self.sp + 5, self.stack.len())]);
                    }
                    self.pc += 1;
                }
                
                // Adjust stack
                Some(Opcode::ADJ) => {
                    let adj_amount = self.text[self.pc] as usize;
                    if self.debug {
                        println!("DEBUG: ADJ adjusting sp by {} from {}", adj_amount, self.sp);
                    }
                    self.sp += adj_amount;
                    if self.debug {
                        println!("DEBUG: ADJ new sp: {}", self.sp);
                        println!("DEBUG: Stack contents around sp: {:?}", &self.stack[self.sp.saturating_sub(5)..std::cmp::min(self.sp + 5, self.stack.len())]);
                    }
                    self.pc += 1;
                }
                
                // Leave function
                Some(Opcode::LEV) => {
                    // Classic C4/C3: set sp to bp, restore bp, return ax
                    self.sp = self.bp;
                    if self.sp == self.stack.len() {
                        // Stack is empty (main return), just return ax
                        if self.debug {
                            println!("DEBUG: LEV returning from main with value: {}", self.ax);
                        }
                        return Ok(self.ax);
                    }
                    if self.sp >= self.stack.len() {
                        return Err(format!("LEV: stack out of bounds for bp: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let old_bp = self.stack[self.sp] as usize;
                    if self.debug {
                        println!("DEBUG: LEV restoring bp from {} to {}", self.bp, old_bp);
                        println!("DEBUG: Stack contents around sp: {:?}", &self.stack[self.sp.saturating_sub(5)..std::cmp::min(self.sp + 5, self.stack.len())]);
                    }
                    self.bp = old_bp;
                    self.sp += 1;
                    return Ok(self.ax);
                }
                
                // Load int
                Some(Opcode::LI) => {
                    let idx = self.ax as usize;
                    if idx >= self.stack.len() {
                        return Err(format!("LI: stack out of bounds: idx={} stack_len={}", idx, self.stack.len()));
                    }
                    self.ax = self.stack[idx];
                }
                
                // Load char
                Some(Opcode::LC) => {
                    let idx = self.ax as usize;
                    if idx >= self.data.len() {
                        return Err(format!("LC: data out of bounds: idx={} data_len={}", idx, self.data.len()));
                    }
                    self.ax = self.data[idx] as i32;
                }
                
                // Store int
                Some(Opcode::SI) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("SI: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let addr = self.stack[self.sp] as usize;
                    self.sp += 1;
                    if addr >= self.stack.len() {
                        return Err(format!("SI: stack out of bounds: addr={} stack_len={}", addr, self.stack.len()));
                    }
                    self.stack[addr] = self.ax;
                }
                
                // Store char
                Some(Opcode::SC) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("SC: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let addr = self.stack[self.sp] as usize;
                    self.sp += 1;
                    if addr >= self.data.len() {
                        return Err(format!("SC: data out of bounds: addr={} data_len={}", addr, self.data.len()));
                    }
                    self.data[addr] = self.ax as u8;
                }
                
                // Push value onto stack
                Some(Opcode::PSH) => {
                    // Decrement sp before writing (downward-growing stack)
                    if self.sp == 0 {
                        return Err("Stack overflow".to_string());
                    }
                    self.sp -= 1;
                    if self.sp >= self.stack.len() {
                        return Err(format!("PSH: stack out of bounds: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    self.stack[self.sp] = self.ax;
                    
                    if self.debug {
                        println!("DEBUG: Pushed value {} onto stack at sp: {}", self.ax, self.sp);
                        println!("DEBUG: Stack contents around sp: {:?}", &self.stack[self.sp.saturating_sub(5)..std::cmp::min(self.sp + 5, self.stack.len())]);
                    }
                }
                
                // Bitwise OR
                Some(Opcode::OR) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("OR: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax |= b;
                }
                
                // Bitwise XOR
                Some(Opcode::XOR) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("XOR: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax ^= b;
                }
                
                // Bitwise AND
                Some(Opcode::AND) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("AND: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax &= b;
                }
                
                // Equal
                Some(Opcode::EQ) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("EQ: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax = if self.ax == b { 1 } else { 0 };
                }
                
                // Not equal
                Some(Opcode::NE) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("NE: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax = if self.ax != b { 1 } else { 0 };
                }
                
                // Less than
                Some(Opcode::LT) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("LT: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax = if self.ax < b { 1 } else { 0 };
                }
                
                // Greater than
                Some(Opcode::GT) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("GT: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax = if self.ax > b { 1 } else { 0 };
                }
                
                // Less than or equal
                Some(Opcode::LE) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("LE: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax = if self.ax <= b { 1 } else { 0 };
                }
                
                // Greater than or equal
                Some(Opcode::GE) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("GE: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax = if self.ax >= b { 1 } else { 0 };
                }
                
                // Shift left
                Some(Opcode::SHL) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("SHL: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax <<= b;
                }
                
                // Shift right
                Some(Opcode::SHR) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("SHR: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax >>= b;
                }
                
                // Add
                Some(Opcode::ADD) => {
                    if self.sp >= self.stack.len() {
                        if self.debug {
                            println!("DEBUG: ADD opcode with no value on stack, keeping ax={}", self.ax);
                        }
                    } else {
                        let b = self.stack[self.sp] as i32;
                        self.sp += 1;
                        if self.debug {
                            println!("DEBUG: ADD: {} + {} = {}", self.ax, b, self.ax + b);
                        }
                        self.ax += b;
                    }
                }
                
                // Subtract
                Some(Opcode::SUB) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("SUB: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax -= b;
                }
                
                // Multiply
                Some(Opcode::MUL) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("MUL: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    self.ax *= b;
                }
                
                // Divide
                Some(Opcode::DIV) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("DIV: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    if b == 0 {
                        return Err("Division by zero".to_string());
                    }
                    self.ax /= b;
                }
                
                // Modulo
                Some(Opcode::MOD) => {
                    if self.sp >= self.stack.len() {
                        return Err(format!("MOD: stack underflow: sp={} stack_len={}", self.sp, self.stack.len()));
                    }
                    let b = self.stack[self.sp] as i32;
                    self.sp += 1;
                    if b == 0 {
                        return Err("Modulo by zero".to_string());
                    }
                    self.ax %= b;
                }
                
                // System calls
                Some(Opcode::OPEN) => self.sys_open()?,
                Some(Opcode::READ) => self.sys_read()?,
                Some(Opcode::CLOS) => self.sys_close()?,
                Some(Opcode::PRTF) => self.sys_printf()?,
                Some(Opcode::MALC) => self.sys_malloc()?,
                Some(Opcode::FREE) => self.sys_free()?,
                Some(Opcode::MSET) => self.sys_memset()?,
                Some(Opcode::MCMP) => self.sys_memcmp()?,
                
                // Exit
                Some(Opcode::EXIT) => {
                    // For EXIT, we return the current value in the accumulator (ax)
                    if self.debug {
                        println!("DEBUG: EXIT with accumulator value {}", self.ax);
                    }
                    return Ok(self.ax);
                }
                
                None => {
                    return Err(format!("Unknown opcode: {}", inst));
                }
            }
        }
        
       
        Ok(self.ax)
    }
    
    // Convert i32 to Opcode
    fn get_opcode(&self, op: i32) -> Option<Opcode> {
        if op == 0 {
            if self.debug {
                println!("DEBUG: Found opcode 0, treating as padding");
            }
            return None;
        }
        
        match op {
            op if op == Opcode::LEA as i32 => Some(Opcode::LEA),
            op if op == Opcode::IMM as i32 => Some(Opcode::IMM),
            op if op == Opcode::JMP as i32 => Some(Opcode::JMP),
            op if op == Opcode::JSR as i32 => Some(Opcode::JSR),
            op if op == Opcode::BZ as i32 => Some(Opcode::BZ),
            op if op == Opcode::BNZ as i32 => Some(Opcode::BNZ),
            op if op == Opcode::ENT as i32 => Some(Opcode::ENT),
            op if op == Opcode::ADJ as i32 => Some(Opcode::ADJ),
            op if op == Opcode::LEV as i32 => Some(Opcode::LEV),
            op if op == Opcode::LI as i32 => Some(Opcode::LI),
            op if op == Opcode::LC as i32 => Some(Opcode::LC),
            op if op == Opcode::SI as i32 => Some(Opcode::SI),
            op if op == Opcode::SC as i32 => Some(Opcode::SC),
            op if op == Opcode::PSH as i32 => Some(Opcode::PSH),
            op if op == Opcode::OR as i32 => Some(Opcode::OR),
            op if op == Opcode::XOR as i32 => Some(Opcode::XOR),
            op if op == Opcode::AND as i32 => Some(Opcode::AND),
            op if op == Opcode::EQ as i32 => Some(Opcode::EQ),
            op if op == Opcode::NE as i32 => Some(Opcode::NE),
            op if op == Opcode::LT as i32 => Some(Opcode::LT),
            op if op == Opcode::GT as i32 => Some(Opcode::GT),
            op if op == Opcode::LE as i32 => Some(Opcode::LE),
            op if op == Opcode::GE as i32 => Some(Opcode::GE),
            op if op == Opcode::SHL as i32 => Some(Opcode::SHL),
            op if op == Opcode::SHR as i32 => Some(Opcode::SHR),
            op if op == Opcode::ADD as i32 => Some(Opcode::ADD),
            op if op == Opcode::SUB as i32 => Some(Opcode::SUB),
            op if op == Opcode::MUL as i32 => Some(Opcode::MUL),
            op if op == Opcode::DIV as i32 => Some(Opcode::DIV),
            op if op == Opcode::MOD as i32 => Some(Opcode::MOD),
            op if op == Opcode::OPEN as i32 => Some(Opcode::OPEN),
            op if op == Opcode::READ as i32 => Some(Opcode::READ),
            op if op == Opcode::CLOS as i32 => Some(Opcode::CLOS),
            op if op == Opcode::PRTF as i32 => Some(Opcode::PRTF),
            op if op == Opcode::MALC as i32 => Some(Opcode::MALC),
            op if op == Opcode::FREE as i32 => Some(Opcode::FREE),
            op if op == Opcode::MSET as i32 => Some(Opcode::MSET),
            op if op == Opcode::MCMP as i32 => Some(Opcode::MCMP),
            op if op == Opcode::EXIT as i32 => Some(Opcode::EXIT),
            _ => None,
        }
    }
    
    // System call implementations
    fn sys_open(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_open called but not implemented");
        }
        Ok(())
    }
    
    fn sys_read(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_read called but not implemented");
        }
        Ok(())
    }
    
    fn sys_close(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_close called but not implemented");
        }
        Ok(())
    }
    
    fn sys_printf(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_printf called but not implemented");
        }
        Ok(())
    }
    
    fn sys_malloc(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_malloc called but not implemented");
        }
        Ok(())
    }
    
    fn sys_free(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_free called but not implemented");
        }
        Ok(())
    }
    
    fn sys_memset(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_memset called but not implemented");
        }
        Ok(())
    }
    
    fn sys_memcmp(&mut self) -> Result<(), String> {
        // Not implemented for simplicity
        if self.debug {
            println!("DEBUG: sys_memcmp called but not implemented");
        }
        Ok(())
    }
}
