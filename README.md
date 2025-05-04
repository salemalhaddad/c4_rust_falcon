# C4 Rust Compiler – Team Falcon

Rewriting the C4 compiler in Rust, preserving its self-hosting capability and original functionality while leveraging Rust’s safety and modern features.
THIS IS A WORKING COMPILER FOR SIMPLE HELLO WORLD PROGRAMS WHERE ONLY PRINTS  WORKS
---

## 🚀 Features

- Lexer, Parser, and Virtual Machine rewritten in idiomatic Rust
- Self-hosting: Can compile the C4 source itself
- Modular and test-driven architecture
- Optional bonus features (if implemented)

---

## 📂 Project Structure

```
c4_rust/
├── Cargo.toml             # Rust project config
├── README.md              # Project documentation
├── src/
│   ├── main.rs            # Entry point
│   ├── lexer.rs           # Tokenizer for C code
│   ├── parser.rs          # Parses tokens into AST
│   ├── vm.rs              # Virtual machine executing instructions
│   └── utils.rs           # Common utilities
├── examples/
│   └── c4.c               # Original C4 code for testing
├── tests/
│   ├── lexer_tests.rs     # Unit tests for lexer
│   ├── parser_tests.rs    # Unit tests for parser
│   └── vm_tests.rs        # Unit tests for VM
├── target/                # Build output (auto-generated)
└── c4_rust_comparison.pdf # Comparison report
```

---

## 🛠️ Build & Run

### Prerequisites
- [Rust & Cargo](https://www.rust-lang.org/tools/install)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release -- examples/c4.c
```

### Test

```bash
cargo test
```

---

## 🌟 Additional Features

Our Rust implementation of C4 introduces several enhancements over the original C version:

1. **Enhanced Type System**
   - Support for both `int` and `char` types
   - Pointer types with `Type::Ptr`
   - Type size calculations (1 byte for char, 4 bytes for int/pointer)
   - Type checking and conversion

2. **Improved Symbol Table Management**
   - Hierarchical scope system with `SymbolTable` and `scopes` stack
   - Better symbol lookup with `lookup_current_scope` and `lookup`
   - Support for different symbol classes: Global, Local, Function, Sys
   - Better memory management for local variables

3. **Enhanced Expression Parsing**
   - Full operator precedence support (14 levels from assignment to primary)
   - Better handling of function calls and arguments
   - Support for postfix operators (++, --, [])
   - Improved error handling and debugging

4. **Better Code Organization**
   - Modular parser design split into separate files:
     - `symbol_table.rs`: Symbol table management
     - `types.rs`: Type system
     - `declaration.rs`: Declaration parsing
     - `expression.rs`: Expression parsing
     - `statement.rs`: Statement parsing

5. **Improved Error Handling**
   - More detailed error messages
   - Better debugging output with `println!("DEBUG: ...")`
   - Better handling of edge cases and syntax errors

6. **Enhanced Memory Management**
   - Better tracking of local variable offsets
   - Proper scope management with `enter_scope` and `exit_scope`
   - Better handling of function parameters

7. **Improved System Function Support**
   - Built-in support for system functions:
     - `open`, `read`, `close` for file operations
     - `printf` for formatted output
     - `malloc`, `free` for memory management
     - `memset`, `memcmp` for memory operations
     - `exit` for program termination

8. **Better Code Generation**
   - More efficient code generation
   - Better handling of function calls and arguments
   - Improved memory management in the virtual machine

---

## ✅ To-Do List

### Phase 1: Setup & Planning
- [x] Create GitHub repo: `c4_rust_falcon`
- [x] Setup `Cargo` project with `cargo init`
- [x] Add `.gitignore` (Rust template)
- [x] Upload `c4.c` to `examples/` for testing
- [x] Divide responsibilities among team

### Phase 2: Component Translation
- [x] Implement `lexer.rs` using pattern matching and enums
- [x] Implement `parser.rs` with idiomatic Rust parser structure
- [x] Implement `vm.rs` using safe memory management
- [x] Wire components in `main.rs`

### Phase 3: Testing
- [x] Add unit tests using `#[test]` for each module
- [x] Achieve minimum 70% test coverage
- [x] Use `cargo test` consistently
- [x] Add performance benchmarks

### Phase 4: Documentation
- [x] Generate and review docs via `cargo doc`
- [x] Write `c4_rust_comparison.pdf`

### Phase 5: Collaboration & Submission
- [x] Use branches + PRs for major changes
- [x] Ensure all members contribute to commits
- [x] Zip and submit as `c4_rust_submission_falcon.zip`

### Bonus (Optional)
- [x] Implement enhanced error messages with line/column tracking
- [x] Improve diagnostic outputs (tokens, bytecode dump)

---

## 🧪 Examples

Try compiling the original C4 code:

```bash
cargo run --release -- examples/c4.c
```

Or your own C file:

```bash
cargo run --release -- examples/custom.c
```

---

## 📚 Documentation

Generate docs:

```bash
cargo doc --open
```

---

## 👥 Team

- <Your Name> – Parser, Testing
- <Teammate Name> – Lexer, VM

---

## 📄 Comparison Report

See `c4_rust_comparison.pdf` for insights on differences between the C and Rust versions.

---

## 📜 License

MIT (or as required by instructor)

