use std::io;

use thiserror::Error;

use crate::syntax::Span;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum AppError {
    #[error(transparent)]
    #[diagnostic(code(io_error), help("I couldn't read or write a file"))]
    IOError(#[from] io::Error),

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

        key: char,
    },

    #[error("Unknown named key: {key}")]
    #[diagnostic(
        code(unknown_named_key),
        help("The following similar keys exist: {similar}")
    )]
    UnknownNamedKey {
        #[label("I don't know this key")]
        span: Span,

        key: String,

        similar: String,
    },

    #[error("Unknown layer: {layer}")]
    #[diagnostic(
        code(unknown_named_layer),
        help("The following similar layers exist: {similar}")
    )]
    UnknownNamedLayer {
        #[label("I don't know this layer")]
        span: Span,

        layer: String,

        similar: String,
    },

    #[error("Inconsistent matrix width")]
    #[diagnostic(
        code(bad_matrix_width),
        help("All rows of a matrix need to have the same number of keys")
    )]
    InconsistentMatrixWidth {
        #[label("This row should have {expected} keys, but it has {got}")]
        bad_row: Span,

        got: u8,
        expected: u8,
    },

    #[error("An option is required")]
    #[diagnostic(
        code(required_option),
        help("The option {option_name} is required for {backend}")
    )]
    OptionRequired {
        option_name: String,
        backend: String,
    },
}
