#![allow(dead_code)]

use lang::{Context, Func, Span};
use nom::{Err, Input, Offset};
use std::{
    cell::UnsafeCell,
    io::{Write, stdin, stdout},
    iter::repeat_n,
};

mod lang;

fn main() {
    let mut buf = UnsafeCell::new(String::new());
    let mut buf_read = 0;
    let mut ctx = Context::new();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let new_buf_read = stdin().read_line(buf.get_mut()).unwrap();
        let span = Span::new(unsafe { &*buf.get() }).take_from(buf_read);
        match lang::program(span) {
            Ok((_, program)) => {
                buf_read += new_buf_read;
                match program {
                    lang::Program::Expression(token) => {
                        match ctx.evaluate_expression(&token) {
                            Ok(result) => println!("{result}"),
                            Err(err) => println!("{err:?}"),
                        }
                    }
                    lang::Program::Func(token) => {
                        ctx.funcs
                            .insert(token.data.ident.data.0.clone(), Func::Custom(token));
                        println!("Ok!")
                    }
                    lang::Program::Var(token) => match ctx.evaluate_expression(&token.data.expr) {
                        Ok(result) => {
                            ctx.vars.insert(token.data.ident.data.0.clone(), result);
                            println!("{result}");
                        }
                        Err(err) => {
                            println!("{err:?}");
                        }
                    },
                }
            }
            Err(err) => {
                match err {
                    Err::Incomplete(needed) => println!("{needed:?}"),
                    Err::Error(err) | Err::Failure(err) => {
                        println!(
                            "{}^- {}",
                            repeat_n(' ', span.offset(&err.input) + 2).collect::<String>(),
                            err.message,
                        );
                    }
                }
            }
        }
    }
}
