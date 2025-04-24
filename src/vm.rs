//! SCAFFOLD: Virtual Machine for C4-inspired language
//! This is a temporary implementation that will be integrated with the real parser.
//! Current status:
//! - Basic AST node types defined
//! - Simple arithmetic and variable operations
//! - Control flow (if/while) ready for testing

use std::collections::HashMap;

// Placeholder for runtime values
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
}

// Placeholder for VM errors
#[derive(Debug)]
pub enum VmError {
    UndefinedVariable(String),
    TypeError(String),
    Runtime(String),
}

// Placeholder AstNode enum, mirror C4 constructs and expected parser output
#[derive(Debug)]
pub enum AstNode {
    Num(i64),
    Id(String),
    Assign { left: Box<AstNode>, right: Box<AstNode> },
    Add(Box<AstNode>, Box<AstNode>),
    Sub(Box<AstNode>, Box<AstNode>),
    Mul(Box<AstNode>, Box<AstNode>),
    Div(Box<AstNode>, Box<AstNode>),
    Mod(Box<AstNode>, Box<AstNode>),
    If {
        cond: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    While {
        cond: Box<AstNode>,
        body: Box<AstNode>,
    },
    Block(Vec<AstNode>),
    Return(Box<AstNode>),

}

pub struct Vm {
    env: HashMap<String, Value>,
}

impl Vm {
    pub fn new() -> Self {
        Vm { env: HashMap::new() }
    }

    pub fn eval(&mut self, node: &AstNode) -> Result<Value, VmError> {
        use AstNode::*;
        match node {
            Num(n) => Ok(Value::Int(*n)),
            Id(name) => self.env.get(name)
                .cloned()
                .ok_or_else(|| VmError::UndefinedVariable(name.clone())),
            Assign { left, right } => {
                if let AstNode::Id(name) = &**left {
                    let val = self.eval(right)?;
                    self.env.insert(name.clone(), val.clone());
                    Ok(val)
                } else {
                    Err(VmError::Runtime("Left side of assignment must be an identifier".into()))
                }
            }
            Add(a, b) => match (self.eval(a)?, self.eval(b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
                _ => Err(VmError::TypeError("Add expects integers".into())),
            },
            Sub(a, b) => match (self.eval(a)?, self.eval(b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
                _ => Err(VmError::TypeError("Sub expects integers".into())),
            },
            Mul(a, b) => match (self.eval(a)?, self.eval(b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
                _ => Err(VmError::TypeError("Mul expects integers".into())),
            },
            Div(a, b) => match (self.eval(a)?, self.eval(b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
                _ => Err(VmError::TypeError("Div expects integers".into())),
            },
            Mod(a, b) => match (self.eval(a)?, self.eval(b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x % y)),
                _ => Err(VmError::TypeError("Mod expects integers".into())),
            },
            If { cond, then_branch, else_branch } => {
                let cond_val = self.eval(cond)?;
                match cond_val {
                    Value::Int(0) => {
                        if let Some(else_br) = else_branch {
                            self.eval(else_br)
                        } else {
                            Ok(Value::Int(0))
                        }
                    }
                    _ => self.eval(then_branch),
                }
            }
            While { cond, body } => {
                let mut result = Value::Int(0);
                while match self.eval(cond)? { Value::Int(0) => false, _ => true } {
                    result = self.eval(body)?;
                }
                Ok(result)
            }
            Block(stmts) => {
                let mut last = Value::Int(0);
                for stmt in stmts {
                    last = self.eval(stmt)?;
                }
                Ok(last)
            }
            Return(expr) => self.eval(expr),
        
        }
    }
}

// Entrypoint for running the VM on an AST
pub fn run_vm(ast: &AstNode) -> Result<Value, VmError> {
    let mut vm = Vm::new();
    vm.eval(ast)
}

// TODO: Replace placeholder AstNode with the real one from your parser when ready.
// TODO: Add more value types, error handling, and features as needed.
