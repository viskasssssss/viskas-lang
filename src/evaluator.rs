use std::collections::HashMap;
use std::fmt;

use crate::parser::{Expr, Statement};
use crate::value::{Type, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    IntegerOutOfRange(isize),
    IntegerOverflow(&'static str),
    DivisionByZero,
    UndefinedVariable(String),
    DuplicateVariable(String),
    ImmutableAssignment(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::IntegerOutOfRange(n) => {
                write!(f, "integer literal {n} is outside the i32 range")
            }
            RuntimeError::IntegerOverflow(op) => {
                write!(f, "i32 overflow while evaluating {op}")
            }
            RuntimeError::DivisionByZero => write!(f, "division by zero"),
            RuntimeError::UndefinedVariable(name) => {
                write!(f, "undefined variable `{name}`")
            }
            RuntimeError::DuplicateVariable(name) => {
                write!(f, "variable `{name}` is already declared")
            }
            RuntimeError::ImmutableAssignment(name) => {
                write!(f, "cannot assign to immutable variable `{name}`")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone, Copy)]
struct RuntimeBinding {
    value: Value,
    mutable: bool,
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeEnv {
    variables: HashMap<String, RuntimeBinding>,
}

impl RuntimeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    fn declare(
        &mut self,
        name: String,
        value: Value,
        mutable: bool,
    ) -> Result<(), RuntimeError> {
        if self.variables.contains_key(&name) {
            return Err(RuntimeError::DuplicateVariable(name));
        }

        self.variables.insert(
            name,
            RuntimeBinding { value, mutable },
        );

        Ok(())
    }

    fn assign(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), RuntimeError> {
        let binding = self
            .variables
            .get_mut(name)
            .ok_or_else(|| RuntimeError::UndefinedVariable(name.to_owned()))?;

        if !binding.mutable {
            return Err(RuntimeError::ImmutableAssignment(name.to_owned()));
        }

        let value = match (binding.value, value) {
            (Value::F64(_), Value::I32(n)) => Value::F64(n as f64),
            (_, value) => value,
        };

        binding.value = value;
        Ok(())
    }

    fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        self.variables
            .get(name)
            .map(|binding| binding.value)
            .ok_or_else(|| RuntimeError::UndefinedVariable(name.to_owned()))
    }
}

pub fn eval_with_env(
    expr: &Expr,
    env: &RuntimeEnv,
) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Int(n) => {
            let n = i32::try_from(*n)
                .map_err(|_| RuntimeError::IntegerOutOfRange(*n))?;
            Ok(Value::I32(n))
        }

        Expr::Float(n) => Ok(Value::F64(*n)),

        Expr::Variable(name) => env.get(name),

        Expr::Neg(inner) => match eval_with_env(inner, env)? {
            Value::I32(n) => n
                .checked_neg()
                .map(Value::I32)
                .ok_or(RuntimeError::IntegerOverflow("unary negation")),
            Value::F64(n) => Ok(Value::F64(-n)),
        },

        Expr::Add(lhs, rhs) => eval_binary(lhs, rhs, "+", env),
        Expr::Sub(lhs, rhs) => eval_binary(lhs, rhs, "-", env),
        Expr::Mul(lhs, rhs) => eval_binary(lhs, rhs, "*", env),
        Expr::Div(lhs, rhs) => eval_binary(lhs, rhs, "/", env),
        Expr::Pow(lhs, rhs) => eval_binary(lhs, rhs, "^", env),
    }
}

fn eval_binary(
    lhs: &Expr,
    rhs: &Expr,
    op: &'static str,
    env: &RuntimeEnv,
) -> Result<Value, RuntimeError> {
    let lhs = eval_with_env(lhs, env)?;
    let rhs = eval_with_env(rhs, env)?;

    if op == "/" && rhs.as_f64() == 0.0 {
        return Err(RuntimeError::DivisionByZero);
    }

    // If either operand is a float, promote the other operand to f64.
    if matches!(lhs, Value::F64(_)) || matches!(rhs, Value::F64(_)) {
        let a = lhs.as_f64();
        let b = rhs.as_f64();

        let result = match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => a / b,
            "^" => a.powf(b),
            _ => unreachable!("unknown operator"),
        };

        return Ok(Value::F64(result));
    }

    let (Value::I32(a), Value::I32(b)) = (lhs, rhs) else {
        unreachable!("float operands handled above");
    };

    match op {
        "+" => a.checked_add(b)
            .map(Value::I32)
            .ok_or(RuntimeError::IntegerOverflow("+")),

        "-" => a.checked_sub(b)
            .map(Value::I32)
            .ok_or(RuntimeError::IntegerOverflow("-")),

        "*" => a.checked_mul(b)
            .map(Value::I32)
            .ok_or(RuntimeError::IntegerOverflow("*")),

        "/" => a.checked_div(b)
            .map(Value::I32)
            .ok_or_else(|| {
                if b == 0 {
                    RuntimeError::DivisionByZero
                } else {
                    RuntimeError::IntegerOverflow("/")
                }
            }),

        "^" if b >= 0 => a.checked_pow(b as u32)
            .map(Value::I32)
            .ok_or(RuntimeError::IntegerOverflow("^")),

        "^" => Ok(Value::F64((a as f64).powf(b as f64))),

        _ => unreachable!("unknown operator"),
    }
}

/// Executes statements in order and returns the last expression's value,
/// if the program ends with an expression statement.
pub fn execute_program(
    statements: &[Statement],
    env: &mut RuntimeEnv,
) -> Result<Option<Value>, RuntimeError> {
    let mut last_value = None;

    for statement in statements {
        match statement {
            Statement::Let {
                name,
                mutable,
                annotation,
                value,
            } => {
                let mut value = eval_with_env(value, env)?;

                if let Some(Type::F64) = annotation {
                    if let Value::I32(n) = value {
                        value = Value::F64(n as f64);
                    }
                }

                env.declare(name.clone(), value, *mutable)?;
                last_value = None;
            }

            Statement::Assign { name, value } => {
                let value = eval_with_env(value, env)?;
                env.assign(name, value)?;
                last_value = None;
            }

            Statement::Expr(expr) => {
                last_value = Some(eval_with_env(expr, env)?);
            }
        }
    }

    Ok(last_value)
}