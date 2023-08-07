use std::borrow::Cow;

use chumsky::span::SimpleSpan;
use locspan::Spanned;
use miette::SourceSpan;

#[derive(Copy, Clone, Debug)]
pub struct Span(pub SourceSpan);

impl debug3::Debug for Span {
    fn fmt(&self, f: &mut debug3::Formatter) {
        debug3::Debug::fmt(
            &format!("{}..{}", self.0.offset(), self.0.offset() + self.0.len()),
            f,
        )
    }
}

impl Span {
    pub fn start_singleton(self) -> Self {
        Self(SourceSpan::new(self.0.offset().into(), 0.into()))
    }

    pub fn end_singleton(self) -> Self {
        Self(SourceSpan::new(
            (self.0.offset() + self.0.len()).into(),
            0.into(),
        ))
    }
}

impl From<SimpleSpan> for Span {
    fn from(span: SimpleSpan) -> Self {
        let s = span.start;
        let e = span.end;

        Self(SourceSpan::new(s.into(), (e - s).into()))
    }
}

impl From<&SimpleSpan> for Span {
    fn from(value: &SimpleSpan) -> Self {
        Span::from(*value)
    }
}

impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        self.0
    }
}

impl Into<SourceSpan> for &Span {
    fn into(self) -> SourceSpan {
        self.0
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct Token<const T: &'static str, S = Span>(pub S);

impl<const T: &'static str, S: Copy> Spanned for Token<T, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.0
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct Ident<'a, S = Span> {
    pub s: &'a str,
    pub span: S,
}

impl<'a, S: Copy> Spanned for Ident<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct File<'a, S = Span> {
    pub layout: Layout<S>,
    pub custom_keys: Vec<CustomKey<'a, S>>,
    pub layers: Vec<Layer<'a, S>>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for File<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct CustomKey<'a, S = Span> {
    pub key_token: Token<"key", S>,
    pub name: Ident<'a, S>,
    pub left_curly: Token<"{", S>,
    pub outputs: Vec<CustomKeyOutput<'a, S>>,
    pub right_curly: Token<"}", S>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for CustomKey<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct CustomKeyOutput<'a, S = Span> {
    pub out_token: Token<"out", S>,
    pub name: Ident<'a, S>,
    pub colon: Token<":", S>,
    pub output: Text<'a, S>,
    pub semi: Token<";", S>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for CustomKeyOutput<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct Text<'a, S = Span> {
    pub left_quote: Token<"\"", S>,
    pub text: Cow<'a, str>,
    pub right_quote: Token<"\"", S>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for Text<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct Layout<S = Span> {
    pub layout_token: Token<"layout", S>,
    pub left_curly: Token<"{", S>,
    pub rows: Vec<LayoutRow<S>>,
    pub right_curly: Token<"}", S>,
    pub span: S,
}

impl<S: Copy> Spanned for Layout<S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct LayoutRow<S = Span> {
    pub items: Vec<LayoutDefn<S>>,
    pub semi: Token<";", S>,
    pub span: S,
}

impl<S: Copy> Spanned for LayoutRow<S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub enum LayoutDefn<S = Span> {
    Keys {
        count: u8,
        k: Token<"k", S>,
        span: S,
    },
    RemappedKey {
        left_bracket: Token<"[", S>,
        position: u8,
        right_bracket: Token<"]", S>,
        span: S,
    },
    Spaces {
        count: u8,
        s: Token<"s", S>,
        span: S,
    },
}

impl<'a, S: Copy> Spanned for LayoutDefn<S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        match self {
            LayoutDefn::Keys {
                count: _,
                k: _,
                span,
            } => *span,
            LayoutDefn::RemappedKey {
                left_bracket: _,
                position: _,
                right_bracket: _,
                span,
            } => *span,
            LayoutDefn::Spaces {
                count: _,
                s: _,
                span,
            } => *span,
        }
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct Layer<'a, S = Span> {
    pub layer_token: Token<"layer", S>,
    pub name: Ident<'a, S>,
    pub left_curly: Token<"{", S>,
    pub rows: Vec<LayerRow<'a, S>>,
    pub right_curly: Token<"}", S>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for Layer<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct LayerRow<'a, S = Span> {
    pub items: Vec<KeyOrChord<'a, S>>,
    pub semi: Token<";", S>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for LayerRow<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub enum KeyOrChord<'a, S = Span> {
    Key(Key<'a, S>),
    Chord(Chord<'a, S>),
}

impl<'a, S: Copy> Spanned for KeyOrChord<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        match self {
            KeyOrChord::Key(k) => k.span(),
            KeyOrChord::Chord(c) => c.span(),
        }
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct Chord<'a, S = Span> {
    pub right_angle: Token<">", S>,
    pub key: Key<'a, S>,
    pub left_angle: Token<"<", S>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for Chord<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub enum Key<'a, S = Span> {
    Plain(PlainKey<'a, S>),
    ModTap {
        tap: PlainKey<'a, S>,
        at: Token<"@", S>,
        hold: PlainKey<'a, S>,
        span: S,
    },
}

impl<'a, S: Copy> Spanned for Key<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        match self {
            Key::Plain(p) => p.span(),
            Key::ModTap { span, .. } => *span,
        }
    }
}

// #[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
// pub enum Named<S = Span> {
//     Esc(Token<"esc", S>),
//     Space(Token<"space", S>),
//     BSpace(Token<"bspace", S>),
//     Del(Token<"del", S>),
//     LShift(Token<"lshift", S>),
//     RShift(Token<"rshift", S>),
//     LCtrl(Token<"lshift", S>),
//     RShift(Token<"rshift", S>),
//     LAlt(Token<"lalt", S>),
//     RAlt(Token<"ralt", S>),
//     Tab(Token<"tab", S>),
//     Win(Token<"win", S>),
//     Enter(Token<"enter", S>),
// }

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub enum PlainKey<'a, S = Span> {
    Named(Ident<'a, S>),
    Layer {
        left_square: Token<"[", S>,
        layer: Ident<'a, S>,
        right_square: Token<"]", S>,
        span: S,
    },
    Char {
        c: char,
        span: S,
    },
}

impl<'a, S: Copy> Spanned for PlainKey<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        match self {
            PlainKey::Named(n) => n.span(),
            PlainKey::Layer { span, .. } => *span,
            PlainKey::Char { span, .. } => *span,
        }
    }
}
