use thiserror::Error;

use crate::syntax::Span;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum AppError {
    #[error("Overlapping keys on layout")]
    #[diagnostic(
        code(overlapping_keys),
        help("Keys should all end up on unique matrix positions")
    )]
    OverlappingKeys {
        #[label("This key")]
        span: Span,

        #[label("Overlaps with this key")]
        other_span: Span,
    },

    #[error("Unknown key: {key}")]
    #[diagnostic(code(unknown_key), help("Try picking a key that exists, huh?"))]
    UnknownKey {
        #[label("I don't understand this key")]
        span: Span,

        key: String,
    },

    #[error("Chords need to be between two normal keys")]
    BadChord {
        #[label("I don't know what to do with this chord")]
        span: Span,
    },
}
