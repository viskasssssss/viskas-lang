mod diagnostics;
mod lexer;
mod parser;
mod evaluator;

use std::io::{self, Write};

use chumsky::Parser;

fn main() {
    println!("Welcome! Enter 'exit' to exit.");

    loop {
        print!("> ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("io::stdout().flush error: {e}");
            break;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = input.trim();

                if command == "exit" {
                    println!("Bye");
                    break;
                }

                if command.is_empty() {
                    continue;
                }

                execute(command);
            }
            Err(error) => {
                eprintln!("Input read error: {error}");
                break;
            }
        }
    }
}


fn execute(source: &str) {
    let tokens = match lexer::lex(source) {
        Ok(tokens) => tokens,
        Err(error) => {
            diagnostics::render_lex_error(source, &error);
            return;
        }
    };

    for token in &tokens {
        println!("{:?}", token);
    }

    // Keep the original spanned tokens for diagnostics.
    let plain_tokens: Vec<lexer::Token> =
        tokens.iter().map(|token| token.kind.clone()).collect();

    let ast = match parser::parser()
        .parse(&plain_tokens)
        .into_result()
    {
        Ok(expr) => {
            println!("[AST]\n{:#?}", expr);
            expr
        }
        Err(errors) => {
            for error in &errors {
                diagnostics::render_parse_error(
                    source,
                    &tokens,
                    error,
                );
            }
            return;
        }
    };

    println!("\n[result]\n{}", evaluator::eval(&ast));
}
