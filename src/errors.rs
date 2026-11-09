#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    PositionError(String, (usize, usize)),
}

pub struct ErrorPrinter<'a> {
    path: &'a str,
    codebase: &'a str,
}

impl<'a> ErrorPrinter<'a> {
    pub fn new(path: &'a str, codebase: &'a str) -> Self {
        Self { path, codebase }
    }

    fn report_one(&self, err: &CompilerError) {
        match err {
            CompilerError::PositionError(message, pos) => {
                let line_and_column = |offset: usize| {
                    let mut line = 0;
                    let mut line_start = 0;

                    for (index, character) in self.codebase.chars().enumerate().take(offset) {
                        if character == '\n' {
                            line += 1;
                            line_start = index + 1;
                        }
                    }

                    (line, offset - line_start)
                };

                let (start_line, start_offset) = line_and_column(pos.0);
                let (end_line, end_offset) = if pos.1 > pos.0 {
                    let (line, column) = line_and_column(pos.1 - 1);
                    (line, column + 1)
                } else {
                    (start_line, start_offset)
                };

                let lines: Vec<(usize, &str)> = self
                    .codebase
                    .split('\n')
                    .enumerate()
                    .skip(start_line)
                    .take(end_line - start_line + 1)
                    .collect();

                println!("error: {}", message);
                println!(
                    "  --> {}:{}:{}",
                    self.path,
                    start_line + 1,
                    start_offset + 1
                );

                for (i, s) in lines {
                    println!("{:>5}| {s}", i + 1);
                    let start = if i == start_line { start_offset } else { 0 };
                    let end = if i == end_line { end_offset } else { s.len() };
                    print!("     | ");
                    for _ in 0..start {
                        print!(" ");
                    }
                    for _ in start..end {
                        print!("^")
                    }
                    println!();
                }
            }
        }
    }

    pub fn report(&self, errors: &[CompilerError]) {
        for err in errors {
            self.report_one(err);
            println!();
        }
        let err_count = errors.len();
        let plural_err = if err_count == 1 { "" } else { "s" };
        println!(
            "error: could not compile `{}` due to {err_count} previous error{plural_err}",
            self.path
        );
    }
}
