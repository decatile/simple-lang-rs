#![allow(dead_code)]

use std::{borrow::Cow, iter, mem::transmute, rc::Rc};

use nom::{
    Input, Offset, Parser,
    branch::alt,
    character::complete::{char, digit1, satisfy},
    combinator::{cut, value},
    multi::many0,
};
use nom_locate::LocatedSpan;

type Pos<'a> = LocatedSpan<&'a str>;

trait PosExt<'a> {
    fn difference(&self, other: Pos) -> Pos<'a>;
}

impl<'a> PosExt<'a> for Pos<'a> {
    fn difference(&self, other: Pos) -> Pos<'a> {
        let offset: usize = self.offset(&other);
        self.take(offset)
    }
}

pub type Result<'a, O> = nom::IResult<Pos<'a>, O, Error<'a>>;

#[derive(Debug, Clone)]
pub struct Error<'a> {
    pub input: Pos<'a>,
    pub message: Cow<'a, str>,
}

impl<'a> Error<'a> {
    pub fn new<M: Into<Cow<'a, str>>>(input: Pos<'a>, message: M) -> Self {
        Self {
            input,
            message: message.into(),
        }
    }
}

impl<'a> nom::error::ParseError<Pos<'a>> for Error<'a> {
    fn from_error_kind(input: Pos<'a>, kind: nom::error::ErrorKind) -> Self {
        Error::new(input, unsafe {
            transmute::<_, &'static str>(kind.description())
        })
    }

    fn append(_: Pos<'a>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> nom::error::FromExternalError<&'a str, Error<'a>> for Error<'a> {
    fn from_external_error(_: &'a str, _: nom::error::ErrorKind, e: Error<'a>) -> Self {
        e
    }
}

impl<'a, E: Into<Cow<'a, str>>> nom::error::FromExternalError<Pos<'a>, E> for Error<'a> {
    fn from_external_error(input: Pos<'a>, _: nom::error::ErrorKind, e: E) -> Self {
        Error::new(input, e)
    }
}

trait ParserExt<'a>: Parser<Pos<'a>> {
    fn parse_or<M: Into<Cow<'a, str>>>(
        &mut self,
        input: Pos<'a>,
        msg: M,
    ) -> Result<'a, Self::Output>;
}

impl<'a, P: Parser<Pos<'a>>> ParserExt<'a> for P {
    fn parse_or<M: Into<Cow<'a, str>>>(&mut self, input: Pos<'a>, msg: M) -> Result<'a, P::Output> {
        self.parse(input)
            .map_err(|err| err.map(|_| Error::new(input, msg)))
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a, T> {
    pub pos: Pos<'a>,
    pub data: T,
}

impl<'a, T> Token<'a, T> {
    fn new(pos: Pos<'a>, data: T) -> Self {
        Self { pos, data }
    }
}

#[derive(Debug, Clone)]
pub struct LparInner;

#[derive(Debug, Clone)]
pub struct RparInner;

#[derive(Debug, Clone)]
pub struct IntInner(i64);

#[derive(Debug, Clone)]
pub struct FloatInner(f64);

type Lpar<'a> = Token<'a, LparInner>;

type Rpar<'a> = Token<'a, RparInner>;

type Int<'a> = Token<'a, IntInner>;

type Float<'a> = Token<'a, FloatInner>;

