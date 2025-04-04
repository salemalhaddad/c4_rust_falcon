# C4 Rust Compiler – Team Falcon

Rewriting the C4 compiler in Rust, preserving its self-hosting capability and original functionality while leveraging Rust’s safety and modern features.

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
│   ├── ast.rs             # (Optional) Abstract Syntax Tree definitions
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

## ✅ To-Do List

### Phase 1: Setup & Planning
- [x] Create GitHub repo: `c4_rust_<team_name>`
- [x] Setup `Cargo` project with `cargo init`
- [ ] Add `.gitignore` (Rust template)
- [ ] Upload `c4.c` to `examples/` for testing
- [ ] Divide responsibilities among team

### Phase 2: Component Translation
- [ ] Implement `lexer.rs` using pattern matching and enums
- [ ] Implement `parser.rs` with idiomatic AST structure
- [ ] Implement `vm.rs` using safe memory management
- [ ] Wire components in `main.rs`

### Phase 3: Testing
- [ ] Add unit tests using `#[test]` for each module
- [ ] Achieve minimum 70% test coverage
- [ ] Use `cargo test` consistently
- [ ] Optional: Add performance benchmarks

### Phase 4: Documentation
- [ ] Use `///` comments for public functions and structs
- [ ] Generate and review docs via `cargo doc`
- [ ] Write `c4_rust_comparison.pdf`

### Phase 5: Collaboration & Submission
- [ ] Use branches + PRs for major changes
- [ ] Ensure all members contribute to commits
- [ ] Zip and submit as `c4_rust_submission_<team_name>.zip`

### Bonus (Optional)
- [ ] Implement enhanced error messages with line/column tracking
- [ ] Add floating-point arithmetic support
- [ ] Improve diagnostic outputs (tokens, AST, bytecode dump)

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

