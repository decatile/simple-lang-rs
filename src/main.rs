use core::str;
use nelang::lang::{Context, Func, Program, Span, program};
use nom::multi::many1;
use nom::{Err, Offset, Parser};
use std::env::args;
use std::io::Read;
use std::process::Command;
use std::{
    io::{Write, stdin, stdout},
    iter::repeat_n,
    mem::transmute,
    process::exit,
    rc::Rc,
};

fn clearscreen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).spawn()
    } else {
        Command::new("clear").spawn()
    }
    .unwrap()
    .wait()
    .unwrap();
}

fn repl_main() {
    let mut storage = Vec::<Rc<String>>::new();
    let mut ctx = Context::new();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut string = String::new();
        stdin().read_line(&mut string).unwrap();
        match string.trim() {
            "help" => {
                println!(
                    "help - print this\nhelp functions - print defined functions and its signature\nclear - clear screen\nexit - exit the program"
                );
                continue;
            }
            "help functions" => {
                ctx.funcs.iter().for_each(|(k, v)| match v {
                    Func::Builtin { argc, .. } => println!("{k}({argc})"),
                    Func::Custom(token) => {
                        println!("{k}({}) builtin", token.data.args.data.0.len())
                    }
                });
                continue;
            }
            "clear" => {
                clearscreen();
                continue;
            }
            "exit" => exit(0),
            _ => {}
        }
        let span = {
            let input_rc = Rc::new(string);
            // SAFETY
            // Преобразование в 'static безопасно, так как мы храним строки,
            // пока программа выполняется
            let span = unsafe { transmute::<_, Span<'static>>(Span::new(&input_rc)) };
            storage.push(input_rc);
            span
        };
        match program(span) {
            Ok((_, program)) => {
                match program {
                    Program::Expression(token) => match ctx.evaluate_expression(&token) {
                        Ok(result) => println!("{result}"),
                        Err(err) => println!("{err}"),
                    },
                    Program::Func(token) => {
                        ctx.funcs
                            .insert(token.data.ident.data.0.clone(), Func::Custom(token));
                        println!("Ok!")
                    }
                    Program::Var(token) => match ctx.evaluate_expression(&token.data.expr) {
                        Ok(result) => {
                            ctx.vars.insert(token.data.ident.data.0.clone(), result);
                            println!("{result}");
                        }
                        Err(err) => {
                            println!("{err}");
                        }
                    },
                };
            }
            Err(err) => match err {
                Err::Incomplete(needed) => println!("{needed:?}"),
                Err::Error(err) | Err::Failure(err) => {
                    println!(
                        "{}^- {}",
                        repeat_n(
                            ' ',
                            unsafe { str::from_utf8_unchecked(span.get_line_beginning()) }
                                .offset(&err.input)
                                + 2
                        )
                        .collect::<String>(),
                        err.message,
                    );
                }
            },
        }
    }
}

fn execute_main() {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).unwrap();
    let mut ctx = Context::new();
    let span = Span::new(&buffer);
    match many1(program).parse(span) {
        Ok((_, programs)) => {
            for program in programs {
                match program {
                    Program::Expression(token) => match ctx.evaluate_expression(&token) {
                        Err(err) => {
                            println!("{err}");
                            return;
                        }
                        _ => {}
                    },
                    Program::Func(token) => {
                        ctx.funcs
                            .insert(token.data.ident.data.0.clone(), Func::Custom(token));
                    }
                    Program::Var(token) => match ctx.evaluate_expression(&token.data.expr) {
                        Ok(result) => {
                            ctx.vars.insert(token.data.ident.data.0.clone(), result);
                        }
                        Err(err) => {
                            println!("{err}");
                            return;
                        }
                    },
                };
            }
        }
        Err(err) => match err {
            Err::Incomplete(needed) => println!("{needed:?}"),
            Err::Error(err) | Err::Failure(err) => {
                println!(
                    "{} at line {}, column {}",
                    err.message,
                    err.input.location_line(),
                    err.input.get_column()
                );
            }
        },
    }
}

fn main() {
    if let Some(arg) = args().nth(1) {
        match arg.as_str() {
            "-e" | "--execute" => {
                execute_main();
            }
            "-h" | "--help" => {
                println!(
                    "NeLang - Simple Expression Interpreter\nCall the program without arguments to enter REPL mode\nTo execute a program, pass it through the pipe with flag '-e' | '--execute'."
                )
            }
            other => {
                println!("Undefined argument '{other}'. Use '-h' or '--help' to print help.")
            }
        }
    } else {
        repl_main();
    }
}
