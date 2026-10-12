use crate::parser::Expr;

pub fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Int(n) => *n as f64,
        Expr::Float(n) => *n,
        Expr::Neg(rhs) => -eval(rhs),
        Expr::Add(lhs, rhs) => eval(lhs) + eval(rhs),
        Expr::Sub(lhs, rhs) => eval(lhs) - eval(rhs),
        Expr::Mul(lhs, rhs) => eval(lhs) * eval(rhs),
        Expr::Div(lhs, rhs) => eval(lhs) / eval(rhs),
        Expr::Pow(lhs, rhs) => eval(lhs).powf(eval(rhs)),
    }
}
