use std::collections::HashMap;
use super::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum Class {
    Global,
    Local,
    Function,
    Sys,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub class: Class,
    pub typ: Type,
    pub val: i64,
    pub offset: i32, // Offset for local variables or function parameters
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    scopes: Vec<Vec<String>>, // Stack of scopes (each scope is a list of symbol names)
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scopes: vec![vec![]], // Initialize with global scope
        }
    }

    pub fn init_builtins(&mut self) {
        // Add built-in types
        self.add_type("int", Type::Int);
        self.add_type("char", Type::Char);

        // Add system functions
        self.add_sys_func("open", Type::Int);
        self.add_sys_func("read", Type::Int);
        self.add_sys_func("close", Type::Int);
        self.add_sys_func("printf", Type::Int);
        self.add_sys_func("malloc", Type::Ptr(Box::new(Type::Int)));
        self.add_sys_func("free", Type::Int);
        self.add_sys_func("memset", Type::Int);
        self.add_sys_func("memcmp", Type::Int);
        self.add_sys_func("exit", Type::Int);
    }

    pub fn all_symbols(&self) -> impl Iterator<Item = (&String, &Symbol)> {
        self.symbols.iter()
    }

    fn add_type(&mut self, name: &str, typ: Type) {
        let symbol = Symbol {
            name: name.to_string(),
            class: Class::Global,
            typ,
            val: 0,
            offset: 0,
        };
        self.symbols.insert(name.to_string(), symbol);
        self.scopes[0].push(name.to_string());
    }

    fn add_sys_func(&mut self, name: &str, ret_type: Type) {
        let symbol = Symbol {
            name: name.to_string(),
            class: Class::Sys,
            typ: ret_type,
            val: 0, // Will be set to the appropriate system call ID
            offset: 0,
        };
        self.symbols.insert(name.to_string(), symbol);
        self.scopes[0].push(name.to_string());
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(vec![]);
    }

    pub fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            // Remove all symbols in the current scope
            for name in scope {
                self.symbols.remove(&name);
            }
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) -> Result<(), String> {
        let name = symbol.name.clone();
        
        // Check if symbol already exists in current scope
        if self.lookup_current_scope(&name).is_some() {
            return Err(format!("Symbol '{}' already defined in current scope", name));
        }
        
        // Add symbol to table and current scope
        self.symbols.insert(name.clone(), symbol);
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(name);
        }
        
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        if let Some(scope) = self.scopes.last() {
            if scope.contains(&name.to_string()) {
                return self.symbols.get(name);
            }
        }
        None
    }

    pub fn update_symbol(&mut self, name: &str, update_fn: impl FnOnce(&mut Symbol)) -> Result<(), String> {
        if let Some(symbol) = self.symbols.get_mut(name) {
            update_fn(symbol);
            Ok(())
        } else {
            Err(format!("Symbol '{}' not found", name))
        }
    }
}
