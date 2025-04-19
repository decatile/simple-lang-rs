use core::str;
use nelang::lang::{
    Context, Func, IExpression, IFloat, Number, Program, Span, Token, VarAssign, VarAssignExpr,
    program,
};
use nom::multi::many1;
use nom::{Err, Offset, Parser};
use std::borrow::Cow;
use std::env::args;
use std::fs::File;
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

fn handle_user_input<'a>(
    ui: &VarAssign<'a>,
    var_name: &str,
) -> Result<Token<'a, IExpression<'a>>, String> {
    let term = if cfg!(target_os = "windows") {
        File::open("CON")
    } else {
        File::open("/dev/tty")
    }
    .unwrap();
    let mut buffer = String::new();
    print!("Enter value for variable '{}': ", var_name);
    stdout().flush().unwrap();
    use std::io::BufRead;
    let mut reader = std::io::BufReader::new(term);
    reader.read_line(&mut buffer).unwrap();
    buffer
        .trim()
        .parse::<f64>()
        .map(|value| {
            Token::new(
                ui.pos,
                Rc::new(IExpression::Number(Number::Float(Token::new(
                    ui.pos,
                    IFloat(value),
                )))),
            )
        })
        .map_err(|_| "Invalid input. Expected a number.".to_string())
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
                    "Available commands:
help - Print this help message.
help functions - List all defined functions and their signatures.
clear - Clear the screen.
exit - Exit the program.
You can also enter expressions, variable declarations, or function definitions directly."
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
                    Program::Var(token) => {
                        let cow = match &token.data.expr {
                            VarAssignExpr::Expression(token) => Cow::Borrowed(token),
                            VarAssignExpr::UserInput(_) => {
                                match handle_user_input(&token, &token.data.ident.data.0) {
                                    Ok(token) => Cow::Owned(token),
                                    Err(err) => {
                                        println!("{err}");
                                        continue;
                                    }
                                }
                            }
                        };
                        match ctx.evaluate_expression(&cow) {
                            Ok(result) => {
                                ctx.vars.insert(token.data.ident.data.0.clone(), result);
                                println!("{result}");
                            }
                            Err(err) => {
                                println!("{err}");
                            }
                        }
                    }
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
                    Program::Var(token) => {
                        let cow = match &token.data.expr {
                            VarAssignExpr::Expression(token) => Cow::Borrowed(token),
                            VarAssignExpr::UserInput(_) => {
                                match handle_user_input(&token, &token.data.ident.data.0) {
                                    Ok(token) => Cow::Owned(token),
                                    Err(err) => {
                                        println!("{err}");
                                        return;
                                    }
                                }
                            }
                        };
                        match ctx.evaluate_expression(&cow) {
                            Ok(result) => {
                                ctx.vars.insert(token.data.ident.data.0.clone(), result);
                            }
                            Err(err) => {
                                println!("{err}");
                                return;
                            }
                        }
                    }
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
                    "Usage: nelang [OPTIONS]

NeLang - Simple Expression Interpreter

Options:
  -e, --execute    Execute a program passed through the pipe.
  -h, --help       Display this help message.

Description:
  Call the program without arguments to enter REPL mode.
  To execute a program, pass it through the pipe with the '-e' or '--execute' flag.

Syntax rules:
  expr           = term (operator term)* | ternary
  term           = number | ident | func_call | '(' expr ')' | unary_operator term
  func_call      = ident '(' args ')' '=' expr
  var            = ident '=' expr
  args           = ident (',' ident)*
  operator       = '+' | '-' | '*' | '/' | '%' | '<' | '<=' | '==' | '!=' | '>=' | '>'
  unary_operator = '+' | '-' | '!'
  ternary        = expr '?' expr ':' expr

Note:
  For ternary expressions, the condition should be wrapped in parentheses if it is complex.
  Example: (x > 0) ? x : -x"
                );
            }
            other => {
                println!("Undefined argument '{other}'. Use '-h' or '--help' to print help.")
            }
        }
    } else {
        repl_main();
    }
}
