use std::iter;

use super::tokens::*;
use super::types::*;
use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::combinator::cut;
use nom::combinator::opt;
use nom::sequence::delimited;
use nom::{
    Input, Offset, Parser,
    branch::alt,
    character::complete::{char, digit1, satisfy},
    combinator::value,
    multi::many0,
};

fn ws<'a, F: Parser<Span<'a>>>(
    inner: F,
) -> impl Parser<Span<'a>, Output = F::Output, Error = F::Error> {
    delimited(space0, inner, space0)
}

fn parsed<'a, F: Parser<Span<'a>>>(
    mut inner: F,
) -> impl Parser<Span<'a>, Output = (F::Output, Span<'a>), Error = F::Error> {
    move |input: Span<'a>| {
        let (rest, result) = inner.parse(input)?;
        Ok((rest, (result, input.diff(&rest))))
    }
}

pub fn eol(input: Span) -> Result<Eol> {
    alt((tag::<_, _, ()>("\r\n"), tag("\n")))
        .map(|chars| Token::new(chars, IEol))
        .parse_or(input, "Expected EOL")
}

pub fn eql(input: Span) -> Result<Eql> {
    ws(parsed(char::<_, ()>('=')))
        .map(|(_, diff)| Token::new(diff, IEql))
        .parse_or(input, "Expected '='")
}

pub fn lpar(input: Span) -> Result<Lpar> {
    ws(parsed(char::<_, ()>('(')))
        .map(|(_, diff)| Token::new(diff, ILpar))
        .parse_or(input, "Expected '('")
}

pub fn rpar(input: Span) -> Result<Rpar> {
    ws(parsed(char::<_, ()>(')')))
        .map(|(_, diff)| Token::new(diff, IRpar))
        .parse_or(input, "Expected ')'")
}

fn no_ws_integer(input: Span) -> Result<Int> {
    let (rest, int) = digit1::<_, ()>.parse_or(input, "Cannot instantiate integer")?;
    match int.parse::<i64>() {
        Ok(int) => Ok((rest, Token::new(input.diff(&rest), IInt(int)))),
        Err(_) => Err(nom::Err::Failure(Error::new(
            input,
            "Cannot instantiate integer",
        ))),
    }
}

fn no_ws_number(input: Span) -> Result<Number> {
    let (rest, integral) = no_ws_integer(input)?;
    // Nah I'd simplify
    if !rest.starts_with('.') {
        return Ok((rest, Number::Int(integral)));
    }
    let (rest, rational) = no_ws_integer.parse_or(
        rest.take_from(1),
        "Cannot instantiate rational part of float",
    )?;
    if let Ok(float) = format!("{}.{}", integral.data.0, rational.data.0).parse() {
        Ok((
            rest,
            Number::Float(Token::new(input.diff(&rest), IFloat(float))),
        ))
    } else {
        Err(nom::Err::Failure(Error::new(
            input,
            "Cannot instantiate float",
        )))
    }
}

fn no_ws_ident(input: Span) -> Result<Ident> {
    let (rest, head) = satisfy::<_, _, ()>(|c| c.is_alphabetic() || c == '_').parse_or(
        input,
        "Identifier should start with alphabetic char or underscore",
    )?;
    let (rest, tail) = many0(satisfy::<_, _, ()>(|c| c.is_alphanumeric() || c == '_')).parse_or(
        rest,
        "Identifier should contain only alphanumeric chars or underscore",
    )?;
    Ok((
        rest,
        Token::new(
            input.diff(&rest),
            IIdent(iter::once(head).chain(tail.into_iter()).collect()),
        ),
    ))
}

pub fn integer(input: Span) -> Result<Int> {
    ws(no_ws_integer).parse(input)
}

pub fn number(input: Span) -> Result<Number> {
    ws(no_ws_number).parse(input)
}

pub fn ident(input: Span) -> Result<Ident> {
    ws(no_ws_ident).parse(input)
}

pub fn func_call(input: Span) -> Result<FuncCall> {
    (
        ident,
        lpar,
        cut((opt((expression, many0((char(','), expression)))), rpar)),
    )
        .map(|(ident, lp, (args, rp))| {
            Token::new(
                input
                    .take_from(input.offset(&ident.pos))
                    .including_diff(&rp.pos),
                IFuncCall {
                    ident,
                    args: Token::new(
                        input
                            .take_from(input.offset(&lp.pos))
                            .including_diff(&rp.pos),
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

pub fn unary_operation(input: Span) -> Result<UnaryOperation> {
    ws(parsed(alt((
        value(IUnaryOperation::Pos, char::<_, ()>('+')),
        value(IUnaryOperation::Neg, char::<_, ()>('-')),
    ))))
    .map(|(inner, diff)| Token::new(diff, inner))
    .parse_or(input, "Expected 'unary+' or 'unary-'")
}

pub fn binary_operation(input: Span) -> Result<BinaryOperation> {
    ws(parsed(alt((
        value(IBinaryOperation::Add, char::<_, ()>('+')),
        value(IBinaryOperation::Sub, char('-')),
        value(IBinaryOperation::Mul, char('*')),
        value(IBinaryOperation::Div, char('/')),
    ))))
    .map(|(inner, diff)| Token::new(diff, inner))
    .parse_or(input, "Expected '+', '-', '*', or '/'")
}

struct ExpressionTokens<'a> {
    operands: Vec<Expression<'a>>,
    operations: Vec<BinaryOperation<'a>>,
}

impl<'a> ExpressionTokens<'a> {
    fn simplify(mut self) -> ExpressionTokens<'a> {
        while let Some(pos) = self
            .operations
            .iter()
            .position(|op| matches!(*op.data, IBinaryOperation::Mul | IBinaryOperation::Div))
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
            (unary_operation, cut(parse_expression_tokens)).map(|(op, mut exp)| {
                let exp0 = exp.operands[0].clone();
                exp.operands[0] = Token::new(
                    input
                        .take_from(input.offset(&op.pos))
                        .including_diff(&exp0.pos),
                    IExpression::Unary(exp0, op),
                );
                ExpressionTokens {
                    operands: vec![exp.simplify().operands.remove(0)],
                    operations: vec![],
                }
            }),
            (lpar, cut((parse_expression_tokens, rpar))).map(|(lp, (exp, rp))| {
                let exp = exp.simplify().operands.remove(0);
                ExpressionTokens {
                    operands: vec![Token::new(
                        input
                            .take_from(input.offset(&lp.pos))
                            .including_diff(&rp.pos),
                        exp.data.clone(),
                    )],
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
        many0((binary_operation, parse_expression_tokens)),
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
        .map(|(ident, _, (expr, eol))| {
            Token::new(
                input.take_from(input.offset(&ident.pos)).diff(&eol.pos),
                IVarAssign { ident, expr },
            )
        })
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
                input.take_from(input.offset(&ident.pos)).diff(&eol.pos),
                IFuncAssign {
                    ident,
                    args: Token::new(
                        input
                            .take_from(input.offset(&lp.pos))
                            .including_diff(&rp.pos),
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

#[derive(Debug)]
pub enum Program<'a> {
    Expression(Expression<'a>),
    Func(FuncAssign<'a>),
    Var(VarAssign<'a>),
}

pub fn program(input: Span) -> Result<Program> {
    alt((
        var_assign.map(Program::Var),
        func_assign.map(Program::Func),
        (expression, eol).map(|(expr, _)| Program::Expression(expr)),
    ))
    .parse(input)
}
