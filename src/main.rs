use nom::{Err, Offset};
use std::{
    io::{Write, stdin, stdout},
    iter::repeat_n,
    time::Instant,
};

mod parser;

fn main() {
    let mut buf = String::new();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buf).unwrap();
        let span = buf.trim().into();
        let i1 = Instant::now();
        match parser::expression(span) {
            Ok((rest, result)) if rest.is_empty() => {
                let i2 = Instant::now();
                println!(
                    "\n{result:?}\n\n{:?}\n\nTook {}us",
                    result.data.try_evaluate(),
                    i2.duration_since(i1).as_micros()
                );
            }
            Ok((rest, _)) => {
                let offset = span.offset(&rest);
                println!(
                    "{}^- Why is that here",
                    repeat_n(' ', offset + 2).collect::<String>(),
                )
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
        buf.clear();
    }
}
