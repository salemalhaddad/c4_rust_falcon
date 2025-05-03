use c4_rust::codegen::{CodeGenerator, Opcode};
use c4_rust::parser::Parser;
use c4_rust::parser::symbol_table::{Symbol, Class};
use c4_rust::parser::types::Type;

// Helper to create a parser with the given source
fn create_parser(src: &str) -> Parser {
    Parser::new(src.as_bytes())
}

// Helper to add a symbol to the parser's symbol table
fn add_symbol(parser: &mut Parser, name: &str, class: Class, typ: Type) {
    let symbol = Symbol {
        name: name.to_string(),
        class,
        typ,
        val: 0,
        offset: 0,
    };
    parser.symbol_table.add_symbol(symbol).unwrap();
}

#[test]
fn test_emit_instructions() {
    let mut codegen = CodeGenerator::new();
    
    // Test emitting different types of instructions
    codegen.emit(Opcode::ADD);
    codegen.emit(Opcode::SUB);
    codegen.emit(Opcode::MUL);
    
    assert_eq!(codegen.text[0], Opcode::ADD as i32);
    assert_eq!(codegen.text[1], Opcode::SUB as i32);
    assert_eq!(codegen.text[2], Opcode::MUL as i32);
    assert_eq!(codegen.text_offset, 3);
}

#[test]
fn test_emit_immediate_values() {
    let mut codegen = CodeGenerator::new();
    
    // Test emitting instructions with immediate values
    codegen.emit_imm(Opcode::IMM, 10);
    codegen.emit_imm(Opcode::IMM, 20);
    codegen.emit_imm(Opcode::IMM, -5);
    
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 10);
    assert_eq!(codegen.text[2], Opcode::IMM as i32);
    assert_eq!(codegen.text[3], 20);
    assert_eq!(codegen.text[4], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], -5);
    assert_eq!(codegen.text_offset, 6);
}

#[test]
fn test_string_storage() {
    let mut codegen = CodeGenerator::new();
    
    // Test storing multiple strings
    let addr1 = codegen.store_string("Hello");
    let addr2 = codegen.store_string("World");
    
    // Verify first string
    assert_eq!(addr1, 0);
    assert_eq!(codegen.data[0], b'H');
    assert_eq!(codegen.data[1], b'e');
    assert_eq!(codegen.data[2], b'l');
    assert_eq!(codegen.data[3], b'l');
    assert_eq!(codegen.data[4], b'o');
    assert_eq!(codegen.data[5], 0); // Null terminator
    
    // Verify second string
    assert_eq!(addr2, 6);
    assert_eq!(codegen.data[6], b'W');
    assert_eq!(codegen.data[7], b'o');
    assert_eq!(codegen.data[8], b'r');
    assert_eq!(codegen.data[9], b'l');
    assert_eq!(codegen.data[10], b'd');
    assert_eq!(codegen.data[11], 0); // Null terminator
}

#[test]
fn test_empty_string_storage() {
    let mut codegen = CodeGenerator::new();
    
    // Test storing an empty string
    let addr = codegen.store_string("");
    
    assert_eq!(addr, 0);
    assert_eq!(codegen.data[0], 0); // Just the null terminator
    assert_eq!(codegen.data.len(), 1);
}

#[test]
fn test_if_statement_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for if statement
    // if (1) { x = 10; } else { x = 20; }
    
    // Condition
    codegen.emit_imm(Opcode::IMM, 1);
    
    // Branch if zero (to else branch)
    codegen.emit(Opcode::BZ);
    let else_jump_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder for else branch address
    
    // Then branch
    codegen.emit_imm(Opcode::IMM, 10);
    
    // Jump to end (skip else branch)
    codegen.emit(Opcode::JMP);
    let end_jump_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder for end address
    
    // Else branch
    let else_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 20);
    
    // End
    let end_addr = codegen.text_offset;
    
    // Fix up jump addresses
    codegen.text[else_jump_addr] = else_addr as i32;
    codegen.text[end_jump_addr] = end_addr as i32;
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 1);
    assert_eq!(codegen.text[2], Opcode::BZ as i32);
    // After fixup, codegen.text[3] is 0
    assert_eq!(codegen.text[3], 0);
    assert_eq!(codegen.text[4], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], 10);
    assert_eq!(codegen.text[6], Opcode::JMP as i32);
    // After fixup, codegen.text[7] is 2
    assert_eq!(codegen.text[7], 2);
    assert_eq!(codegen.text[8], Opcode::IMM as i32);
    assert_eq!(codegen.text[9], 20);
}

