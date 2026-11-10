use crate::{
    errors::CompilerError,
    lexer::{AnnotatedToken, Atom, Lexer, OpToken, Token},
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    BinOp(OpToken, Box<AnnotatedExpr>, Box<AnnotatedExpr>),
    PreOp(OpToken, Box<AnnotatedExpr>),
    PostOp(OpToken, Box<AnnotatedExpr>),
    Call(Box<AnnotatedExpr>, Vec<AnnotatedExpr>),
    Comprehension(
        Option<Box<AnnotatedExpr>>,
        Box<AnnotatedExpr>,
        Vec<AnnotatedExpr>,
    ),
    IndexExpr(Box<AnnotatedExpr>, Vec<AnnotatedExpr>),
    Identifier(String),
    Number(String),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AnnotatedExpr {
    expr: Expr,
    pos: (usize, usize),
}

type ParsingResult<T> = Result<T, Vec<CompilerError>>;

macro_rules! merge {
    ($( $expression:expr => $name:ident ),+ $(,)?) => {{
        let mut errors = Vec::new();

        $(
            let $name = match $expression {
                Ok(value) => Some(value),
                Err(mut error) => {
                    errors.append(&mut error);
                    None
                }
            };
        )+

        if errors.is_empty() {
            Ok((
                $(
                    match $name {
                        Some(value) => value,
                        None => unreachable!(),
                    }
                ),+
            ))
        } else {
            Err(errors)
        }
    }};
}

pub fn expr(lexer: &mut Lexer) -> ParsingResult<Vec<(AnnotatedExpr, AnnotatedExpr)>> {
    let mut out = Vec::new();

    let mut errors = Vec::new();

    loop {
        let lexer_next = lexer.next();
        match lexer_next.token {
            Token::Op(OpToken::Def) => {}
            Token::Eof => break,
            _ => {
                errors.push(CompilerError::PositionError(
                    "expected `def` keyword here".to_string(),
                    lexer_next.pos,
                ));
                escape_to(lexer, &[OpToken::Def]);
                continue;
            }
        }
        let signature = expr_bp(lexer, 0);

        if signature.is_err() {
            escape_to(lexer, &[OpToken::As, OpToken::Def]);
        }

        let lexer_next = lexer.next();
        match lexer_next.token {
            Token::Op(OpToken::As) => {}
            Token::Eof => {
                if let Err(mut a) = signature {
                    errors.append(&mut a);
                }
                errors.push(CompilerError::PositionError(
                    "expected `as` keyword here".to_string(),
                    lexer_next.pos,
                ));
                break;
            }
            _ => {
                if let Err(mut a) = signature {
                    errors.append(&mut a);
                }
                errors.push(CompilerError::PositionError(
                    "expected `as` keyword here".to_string(),
                    lexer_next.pos,
                ));
                escape_to(lexer, &[OpToken::Def, OpToken::As, OpToken::Semicolon]);
                continue;
            }
        }

        let body = expr_bp(lexer, 0);

        if signature.is_err() || body.is_err() {
            escape_to(lexer, &[OpToken::Def]);
        }

        match merge!(signature => a, body => b) {
            Ok(a) => out.push(a),
            Err(mut b) => errors.append(&mut b),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(out)
}

fn escape_to(lexer: &mut Lexer, token: &[OpToken]) {
    loop {
        match lexer.peek().token {
            Token::Eof => break,
            Token::Op(a) if token.contains(&a) => break,
            _ => {
                lexer.next();
            }
        }
    }
}

fn expect_symbol(token: AnnotatedToken, target: Token, message: &str) -> ParsingResult<()> {
    if token.token == target {
        return Ok(());
    }

    Err(vec![CompilerError::PositionError(
        message.to_string(),
        token.pos,
    )])
}

fn escape_tokens() -> &'static [OpToken] {
    &[
        OpToken::Def,
        OpToken::As,
        OpToken::Semicolon,
        OpToken::RBrace,
        OpToken::RBracket,
        OpToken::RParen,
        OpToken::Comma,
    ]
}

/// parse through the inside of a comprehension
fn expr_comprehension(
    lexer: &mut Lexer,
) -> ParsingResult<(AnnotatedExpr, Vec<AnnotatedExpr>, usize)> {
    let mhs = expr_bp(lexer, 0)?;
    let next_token = lexer.next();
    match next_token.token {
        Token::Op(OpToken::Arrow) => {
            let mut errors = Vec::new();
            let lbracket = expect_symbol(
                lexer.next(),
                Token::Op(OpToken::LBracket),
                "expected left bracket here",
            );
            if let Err(mut bracket_errors) = lbracket {
                errors.append(&mut bracket_errors);
                escape_to(
                    lexer,
                    &[
                        OpToken::RBracket,
                        OpToken::RBrace,
                        OpToken::Def,
                        OpToken::As,
                    ],
                );
            }
            let series = expr_commas(lexer, OpToken::RBracket);
            let indices = match series {
                Ok(indices) => Some(indices),
                Err(mut series_errors) => {
                    errors.append(&mut series_errors);
                    None
                }
            };
            if indices.is_none() {
                escape_to(
                    lexer,
                    &[
                        OpToken::RBracket,
                        OpToken::RBrace,
                        OpToken::Def,
                        OpToken::As,
                    ],
                );
            }

            if lexer.peek().token == Token::Op(OpToken::RBracket) {
                lexer.next();
            } else {
                let token = lexer.peek();
                errors.push(CompilerError::PositionError(
                    "expected right bracket here".to_string(),
                    token.pos,
                ));
            }

            let rbrace = if lexer.peek().token == Token::Op(OpToken::RBrace) {
                lexer.next()
            } else {
                let token = lexer.peek();
                errors.push(CompilerError::PositionError(
                    "expected end of comprehension here".to_string(),
                    token.pos,
                ));
                escape_to(
                    lexer,
                    &[
                        OpToken::RBrace,
                        OpToken::Def,
                        OpToken::As,
                        OpToken::Semicolon,
                    ],
                );
                lexer.peek()
            };

            if rbrace.token != Token::Op(OpToken::RBrace) {
                return Err(errors);
            }
            if !errors.is_empty() {
                return Err(errors);
            }
            let indices = match indices {
                Some(indices) => indices,
                None => return Err(errors),
            };
            Ok((mhs, indices, rbrace.pos.1))
        }
        Token::Op(OpToken::RBrace) => Ok((mhs, Vec::new(), next_token.pos.1)),
        _ => {
            escape_to(
                lexer,
                &[
                    OpToken::RBrace,
                    OpToken::Def,
                    OpToken::As,
                    OpToken::Semicolon,
                ],
            );
            Err(vec![CompilerError::PositionError(
                "expected `->` or `}` here".to_string(),
                next_token.pos,
            )])
        }
    }
}

/// parse through the inside of something full of commas
fn expr_commas(lexer: &mut Lexer, end: OpToken) -> ParsingResult<Vec<AnnotatedExpr>> {
    let mut out = Vec::new();

    let mut errors = Vec::new();

    if Token::Op(end) == lexer.peek().token {
        return Ok(out);
    }
    match expr_bp(lexer, 0) {
        Ok(a) => out.push(a),
        Err(mut e) => {
            errors.append(&mut e);
            escape_to(lexer, &[end, OpToken::Comma]);
        }
    }

    while let Token::Op(OpToken::Comma) = lexer.peek().token {
        lexer.next();
        if Token::Op(end) == lexer.peek().token {
            if errors.is_empty() {
                return Ok(out);
            } else {
                return Err(errors);
            }
        }

        match expr_bp(lexer, 0) {
            Ok(a) => out.push(a),
            Err(mut e) => {
                errors.append(&mut e);
                escape_to(lexer, &[end, OpToken::Comma]);
            }
        }
    }
    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors)
    }
}

