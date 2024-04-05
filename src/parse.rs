use std::borrow::Cow;

use chumsky::{
    combinator::{Map, ToSpan},
    label::Labelled,
    prelude::*,
    primitive::Just,
    text::int,
};
use itertools::Itertools;
use thiserror::Error;

use crate::syntax::{
    Chord, CustomKey, CustomKeyOutput, File, Ident, Key, KeyOrChord, Layer, LayerRow, Layout,
    LayoutDefn, LayoutRow, ModTapType, Options, OptionsFor, OptionsItem, PlainKey, Span, Text,
    Token,
};

pub fn file<'a>() -> impl Parser<'a, &'a str, File<'a>, extra::Err<Rich<'a, char>>> {
    group((
        layout(),
        options().padded().repeated().collect(),
        custom_key().padded().repeated().collect(),
        layer().padded().repeated().collect(),
    ))
    .map_with_span(|(layout, options, custom_keys, layers), span| File {
        layout,
        options,
        custom_keys,
        layers,
        span: span.into(),
    })
}

pub fn layout<'a>() -> impl Parser<'a, &'a str, Layout, extra::Err<Rich<'a, char>>> {
    token::<"layout">()
        .padded()
        .then(token::<"{">().padded())
        .then(layout_row().padded().repeated().collect())
        .then(token::<"}">().padded())
        .map_with_span(
            |(((layout_token, left_curly), rows), right_curly), span| Layout {
                layout_token,
                left_curly,
                rows,
                right_curly,
                span: span.into(),
            },
        )
}

fn layout_row<'a>() -> impl Parser<'a, &'a str, LayoutRow, extra::Err<Rich<'a, char>>> {
    layout_defn()
        .padded()
        .repeated()
        .at_least(1)
        .collect()
        .then(token::<";">())
        .padded()
        .map_with_span(|(items, semi), span| LayoutRow {
            items,
            semi,
            span: span.into(),
        })
        .labelled("layout row")
}

pub fn layout_defn<'a>() -> impl Parser<'a, &'a str, LayoutDefn, extra::Err<Rich<'a, char>>> {
    let i = int(10).try_map(|s: &str, span| s.parse().map_err(|e| Rich::custom(span, e)));

    let k = i
        .then(token::<"k">())
        .map_with_span(|(count, k), span| LayoutDefn::Keys {
            count,
            k,
            span: span.into(),
        });

    let s = i
        .then(token::<"s">())
        .map_with_span(|(count, s), span| LayoutDefn::Spaces {
            count,
            s,
            span: span.into(),
        });

    let remapped = token::<"[">().then(i).then(token::<"]">()).map_with_span(
        |((left_bracket, position), right_bracket), span| LayoutDefn::RemappedKey {
            left_bracket,
            position,
            right_bracket,
            span: span.into(),
        },
    );

    k.or(s).or(remapped)
}

pub fn options<'a>() -> impl Parser<'a, &'a str, Options<'a>, extra::Err<Rich<'a, char>>> {
    group((
        token::<"options">().padded(),
        options_for().padded(),
        token::<"{">().padded(),
        options_item().padded().repeated().collect(),
        token::<"}">().padded(),
    ))
    .map_with_span(
        |(options_token, for_, left_curly, items, right_curly), span| Options {
            options_token,
            for_,
            left_curly,
            items,
            right_curly,
            span: span.into(),
        },
    )
}
pub fn options_for<'a>() -> impl Parser<'a, &'a str, OptionsFor, extra::Err<Rich<'a, char>>> {
    choice((
        token::<"rusty_dilemma">().map(OptionsFor::RustyDilemma),
        token::<"keymap_drawer">().map(OptionsFor::KeymapDrawer),
        token::<"formatter">().map(OptionsFor::Formatter),
    ))
}

pub fn options_item<'a>() -> impl Parser<'a, &'a str, OptionsItem<'a>, extra::Err<Rich<'a, char>>> {
    group((
        ident().padded(),
        token::<":">().padded(),
        text().padded(),
        token::<";">().padded(),
    ))
    .map_with_span(|(name, colon, value, semi), span| OptionsItem {
        name,
        colon,
        value,
        semi,
        span: span.into(),
    })
    .labelled("custom key output")
}

pub fn custom_key<'a>() -> impl Parser<'a, &'a str, CustomKey<'a>, extra::Err<Rich<'a, char>>> {
    group((
        token::<"key">().padded(),
        ident().padded(),
        token::<"{">().padded(),
        custom_key_output().padded().repeated().collect(),
        token::<"}">().padded(),
    ))
    .map_with_span(
        |(key_token, name, left_curly, outputs, right_curly), span| CustomKey {
            key_token,
            name,
            left_curly,
            outputs,
            right_curly,
            span: span.into(),
        },
    )
}

pub fn custom_key_output<'a>(
) -> impl Parser<'a, &'a str, CustomKeyOutput<'a>, extra::Err<Rich<'a, char>>> {
    group((
        token::<"out">().padded(),
        ident().padded(),
        token::<":">().padded(),
        text().padded(),
        token::<";">().padded(),
    ))
    .map_with_span(
        |(out_token, name, colon, output, semi), span| CustomKeyOutput {
            out_token,
            name,
            colon,
            output,
            semi,
            span: span.into(),
        },
    )
    .labelled("custom key output")
}

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
                span: span.into(),
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
        .map_with_span(|(items, semi), span| LayerRow {
            items,
            semi,
            span: span.into(),
        })
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
            span: span.into(),
        })
        .labelled("chord")
}