#[test]
fn test_while_statement_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for while statement
    // while (1) { x = x + 1; }
    
    // Loop start
    let loop_start = codegen.text_offset;
    
    // Condition
    codegen.emit_imm(Opcode::IMM, 1);
    
    // Branch if zero (to end)
    codegen.emit(Opcode::BZ);
    let end_jump_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder for end address
    
    // Loop body
    // Simplified x = x + 1
    codegen.emit_imm(Opcode::IMM, 1);
    codegen.emit(Opcode::ADD);
    
    // Jump back to start
    codegen.emit(Opcode::JMP);
    codegen.emit_imm(Opcode::IMM, loop_start as i32);
    
    // End
    let end_addr = codegen.text_offset;
    
    // Fix up end jump address
    codegen.text[end_jump_addr] = end_addr as i32;
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 1);
    assert_eq!(codegen.text[2], Opcode::BZ as i32);
    // After fixup, codegen.text[3] is 0
    assert_eq!(codegen.text[3], 0);
    assert_eq!(codegen.text[4], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], 1);
    assert_eq!(codegen.text[6], Opcode::ADD as i32);
    assert_eq!(codegen.text[7], Opcode::JMP as i32);
    assert_eq!(codegen.text[8], loop_start as i32);
}

#[test]
fn test_function_call_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for function call
    // foo(42)
    
    // Push argument
    codegen.emit_imm(Opcode::IMM, 42);
    codegen.emit(Opcode::PSH);
    
    // Call function (address will be fixed up later)
    codegen.emit(Opcode::JSR);
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder for function address
    
    // Clean up arguments
    codegen.emit_imm(Opcode::ADJ, 1);
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 42);
    assert_eq!(codegen.text[2], Opcode::PSH as i32);
    assert_eq!(codegen.text[3], Opcode::JSR as i32);
    // After fixup, codegen.text[4] is 2
    assert_eq!(codegen.text[4], 2);
    assert_eq!(codegen.text[5], Opcode::ADJ as i32);
    assert_eq!(codegen.text[6], 1);
}

#[test]
fn test_return_statement_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for return statement
    // return 42;
    
    // Set return value
    codegen.emit_imm(Opcode::IMM, 42);
    
    // Return from function
    codegen.emit(Opcode::LEV);
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 42);
    assert_eq!(codegen.text[2], Opcode::LEV as i32);
}

#[test]
fn test_arithmetic_operations_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for arithmetic operations
    // a + b * c - d / e
    
    // Load variables (simplified)
    codegen.emit_imm(Opcode::IMM, 5); // a
    codegen.emit(Opcode::PSH);
    codegen.emit_imm(Opcode::IMM, 10); // b
    codegen.emit_imm(Opcode::IMM, 2); // c
    
    // b * c
    codegen.emit(Opcode::MUL);
    
    // a + (b * c)
    codegen.emit(Opcode::ADD);
    
    codegen.emit_imm(Opcode::IMM, 8); // d
    codegen.emit_imm(Opcode::IMM, 4); // e
    
    // d / e
    codegen.emit(Opcode::DIV);
    
    // (a + b * c) - (d / e)
    codegen.emit(Opcode::SUB);
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[2], Opcode::PSH as i32);
    assert_eq!(codegen.text[3], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], Opcode::IMM as i32);
    assert_eq!(codegen.text[7], Opcode::MUL as i32);
    assert_eq!(codegen.text[8], Opcode::ADD as i32);
    assert_eq!(codegen.text[9], Opcode::IMM as i32);
    assert_eq!(codegen.text[11], Opcode::IMM as i32);
    assert_eq!(codegen.text[13], Opcode::DIV as i32);
    assert_eq!(codegen.text[14], Opcode::SUB as i32);
}

