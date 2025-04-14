use lang::{Context, Func, Span};
use nom::{Err, Input, Offset};
use std::{
    cell::UnsafeCell,
    collections::HashMap,
    io::{Write, stdin, stdout},
    iter::repeat_n,
    time::Instant,
};

mod lang;

fn main() {
    let mut buf = UnsafeCell::new(String::new());
    let mut buf_read = 0;
    let mut ctx = Context::default();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let new_buf_read = stdin().read_line(&mut *buf.get_mut()).unwrap();
        let span: Span = unsafe { &*buf.get() }.as_str().into();
        match lang::program(span.take_from(buf_read)) {
            Ok((_, program)) => {
                buf_read += new_buf_read;
                match program {
                    lang::Program::Expression(token) => {
                        println!("{:?}", ctx.evaluate_expression(&token))
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
                unsafe { &mut *buf.get() }.shrink_to(buf_read);
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
