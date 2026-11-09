use core::fmt;
use std::{
    collections::BTreeMap,
    ops::{Add, Mul},
};

use crate::parser::Expr;

pub struct TypedExpr {
    tipe: ExpressionType,
    expr: Expr,
}

#[derive(Clone)]
pub struct PolyType {
    terms: BTreeMap<BTreeMap<String, i64>, i64>,
}

impl PolyType {
    pub fn new() -> Self {
        PolyType {
            terms: BTreeMap::new(),
        }
    }

    pub fn mon(sym: String) -> Self {
        let mut terms = BTreeMap::new();
        let mut monomial = BTreeMap::new();
        monomial.insert(sym, 1);
        terms.insert(monomial, 1);
        PolyType { terms }
    }

    pub fn constant(num: i64) -> Self {
        let mut terms = BTreeMap::new();
        let constant = BTreeMap::new();
        terms.insert(constant, num);
        PolyType { terms }
    }
}

impl Add for &PolyType {
    type Output = PolyType;

    fn add(self, rhs: Self) -> Self::Output {
        let mut out = self.clone();
        for (term, factor) in &rhs.terms {
            let factor_out = out.terms.entry(term.clone()).or_insert(0);
            *factor_out += factor;
        }
        out
    }
}

impl Mul for &PolyType {
    type Output = PolyType;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut out = PolyType::new();
        for (term_self, factor_self) in &self.terms {
            for (term_other, factor_other) in &rhs.terms {
                let mut term = term_self.clone();
                for (sym, pow) in term_other {
                    let pow_self = term.entry(sym.clone()).or_insert(0);
                    *pow_self += pow;
                }

                let factor_out = out.terms.entry(term).or_insert(0);
                *factor_out += factor_self * factor_other;
            }
        }
        out
    }
}

impl Mul<i64> for &PolyType {
    type Output = PolyType;

    fn mul(self, rhs: i64) -> Self::Output {
        self * &PolyType::constant(rhs)
    }
}

impl Mul<&PolyType> for i64 {
    type Output = PolyType;

    fn mul(self, rhs: &PolyType) -> Self::Output {
        rhs * &PolyType::constant(self)
    }
}

impl Add<i64> for &PolyType {
    type Output = PolyType;

    fn add(self, rhs: i64) -> Self::Output {
        self + &PolyType::constant(rhs)
    }
}

impl Add<&PolyType> for i64 {
    type Output = PolyType;

    fn add(self, rhs: &PolyType) -> Self::Output {
        rhs + &PolyType::constant(self)
    }
}

impl fmt::Display for PolyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the write! macro to push the formatted string into the Formatter
        // Do not add a trailing semicolon so that the Result is returned implicitly
        let mut first_term = true;
        for (term, factor) in &self.terms {
            if !first_term {
                write!(f, " + ")?;
            } else {
                first_term = false;
            }

            write!(f, "{}", factor)?;

            for (sym, pow) in term {
                write!(f, " * {}^{}", sym, pow)?;
            }
        }
        Ok(())
    }
}

pub enum ScalarType {
    Float,
    Long,
    Boolean,
    Unit,
}
 
pub enum ExpressionType {
    UnboundVar,
    TypeVar,
    Shape(ScalarType, Vec<PolyType>),
}
