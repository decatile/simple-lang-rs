use std::{collections::HashMap, rc::Rc};
use std::fmt;

use super::tokens::{
    Expression, FuncAssign, FuncCall, IBinaryOperation, IExpression, IUnaryOperation, Ident, Number,
};

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
        Context::default()
    }
}

#[derive(Debug)]
pub enum EvaluateExpressionError<'a> {
    InvalidFunctionArgc(FuncCall<'a>, usize),
    UndefinedFunction(FuncCall<'a>),
    UndefinedVar(Ident<'a>),
    DivisionByZero(Expression<'a>),
    Overflow(Expression<'a>),
}

impl<'a> fmt::Display for EvaluateExpressionError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvaluateExpressionError::InvalidFunctionArgc(func_call, expected_argc) => {
                write!(f, "Invalid number of arguments for function '{}': expected {}, got {}", 
                       func_call.data.ident, expected_argc, func_call.data.args.data.0.len())
            },
            EvaluateExpressionError::UndefinedFunction(func_call) => {
                write!(f, "Undefined function: '{}'", func_call.data.ident)
            },
            EvaluateExpressionError::UndefinedVar(ident) => {
                write!(f, "Undefined variable: '{}'", ident)
            },
            EvaluateExpressionError::DivisionByZero(expr) => {
                write!(f, "Division by zero in expression: '{}'", expr)
            },
            EvaluateExpressionError::Overflow(expr) => {
                write!(f, "Numeric overflow in expression: '{}'", expr)
            },
        }
    }
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
            IExpression::Unary(hs, op) => {
                let hr = self.evaluate_expression(hs)?;
                let r = match *op.data {
                    IUnaryOperation::Neg => -hr,
                    IUnaryOperation::Pos => hr,
                };
                if r.is_finite() {
                    Ok(r)
                } else {
                    Err(EvaluateExpressionError::Overflow(expr.clone()))
                }
            }
            IExpression::Binary(lhs, op, rhs) => {
                let lr = self.evaluate_expression(lhs)?;
                let rr = self.evaluate_expression(rhs)?;

                fn b2f(value: bool) -> f64 {
                    if value { 1. } else { 0. }
                }

                let r = match *op.data {
                    IBinaryOperation::Add => lr + rr,
                    IBinaryOperation::Sub => lr - rr,
                    IBinaryOperation::Mul => lr * rr,
                    IBinaryOperation::Div => {
                        if rr == 0. {
                            return Err(EvaluateExpressionError::DivisionByZero(rhs.clone()));
                        } else {
                            lr / rr
                        }
                    }
                    IBinaryOperation::Lt => b2f(lr < rr),
                    IBinaryOperation::Le => b2f(lr <= rr),
                    IBinaryOperation::Eq => b2f(lr == rr),
                    IBinaryOperation::Ne => b2f(lr != rr),
                    IBinaryOperation::Ge => b2f(lr >= rr),
                    IBinaryOperation::Gt => b2f(lr > rr),
                };
                if r.is_finite() {
                    Ok(r)
                } else {
                    Err(EvaluateExpressionError::Overflow(expr.clone()))
                }
            }
            IExpression::Ternary(cond, lhs, rhs) => {
                if self.evaluate_expression(cond)? != 0. {
                    self.evaluate_expression(lhs)
                } else {
                    self.evaluate_expression(rhs)
                }
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
