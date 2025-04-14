use std::iter;

use super::tokens::*;
use super::types::*;
use nom::combinator::cut;
use nom::combinator::opt;
use nom::{
    Input, Offset, Parser,
    branch::alt,
    character::complete::{char, digit1, satisfy},
    combinator::value,
    multi::many0,
};

pub fn eol(input: Span) -> Result<Eol> {
    char::<_, ()>('\n')
        .map(|_| Token::new(input.take(1), IEol))
        .parse_or(input, "Expected EOL")
}

pub fn eql(input: Span) -> Result<Eql> {
    char::<_, ()>('=')
        .map(|_| Token::new(input.take(1), IEql))
        .parse_or(input, "Expected '='")
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
    let (rest, minus) = opt(char('-')).map(|c| c.is_some()).parse(input)?;
    let (rest, int) = digit1::<_, ()>.parse_or(rest, "Cannot instantiate integer")?;
    match int.parse::<i64>() {
        Ok(int) => Ok((
            rest,
            Token::new(input.diff(rest), IInt(if minus { -int } else { int })),
        )),
        Err(_) => Err(nom::Err::Failure(Error::new(
            input,
            "Cannot instantiate integer",
        ))),
    }
}

fn non_negative_integer(input: Span) -> Result<Int> {
    let (rest, int) = digit1::<_, ()>.parse_or(input, "Cannot instantiate integer")?;
    match int.parse::<i64>() {
        Ok(int) => Ok((rest, Token::new(input.diff(rest), IInt(int)))),
        Err(_) => Err(nom::Err::Failure(Error::new(
            input,
            "Cannot instantiate integer",
        ))),
    }
}

pub fn number(input: Span) -> Result<Number> {
    let (rest, integral) = integer(input)?;
    // Nah I'd simplify
    if !rest.starts_with('.') {
        return Ok((rest, Number::Int(integral)));
    }
    let (rest, rational) = non_negative_integer.parse_or(
        rest.take_from(1),
        "Cannot instantiate rational part of float",
    )?;
    if let Ok(float) = format!("{}.{}", integral.data.0, rational.data.0).parse() {
        Ok((
            rest,
            Number::Float(Token::new(input.diff(rest), IFloat(float))),
        ))
    } else {
        Err(nom::Err::Failure(Error::new(
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

pub fn func_call(input: Span) -> Result<FuncCall> {
    (
        ident,
        lpar,
        cut((opt((expression, many0((char(','), expression)))), rpar)),
    )
        .map(|(ident, lp, (args, rp))| {
            Token::new(
                input.including_diff(rp.pos),
                IFuncCall {
                    ident,
                    args: Token::new(
                        input
                            .take_from(input.offset(&lp.pos))
                            .including_diff(rp.pos),
                        IFuncCallArgs(match args {
                            Some((arg0, args)) => iter::once(arg0)
                                .chain(args.into_iter().map(|(_, arg)| arg))
                                .collect::<Vec<_>>(),
                            None => vec![],
                        }),
                    ),
                },
            )
        })
        .parse(input)
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
    operands: Vec<Expression<'a>>,
    operations: Vec<Operation<'a>>,
}

impl<'a> ExpressionTokens<'a> {
    fn simplify(mut self) -> ExpressionTokens<'a> {
        while let Some(pos) = self
            .operations
            .iter()
            .position(|op| matches!(*op.data, IOperation::Mul | IOperation::Div))
        {
            let op = self.operations.remove(pos);
            let rhs = self.operands.remove(pos + 1);
            let lhs = self.operands[pos].clone();
            self.operands[pos] = Token::new(lhs.pos, IExpression::Binary(lhs, op, rhs));
        }
        while !self.operations.is_empty() {
            let op = self.operations.remove(0);
            let rhs = self.operands.remove(1);
            let lhs = self.operands[0].clone();
            self.operands[0] = Token::new(lhs.pos, IExpression::Binary(lhs, op, rhs));
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
                ExpressionTokens {
                    operands: vec![Token::new(input.including_diff(rp.pos), exp.data.clone())],
                    operations: vec![],
                }
            }),
            func_call.map(|call| ExpressionTokens {
                operands: vec![Token::new(call.pos, IExpression::Call(call))],
                operations: vec![],
            }),
            ident.map(|id| ExpressionTokens {
                operands: vec![Token::new(id.pos, IExpression::Ident(id))],
                operations: vec![],
            }),
            number.map(|num| ExpressionTokens {
                operands: vec![Token::new(
                    match &num {
                        Number::Int(token) => token.pos,
                        Number::Float(token) => token.pos,
                    },
                    IExpression::Number(num),
                )],
                operations: vec![],
            }),
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

pub fn expression(input: Span) -> Result<Expression> {
    parse_expression_tokens
        .map(|tok| tok.simplify().operands.remove(0))
        .parse(input)
}

pub fn var_assign(input: Span) -> Result<VarAssign> {
    (ident, eql, cut((expression, eol)))
        .map(|(ident, _, (expr, eol))| Token::new(input.diff(eol.pos), IVarAssign { ident, expr }))
        .parse(input)
}

pub fn func_assign(input: Span) -> Result<FuncAssign> {
    (
        ident,
        lpar,
        opt((ident, many0((char(','), ident)))),
        rpar,
        eql,
        cut((expression, eol)),
    )
        .map(|(ident, lp, args, rp, _, (exp, eol))| {
            Token::new(
                input.diff(eol.pos),
                IFuncAssign {
                    ident,
                    args: Token::new(
                        input
                            .take_from(input.offset(&lp.pos))
                            .including_diff(rp.pos),
                        IFuncAssignArgs(match args {
                            Some((arg0, args)) => iter::once(arg0)
                                .chain(args.into_iter().map(|(_, arg)| arg))
                                .collect::<Vec<_>>(),
                            None => vec![],
                        }),
                    ),
                    expr: exp,
                },
            )
        })
        .parse(input)
}

pub enum Program<'a> {
    Expression(Expression<'a>),
    Func(FuncAssign<'a>),
    Var(VarAssign<'a>),
}

pub fn program(input: Span) -> Result<Program> {
    alt((
        (expression, eol).map(|(expr, _)| Program::Expression(expr)),
        func_assign.map(Program::Func),
        var_assign.map(Program::Var),
    ))
    .parse(input)
}
