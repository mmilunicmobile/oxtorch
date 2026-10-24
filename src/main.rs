use std::io;

use crate::lexer::{Lexer, Token};

mod lexer;

fn main() {
    println!("Enter your string:");

    // Create a mutable string to store the input
    let mut input = String::new();

    // Read the line from standard input
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Trim the trailing newline character
    let mut name = Lexer::new(&input);

    loop {
        let element = name.next();
        println!("{:?}", element);
        if let Token::Eof = element {
            break;
        }
    }
}
