use std::fs::File;
use std::io::{self, Read, Write};

use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Char,
    Int,
    Ptr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Class {
    Fun,
    Sys,
    Glo,
    Loc,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    name: String,
    class: Class,
    ty: Type,
    value: i64,
    // For local variables, we need to save previous state
    prev_class: Option<Class>,
    prev_type: Option<Type>,
    prev_value: Option<i64>,
}

// Opcodes matching C4 implementation
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    LEA,  // Load effective address
    IMM,  // Load immediate value
    JMP,  // Jump
    JSR,  // Jump to subroutine
    BZ,   // Branch if zero
    BNZ,  // Branch if not zero
    ENT,  // Enter subroutine
    ADJ,  // Adjust stack
    LEV,  // Leave subroutine
    LI,   // Load int
    LC,   // Load char
    SI,   // Store int
    SC,   // Store char
    PSH,  // Push
    OR,   // Bitwise OR
    XOR,  // Bitwise XOR
    AND,  // Bitwise AND
    EQ,   // Equal
    NE,   // Not equal
    LT,   // Less than
    GT,   // Greater than
    LE,   // Less than or equal
    GE,   // Greater than or equal
    SHL,  // Shift left
    SHR,  // Shift right
    ADD,  // Add
    SUB,  // Subtract
    MUL,  // Multiply
    DIV,  // Divide
    MOD,  // Modulo
    OPEN, // Open file
    READ, // Read file
    CLOS, // Close file
    PRTF, // Printf
    MALC, // Malloc
    FREE, // Free
    MSET, // Memset
    MCMP, // Memcmp
    EXIT  // Exit
}

pub struct VM {
    // Memory and registers
    memory: Vec<i64>,
    pc: usize,      // Program counter
    sp: usize,      // Stack pointer
    bp: usize,      // Base pointer
    ax: i64,        // Accumulator
    cycle: i64,     // Instruction cycle count
    debug: bool,    // Debug mode flag
    
    // Symbol table and data section
    symbols: HashMap<String, Symbol>,
    data: Vec<u8>,  // Data/BSS section
    data_ptr: usize,// Current position in data section
    
    // Command line arguments
    args: Vec<String>,
}

