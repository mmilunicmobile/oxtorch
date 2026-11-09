use crate::lexer::{Atom, Lexer, OpToken, Token};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    BinOp(OpToken, Box<Expr>, Box<Expr>),
    PreOp(OpToken, Box<Expr>),
    PostOp(OpToken, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Comprehension(Option<Box<Expr>>, Box<Expr>, Vec<Expr>),
    IndexExpr(Box<Expr>, Vec<Expr>),
    Identifier(String),
    Number(String),
}

pub fn expr(lexer: &mut Lexer) -> Vec<(Expr, Expr)> {
    let mut out = Vec::new();
    loop {
        match lexer.next() {
            Token::Op(OpToken::Def) => {}
            Token::Eof => break,
            _ => panic!("expected function definition"),
        }
        let signature = expr_bp(lexer, 0);

        match lexer.next() {
            Token::Op(OpToken::As) => {}
            Token::Eof => break,
            _ => panic!("expected function definition"),
        }

        let body = expr_bp(lexer, 0);

        out.push((signature, body));
    }
    out
}

/// parse through the inside of a comprehension
fn expr_comprehension(lexer: &mut Lexer) -> (Expr, Vec<Expr>) {
    let mhs = expr_bp(lexer, 0);
    match lexer.next() {
        Token::Op(OpToken::Arrow) => {
            assert_eq!(lexer.next(), Token::Op(OpToken::LBracket), "expected bracket");
            let series = expr_commas(lexer, Token::Op(OpToken::RBracket));
            assert_eq!(lexer.next(), Token::Op(OpToken::RBracket), "expected bracket");
            assert_eq!(
                lexer.next(),
                Token::Op(OpToken::RBrace),
                "expected end of comprehension"
            );
            (mhs, series)
        }
        Token::Op(OpToken::RBrace) => (mhs, Vec::new()),
        _ => panic!("expected end of comprehension"),
    }
}

/// parse through the inside of something full of commas
fn expr_commas(lexer: &mut Lexer, end: Token) -> Vec<Expr> {
    let mut out = Vec::new();
    if end == lexer.peek() {
        return out;
    }
    out.push(expr_bp(lexer, 0));
    while let Token::Op(OpToken::Comma) = lexer.peek()
    {
        lexer.next();
        if end == lexer.peek() {
            return out;
        }
        out.push(expr_bp(lexer, 0));
    }
    out
}

fn expr_bp(lexer: &mut Lexer, min_bp: u8) -> Expr {
    let mut lhs = match lexer.next() {
        Token::Atom(Atom::Identifier(s)) => Expr::Identifier(s),
        Token::Atom(Atom::Number(s)) => Expr::Number(s),
        Token::Op(OpToken::LParen)=> {
            let lhs = expr_bp(lexer, 0);
            assert_eq!(
                lexer.next(),
                Token::Op(OpToken::RParen),
                "expected parenthesis"
            );
            lhs
        }
        Token::Op(OpToken::LBrace) => {
            let (mhs, indices) = expr_comprehension(lexer);
            Expr::Comprehension(None, Box::new(mhs), indices)
        }
        Token::Op(s) => {
            let ((), bp) = prefix_binding_power(s);
            let lhs = expr_bp(lexer, bp);
            Expr::PreOp(s, Box::new(lhs))
        }
        Token::Eof => panic!("expected expression"),
    };

    loop {
        let op = match lexer.peek() {
            Token::Eof => break,
            Token::Op(op) => op,
            _ => panic!("expected operator"),
        };

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();

            lhs = if op == OpToken::LBracket {
                let indices = expr_commas(lexer, Token::Op(OpToken::RBracket));
                assert_eq!(lexer.next(), Token::Op(OpToken::RBracket), "expected bracket");
                Expr::IndexExpr(Box::new(lhs), indices)
            } else if op == OpToken::LParen {
                let parameters = expr_commas(lexer, Token::Op(OpToken::RParen));
                assert_eq!(
                    lexer.next(),
                    Token::Op(OpToken::RParen),
                    "expected parenthesis"
                );
                Expr::Call(Box::new(lhs), parameters)
            } else if op == OpToken::LBrace {
                let (mhs, indices) = expr_comprehension(lexer);
                Expr::Comprehension(Some(Box::new(lhs)), Box::new(mhs), indices)
            } else {
                Expr::PostOp(op, Box::new(lhs))
            };

            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();

            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(expr_bp(lexer, r_bp)));
            continue;
        }

        break;
    }

    lhs
}

fn postfix_binding_power(op: OpToken) -> Option<(u8, ())> {
    use  OpToken::*;
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
        Plus| Minus | Bang => ((), 21),
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
        Plus | Minus| At => (15, 16),
        Times | Div => (17, 18),
        _ => return None,
    })
}
