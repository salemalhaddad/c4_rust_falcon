use c4_rust::codegen::{CodeGenerator, Opcode};
use c4_rust::lexer::Lexer;
use c4_rust::parser::Parser;
use c4_rust::parser::types::Type;
use c4_rust::parser::symbol_table::{Class, Symbol};

#[test]
fn test_basic_bytecode_emission() {
    let mut codegen = CodeGenerator::new();
    
    // Test basic instruction emission
    codegen.emit(Opcode::ADD);
    assert_eq!(codegen.text[0], Opcode::ADD as i32);
    
    codegen.emit(Opcode::SUB);
    assert_eq!(codegen.text[1], Opcode::SUB as i32);
    
    // Test immediate value emission
    codegen.emit_imm(Opcode::IMM, 42);
    assert_eq!(codegen.text[2], Opcode::IMM as i32);
    assert_eq!(codegen.text[3], 42);
    
    // Verify text segment size and offset
    assert_eq!(codegen.text.len(), 4);
    assert_eq!(codegen.text_offset, 4);
}

#[test]
fn test_string_storage() {
    let mut codegen = CodeGenerator::new();
    
    // Test storing strings in the data segment
    let addr1 = codegen.store_string("Hello");
    let addr2 = codegen.store_string("World");
    
    // Verify addresses
    assert_eq!(addr1, 0);
    assert_eq!(addr2, 6); // "Hello\0" takes 6 bytes
    
    // Verify string contents
    assert_eq!(&codegen.data[0..6], b"Hello\0");
    assert_eq!(&codegen.data[6..12], b"World\0");
}

#[test]
fn test_if_statement_bytecode_structure() {
    let mut codegen = CodeGenerator::new();
    
    // Simulate bytecode for: if (condition) { then_branch } else { else_branch }
    
    // 1. Condition evaluation (placeholder)
    codegen.emit_imm(Opcode::IMM, 1);
    
    // 2. Branch if zero (BZ to else branch)
    codegen.emit(Opcode::BZ);
    let else_jump_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder for else branch address
    
    // 3. Then branch code (placeholder)
    codegen.emit_imm(Opcode::IMM, 10);
    
    // 4. Jump to end (skip else branch)
    codegen.emit(Opcode::JMP);
    let end_jump_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder for end address
    
    // 5. Else branch location
    let else_branch_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 20);
    
    // 6. End location
    let end_addr = codegen.text_offset;
    
    // 7. Fix up jump addresses
    codegen.text[else_jump_addr] = else_branch_addr as i32;
    codegen.text[end_jump_addr] = end_addr as i32;
    
    // Verify the bytecode structure matches our if-else pattern
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 1);
    assert_eq!(codegen.text[2], Opcode::BZ as i32);
    assert_eq!(codegen.text[3], else_branch_addr as i32);
    assert_eq!(codegen.text[4], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], 10);
    assert_eq!(codegen.text[6], Opcode::JMP as i32);
    assert_eq!(codegen.text[7], end_addr as i32);
    assert_eq!(codegen.text[8], Opcode::IMM as i32);
    assert_eq!(codegen.text[9], 20);
}

#[test]
fn test_while_loop_bytecode_structure() {
    let mut codegen = CodeGenerator::new();
    
    // Simulate bytecode for: while (condition) { body }
    
    // 1. Loop start (condition evaluation)
    let loop_start = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 1); // Placeholder for condition
    
    // 2. Branch if zero (exit loop if condition is false)
    codegen.emit(Opcode::BZ);
    let exit_jump_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder for exit address
    
    // 3. Loop body (placeholder)
    codegen.emit_imm(Opcode::IMM, 42);
    
    // 4. Jump back to condition
    codegen.emit(Opcode::JMP);
    codegen.emit_imm(Opcode::IMM, loop_start as i32);
    
    // 5. Loop exit location
    let exit_addr = codegen.text_offset;
    
    // 6. Fix up exit jump address
    codegen.text[exit_jump_addr] = exit_addr as i32;
    
    // Verify the bytecode structure matches our while loop pattern
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 1);
    assert_eq!(codegen.text[2], Opcode::BZ as i32);
    assert_eq!(codegen.text[3], exit_addr as i32);
    assert_eq!(codegen.text[4], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], 42);
    assert_eq!(codegen.text[6], Opcode::JMP as i32);
    assert_eq!(codegen.text[7], loop_start as i32);
}

#[test]
fn test_function_call_bytecode() {
    let mut codegen = CodeGenerator::new();
    
    // Simulate bytecode for a function call: foo(42)
    
    // 1. Push argument onto stack
    codegen.emit_imm(Opcode::IMM, 42);
    codegen.emit(Opcode::PSH);
    
    // 2. Call function (JSR)
    codegen.emit(Opcode::JSR);
    codegen.emit_imm(Opcode::IMM, 100); // Function address (arbitrary for test)
    
    // 3. Adjust stack after call (remove arguments)
    codegen.emit_imm(Opcode::ADJ, 1);
    
    // Verify the bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 42);
    assert_eq!(codegen.text[2], Opcode::PSH as i32);
    assert_eq!(codegen.text[3], Opcode::JSR as i32);
    assert_eq!(codegen.text[4], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], 100);
    assert_eq!(codegen.text[6], Opcode::ADJ as i32);
    assert_eq!(codegen.text[7], 1);
}
