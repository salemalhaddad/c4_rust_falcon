# C4 Rust Compiler – Team Zubair

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
c4-rust/
├── Cargo.toml
├── src/
│   ├── lexer.rs
│   ├── parser/
│   │   ├── mod.rs           # Public API: re-exports sub-modules
│   │   ├── symbol_table.rs  # Symbol, Class, Type & scope management
│   │   ├── types.rs         # Type enum and utilities
│   │   ├── declaration.rs   # Parse globals, functions, variable declarations
│   │   ├── expression.rs    # Precedence-climbing expression parser
│   │   └── statement.rs     # Statement parser (if, while, return, compound)
│   ├── codegen.rs
│   ├── vm.rs
│   └── main.rs
└── tests/
    ├── xxxx.c
    └── xxxx.c
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