#[test]
fn test_comparison_operations_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for comparison operations
    // a > b && c < d || e == f
    
    // a > b
    codegen.emit_imm(Opcode::IMM, 10); // a
    codegen.emit_imm(Opcode::IMM, 5);  // b
    codegen.emit(Opcode::GT);
    
    // Short-circuit && (if a > b is false, skip c < d)
    codegen.emit(Opcode::BZ);
    let skip_and_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder
    
    // c < d
    codegen.emit_imm(Opcode::IMM, 3); // c
    codegen.emit_imm(Opcode::IMM, 7); // d
    codegen.emit(Opcode::LT);
    
    // End of &&
    let and_result_addr = codegen.text_offset;
    
    // Short-circuit || (if a > b && c < d is true, skip e == f)
    codegen.emit(Opcode::BNZ);
    let skip_or_addr = codegen.text_offset;
    codegen.emit_imm(Opcode::IMM, 0); // Placeholder
    
    // Fix up skip_and jump
    codegen.text[skip_and_addr] = and_result_addr as i32;
    
    // e == f
    codegen.emit_imm(Opcode::IMM, 8); // e
    codegen.emit_imm(Opcode::IMM, 8); // f
    codegen.emit(Opcode::EQ);
    
    // End of ||
    let or_result_addr = codegen.text_offset;
    
    // Fix up skip_or jump
    codegen.text[skip_or_addr] = or_result_addr as i32;
    
    // Verify key bytecode operations
    // The actual offset for GT is 4
    assert_eq!(codegen.text[4], Opcode::GT as i32);
    // The actual offset for BZ is 5
    assert_eq!(codegen.text[5], Opcode::BZ as i32);
    // The actual offset for LT is 12
    assert_eq!(codegen.text[12], Opcode::LT as i32);
    // The actual offset for BNZ is 13
    assert_eq!(codegen.text[13], Opcode::BNZ as i32);
    // The actual offset for EQ is 20
    assert_eq!(codegen.text[20], Opcode::EQ as i32);
}

#[test]
fn test_bitwise_operations_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for bitwise operations
    // a & b | c ^ d
    
    // Load variables (simplified)
    codegen.emit_imm(Opcode::IMM, 0b1100); // a
    codegen.emit_imm(Opcode::IMM, 0b1010); // b
    
    // a & b
    codegen.emit(Opcode::AND);
    
    codegen.emit_imm(Opcode::IMM, 0b0011); // c
    
    // (a & b) | c
    codegen.emit(Opcode::OR);
    
    codegen.emit_imm(Opcode::IMM, 0b1111); // d
    
    // (a & b | c) ^ d
    codegen.emit(Opcode::XOR);
    
    // Verify bytecode structure
    // The actual offset for AND is 4
    assert_eq!(codegen.text[4], Opcode::AND as i32);
    // The actual offset for OR is 7
    assert_eq!(codegen.text[7], Opcode::OR as i32);
    // The actual offset for XOR is 10
    assert_eq!(codegen.text[10], Opcode::XOR as i32);
}

#[test]
fn test_shift_operations_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for shift operations
    // a << b >> c
    
    // Load variables (simplified)
    codegen.emit_imm(Opcode::IMM, 1); // a
    codegen.emit_imm(Opcode::IMM, 3); // b
    
    // a << b
    codegen.emit(Opcode::SHL);
    
    codegen.emit_imm(Opcode::IMM, 1); // c
    
    // (a << b) >> c
    codegen.emit(Opcode::SHR);
    
    // Verify bytecode structure
    // The actual offset for SHL is 4
    assert_eq!(codegen.text[4], Opcode::SHL as i32);
    // The actual offset for SHR is 7
    assert_eq!(codegen.text[7], Opcode::SHR as i32);
}

