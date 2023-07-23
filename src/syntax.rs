use chumsky::span::SimpleSpan;
use locspan::Spanned;
use miette::SourceSpan;

#[derive(Copy, Clone, Debug)]
pub struct Span(pub SourceSpan);

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
pub struct File<'a, S = Span> {
    pub layout: Layout<S>,
    pub layers: Vec<Layer<'a, S>>,
    pub span: S,
}

impl<'a, S: Copy> Spanned for File<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
            LayoutDefn::Keys { count, k, span } => *span,
            LayoutDefn::RemappedKey {
                left_bracket,
                position,
                right_bracket,
                span,
            } => *span,
            LayoutDefn::Spaces { count, s, span } => *span,
        }
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