fn expr_bp(lexer: &mut Lexer, min_bp: u8) -> ParsingResult<AnnotatedExpr> {
    let next_token = lexer.peek();
    if matches!(
        next_token.token,
        Token::Eof
            | Token::Op(
                OpToken::Def
                    | OpToken::As
                    | OpToken::Semicolon
                    | OpToken::Comma
                    | OpToken::Arrow
                    | OpToken::RBrace
                    | OpToken::RBracket
                    | OpToken::RParen
            )
    ) {
        return Err(vec![CompilerError::PositionError(
            "expected expression here".to_string(),
            next_token.pos,
        )]);
    }

    let next_tok = lexer.next();
    let pos = next_tok.pos;

    let mut lhs = match next_tok.token {
        Token::Atom(Atom::Identifier(s)) => AnnotatedExpr {
            expr: Expr::Identifier(s),
            pos,
        },
        Token::Atom(Atom::Number(s)) => AnnotatedExpr {
            expr: Expr::Number(s),
            pos,
        },
        Token::Op(OpToken::LParen) => {
            let lhs = expr_bp(lexer, 0);
            let lhs = match lhs {
                Ok(lhs) => lhs,
                Err(mut errors) => {
                    escape_to(
                        lexer,
                        &[
                            OpToken::Semicolon,
                            OpToken::RParen,
                            OpToken::Def,
                            OpToken::As,
                        ],
                    );

                    while lexer.peek().token == Token::Op(OpToken::Semicolon) {
                        lexer.next();
                        match expr_bp(lexer, 0) {
                            Ok(_) => {}
                            Err(mut more_errors) => {
                                errors.append(&mut more_errors);
                                escape_to(
                                    lexer,
                                    &[
                                        OpToken::Semicolon,
                                        OpToken::RParen,
                                        OpToken::Def,
                                        OpToken::As,
                                    ],
                                );
                            }
                        }
                    }

                    if lexer.peek().token != Token::Op(OpToken::RParen) {
                        let token = lexer.peek();
                        errors.push(CompilerError::PositionError(
                            "expected right parenthesis here".to_string(),
                            token.pos,
                        ));
                    } else {
                        lexer.next();
                    }

                    return Err(errors);
                }
            };
            let rparen = lexer.next();
            if rparen.token != Token::Op(OpToken::RParen) {
                escape_to(
                    lexer,
                    &[
                        OpToken::RParen,
                        OpToken::Def,
                        OpToken::As,
                        OpToken::Semicolon,
                    ],
                );
                return Err(vec![CompilerError::PositionError(
                    "expected right parenthesis here".to_string(),
                    rparen.pos,
                )]);
            }
            AnnotatedExpr {
                expr: lhs.expr,
                pos: (pos.0, rparen.pos.1),
            }
        }
        Token::Op(OpToken::LBrace) => {
            let (mhs, indices, end) = expr_comprehension(lexer)?;
            let lhs = Expr::Comprehension(None, Box::new(mhs), indices);
            AnnotatedExpr {
                expr: lhs,
                pos: (pos.0, end),
            }
        }
        Token::Op(s) => {
            let (_, bp) = match prefix_binding_power(s) {
                Some(binding_power) => binding_power,
                None => {
                    escape_to(lexer, escape_tokens());
                    return Err(vec![CompilerError::PositionError(
                        "expected expression here".to_string(),
                        pos,
                    )]);
                }
            };
            let lhs = expr_bp(lexer, bp)?;
            let new_pos = (pos.0, lhs.pos.1);
            AnnotatedExpr {
                expr: Expr::PreOp(s, Box::new(lhs)),
                pos: new_pos,
            }
        }
        Token::Eof => {
            return Err(vec![CompilerError::PositionError(
                "expected expression here".to_string(),
                pos,
            )]);
        }
    };

    loop {
        let op_token = lexer.peek();
        let op = match op_token.token {
            Token::Eof => break,
            Token::Op(op) => op,
            _ => {
                return Err(vec![CompilerError::PositionError(
                    "expected operator here".to_string(),
                    op_token.pos,
                )]);
            }
        };

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();

            let lhs_pos = lhs.pos;

            lhs = if op == OpToken::LBracket {
                let indices = expr_commas(lexer, OpToken::RBracket);
                let last_token = lexer.next();
                if last_token.token != Token::Op(OpToken::RBracket) {
                    escape_to(lexer, escape_tokens());
                    return Err(vec![CompilerError::PositionError(
                        "expected right bracket here".to_string(),
                        last_token.pos,
                    )]);
                }
                let indices = indices?;
                let indexpr = Expr::IndexExpr(Box::new(lhs), indices);
                AnnotatedExpr {
                    expr: indexpr,
                    pos: (lhs_pos.0, last_token.pos.1),
                }
            } else if op == OpToken::LParen {
                let parameters = expr_commas(lexer, OpToken::RParen);
                let last_token = lexer.next();
                if last_token.token != Token::Op(OpToken::RParen) {
                    escape_to(lexer, escape_tokens());
                    return Err(vec![CompilerError::PositionError(
                        "expected right parenthesis here".to_string(),
                        last_token.pos,
                    )]);
                }
                let parameters = parameters?;
                let callexpr = Expr::Call(Box::new(lhs), parameters);
                AnnotatedExpr {
                    expr: callexpr,
                    pos: (lhs_pos.0, last_token.pos.1),
                }
            } else if op == OpToken::LBrace {
                let (mhs, indices, end) = expr_comprehension(lexer)?;
                AnnotatedExpr {
                    expr: Expr::Comprehension(Some(Box::new(lhs)), Box::new(mhs), indices),
                    pos: (lhs_pos.0, end),
                }
            } else {
                AnnotatedExpr {
                    expr: Expr::PostOp(op, Box::new(lhs)),
                    pos: (lhs_pos.0, op_token.pos.1),
                }
            };

            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            let rhs = expr_bp(lexer, r_bp)?;
            let pos = (lhs.pos.0, rhs.pos.1);
            lhs = AnnotatedExpr {
                expr: Expr::BinOp(op, Box::new(lhs), Box::new(rhs)),
                pos,
            };
            continue;
        }

        break;
    }

    Ok(lhs)
}

fn postfix_binding_power(op: OpToken) -> Option<(u8, ())> {
    use OpToken::*;
    match op {
        LBracket => Some((31, ())),
        LParen => Some((31, ())),
        LBrace => Some((31, ())),
        _ => None,
    }
}

fn prefix_binding_power(op: OpToken) -> Option<((), u8)> {
    use OpToken::*;
    match op {
        Plus | Minus | Bang => Some(((), 21)),
        _ => None,
    }
}

fn infix_binding_power(op: OpToken) -> Option<(u8, u8)> {
    use OpToken::*;
    Some(match op {
        Semicolon => (3, 4),
        Assign => (5, 6),
        Colon => (7, 24),
        Or => (9, 10),
        And => (11, 12),
        Eq | Neq | Leq | Geq | Lt | Gt => (13, 14),
        Plus | Minus | At => (15, 16),
        Times | Div => (17, 18),
        _ => return None,
    })
}
