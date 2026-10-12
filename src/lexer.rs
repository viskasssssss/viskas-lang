use logos::Logos;
use std::ops::Range;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n]+")]
#[logos(error = String)]
pub enum Token {
    // Keywords and types.
    #[token("let")]
    Let,

    #[token("mut")]
    Mut,

    #[token("i32")]
    I32,

    #[token("f64")]
    F64,

    // Declaration and assignment operators.
    #[token(":=")]
    InferAssign,

    #[token(":")]
    Colon,

    #[token("=")]
    Assign,

    #[token(";")]
    Semicolon,

    // Arithmetic operators.
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Multiply,

    #[token("/")]
    Divide,

    #[token("^")]
    Power,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    // Numeric literals.
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().unwrap())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<isize>().unwrap())]
    Integer(isize),

    // Variable names.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub kind: Token,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub lexeme: String,
    pub span: Range<usize>,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unexpected character {:?} at position {}",
            self.lexeme, self.span.start
        )
    }
}

impl std::error::Error for LexError {}

pub struct Lexer;

impl Lexer {
    pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
        Token::lexer(source)
            .spanned()
            .map(|(result, span)| {
                result
                    .map(|kind| SpannedToken {
                        kind,
                        span: span.clone(),
                    })
                    .map_err(|_| LexError {
                        lexeme: source[span.clone()].to_owned(),
                        span,
                    })
            })
            .collect()
    }
}

pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    Lexer::lex(source)
}