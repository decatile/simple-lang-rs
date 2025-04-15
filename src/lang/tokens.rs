use std::rc::Rc;

use super::types::Span;

pub type Eol<'a> = Token<'a, IEol>;

pub type Eql<'a> = Token<'a, IEql>;

pub type Lpar<'a> = Token<'a, ILpar>;

pub type Rpar<'a> = Token<'a, IRpar>;

pub type Int<'a> = Token<'a, IInt>;

pub type Float<'a> = Token<'a, IFloat>;

pub type Ident<'a> = Token<'a, IIdent>;

pub type FuncCall<'a> = Token<'a, IFuncCall<'a>>;

pub type FuncCallArgs<'a> = Token<'a, IFuncCallArgs<'a>>;

pub type UnaryOperation<'a> = Token<'a, IUnaryOperation>;

pub type BinaryOperation<'a> = Token<'a, IBinaryOperation>;

pub type Expression<'a> = Token<'a, IExpression<'a>>;

pub type VarAssign<'a> = Token<'a, IVarAssign<'a>>;

pub type FuncAssign<'a> = Token<'a, IFuncAssign<'a>>;

pub type FuncAssignArgs<'a> = Token<'a, IFuncAssignArgs<'a>>;

#[derive(Debug, Clone)]
pub struct Token<'a, T> {
    pub pos: Span<'a>,
    pub data: Rc<T>,
}

impl<'a, T> Token<'a, T> {
    pub fn new<I: Into<Rc<T>>>(pos: Span<'a>, data: I) -> Self {
        Self {
            pos,
            data: data.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IEol;

#[derive(Debug, Clone)]
pub struct IEql;

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
pub struct IFuncCallArgs<'a>(pub Vec<Expression<'a>>);

#[derive(Debug, Clone)]
pub struct IFuncCall<'a> {
    pub ident: Ident<'a>,
    pub args: FuncCallArgs<'a>,
}

#[derive(Debug, Clone)]
pub enum IUnaryOperation {
    Neg,
    Pos,
}

#[derive(Debug, Clone)]
pub enum IBinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum IExpression<'a> {
    Call(FuncCall<'a>),
    Ident(Ident<'a>),
    Number(Number<'a>),
    Unary(Expression<'a>, UnaryOperation<'a>),
    Binary(Expression<'a>, BinaryOperation<'a>, Expression<'a>),
}

#[derive(Debug, Clone)]
pub struct IVarAssign<'a> {
    pub ident: Ident<'a>,
    pub expr: Expression<'a>,
}

#[derive(Debug, Clone)]
pub struct IFuncAssign<'a> {
    pub ident: Ident<'a>,
    pub args: FuncAssignArgs<'a>,
    pub expr: Expression<'a>,
}

#[derive(Debug, Clone)]
pub struct IFuncAssignArgs<'a>(pub Vec<Ident<'a>>);
