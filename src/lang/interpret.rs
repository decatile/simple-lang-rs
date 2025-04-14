use std::{collections::HashMap, rc::Rc};

use super::tokens::{Expression, FuncAssign, FuncCall, IExpression, IOperation, Ident, Number};

#[derive(Clone)]
pub struct BuiltinFunc {
    inner: Rc<dyn Fn(&[f64])>,
    argc: usize,
}

#[derive(Clone)]
pub enum Func<'a> {
    Builtin(BuiltinFunc),
    Custom(FuncAssign<'a>),
}

#[derive(Default)]
pub struct Context<'a> {
    pub vars: HashMap<String, f64>,
    pub funcs: HashMap<String, Func<'a>>,
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
                    Func::Builtin(builtin_func) => {
                        if builtin_func.argc != argc {
                            return Err(EvaluateExpressionError::InvalidFunctionArgc(
                                token.clone(),
                                builtin_func.argc,
                            ));
                        }
                        let mut args = vec![0.; argc];
                        for (idx, tok) in token.data.args.data.0.iter().enumerate() {
                            args[idx] = self.evaluate_expression(tok)?;
                        }
                        (builtin_func.inner)(&args);
                        todo!()
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
