use std::{collections::HashMap, rc::Rc};

use super::tokens::{Expression, FuncAssign, FuncCall, IExpression, IOperation, Ident, Number};

#[macro_export]
macro_rules! builtin_func {
    ($name:ident, $argc:expr, $closure:expr) => {
        (
            stringify!($name).into(),
            Func::Builtin {
                inner: Rc::new($closure),
                argc: $argc,
            },
        )
    };
}

#[derive(Clone)]
pub enum Func<'a> {
    Builtin {
        inner: Rc<dyn Fn(&[f64]) -> f64>,
        argc: usize,
    },
    Custom(FuncAssign<'a>),
}

#[derive(Default)]
pub struct Context<'a> {
    pub vars: HashMap<String, f64>,
    pub funcs: HashMap<String, Func<'a>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        let mut this = Context::default();
        this.funcs
            .extend([builtin_func!(abs, 1, |args| if args[0] >= 0. {
                args[0]
            } else {
                -args[0]
            })]);
        this
    }
}

#[derive(Debug)]
pub enum EvaluateExpressionError<'a> {
    InvalidFunctionArgc(FuncCall<'a>, usize),
    UndefinedFunction(FuncCall<'a>),
    UndefinedVar(Ident<'a>),
    DivisionByZero(Expression<'a>),
}

impl<'a> Context<'a> {
    pub fn evaluate_expression(
        &self,
        expr: &Expression<'a>,
    ) -> Result<f64, EvaluateExpressionError<'a>> {
        match expr.data.as_ref() {
            IExpression::Ident(token) => self
                .vars
                .get(token.data.0.as_str())
                .cloned()
                .ok_or(EvaluateExpressionError::UndefinedVar(token.clone())),
            IExpression::Number(number) => match number {
                Number::Int(token) => Ok(token.data.0 as f64),
                Number::Float(token) => Ok(token.data.0),
            },
            IExpression::Binary(lhs, op, rhs) => {
                let lr = self.evaluate_expression(lhs)?;
                let rr = self.evaluate_expression(rhs)?;
                Ok(match *op.data {
                    IOperation::Add => lr + rr,
                    IOperation::Sub => lr - rr,
                    IOperation::Mul => lr * rr,
                    IOperation::Div => {
                        if rr == 0. {
                            return Err(EvaluateExpressionError::DivisionByZero(rhs.clone()));
                        } else {
                            lr / rr
                        }
                    }
                })
            }
            IExpression::Call(token) => {
                let argc = token.data.args.data.0.len();
                match self
                    .funcs
                    .get(token.data.ident.data.0.as_str())
                    .ok_or(EvaluateExpressionError::UndefinedFunction(token.clone()))?
                {
                    Func::Builtin {
                        argc: builtin_func_argc,
                        inner: builtin_func_inner,
                    } => {
                        if *builtin_func_argc != argc {
                            return Err(EvaluateExpressionError::InvalidFunctionArgc(
                                token.clone(),
                                *builtin_func_argc,
                            ));
                        }
                        let mut args = vec![0.; argc];
                        for (idx, tok) in token.data.args.data.0.iter().enumerate() {
                            args[idx] = self.evaluate_expression(tok)?;
                        }
                        Ok((builtin_func_inner)(&args))
                    }
                    Func::Custom(custom_func) => {
                        let custom_func_argc = custom_func.data.args.data.0.len();
                        if custom_func_argc != argc {
                            return Err(EvaluateExpressionError::InvalidFunctionArgc(
                                token.clone(),
                                custom_func_argc,
                            ));
                        }
                        let mut ctx: Context<'a> = Context {
                            funcs: self.funcs.clone(),
                            vars: HashMap::new(),
                        };
                        for (idx, i) in token.data.args.data.0.iter().enumerate() {
                            ctx.vars.insert(
                                custom_func.data.args.data.0[idx].data.0.clone(),
                                self.evaluate_expression(i)?,
                            );
                        }
                        ctx.evaluate_expression(&custom_func.data.expr)
                    }
                }
            }
        }
    }
}
