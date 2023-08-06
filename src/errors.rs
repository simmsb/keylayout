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

    #[error("Badly positioned chord")]
    #[diagnostic(
        code(bad_chord_positioning),
        help("Chords should be positioned between two normal keys")
    )]
    BadChordPositions {
        #[label("For this to be a valid chord")]
        bad_chord: Span,

        #[label("There should be a key here")]
        prev_item: Span,

        #[label("There also needs to be a key here")]
        next_item: Span,
    },

    #[error("Unknown key: {key}")]
    #[diagnostic(code(unknown_key), help("Try picking a key that exists, huh?"))]
    UnknownKey {
        #[label("I don't understand this key")]
        span: Span,

        key: String,
    },
}