#[derive(Debug, Clone)]
pub enum Number<'a> {
    Int(Int<'a>),
    Float(Float<'a>),
}

#[derive(Debug, Clone)]
pub struct IdentInner(String);

type Ident<'a> = Token<'a, IdentInner>;

#[derive(Debug, Clone)]
pub enum OperationInner {
    Add,
    Sub,
    Mul,
    Div,
}

type Operation<'a> = Token<'a, OperationInner>;

#[derive(Debug, Clone)]
pub enum ExpressionInner<'a> {
    Ident(Ident<'a>),
    Number(Number<'a>),
    Binary(Rc<Expression<'a>>, Operation<'a>, Rc<Expression<'a>>),
}

pub type Expression<'a> = Token<'a, ExpressionInner<'a>>;

#[derive(Debug, Clone)]
pub enum TryEvaluateError<'a> {
    Var(Pos<'a>),
    Overflow(Pos<'a>),
    DivisionByZero(Pos<'a>),
}

impl<'a> ExpressionInner<'a> {
    pub fn try_evaluate(&self) -> std::result::Result<f64, TryEvaluateError<'a>> {
        match self {
            ExpressionInner::Ident(id) => Err(TryEvaluateError::Var(id.pos)),
            ExpressionInner::Number(num) => Ok(match num {
                Number::Int(int) => int.data.0 as f64,
                Number::Float(float) => float.data.0,
            }),
            ExpressionInner::Binary(ex1, op, ex2) => {
                let r1 = ex1.data.try_evaluate()?;
                let r2 = ex2.data.try_evaluate()?;
                let r = match op.data {
                    OperationInner::Add => r1 + r2,
                    OperationInner::Sub => r1 - r2,
                    OperationInner::Mul => r1 * r2,
                    OperationInner::Div => {
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

pub fn lpar(input: Pos) -> Result<Lpar> {
    char::<_, ()>('(')
        .map(|_| Token::new(input.take(1), LparInner))
        .parse_or(input, "Expected '('")
}

pub fn rpar(input: Pos) -> Result<Rpar> {
    char::<_, ()>(')')
        .map(|_| Token::new(input.take(1), RparInner))
        .parse_or(input, "Expected ')'")
}

pub fn integer(input: Pos) -> Result<Int> {
    let (rest, int) = digit1::<_, ()>
        .map_opt(|integral: Pos| integral.parse().ok())
        .parse_or(input, "Cannot instantiate integer")?;
    Ok((rest, Token::new(input.difference(rest), IntInner(int))))
}

pub fn number(input: Pos) -> Result<Number> {
    let (rest, integral) = integer(input)?;
    // Nah I'd simplify
    if !rest.starts_with('.') {
        return Ok((rest, Number::Int(integral)));
    }
    let (rest, rational) = integer(rest.take_from(1)).map_err(|err| {
        err.map(|_| Error::new(input, "Cannot instantiate rational part of float"))
    })?;
    if let Ok(float) = format!("{}.{}", integral.data.0, rational.data.0).parse() {
        Ok((
            rest,
            Number::Float(Token::new(input.difference(rest), FloatInner(float))),
        ))
    } else {
        Err(nom::Err::Error(Error::new(
            input,
            "Cannot instantiate float",
        )))
    }
}

pub fn ident(input: Pos) -> Result<Ident> {
    let (rest, head) = satisfy::<_, _, ()>(|c| c.is_alphabetic() || c == '_').parse_or(
        input,
        "Identifier should start with alphabetic char or underscore",
    )?;
    let (rest, id) = many0(satisfy::<_, _, ()>(|c| c.is_alphanumeric() || c == '_'))
        .map(|tail| iter::once(head).chain(tail.into_iter()).collect())
        .parse_or(
            rest,
            "Identifier should contain only alphanumeric chars or underscore",
        )?;
    Ok((rest, Token::new(input.difference(rest), IdentInner(id))))
}

pub fn operation(input: Pos) -> Result<Operation> {
    alt((
        value(OperationInner::Add, char::<_, ()>('+')),
        value(OperationInner::Sub, char('-')),
        value(OperationInner::Mul, char('*')),
        value(OperationInner::Div, char('/')),
    ))
    .map(|inner| Token::new(input.take(1), inner))
    .parse_or(input, "Expected '+', '-', '*', or '/'")
}

struct ExpressionTokens<'a> {
    operands: Vec<Rc<Expression<'a>>>,
    operations: Vec<Operation<'a>>,
}

impl<'a> ExpressionTokens<'a> {
    fn simplify(mut self) -> ExpressionTokens<'a> {
        while let Some(pos) = self
            .operations
            .iter()
            .position(|op| matches!(op.data, OperationInner::Mul | OperationInner::Div))
        {
            let op = self.operations.remove(pos);
            let rhs = self.operands.remove(pos + 1);
            let lhs = self.operands[pos].clone();
            self.operands[pos] =
                Rc::new(Token::new(lhs.pos, ExpressionInner::Binary(lhs, op, rhs)));
        }
        while !self.operations.is_empty() {
            let op = self.operations.remove(0);
            let rhs = self.operands.remove(1);
            let lhs = self.operands[0].clone();
            self.operands[0] = Rc::new(Token::new(lhs.pos, ExpressionInner::Binary(lhs, op, rhs)));
        }
        self
    }

    fn extend(&mut self, mut other: Self) {
        self.operands.append(&mut other.operands);
        self.operations.append(&mut other.operations);
    }
}

fn parse_expression_tokens(input: Pos) -> Result<ExpressionTokens> {
    (
        cut(alt((
            (lpar, parse_expression_tokens, rpar).map(|(_, exp, rp)| {
                let exp = exp.simplify().operands.remove(0);
                let len = input.offset(&rp.pos) + rp.pos.len();
                ExpressionTokens {
                    operands: vec![Rc::new(Token::new(input.take(len), exp.data.clone()))],
                    operations: vec![],
                }
            }),
            number.map(|num| ExpressionTokens {
                operands: vec![Rc::new(Token::new(
                    match &num {
                        Number::Int(token) => token.pos,
                        Number::Float(token) => token.pos,
                    },
                    ExpressionInner::Number(num),
                ))],
                operations: vec![],
            }),
        ))),
        many0((operation, parse_expression_tokens)),
    )
        .map(|(mut tok, toks)| {
            for (op, tt) in toks {
                tok.operations.push(op);
                tok.extend(tt);
            }
            tok
        })
        .parse(input)
}

pub fn expression(input: Pos) -> Result<Rc<Expression>> {
    parse_expression_tokens
        .map(|tok| tok.simplify().operands.remove(0))
        .parse(input)
}
