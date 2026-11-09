use crate::errors::CompilerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Atom(Atom),
    Op(OpToken),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedToken {
    pub token: Token,
    pub pos: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpToken {
    Def,

    Plus,
    Minus,
    Times,
    Div,
    At,

    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,

    Eq,
    Neq,

    Leq,
    Lt,
    Geq,
    Gt,

    Colon,
    Comma,

    Assign,
    Arrow,

    Bang,
    And,
    Or,

    Semicolon,
    As,
}

const OPERATOR_SPELLINGS: &[&str] = &[
    "==", "!=", "->", "&&", "||", "+", "-", "*", "/", "@", "{", "}", "[", "]", "(", ")", ":", ",",
    "=", "!", ";", "<=", "<", ">=", ">",
];

const MAX_OPERATOR_LEN: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    Identifier(String),
    Number(String),
}

pub struct Lexer {
    tokens: Vec<AnnotatedToken>,
    start_len: usize,
}

impl Lexer {
    pub fn new(input: &str) -> (Self, Vec<CompilerError>) {
        let mut letters: Vec<(usize, char)> = input.chars().enumerate().collect();
        letters.reverse();

        let mut errors = Vec::new();

        let start_len = letters.len();
        let mut tokens = Vec::new();

        while let Some((_, a)) = letters.last() {
            let a = *a;
            if a.is_whitespace() {
                Lexer::take_whitespace(&mut letters);
                continue;
            }
            let maybe_token = fun_name(&mut letters, start_len, a);
            match maybe_token {
                Ok(a) => tokens.push(a),
                Err(e) => {
                    errors.push(e);
                    letters.pop();
                }
            }
        }

        tokens.reverse();

        (Lexer { tokens, start_len }, errors)
    }

    pub fn next(&mut self) -> AnnotatedToken {
        self.tokens.pop().unwrap_or(AnnotatedToken {
            token: Token::Eof,
            pos: (self.start_len, self.start_len),
        })
    }

    pub fn peek(&mut self) -> AnnotatedToken {
        self.tokens.last().cloned().unwrap_or(AnnotatedToken {
            token: Token::Eof,
            pos: (self.start_len, self.start_len),
        })
    }

    /// Takes until it runs into an ending of the word
    fn take_word(letters: &mut Vec<(usize, char)>) -> Result<String, CompilerError> {
        enum ParsingState {
            Start,
            Rest,
        }

        let mut out = Vec::new();
        let mut state = ParsingState::Start;

        while let Some((a_position, a)) = letters.last() {
            state = match state {
                ParsingState::Start if a.is_alphabetic() || *a == '_' => ParsingState::Rest,
                ParsingState::Rest if a.is_alphanumeric() || *a == '_' => ParsingState::Rest,
                ParsingState::Start => {
                    return Err(CompilerError::PositionError(
                        "expected an identifier here".to_string(),
                        (*a_position, *a_position + 1),
                    ));
                }
                _ => {
                    break;
                }
            };

            out.push(*a);
            letters.pop();
        }

        Ok(out.iter().collect())
    }

    /// Takes until it runs into an ending of the number, or returns nothing if no valid number
    fn take_number(letters: &mut Vec<(usize, char)>) -> Result<String, CompilerError> {
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

        while let Some((a_position, a)) = letters.last() {
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
                    return Err(CompilerError::PositionError(
                        "invalid character detected here when parsing number".to_string(),
                        (*a_position, *a_position + 1),
                    ));
                }
                (
                    _,
                    ParsingState::Start | ParsingState::MantissaSign | ParsingState::MantissaStart,
                ) => {
                    return Err(CompilerError::PositionError(
                        "expected a digit when parsing number".to_string(),
                        (*a_position, *a_position + 1),
                    ));
                }
                (_, _) => {
                    break;
                }
            };
            out.push(*a);
            letters.pop();
        }

        Ok(out.iter().collect())
    }

    fn take_operator(letters: &mut Vec<(usize, char)>) -> Result<OpToken, CompilerError> {
        let double_pattern: String = letters
            .iter()
            .rev()
            .map(|(_, b)| b)
            .take(MAX_OPERATOR_LEN)
            .collect();

        let spelling = OPERATOR_SPELLINGS
            .iter()
            .filter(|operator| double_pattern.starts_with(*operator))
            .max_by_key(|operator| operator.len())
            .copied()
            .ok_or_else(|| {
                let idx = letters
                    .last()
                    .expect("should not be calling take_operator on empty `letters`")
                    .0;
                CompilerError::PositionError(
                    "this is not a valid operator".to_string(),
                    (idx, idx + 1),
                )
            })?;

        for _ in spelling.chars() {
            letters.pop();
        }

        Ok(match spelling {
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
        })
    }

    fn take_whitespace(letters: &mut Vec<(usize, char)>) {
        while letters
            .last()
            .map(|(_, c)| c.is_whitespace())
            .unwrap_or(false)
        {
            letters.pop();
        }
    }
}

fn fun_name(
    letters: &mut Vec<(usize, char)>,
    start_len: usize,
    a: char,
) -> Result<AnnotatedToken, CompilerError> {
    let start = start_len - letters.len();
    let token = match a {
        '0'..='9' => Token::Atom(Atom::Number(Lexer::take_number(letters)?)),
        _ if a.is_alphabetic() || a == '_' => {
            let w = Lexer::take_word(letters)?;
            match w.as_str() {
                "def" => Token::Op(OpToken::Def),
                "as" => Token::Op(OpToken::As),
                _ => Token::Atom(Atom::Identifier(w)),
            }
        }
        _ => {
            let w = Lexer::take_operator(letters)?;
            Token::Op(w)
        }
    };
    let end = start_len - letters.len();
    Ok(AnnotatedToken {
        token,
        pos: (start, end),
    })
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    #[test]
    fn test_lexer_integer() {
        let mut number: Vec<(usize, char)> = "5-".chars().enumerate().collect();
        number.reverse();
        assert_eq!(Lexer::take_number(&mut number), Ok(String::from("5")));
    }

    #[test]
    fn test_lexer_float() {
        let mut number: Vec<(usize, char)> = "532.15e2+4".chars().enumerate().collect();
        number.reverse();
        assert_eq!(
            Lexer::take_number(&mut number),
            Ok(String::from("532.15e2"))
        );
    }

    #[test]
    fn test_lexer_panic() {
        let mut number: Vec<(usize, char)> = "24623.12521.325*2".chars().enumerate().collect();
        number.reverse();
        assert!(Lexer::take_number(&mut number).is_err());
    }

    #[test]
    fn test_lexer_word() {
        let mut word: Vec<(usize, char)> = "hi_5_542^34".chars().enumerate().collect();
        word.reverse();
        assert_eq!(Lexer::take_word(&mut word), Ok(String::from("hi_5_542")));
    }

    #[test]
    fn test_lexer_word_long() {
        let mut word: Vec<(usize, char)> = "hello_world*aefea".chars().enumerate().collect();
        word.reverse();
        assert_eq!(Lexer::take_word(&mut word), Ok(String::from("hello_world")));
    }

    #[test]
    #[should_panic = "tried to take invalid word"]
    fn test_lexer_word_panic() {
        let mut word: Vec<(usize, char)> = "231*4".chars().enumerate().collect();
        word.reverse();
        Lexer::take_word(&mut word);
    }
}