impl VM {
    pub fn new(memory_size: usize, args: Vec<String>) -> Self {
        VM {
            memory: vec![0; memory_size],
            pc: 0,
            sp: memory_size,
            bp: memory_size,
            ax: 0,
            cycle: 0,
            debug: false,
            symbols: HashMap::new(),
            data: vec![0; memory_size],
            data_ptr: 0,
            args,
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn load_program(&mut self, program: &[i64]) {
        // Load program into memory starting at address 0
        self.memory[..program.len()].copy_from_slice(program);
    }

    pub fn write_data(&mut self, addr: usize, value: i64) {
        // Write data to memory at specified address
        if addr < self.memory.len() {
            self.memory[addr] = value;
        }
    }

    pub fn add_symbol(&mut self, name: String, class: Class, ty: Type, value: i64) {
        self.symbols.insert(name.clone(), Symbol {
            name,
            class,
            ty,
            value,
            prev_class: None,
            prev_type: None,
            prev_value: None,
        });
    }

    pub fn enter_scope(&mut self) {
        // Save current state of local variables
        for symbol in self.symbols.values_mut() {
            if symbol.class == Class::Loc {
                symbol.prev_class = Some(symbol.class.clone());
                symbol.prev_type = Some(symbol.ty);
                symbol.prev_value = Some(symbol.value);
            }
        }
    }

    pub fn leave_scope(&mut self) {
        // Restore previous state of local variables
        let mut to_remove = Vec::new();
        for (name, symbol) in self.symbols.iter_mut() {
            if symbol.class == Class::Loc {
                if let (Some(class), Some(ty), Some(value)) = (
                    symbol.prev_class.take(),
                    symbol.prev_type.take(),
                    symbol.prev_value.take(),
                ) {
                    symbol.class = class;
                    symbol.ty = ty;
                    symbol.value = value;
                } else {
                    to_remove.push(name.clone());
                }
            }
        }
        // Remove local variables that didn't exist before this scope
        for name in to_remove {
            self.symbols.remove(&name);
        }
    }

    pub fn init_program(&mut self) -> io::Result<()> {
        // Setup initial stack frame
        self.sp = self.memory.len();
        self.bp = self.sp;

        // Push exit handler
        self.sp -= 1;
        self.memory[self.sp] = OpCode::EXIT as i64;

        // Push PSH instruction
        self.sp -= 1;
        self.memory[self.sp] = OpCode::PSH as i64;
        let temp_sp = self.sp;

        // Push argc
        self.sp -= 1;
        self.memory[self.sp] = self.args.len() as i64;

        // Push argv (as string pointers into data section)
        self.sp -= 1;
        let argv_ptr = self.data_ptr;
        self.memory[self.sp] = argv_ptr as i64;

        // Copy args into data section
        for arg in &self.args {
            let bytes = arg.as_bytes();
            self.data[self.data_ptr..self.data_ptr + bytes.len()].copy_from_slice(bytes);
            self.data_ptr += bytes.len();
            self.data[self.data_ptr] = 0; // Null terminator
            self.data_ptr += 1;
        }

        // Push temp_sp
        self.sp -= 1;
        self.memory[self.sp] = temp_sp as i64;

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<i64> {
        while self.pc < self.memory.len() {
            self.cycle += 1;
            
            let op = unsafe { std::mem::transmute::<u8, OpCode>(self.memory[self.pc] as u8) };
            self.pc += 1;

            if self.debug {
                println!("{:?} cycle={} ax={} sp={}", op, self.cycle, self.ax, self.sp);
            }

            match op {
                OpCode::IMM => {
                    self.ax = self.memory[self.pc];
                    self.pc += 1;
                }
                OpCode::LEA => {
                    self.ax = (self.bp as i64 + self.memory[self.pc]) as i64;
                    self.pc += 1;
                }
                OpCode::JMP => {
                    self.pc = self.ax as usize;
                }
                OpCode::JSR => {
                    self.sp -= 1;
                    self.memory[self.sp] = self.pc as i64;
                    self.pc = self.ax as usize;
                }
                OpCode::BZ => {
                    let addr = self.memory[self.pc] as usize;
                    self.pc = if self.ax == 0 { addr } else { self.pc + 1 };
                }
                OpCode::BNZ => {
                    let addr = self.memory[self.pc] as usize;
                    self.pc = if self.ax != 0 { addr } else { self.pc + 1 };
                }
                OpCode::ENT => {
                    self.sp -= 1;
                    self.memory[self.sp] = self.bp as i64;
                    self.bp = self.sp;
                    self.sp -= self.memory[self.pc] as usize;
                    self.pc += 1;
                }
                OpCode::ADJ => {
                    self.sp += self.memory[self.pc] as usize;
                    self.pc += 1;
                }
                OpCode::LEV => {
                    self.sp = self.bp;
                    self.bp = self.memory[self.sp] as usize;
                    self.sp += 1;
                    self.pc = self.memory[self.sp] as usize;
                    self.sp += 1;
                }
                OpCode::LI => {
                    self.ax = self.memory[self.ax as usize];
                }
                OpCode::LC => {
                    self.ax = self.memory[self.ax as usize] & 0xff;
                }
                OpCode::SI => {
                    let addr = self.memory[self.sp] as usize;
                    self.memory[addr] = self.ax;
                    self.sp += 1;
                }
                OpCode::SC => {
                    let addr = self.memory[self.sp] as usize;
                    self.memory[addr] = self.ax & 0xff;
                    self.sp += 1;
                }
                OpCode::PSH => {
                    self.sp -= 1;
                    self.memory[self.sp] = self.ax;
                }
                OpCode::OR  => { self.ax = self.memory[self.sp] | self.ax; self.sp += 1; }
                OpCode::XOR => { self.ax = self.memory[self.sp] ^ self.ax; self.sp += 1; }
                OpCode::AND => { self.ax = self.memory[self.sp] & self.ax; self.sp += 1; }
                OpCode::EQ  => { self.ax = (self.memory[self.sp] == self.ax) as i64; self.sp += 1; }
                OpCode::NE  => { self.ax = (self.memory[self.sp] != self.ax) as i64; self.sp += 1; }
                OpCode::LT  => { self.ax = (self.memory[self.sp] < self.ax) as i64; self.sp += 1; }
                OpCode::GT  => { self.ax = (self.memory[self.sp] > self.ax) as i64; self.sp += 1; }
                OpCode::LE  => { self.ax = (self.memory[self.sp] <= self.ax) as i64; self.sp += 1; }
                OpCode::GE  => { self.ax = (self.memory[self.sp] >= self.ax) as i64; self.sp += 1; }
                OpCode::SHL => { self.ax = self.memory[self.sp] << self.ax; self.sp += 1; }
                OpCode::SHR => { self.ax = self.memory[self.sp] >> self.ax; self.sp += 1; }
                OpCode::ADD => { self.ax = self.memory[self.sp] + self.ax; self.sp += 1; }
                OpCode::SUB => { self.ax = self.memory[self.sp] - self.ax; self.sp += 1; }
                OpCode::MUL => { self.ax = self.memory[self.sp] * self.ax; self.sp += 1; }
                OpCode::DIV => { 
                    if self.ax == 0 {
                        return Err(io::Error::new(io::ErrorKind::Other, "division by zero"));
                    }
                    self.ax = self.memory[self.sp] / self.ax; 
                    self.sp += 1; 
                }
                OpCode::MOD => {
                    if self.ax == 0 {
                        return Err(io::Error::new(io::ErrorKind::Other, "division by zero"));
                    }
                    self.ax = self.memory[self.sp] % self.ax;
                    self.sp += 1;
                }
                OpCode::EXIT => {
                    return Ok(self.memory[self.sp]);
                }
                OpCode::OPEN => {
                    let path = unsafe {
                        let ptr = self.memory[self.sp + 1] as *const i8;
                        std::ffi::CStr::from_ptr(ptr)
                    };
                    let _flags = self.memory[self.sp] as i32;
                    self.sp += 2;
                    
                    match File::open(path.to_str().unwrap()) {
                        Ok(f) => self.ax = f.as_raw_handle() as i64,
                        Err(_e) => self.ax = -1
                    }
                }
                OpCode::READ => {
                    let fd = self.memory[self.sp + 2] as i32;
                    let buf = self.memory[self.sp + 1] as *mut u8;
                    let count = self.memory[self.sp] as usize;
                    self.sp += 3;
                    
                    let mut file = unsafe { File::from_raw_handle(fd as *mut _) };
                    let mut buffer = vec![0u8; count];
                    match file.read_exact(&mut buffer) {
                        Ok(_) => {
                            unsafe {
                                std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf, count);
                            }
                            self.ax = count as i64;
                        }
                        Err(_) => self.ax = -1
                    }
                    std::mem::forget(file); // Don't close the file
                }
                OpCode::CLOS => {
                    let fd = self.memory[self.sp] as i32;
                    self.sp += 1;
                    unsafe {
                        File::from_raw_handle(fd as *mut _);
                    } // File is closed when dropped
                    self.ax = 0;
                }
                OpCode::PRTF => {
                    let args = self.memory[self.pc] as usize;
                    let fmt = unsafe {
                        let ptr = self.memory[self.sp + args - 1] as *const i8;
                        std::ffi::CStr::from_ptr(ptr).to_str().unwrap()
                    };
                    
                    // Simple printf implementation supporting %d and %s
                    let mut result = String::new();
                    let mut chars = fmt.chars();
                    let mut arg_index = args - 2;
                    
                    while let Some(c) = chars.next() {
                        if c == '%' {
                            match chars.next() {
                                Some('d') => {
                                    result.push_str(&self.memory[self.sp + arg_index].to_string());
                                    arg_index -= 1;
                                }
                                Some('s') => {
                                    let s = unsafe {
                                        let ptr = self.memory[self.sp + arg_index] as *const i8;
                                        std::ffi::CStr::from_ptr(ptr).to_str().unwrap()
                                    };
                                    result.push_str(s);
                                    arg_index -= 1;
                                }
                                Some(c) => result.push(c),
                                None => break,
                            }
                        } else {
                            result.push(c);
                        }
                    }
                    
                    print!("{}", result);
                    io::stdout().flush()?;
                    
                    self.sp += args;
                    self.pc += 1;
                    self.ax = result.len() as i64;
                }
                OpCode::MALC => {
                    let size = self.memory[self.sp] as usize;
                    self.sp += 1;
                    
                    unsafe {
                        let layout = Layout::from_size_align(size, 8).unwrap();
                        let ptr = alloc(layout);
                        self.ax = ptr as i64;
                    }
                }
                OpCode::FREE => {
                    let ptr = self.memory[self.sp] as *mut u8;
                    self.sp += 1;
                    
                    unsafe {
                        let layout = Layout::from_size_align(1, 8).unwrap(); // Size doesn't matter for dealloc
                        dealloc(ptr, layout);
                    }
                }
                OpCode::MSET => {
                    let ptr = self.memory[self.sp + 2] as *mut u8;
                    let value = self.memory[self.sp + 1] as u8;
                    let count = self.memory[self.sp] as usize;
                    self.sp += 3;
                    
                    unsafe {
                        std::ptr::write_bytes(ptr, value, count);
                    }
                    self.ax = ptr as i64;
                }
                OpCode::MCMP => {
                    let s1 = self.memory[self.sp + 2] as *const u8;
                    let s2 = self.memory[self.sp + 1] as *const u8;
                    let count = self.memory[self.sp] as usize;
                    self.sp += 3;
                    
                    unsafe {
                        let slice1 = std::slice::from_raw_parts(s1, count);
                        let slice2 = std::slice::from_raw_parts(s2, count);
                        self.ax = match slice1.cmp(slice2) {
                            std::cmp::Ordering::Less => -1,
                            std::cmp::Ordering::Equal => 0,
                            std::cmp::Ordering::Greater => 1,
                        };
                    }
                }
            }
        }
        Ok(0)
    }
}
