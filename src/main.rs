use nelang::lang::{Context, Func, Program, Span, program};
use nom::{Err, Offset};
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

fn main() {
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
                    Func::Custom(token) => println!("{k}({}) builtin", token.data.args.data.0.len()),
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
                        repeat_n(' ', span.offset(&err.input) + 2).collect::<String>(),
                        err.message,
                    );
                }
            },
        }
    }
}
