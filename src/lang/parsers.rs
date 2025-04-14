use std::{iter, rc::Rc};

use super::tokens::*;
use super::types::*;
use nom::{
    Input, Offset, Parser,
    branch::alt,
    character::complete::{char, digit1, satisfy},
    combinator::value,
    multi::many0,
};

pub fn soft<'a, P: Parser<Span<'a>, Error = Error<'a>>>(
    mut p: P,
    i: Span<'a>,
) -> Result<'a, P::Output> {
    match p.parse(i) {
        Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e)), // понижаем уровень ошибки
        other => other,
    }
}

pub fn lpar(input: Span) -> Result<Lpar> {
    char::<_, ()>('(')
        .map(|_| Token::new(input.take(1), ILpar))
        .parse_or(input, "Expected '('")
}

pub fn rpar(input: Span) -> Result<Rpar> {
    char::<_, ()>(')')
        .map(|_| Token::new(input.take(1), IRpar))
        .parse_or(input, "Expected ')'")
}

pub fn integer(input: Span) -> Result<Int> {
    let (rest, int) = digit1::<_, ()>
        .map_opt(|integral: Span| integral.parse().ok())
        .parse_or(input, "Cannot instantiate integer")?;
    Ok((rest, Token::new(input.diff(rest), IInt(int))))
}

pub fn number(input: Span) -> Result<Number> {
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
            Number::Float(Token::new(input.diff(rest), IFloat(float))),
        ))
    } else {
        Err(nom::Err::Error(Error::new(
            input,
            "Cannot instantiate float",
        )))
    }
}

pub fn ident(input: Span) -> Result<Ident> {
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
    Ok((rest, Token::new(input.diff(rest), IIdent(id))))
}

pub fn operation(input: Span) -> Result<Operation> {
    alt((
        value(IOperation::Add, char::<_, ()>('+')),
        value(IOperation::Sub, char('-')),
        value(IOperation::Mul, char('*')),
        value(IOperation::Div, char('/')),
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
            .position(|op| matches!(op.data, IOperation::Mul | IOperation::Div))
        {
            let op = self.operations.remove(pos);
            let rhs = self.operands.remove(pos + 1);
            let lhs = self.operands[pos].clone();
            self.operands[pos] = Rc::new(Token::new(lhs.pos, IExpression::Binary(lhs, op, rhs)));
        }
        while !self.operations.is_empty() {
            let op = self.operations.remove(0);
            let rhs = self.operands.remove(1);
            let lhs = self.operands[0].clone();
            self.operands[0] = Rc::new(Token::new(lhs.pos, IExpression::Binary(lhs, op, rhs)));
        }
        self
    }

    fn extend(&mut self, mut other: Self) {
        self.operands.append(&mut other.operands);
        self.operations.append(&mut other.operations);
    }
}

fn parse_expression_tokens(input: Span) -> Result<ExpressionTokens> {
    (
        alt((
            (lpar, parse_expression_tokens, rpar).map(|(_, exp, rp)| {
                let exp = exp.simplify().operands.remove(0);
                let len = input.offset(&rp.pos) + rp.pos.len();
                ExpressionTokens {
                    operands: vec![Rc::new(Token::new(input.take(len), exp.data.clone()))],
                    operations: vec![],
                }
            }),
            ident.map(|id| ExpressionTokens {
                operands: vec![Rc::new(Token::new(id.pos, IExpression::Ident(id)))],
                operations: vec![],
            }),
            number.map(|num| ExpressionTokens {
                operands: vec![Rc::new(Token::new(
                    match &num {
                        Number::Int(token) => token.pos,
                        Number::Float(token) => token.pos,
                    },
                    IExpression::Number(num),
                ))],
                operations: vec![],
            }),
            |_: Span| Err(nom::Err::Failure(Error::new(input, "Invalid expression"))),
        )),
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

pub fn expression(input: Span) -> Result<Rc<Expression>> {
    soft(
        parse_expression_tokens.map(|tok| tok.simplify().operands.remove(0)),
        input,
    )
}
