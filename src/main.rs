use nom::{Err, Offset};
use std::{
    io::{Write, stdin, stdout},
    iter::repeat_n,
    time::Instant,
};

mod lang;

fn main() {
    let mut buf = String::new();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut buf).unwrap();
        let span = buf.trim().into();
        let i1 = Instant::now();
        match lang::expression(span) {
            Ok((rest, result)) if rest.is_empty() => {
                let i2 = Instant::now();
                println!(
                    "\n{result:?}\n\n{:?}\n\nTook {}us",
                    result.data.try_evaluate(),
                    i2.duration_since(i1).as_micros()
                );
            }
            Ok((rest, _)) => {
                println!(
                    "{}^- Why is that here? Parser recorded end of known expression (char to left)",
                    repeat_n(' ', span.offset(&rest) + 2).collect::<String>(),
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
