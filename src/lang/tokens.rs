use std::rc::Rc;

use super::types::Span;

pub type Lpar<'a> = Token<'a, ILpar>;

pub type Rpar<'a> = Token<'a, IRpar>;

pub type Int<'a> = Token<'a, IInt>;

pub type Float<'a> = Token<'a, IFloat>;

pub type Ident<'a> = Token<'a, IIdent>;

pub type Operation<'a> = Token<'a, IOperation>;

pub type Expression<'a> = Token<'a, IExpression<'a>>;

#[derive(Debug, Clone)]
pub struct Token<'a, T> {
    pub pos: Span<'a>,
    pub data: T,
}

impl<'a, T> Token<'a, T> {
    pub fn new(pos: Span<'a>, data: T) -> Self {
        Self { pos, data }
    }
}

#[derive(Debug, Clone)]
pub struct ILpar;

#[derive(Debug, Clone)]
pub struct IRpar;

#[derive(Debug, Clone)]
pub struct IInt(pub i64);

#[derive(Debug, Clone)]
pub struct IFloat(pub f64);

#[derive(Debug, Clone)]
pub enum Number<'a> {
    Int(Int<'a>),
    Float(Float<'a>),
}

#[derive(Debug, Clone)]
pub struct IIdent(pub String);

#[derive(Debug, Clone)]
pub enum IOperation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum IExpression<'a> {
    Ident(Ident<'a>),
    Number(Number<'a>),
    Binary(Rc<Expression<'a>>, Operation<'a>, Rc<Expression<'a>>),
}

#[derive(Debug, Clone)]
pub enum TryEvaluateError<'a> {
    Var(Span<'a>),
    Overflow(Span<'a>),
    DivisionByZero(Span<'a>),
}

impl<'a> IExpression<'a> {
    pub fn try_evaluate(&self) -> std::result::Result<f64, TryEvaluateError<'a>> {
        match self {
            IExpression::Ident(id) => Err(TryEvaluateError::Var(id.pos)),
            IExpression::Number(num) => Ok(match num {
                Number::Int(int) => int.data.0 as f64,
                Number::Float(float) => float.data.0,
            }),
            IExpression::Binary(ex1, op, ex2) => {
                let r1 = ex1.data.try_evaluate()?;
                let r2 = ex2.data.try_evaluate()?;
                let r = match op.data {
                    IOperation::Add => r1 + r2,
                    IOperation::Sub => r1 - r2,
                    IOperation::Mul => r1 * r2,
                    IOperation::Div => {
                        if r2 == 0. {
                            return Err(TryEvaluateError::DivisionByZero(ex2.pos));
                        }
                        r1 / r2
                    }
                };
                if r.is_finite() {
                    Ok(r)
                } else {
                    Err(TryEvaluateError::Overflow(op.pos))
                }
            }
        }
    }
}
