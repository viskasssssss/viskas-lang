use std::collections::HashMap;
use std::fmt;

use crate::parser::{Expr, Statement};
use crate::value::Type;

#[derive(Debug, Clone, Copy)]
struct Binding {
    ty: Type,
    mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    DuplicateVariable(String),
    UnknownVariable(String),
    ImmutableAssignment(String),
    TypeMismatch {
        expected: Type,
        found: Type,
    },
    IntegerOutOfRange(isize),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::DuplicateVariable(name) => {
                write!(f, "variable `{name}` is already declared")
            }
            TypeError::UnknownVariable(name) => {
                write!(f, "unknown variable `{name}`")
            }
            TypeError::ImmutableAssignment(name) => {
                write!(f, "cannot assign to immutable variable `{name}`")
            }
            TypeError::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected `{expected}`, found `{found}`")
            }
            TypeError::IntegerOutOfRange(n) => {
                write!(f, "integer literal {n} is outside the i32 range")
            }
        }
    }
}

impl std::error::Error for TypeError {}

#[derive(Default, Clone)]
pub struct TypeChecker {
    variables: HashMap<String, Binding>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_program(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), TypeError> {
        for statement in statements {
            self.check_statement(statement)?;
        }

        Ok(())
    }

    fn check_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<(), TypeError> {
        match statement {
            Statement::Let {
                name,
                mutable,
                annotation,
                value,
            } => {
                if self.variables.contains_key(name) {
                    return Err(TypeError::DuplicateVariable(name.clone()));
                }

                let inferred = self.check_expr(value)?;
                let ty = match annotation {
                    Some(declared) => {
                        self.check_assignable(*declared, inferred)?;
                        *declared
                    }
                    None => inferred,
                };

                self.variables.insert(
                    name.clone(),
                    Binding {
                        ty,
                        mutable: *mutable,
                    },
                );

                Ok(())
            }

            Statement::Assign { name, value } => {
                let binding = self
                    .variables
                    .get(name)
                    .copied()
                    .ok_or_else(|| TypeError::UnknownVariable(name.clone()))?;

                if !binding.mutable {
                    return Err(TypeError::ImmutableAssignment(name.clone()));
                }

                let found = self.check_expr(value)?;
                self.check_assignable(binding.ty, found)
            }

            Statement::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
        }
    }

    fn check_expr(&self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Int(n) => {
                i32::try_from(*n)
                    .map_err(|_| TypeError::IntegerOutOfRange(*n))?;
                Ok(Type::I32)
            }

            Expr::Float(_) => Ok(Type::F64),

            Expr::Variable(name) => self
                .variables
                .get(name)
                .map(|binding| binding.ty)
                .ok_or_else(|| TypeError::UnknownVariable(name.clone())),

            Expr::Neg(inner) => self.check_expr(inner),

            Expr::Add(lhs, rhs)
            | Expr::Sub(lhs, rhs)
            | Expr::Mul(lhs, rhs)
            | Expr::Div(lhs, rhs)
            | Expr::Pow(lhs, rhs) => {
                let lhs_ty = self.check_expr(lhs)?;
                let rhs_ty = self.check_expr(rhs)?;

                // Mixed numeric expressions promote to f64.
                if lhs_ty == Type::F64 || rhs_ty == Type::F64 {
                    Ok(Type::F64)
                } else {
                    Ok(Type::I32)
                }
            }
        }
    }

    fn check_assignable(
        &self,
        expected: Type,
        found: Type,
    ) -> Result<(), TypeError> {
        if expected == found
            || (expected == Type::F64 && found == Type::I32)
        {
            Ok(())
        } else {
            Err(TypeError::TypeMismatch { expected, found })
        }
    }
}