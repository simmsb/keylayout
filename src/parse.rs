use chumsky::{
    combinator::{Map, ToSpan},
    prelude::*,
    primitive::Just,
};
use itertools::Itertools;
use thiserror::Error;

use crate::syntax::{Chord, Ident, Key, KeyOrChord, Layer, LayerRow, PlainKey, Token};

pub fn layer<'a>() -> impl Parser<'a, &'a str, Layer<'a>, extra::Err<Rich<'a, char>>> {
    token::<"layer">()
        .padded()
        .then(ident().padded())
        .then(token::<"{">().padded())
        .then(layer_row().padded().repeated().collect())
        .then(token::<"}">().padded())
        .map_with_span(
            |((((layer_token, name), left_curly), rows), right_curly), span| Layer {
                layer_token,
                name,
                left_curly,
                rows,
                right_curly,
                span,
            },
        )
        .labelled("layer")
}

fn layer_row<'a>() -> impl Parser<'a, &'a str, LayerRow<'a>, extra::Err<Rich<'a, char>>> {
    key_or_chord()
        .padded()
        .repeated()
        .at_least(1)
        .collect()
        .then(token::<";">())
        .padded()
        .map_with_span(|(items, semi), span| LayerRow { items, semi, span })
        .labelled("row")
}

fn key_or_chord<'a>() -> impl Parser<'a, &'a str, KeyOrChord<'a>, extra::Err<Rich<'a, char>>> {
    key()
        .map(KeyOrChord::Key)
        .or(chord().map(KeyOrChord::Chord))
}

fn chord<'a>() -> impl Parser<'a, &'a str, Chord<'a>, extra::Err<Rich<'a, char>>> {
    token::<">">()
        .then(key())
        .then(token::<"<">())
        .map_with_span(|((right_angle, key), left_angle), span| Chord {
            right_angle,
            key,
            left_angle,
            span,
        }).labelled("chord")
}

fn key<'a>() -> impl Parser<'a, &'a str, Key<'a>, extra::Err<Rich<'a, char>>> {
    let p = plainkey().map(Key::Plain);
    let mt = plainkey()
        .then(token::<"@">())
        .then(plainkey())
        .map_with_span(|((tap, at), hold), span| Key::ModTap {
            tap,
            at,
            hold,
            span,
        });

    mt.or(p).labelled("key")
}

fn plainkey<'a>() -> impl Parser<'a, &'a str, PlainKey<'a>, extra::Err<Rich<'a, char>>> {
    let i = ident().map(PlainKey::Named);
    let c = any()
        .delimited_by(just('\''), just('\''))
        .map_with_span(|c, span| PlainKey::Char { c, span });
    let c2 = any()
        .delimited_by(just('"'), just('"'))
        .map_with_span(|c, span| PlainKey::Char { c, span });

    i.or(c).or(c2).labelled("plain key")
}

fn token<'a, const T: &'static str>() -> Map<
    ToSpan<Just<&'static str, &'a str, extra::Err<Rich<'a, char>>>, &'static str>,
    SimpleSpan,
    fn(SimpleSpan) -> Token<T>,
> {
    just(T)
        .to_span()
        .map((|s: SimpleSpan| Token(s)) as fn(_) -> _)
}

fn ident<'a>() -> impl Parser<'a, &'a str, Ident<'a>, extra::Err<Rich<'a, char>>> {
    any()
        .filter(|c: &char| c.is_alphabetic())
        .repeated()
        .at_least(1)
        .map_slice(|t| t)
        .map_with_span(|t, s| Ident { s: t, span: s })
}

#[derive(Error, Debug, miette::Diagnostic)]
#[error("While parsing {name}")]
pub struct LabelNote {
    #[label("{name}")]
    pub err_span: miette::SourceSpan,

    pub name: String,
}

#[derive(Error, Debug, miette::Diagnostic)]
#[error("Failed to parse")]
pub enum ParseError {
    #[error("Unexpected input: {found}")]
    UnexpectedInput {
        #[label("{expected_msg}")]
        err_span: miette::SourceSpan,

        expected_msg: String,

        found: String,

        #[related]
        contexts: Vec<LabelNote>,
    },

    #[error("{custom}")]
    Custom {
        #[label]
        err_span: miette::SourceSpan,

        custom: String,

        #[related]
        contexts: Vec<LabelNote>,
    },

    #[error("Multiple errors happened")]
    Multiple {
        #[label]
        err_span: miette::SourceSpan,

        #[related]
        contexts: Vec<LabelNote>,

        #[related]
        errors: Vec<Self>,
    },
}

fn convert_span(span: SimpleSpan) -> miette::SourceSpan {
    let s = span.start();
    let e = span.end();

    miette::SourceSpan::new(s.into(), (e - s).into())
}

pub fn convert_error<'a>(err: Rich<'a, char>) -> ParseError {
    let contexts = err
        .contexts()
        .map(|(l, span)| LabelNote {
            err_span: convert_span(*span),
            name: l.to_string(),
        })
        .collect::<Vec<_>>();

    let span = convert_span(*err.span());

    match err.reason() {
        chumsky::error::RichReason::ExpectedFound { expected, found } => {
            let expected = expected.iter().map(|x| x.to_string()).join(", ");
            let found = if let Some(m) = found {
                m.to_string()
            } else {
                "<nothing>".to_string()
            };

            ParseError::UnexpectedInput {
                err_span: span,
                expected_msg: format!("Expected: {expected}"),
                found,
                contexts,
            }
        }
        chumsky::error::RichReason::Custom(m) => ParseError::Custom {
            err_span: span,
            custom: m.to_string(),
            contexts,
        },
        chumsky::error::RichReason::Many(o) => {
            panic!("idk")
        }
    }
}