fn key<'a>() -> impl Parser<'a, &'a str, Key<'a>, extra::Err<Rich<'a, char>>> {
    let p = plainkey().map(Key::Plain);
    let mt = plainkey()
        .then(
            token::<"@~">()
                .map(ModTapType::OnOtherKey)
                .or(token::<"@">().map(ModTapType::Permissive)),
        )
        .then(plainkey())
        .map_with_span(|((tap, at), hold), span| Key::ModTap {
            tap,
            at,
            hold,
            span: span.into(),
        });

    mt.or(p).labelled("key")
}

fn plainkey<'a>() -> impl Parser<'a, &'a str, PlainKey<'a>, extra::Err<Rich<'a, char>>> {
    let i = ident().map(PlainKey::Named);
    let l = token::<"[">()
        .then(ident())
        .then(token::<"]">())
        .map_with_span(
            |((left_square, layer), right_square), span| PlainKey::Layer {
                left_square,
                layer,
                right_square,
                span: span.into(),
            },
        );
    let c = group((token::<"'">(), any(), token::<"'">())).map_with_span(
        |(left_quote, c, right_quote), span: SimpleSpan| PlainKey::Char {
            left_quote,
            c,
            right_quote,
            span: span.into(),
        },
    );

    i.or(l).or(c).labelled("plain key")
}

fn token<'a, const T: &'static str>() -> Labelled<
    Map<
        ToSpan<Just<&'static str, &'a str, extra::Err<Rich<'a, char>>>, &'static str>,
        SimpleSpan,
        fn(SimpleSpan) -> Token<T>,
    >,
    &'static str,
> {
    just(T)
        .to_span()
        .map((|s: SimpleSpan| Token(s.into())) as fn(_) -> _)
        .labelled(T)
}

fn ident<'a>() -> impl Parser<'a, &'a str, Ident<'a>, extra::Err<Rich<'a, char>>> {
    group((
        any()
            .filter(|c: &char| c.is_alphabetic() || "-_".contains(*c))
            .ignored(),
        any()
            .filter(|c: &char| c.is_alphanumeric() || "-_".contains(*c))
            .repeated()
            .ignored(),
    ))
    .slice()
    .map_with_span(|t, s: SimpleSpan| Ident {
        s: t,
        span: s.into(),
    })
}

fn text<'a>() -> impl Parser<'a, &'a str, Text<'a>, extra::Err<Rich<'a, char>>> {
    let escape = just('\\').then(choice((just('\\'), just('"')))).ignored();

    let escaped_string = none_of("\n\\\"")
        .ignored()
        .or(escape)
        .ignored()
        .repeated()
        .slice()
        .map(|text: &str| {
            if text.contains('\\') {
                Cow::Owned(text.replace("\\\"", "\"").replace("\\\\", "\\"))
            } else {
                Cow::Borrowed(text)
            }
        });

    group((token::<"\"">(), escaped_string, token::<"\"">()))
        .map_with_span(|(left_quote, text, right_quote), span| Text {
            left_quote,
            text,
            right_quote,
            span: span.into(),
        })
        .labelled("a quoted string")
}

#[derive(Error, Debug, miette::Diagnostic)]
#[error("While parsing {name}")]
pub struct LabelNote {
    #[label("{name}")]
    pub err_span: Span,

    pub name: String,
}

#[derive(Error, Debug, miette::Diagnostic)]
#[error("Failed to parse")]
pub enum ParseError {
    #[error("Unexpected input: {found}")]
    UnexpectedInput {
        #[label("{expected_msg}")]
        err_span: Span,

        expected_msg: String,

        found: String,

        #[related]
        contexts: Vec<LabelNote>,
    },

    #[error("{custom}")]
    Custom {
        #[label]
        err_span: Span,

        custom: String,

        #[related]
        contexts: Vec<LabelNote>,
    },
    // #[error("Multiple errors happened")]
    // Multiple {
    //     #[label]
    //     err_span: miette::SourceSpan,

    //     #[related]
    //     contexts: Vec<LabelNote>,

    //     #[related]
    //     errors: Vec<Self>,
    // },
}

pub fn convert_error<'a>(err: Rich<'a, char>) -> ParseError {
    let contexts = err
        .contexts()
        .map(|(l, span)| LabelNote {
            err_span: span.into(),
            name: l.to_string(),
        })
        .collect::<Vec<_>>();

    match err.reason() {
        chumsky::error::RichReason::ExpectedFound { .. } => {
            let expected = err.expected().map(|x| x.to_string()).join(", ");
            let found = if let Some(m) = err.found() {
                format!("{:?}", m.to_string())
            } else {
                "EOF".to_string()
            };

            ParseError::UnexpectedInput {
                err_span: err.span().into(),
                expected_msg: format!("Expected: {expected}"),
                found,
                contexts,
            }
        }
        chumsky::error::RichReason::Custom(m) => ParseError::Custom {
            err_span: err.span().into(),
            custom: m.to_string(),
            contexts,
        },
        chumsky::error::RichReason::Many(_o) => {
            panic!("idk")
        }
    }
}
