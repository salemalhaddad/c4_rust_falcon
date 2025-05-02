#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Char,
    Int,
    Ptr(Box<Type>),
}

impl Type {
    pub fn size(&self) -> i32 {
        match self {
            Type::Char => 1,
            Type::Int => 4,
            Type::Ptr(_) => 4, // Pointers are 4 bytes on 32-bit systems
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Char | Type::Int)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Ptr(_))
    }

    pub fn get_base_type(&self) -> Option<&Type> {
        match self {
            Type::Ptr(base) => Some(base),
            _ => None,
        }
    }

    pub fn to_pointer(self) -> Self {
        Type::Ptr(Box::new(self))
    }
}
