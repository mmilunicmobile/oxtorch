#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Atom(Atom),
    Op(OpToken),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedToken {
    pub token: Token,
    pub pos: (usize, usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpToken {
    Def,

    Plus,
    Minus,
    Times,
    Div,
    At,

    LBrace, RBrace,
    LBracket, RBracket,
    LParen, RParen,

    Eq, Neq,

    Leq, Lt, Geq, Gt,

    Colon,
    Comma,

    Assign,
    Arrow,

    Bang,
    And, Or,

    Semicolon,
    As
}

const OPERATOR_SPELLINGS: &[&str] = &[
    "==", "!=", "->", "&&", "||",
    "+", "-", "*", "/", "@",
    "{", "}", "[", "]", "(", ")",
    ":", ",", "=", "!", ";",
    "<=", "<", ">=", ">",
];

const MAX_OPERATOR_LEN: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    Identifier(String),
    Number(String),
}

pub struct Lexer {
    tokens: Vec<AnnotatedToken>,
    start_len: usize
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut letters: Vec<char> = input.chars().rev().collect();
        let start_len = letters.len();
        let mut tokens = Vec::new();

        while let Some(a) = letters.last() {
            let start = start_len - letters.len();
            let token = match a {
                '0'..='9' => {
                    Token::Atom(Atom::Number(Lexer::take_number(&mut letters)))
                }
                _ if a.is_alphabetic() || *a == '_' => {
                    let w = Lexer::take_word(&mut letters);
                    match w.as_str() {
                         "def" => Token::Op(OpToken::Def),
                         "as" => Token::Op(OpToken::As),
                         _ => Token::Atom(Atom::Identifier(w))
                    }
                }
                _ if a.is_whitespace() => {
                    Lexer::take_whitespace(&mut letters);
                    continue;
                }
                _ => {
                    let w = Lexer::take_operator(&mut letters);
                    Token::Op(w)
                }
            };
            let end = start_len - letters.len();
            tokens.push(AnnotatedToken {token, pos: (start, end)});
        }

        tokens.reverse();

        Lexer { tokens, start_len }
    }

    pub fn next(&mut self) -> AnnotatedToken {
        self.tokens.pop().unwrap_or(AnnotatedToken { token: Token::Eof, pos: (self.start_len, self.start_len) } )
    }

    pub fn peek(&mut self) -> AnnotatedToken {
        self.tokens.last().cloned().unwrap_or(AnnotatedToken { token: Token::Eof, pos: (self.start_len, self.start_len) } )
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

    fn take_operator(letters: &mut Vec<char>) -> OpToken {
        let double_pattern: String = letters.iter().rev().take(MAX_OPERATOR_LEN).collect();

        let spelling = OPERATOR_SPELLINGS
            .iter()
            .filter(|operator| double_pattern.starts_with(*operator))
            .max_by_key(|operator| operator.len())
            .copied()
            .expect("tried to take invalid operator");

        for _ in spelling.chars() {
            letters.pop();
        }

        match spelling {
            "==" => OpToken::Eq,
            "!=" => OpToken::Neq,
            "<=" => OpToken::Leq,
            ">=" => OpToken::Geq,
            "<" => OpToken::Lt,
            ">" => OpToken::Gt,
            "->" => OpToken::Arrow,
            "&&" => OpToken::And,
            "||" => OpToken::Or,
            "+" => OpToken::Plus,
            "-" => OpToken::Minus,
            "*" => OpToken::Times,
            "/" => OpToken::Div,
            "@" => OpToken::At,
            "{" => OpToken::LBrace,
            "}" => OpToken::RBrace,
            "[" => OpToken::LBracket,
            "]" => OpToken::RBracket,
            "(" => OpToken::LParen,
            ")" => OpToken::RParen,
            ":" => OpToken::Colon,
            "," => OpToken::Comma,
            "=" => OpToken::Assign,
            "!" => OpToken::Bang,
            ";" => OpToken::Semicolon,
            _ => unreachable!("operator spelling has no token"),
        }
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
