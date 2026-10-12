use chumsky::{
    IterParser, Parser,
    primitive::{end, just},
    recursive::recursive,
    select,
    error::Rich,
};

use crate::lexer::Token;

#[derive(Debug)]
pub enum Expr {
    // Integer literal.
    Int(isize),

    // Floating point literal.
    Float(f64),

    // Unary minus.
    Neg(Box<Expr>),

    // Binary operators.
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
}

pub fn parser<'src>(
) -> impl Parser<'src, &'src [Token], Expr, chumsky::extra::Err<Rich<'src, Token>>>
{
    let expression = recursive(|p| {
        
        let atom = {
            let parenthesized = p
                .clone()
                .delimited_by(
                    just(Token::LParen),
                    just(Token::RParen).labelled("`)`"),
                );

            let float = select! {
                Token::Float(n) => Expr::Float(n),
            };

            let integer = select! {
                Token::Integer(n) => Expr::Int(n),
            };

            let literal = float
                .or(integer)
                .labelled("an expression");

            parenthesized.or(literal)
        };

        
        let unary = recursive(|unary| {
            let power = atom
                .clone()
                .then(
                    just(Token::Power)
                        .ignore_then(unary.clone())
                        .or_not(),
                )
                .map(|(lhs, rhs)| match rhs {
                    Some(rhs) => Expr::Pow(Box::new(lhs), Box::new(rhs)),
                    None => lhs,
                });

            just(Token::Plus)
                .or(just(Token::Minus))
                .repeated()
                .foldr(power, |_op, rhs| match _op {
                    Token::Plus => rhs,
                    Token::Minus => Expr::Neg(Box::new(rhs)),
                    _ => unreachable!(),
                })
        });

        let binary_1 = unary.clone().foldl(
            just(Token::Multiply)
                .or(just(Token::Divide))
                .then(unary)
                .repeated(),
            |lhs, (op, rhs)| match op {
                Token::Multiply => Expr::Mul(Box::new(lhs), Box::new(rhs)),
                Token::Divide => Expr::Div(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            },
        );

        let binary_2 = binary_1.clone().foldl(
            just(Token::Plus)
                .or(just(Token::Minus))
                .then(binary_1)
                .repeated(),
            |lhs, (op, rhs)| match op {
                Token::Plus => Expr::Add(Box::new(lhs), Box::new(rhs)),
                Token::Minus => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            },
        );

        binary_2
    });

    expression.then_ignore(end())
}