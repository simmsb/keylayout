use chumsky::span::SimpleSpan as Span;
use locspan::Spanned;

pub trait IsToken {
    const VALUE: &'static str;
}

// struct Layer;
// struct OpenCurly;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<const T: &'static str, S = Span>(pub S);

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlainKey<'a, S = Span> {
    Named(Ident<'a, S>),
    Char { c: char, span: S },
}

impl<'a, S: Copy> Spanned for PlainKey<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        match self {
            PlainKey::Named(n) => n.span(),
            PlainKey::Char { span, .. } => *span,
        }
    }
}
