use chumsky::{
    error::Rich,
    primitive::{end, just},
    recursive::recursive,
    select, IterParser, Parser,
};

use crate::{
    lexer::Token,
    value::Type,
};

#[derive(Debug)]
pub enum Expr {
    Int(isize),
    Float(f64),
    Variable(String),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum Statement {
    Let {
        name: String,
        mutable: bool,
        annotation: Option<Type>,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Expr(Expr),
}

fn expression_parser<'src>(
) -> impl Parser<'src, &'src [Token], Expr, chumsky::extra::Err<Rich<'src, Token>>> {
    recursive(|p| {
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

            let variable = select! {
                Token::Identifier(name) => Expr::Variable(name.clone()),
            };

            let literal = float
                .or(integer)
                .or(variable)
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
                .foldr(power, |op, rhs| match op {
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

        binary_1.clone().foldl(
            just(Token::Plus)
                .or(just(Token::Minus))
                .then(binary_1)
                .repeated(),
            |lhs, (op, rhs)| match op {
                Token::Plus => Expr::Add(Box::new(lhs), Box::new(rhs)),
                Token::Minus => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            },
        )
    })
}

/// Parses declarations, assignments, and expression statements.
/// Multiple statements can be separated by semicolons.
pub fn statement_parser<'src>(
) -> impl Parser<
    'src,
    &'src [Token],
    Vec<Statement>,
    chumsky::extra::Err<Rich<'src, Token>>,
> {
    let identifier = select! {
        Token::Identifier(name) => name.clone(),
    };

    let type_annotation = select! {
        Token::I32 => Type::I32,
        Token::F64 => Type::F64,
    };

    let inferred_declaration = just(Token::Let)
        .ignore_then(just(Token::Mut).or_not())
        .then(identifier.clone())
        .then_ignore(just(Token::InferAssign))
        .then(expression_parser())
        .map(|((mutable, name), value)| Statement::Let {
            name,
            mutable: mutable.is_some(),
            annotation: None,
            value,
        });

    let explicit_declaration = just(Token::Let)
        .ignore_then(just(Token::Mut).or_not())
        .then(identifier.clone())
        .then_ignore(just(Token::Colon))
        .then(type_annotation)
        .then_ignore(just(Token::Assign))
        .then(expression_parser())
        .map(|(((mutable, name), annotation), value)| Statement::Let {
            name,
            mutable: mutable.is_some(),
            annotation: Some(annotation),
            value,
        });

    let assignment = identifier
        .then_ignore(just(Token::Assign))
        .then(expression_parser())
        .map(|(name, value)| Statement::Assign { name, value });

    let expression_statement =
        expression_parser().map(Statement::Expr);

    let statement = inferred_declaration
        .or(explicit_declaration)
        .or(assignment)
        .or(expression_statement);

    statement
        .separated_by(just(Token::Semicolon))
        .allow_trailing()
        .collect::<Vec<_>>()
        .then_ignore(end())
}