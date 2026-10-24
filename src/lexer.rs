#[derive(Clone, Debug)]
pub enum Token {
    Atom(Atom),
    Op(String),
    Eof,
}

#[derive(Clone, Debug)]
pub enum Atom {
    Identifier(String),
    Number(String),
}

pub struct Lexer {
    tokens: Vec<Token>,
}

pub const OPERATORS: &[&str] = &[
    "fun", "+", "-", "*", "/", "@", "{", "}", "[", "]", "(", ")", "<", "<=", ">", ">=", "==", "!=",
    ":", ",", "=", "->", "!", "&&", "||", "&", "|", "^", "~"
];

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut letters: Vec<char> = input.chars().rev().collect();
        let mut tokens = Vec::new();

        while let Some(a) = letters.last() {
            match a {
                '0'..='9' => {
                    tokens.push(Token::Atom(Atom::Number(Lexer::take_number(&mut letters))));
                }
                _ if a.is_alphabetic() || *a == '_' => {
                    let w = Lexer::take_word(&mut letters);
                    tokens.push(if OPERATORS.contains(&w.as_str()) {
                        Token::Op(w)
                    } else {
                        Token::Atom(Atom::Identifier(w))
                    });
                }
                _ if a.is_whitespace() => {
                    Lexer::take_whitespace(&mut letters);
                }
                _ => {
                    let w = Lexer::take_operator(&mut letters);
                    if !OPERATORS.contains(&w.as_str()) {
                        panic!("tried to take invalid operator")
                    }
                    tokens.push(Token::Op(w));
                }
            }
        }

        tokens.reverse();

        Lexer { tokens }
    }

    pub fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }

    pub fn peek(&mut self) -> Token {
        self.tokens.last().cloned().unwrap_or(Token::Eof)
    }

    /// Takes until it runs into an ending of the word
    fn take_word(letters: &mut Vec<char>) -> String {
        enum ParsingState {
            Start,
            Rest,
        }

        let mut out = Vec::new();
        let mut state = ParsingState::Start;

        while let Some(a) = letters.last() {
            state = match state {
                ParsingState::Start if a.is_alphabetic() || *a == '_' => ParsingState::Rest,
                ParsingState::Rest if a.is_alphanumeric() || *a == '_' => ParsingState::Rest,
                tmp => {
                    state = tmp;
                    break;
                }
            };

            out.push(*a);
            letters.pop();
        }

        if let ParsingState::Start = state {
            panic!("tried to take invalid word");
        }

        out.iter().collect()
    }

    /// Takes until it runs into an ending of the number, or returns nothing if no valid number
    fn take_number(letters: &mut Vec<char>) -> String {
        enum ParsingState {
            Start,
            Zero,
            Nonzero,
            Decimal,
            MantissaSign,
            MantissaStart,
            Mantissa,
        }

        let mut out = Vec::new();

        let mut state = ParsingState::Start;

        while let Some(a) = letters.last() {
            if a.is_whitespace() {
                break;
            }
            state = match (a, state) {
                ('0', ParsingState::Start) => ParsingState::Zero,
                ('1'..='9', ParsingState::Start) => ParsingState::Nonzero,
                ('0'..='9', ParsingState::Nonzero) => ParsingState::Nonzero,
                ('.', ParsingState::Start | ParsingState::Zero | ParsingState::Nonzero) => {
                    ParsingState::Decimal
                }
                ('0'..='9', ParsingState::Decimal) => ParsingState::Decimal,
                ('e' | 'E', ParsingState::Zero | ParsingState::Nonzero | ParsingState::Decimal) => {
                    ParsingState::MantissaSign
                }
                ('+' | '-', ParsingState::MantissaSign) => ParsingState::MantissaStart,
                (
                    '0'..='9',
                    ParsingState::Mantissa
                    | ParsingState::MantissaStart
                    | ParsingState::MantissaSign,
                ) => ParsingState::Mantissa,
                ('.' | 'e', _) => {
                    panic!("tried to take invalid number");
                }
                (_, tmp) => {
                    state = tmp;
                    break;
                }
            };
            out.push(*a);
            letters.pop();
        }

        if let ParsingState::Start | ParsingState::MantissaSign | ParsingState::MantissaStart =
            state
        {
            panic!("tried to take invalid number");
        }

        out.iter().collect()
    }

    fn take_operator(letters: &mut Vec<char>) -> String {
        let next = letters.last().unwrap_or(&' ');
        let next_next = letters.get(letters.len() - 2).unwrap_or(&' ');

        let double_pattern = match (next, next_next) {
            ('=', '=') => String::from("=="),
            ('!', '=') => String::from("!="),
            ('-', '>') => String::from("->"),
            ('<', '=') => String::from("<="),
            ('>', '=') => String::from(">="),
            (a, _) if a.is_whitespace() => panic!("tried to take invalid operator"),
            (a, _) => a.to_string(),
        };

        for _ in 0..double_pattern.len() {
            letters.pop();
        }

        double_pattern
    }

    fn take_whitespace(letters: &mut Vec<char>) {
        while letters.last().map(|c| c.is_whitespace()).unwrap_or(false) {
            letters.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    #[test]
    fn test_lexer_integer() {
        let mut number = "5-".chars().rev().collect();
        assert_eq!(Lexer::take_number(&mut number), String::from("5"));
    }

    #[test]
    fn test_lexer_float() {
        let mut number = "532.15e2+4".chars().rev().collect();
        assert_eq!(Lexer::take_number(&mut number), String::from("532.15e2"));
    }

    #[test]
    #[should_panic = "tried to take invalid number"]
    fn test_lexer_panic() {
        let mut number = "24623.12521.325*2".chars().rev().collect();
        Lexer::take_number(&mut number);
    }

    #[test]
    fn test_lexer_word() {
        let mut word = "hi_5_542^34".chars().rev().collect();
        assert_eq!(Lexer::take_word(&mut word), String::from("hi_5_542"));
    }

    #[test]
    fn test_lexer_word_long() {
        let mut word = "hello_world*aefea".chars().rev().collect();
        assert_eq!(Lexer::take_word(&mut word), String::from("hello_world"));
    }

    #[test]
    #[should_panic = "tried to take invalid word"]
    fn test_lexer_word_panic() {
        let mut word = "231*4".chars().rev().collect();
        Lexer::take_word(&mut word);
    }
}
