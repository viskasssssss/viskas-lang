use std::fmt;

/// A value's static type in the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I32,
    F64,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::F64 => write!(f, "f64"),
        }
    }
}

/// A value that exists at runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
    F64(f64),
}

impl Value {
    pub fn ty(self) -> Type {
        match self {
            Value::I32(_) => Type::I32,
            Value::F64(_) => Type::F64,
        }
    }

    /// Convert a numeric value to f64 for mixed-type arithmetic.
    pub fn as_f64(self) -> f64 {
        match self {
            Value::I32(n) => n as f64,
            Value::F64(n) => n,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::I32(n) => write!(f, "{n}"),
            Value::F64(n) => write!(f, "{n}"),
        }
    }
}