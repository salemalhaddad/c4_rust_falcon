use std::collections::VecDeque;

pub struct VM {
    code: Vec<i32>,
    data: Vec<u8>,
    stack: VecDeque<i32>,
    memory: Vec<i32>,
    pc: usize,
    sp: usize,
    debug_mode: bool,
}

impl VM {
    pub fn new(code: Vec<i32>, data: Vec<u8>, stack_size: usize, debug_mode: bool) -> Self {
        VM {
            code,
            data,
            stack: VecDeque::new(),
            memory: vec![0; stack_size],
            pc: 0,
            sp: 0,
            debug_mode,
        }
    }

    pub fn run(&mut self) -> Result<i32, String> {
        self.pc = 0;
        self.sp = 0;
        self.stack.clear();

        while self.pc < self.code.len() {
            let instruction = self.code[self.pc];
            self.execute_instruction(instruction)?;
            self.pc += 1;
        }

        Ok(self.stack.pop_back().unwrap_or(0))
    }

    fn execute_instruction(&mut self, instruction: i32) -> Result<(), String> {
        // TODO: Implement actual instruction execution
		
        Ok(())
    }
}
