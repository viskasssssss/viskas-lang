mod diagnostics;
mod lexer;
mod parser;
mod value;
mod type_checker;
mod evaluator;

use std::io::{self, Write};

use chumsky::Parser;

fn main() {
    println!("Welcome! Enter 'exit' to exit.");

    // Keep both environments alive for the entire REPL session.
    let mut checker = type_checker::TypeChecker::new();
    let mut runtime = evaluator::RuntimeEnv::new();

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

                execute(command, &mut checker, &mut runtime);
            }
            Err(error) => {
                eprintln!("Input read error: {error}");
                break;
            }
        }
    }
}

fn execute(
    source: &str,
    checker: &mut type_checker::TypeChecker,
    runtime: &mut evaluator::RuntimeEnv,
) {
    let tokens = match lexer::lex(source) {
        Ok(tokens) => tokens,
        Err(error) => {
            diagnostics::render_lex_error(source, &error);
            return;
        }
    };

    // Keep the original spanned tokens for diagnostics.
    let plain_tokens: Vec<lexer::Token> =
        tokens.iter().map(|token| token.kind.clone()).collect();

    let statements = match parser::statement_parser()
        .parse(&plain_tokens)
        .into_result()
    {
        Ok(statements) => {
            //println!("[AST]\n{:#?}", statements);
            statements
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

    // Check a copy first so failed checks don't modify persistent state.
    let mut candidate_checker = checker.clone();

    if let Err(error) = candidate_checker.check_program(&statements) {
        eprintln!("Type error: {error}");
        return;
    }

    // Execute against a copy so runtime errors don't commit partial changes.
    let mut candidate_runtime = runtime.clone();

    match evaluator::execute_program(
        &statements,
        &mut candidate_runtime,
    ) {
        Ok(result) => {
            // Commit both environments only after successful execution.
            *checker = candidate_checker;
            *runtime = candidate_runtime;

            if let Some(value) = result {
                println!("{value}");
            }
        }
        Err(error) => {
            eprintln!("Runtime error: {error}");
        }
    }
}