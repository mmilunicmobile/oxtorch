use std::{env, fs};

use string_interner::DefaultStringInterner;

use crate::{errors::ErrorPrinter, lexer::Lexer};

mod errors;
mod lexer;
mod parser;
mod typer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("requires 2 arguments");

    match fs::read_to_string(path) {
        Ok(contents) => {
            let mut interner: DefaultStringInterner = DefaultStringInterner::new();
            let (mut name, lexer_errors) =
                Lexer::new(&contents, |identifier| interner.get_or_intern(identifier));
            let error_printer = ErrorPrinter::new(path, &contents);
            let (expr, parser_errors) = match parser::expr(&mut name) {
                Ok(expr) => (expr, Vec::new()),
                Err(errors) => (Vec::new(), errors),
            };

            let all_errors: Vec<errors::CompilerError> = vec![lexer_errors, parser_errors]
                .into_iter()
                .flatten()
                .collect();

            if all_errors.is_empty() {
                println!("{:?}", expr);
            } else {
                error_printer.report(&all_errors);
            }
        }
        Err(err) => {
            panic!("Error {} reading file '{}'.", err, path);
        }
    }
}