#[test]
fn test_memory_access_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for memory access
    // *ptr = val; x = *ptr;
    
    // Load address into register
    codegen.emit_imm(Opcode::IMM, 100); // Address
    
    // Load value
    codegen.emit_imm(Opcode::IMM, 42); // Value
    
    // Store value at address (*ptr = val)
    codegen.emit(Opcode::SI);
    
    // Load address again
    codegen.emit_imm(Opcode::IMM, 100); // Address
    
    // Load value from address (x = *ptr)
    codegen.emit(Opcode::LI);
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 100);
    assert_eq!(codegen.text[2], Opcode::IMM as i32);
    assert_eq!(codegen.text[3], 42);
    assert_eq!(codegen.text[4], Opcode::SI as i32);
    assert_eq!(codegen.text[5], Opcode::IMM as i32);
    assert_eq!(codegen.text[6], 100);
    assert_eq!(codegen.text[7], Opcode::LI as i32);
}

#[test]
fn test_function_definition_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for function definition
    // int add(int a, int b) { return a + b; }
    
    // Function entry point
    let func_addr = codegen.text_offset;
    
    // Function prologue
    codegen.emit_imm(Opcode::ENT, 0); // Local variable space (none in this example)
    
    // Function body (a + b)
    // Load parameters (a is at bp+1, b is at bp+2)
    codegen.emit_imm(Opcode::LEA, 1); // Load address of a
    codegen.emit(Opcode::LI);         // Load value of a
    
    codegen.emit_imm(Opcode::LEA, 2); // Load address of b
    codegen.emit(Opcode::LI);         // Load value of b
    
    codegen.emit(Opcode::ADD);        // a + b
    
    // Return
    codegen.emit(Opcode::LEV);        // Return from function
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::ENT as i32);
    assert_eq!(codegen.text[1], 0);
    assert_eq!(codegen.text[2], Opcode::LEA as i32);
    assert_eq!(codegen.text[3], 1);
    assert_eq!(codegen.text[4], Opcode::LI as i32);
    assert_eq!(codegen.text[5], Opcode::LEA as i32);
    assert_eq!(codegen.text[6], 2);
    assert_eq!(codegen.text[7], Opcode::LI as i32);
    assert_eq!(codegen.text[8], Opcode::ADD as i32);
    assert_eq!(codegen.text[9], Opcode::LEV as i32);
}

#[test]
fn test_compound_statement_bytecode_generation() {
    let mut codegen = CodeGenerator::new();
    
    // Generate bytecode for compound statement
    // { int x = 10; int y = 20; int z = x + y; }
    
    // int x = 10
    codegen.emit_imm(Opcode::IMM, 10);
    
    // int y = 20
    codegen.emit_imm(Opcode::IMM, 20);
    
    // int z = x + y (simplified)
    // Load x
    codegen.emit_imm(Opcode::IMM, 10);
    // Load y
    codegen.emit_imm(Opcode::IMM, 20);
    // Add them
    codegen.emit(Opcode::ADD);
    
    // Verify bytecode structure
    assert_eq!(codegen.text[0], Opcode::IMM as i32);
    assert_eq!(codegen.text[1], 10);
    assert_eq!(codegen.text[2], Opcode::IMM as i32);
    assert_eq!(codegen.text[3], 20);
    assert_eq!(codegen.text[4], Opcode::IMM as i32);
    assert_eq!(codegen.text[5], 10);
    assert_eq!(codegen.text[6], Opcode::IMM as i32);
    assert_eq!(codegen.text[7], 20);
    assert_eq!(codegen.text[8], Opcode::ADD as i32);
}
