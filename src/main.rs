#![allow(dead_code)]

use lang::{Context, Func, Span};
use nom::{Err, Input, Offset};
use std::{
    cell::{RefCell, UnsafeCell},
    io::{Write, stdin, stdout},
    iter::repeat_n,
    rc::Rc,
};

mod lang;

fn main() {
    let mut buf: Vec<UnsafeCell<String>> = vec![];
    let mut ctx = Context::new();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        // SAFETY
        // Раст не всегда понимает как используются данные
        // В этом случае, мы используем не всю структуру строки для Span-ов,
        // А лишь поинтер на хипе. Если мы муваем строку, хипа очевидно не изменится,
        // А указатели не инвалидируются. Поэтому всё ок. (Я не придумал ничего лучше).
        let mut string = UnsafeCell::new(String::new());
        stdin().read_line(string.get_mut()).unwrap();
        let span = Span::new(unsafe { &mut *string.get() }.as_str());
        match lang::program(span) {
            Ok((_, program)) => {
                match program {
                    lang::Program::Expression(token) => match ctx.evaluate_expression(&token) {
                        Ok(result) => println!("{result}"),
                        Err(err) => println!("{err:?}"),
                    },
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
                };
                buf.push(string);
            }
            Err(err) => match err {
                Err::Incomplete(needed) => println!("{needed:?}"),
                Err::Error(err) | Err::Failure(err) => {
                    println!(
                        "{}^- {}",
                        repeat_n(' ', span.offset(&err.input) + 2).collect::<String>(),
                        err.message,
                    );
                }
            },
        }
    }
}
