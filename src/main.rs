use std::{env, fs};

use crate::lexer::Lexer;

mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("requires 2 arguments");

    match fs::read_to_string(path) {
        Ok(contents) => {
            let mut name = Lexer::new(&contents);
            let expr = parser::expr_bp(&mut name, 0);

            println!("{:?}", expr);
        }
        Err(err) => {
            panic!("Error {} reading file '{}'.", err, path);
        }
    }
}
