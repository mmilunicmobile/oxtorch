use crate::{
    errors::CompilerError,
    lexer::{Atom, Lexer, OpToken, Token},
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

pub fn expr(lexer: &mut Lexer) -> (Vec<(AnnotatedExpr, AnnotatedExpr)>, Vec<CompilerError>) {
    let mut out = Vec::new();

    loop {
        match lexer.next().token {
            Token::Op(OpToken::Def) => {}
            Token::Eof => break,
            _ => panic!("expected function definition"),
        }
        let signature = expr_bp(lexer, 0);

        match lexer.next().token {
            Token::Op(OpToken::As) => {}
            Token::Eof => break,
            _ => panic!("expected function definition"),
        }

        let body = expr_bp(lexer, 0);

        out.push((signature, body));
    }
    (out, Vec::new())
}

/// parse through the inside of a comprehension
fn expr_comprehension(lexer: &mut Lexer) -> (AnnotatedExpr, Vec<AnnotatedExpr>, usize) {
    let mhs = expr_bp(lexer, 0);
    let next_token = lexer.next();
    match next_token.token {
        Token::Op(OpToken::Arrow) => {
            assert_eq!(
                lexer.next().token,
                Token::Op(OpToken::LBracket),
                "expected bracket"
            );
            let series = expr_commas(lexer, Token::Op(OpToken::RBracket));
            assert_eq!(
                lexer.next().token,
                Token::Op(OpToken::RBracket),
                "expected bracket"
            );
            let rbrace = lexer.next();
            assert_eq!(
                rbrace.token,
                Token::Op(OpToken::RBrace),
                "expected end of comprehension"
            );
            (mhs, series, rbrace.pos.1)
        }
        Token::Op(OpToken::RBrace) => (mhs, Vec::new(), next_token.pos.1),
        _ => panic!("expected end of comprehension"),
    }
}

/// parse through the inside of something full of commas
fn expr_commas(lexer: &mut Lexer, end: Token) -> Vec<AnnotatedExpr> {
    let mut out = Vec::new();
    if end == lexer.peek().token {
        return out;
    }
    out.push(expr_bp(lexer, 0));
    while let Token::Op(OpToken::Comma) = lexer.peek().token {
        lexer.next();
        if end == lexer.peek().token {
            return out;
        }
        out.push(expr_bp(lexer, 0));
    }
    out
}

fn expr_bp(lexer: &mut Lexer, min_bp: u8) -> AnnotatedExpr {
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
            let rparen = lexer.next();
            assert_eq!(
                rparen.token,
                Token::Op(OpToken::RParen),
                "expected parenthesis"
            );
            AnnotatedExpr {
                expr: lhs.expr,
                pos: (pos.0, rparen.pos.1),
            }
        }
        Token::Op(OpToken::LBrace) => {
            let (mhs, indices, end) = expr_comprehension(lexer);
            let lhs = Expr::Comprehension(None, Box::new(mhs), indices);
            AnnotatedExpr {
                expr: lhs,
                pos: (pos.0, end),
            }
        }
        Token::Op(s) => {
            let ((), bp) = prefix_binding_power(s);
            let lhs = expr_bp(lexer, bp);
            let new_pos = (pos.0, lhs.pos.1);
            AnnotatedExpr {
                expr: Expr::PreOp(s, Box::new(lhs)),
                pos: new_pos,
            }
        }
        Token::Eof => panic!("expected expression"),
    };

    loop {
        let op_token = lexer.peek();
        let op = match op_token.token {
            Token::Eof => break,
            Token::Op(op) => op,
            _ => panic!("expected operator"),
        };

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();

            let lhs_pos = lhs.pos;

            lhs = if op == OpToken::LBracket {
                let indices = expr_commas(lexer, Token::Op(OpToken::RBracket));
                let last_token = lexer.next();
                assert_eq!(
                    last_token.token,
                    Token::Op(OpToken::RBracket),
                    "expected bracket"
                );
                let indexpr = Expr::IndexExpr(Box::new(lhs), indices);
                AnnotatedExpr {
                    expr: indexpr,
                    pos: (lhs_pos.0, last_token.pos.1),
                }
            } else if op == OpToken::LParen {
                let parameters = expr_commas(lexer, Token::Op(OpToken::RParen));
                let last_token = lexer.next();
                assert_eq!(
                    last_token.token,
                    Token::Op(OpToken::RParen),
                    "expected parenthesis"
                );
                let callexpr = Expr::Call(Box::new(lhs), parameters);
                AnnotatedExpr {
                    expr: callexpr,
                    pos: (lhs_pos.0, last_token.pos.1),
                }
            } else if op == OpToken::LBrace {
                let (mhs, indices, end) = expr_comprehension(lexer);
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
            let rhs = expr_bp(lexer, r_bp);
            let pos = (lhs.pos.0, rhs.pos.1);
            lhs = AnnotatedExpr {
                expr: Expr::BinOp(op, Box::new(lhs), Box::new(rhs)),
                pos,
            };
            continue;
        }

        break;
    }

    lhs
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

fn prefix_binding_power(op: OpToken) -> ((), u8) {
    use OpToken::*;
    match op {
        Plus | Minus | Bang => ((), 21),
        _ => panic!("unrecognized prefix operator"),
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
