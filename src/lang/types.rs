use std::{borrow::Cow, mem::transmute};

use nom::{Input, Offset, Parser};
use nom_locate::LocatedSpan;

pub type Result<'a, O> = nom::IResult<Span<'a>, O, Error<'a>>;

pub type Span<'a> = LocatedSpan<&'a str>;

pub trait SpanExt<'a> {
    fn diff(&self, other: Span) -> Span<'a>;
}

impl<'a> SpanExt<'a> for Span<'a> {
    fn diff(&self, other: Span) -> Span<'a> {
        let offset: usize = self.offset(&other);
        self.take(offset)
    }
}

#[derive(Debug, Clone)]
pub struct Error<'a> {
    pub input: Span<'a>,
    pub message: Cow<'a, str>,
}

impl<'a> Error<'a> {
    pub fn new<M: Into<Cow<'a, str>>>(input: Span<'a>, message: M) -> Self {
        Self {
            input,
            message: message.into(),
        }
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for Error<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Error::new(input, unsafe {
            transmute::<_, &'static str>(kind.description())
        })
    }

    fn append(_: Span<'a>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> nom::error::FromExternalError<&'a str, Error<'a>> for Error<'a> {
    fn from_external_error(_: &'a str, _: nom::error::ErrorKind, e: Error<'a>) -> Self {
        e
    }
}

impl<'a, E: Into<Cow<'a, str>>> nom::error::FromExternalError<Span<'a>, E> for Error<'a> {
    fn from_external_error(input: Span<'a>, _: nom::error::ErrorKind, e: E) -> Self {
        Error::new(input, e)
    }
}

pub trait ParserExt<'a>: Parser<Span<'a>> {
    fn parse_or<M: Into<Cow<'a, str>>>(
        &mut self,
        input: Span<'a>,
        msg: M,
    ) -> Result<'a, Self::Output>;
}

impl<'a, P: Parser<Span<'a>>> ParserExt<'a> for P {
    fn parse_or<M: Into<Cow<'a, str>>>(
        &mut self,
        input: Span<'a>,
        msg: M,
    ) -> Result<'a, P::Output> {
        self.parse(input)
            .map_err(|err| err.map(|_| Error::new(input, msg)))
    }
}
