use std::{borrow::Cow, collections::HashSet};

use chumsky::span::SimpleSpan;
use locspan::Spanned;
use miette::SourceSpan;
use pad::PadStr;
use pretty::RcDoc;

use crate::format::KeySpacing;

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
        Self(SourceSpan::new(self.0.offset().into(), 0))
    }

    pub fn end_singleton(self) -> Self {
        Self(SourceSpan::new((self.0.offset() + self.0.len()).into(), 0))
    }

    pub fn len(&self) -> usize {
        self.0.len()
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

impl<const T: &'static str, S> Token<T, S> {
    pub fn to_doc(&self) -> RcDoc {
        RcDoc::text(T)
    }
}

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

impl<'a, S> Ident<'a, S> {
    pub fn to_doc(&self) -> RcDoc {
        RcDoc::text(self.s)
    }
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
    pub options: Vec<Options<'a, S>>,
    pub custom_keys: Vec<CustomKey<'a, S>>,
    pub layers: Vec<Layer<'a, S>>,
    pub span: S,
}

impl<'a> File<'a> {
    pub fn to_doc(&self, spacing: &[KeySpacing], empties: &HashSet<(u8, u8)>) -> RcDoc {
        let twoline = RcDoc::line().append(RcDoc::line_());
        self.layout
            .to_doc()
            .append(twoline.clone())
            .append(RcDoc::intersperse(
                self.options.iter().map(|o| o.to_doc()),
                twoline.clone(),
            ))
            .append(twoline.clone())
            .append(RcDoc::intersperse(
                self.custom_keys.iter().map(|o| o.to_doc()),
                twoline.clone(),
            ))
            .append(twoline.clone())
            .append(RcDoc::intersperse(
                self.layers.iter().map(|o| o.to_doc(spacing, empties)),
                twoline.clone(),
            ))
            .append(RcDoc::line())
    }
}

impl<'a, S: Copy> Spanned for File<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct Options<'a, S = Span> {
    pub options_token: Token<"options", S>,
    pub for_: OptionsFor<S>,
    pub left_curly: Token<"{", S>,
    pub items: Vec<OptionsItem<'a, S>>,
    pub right_curly: Token<"}", S>,
    pub span: S,
}

impl<'a> Options<'a> {
    pub fn to_doc(&self) -> RcDoc {
        self.options_token
            .to_doc()
            .append(RcDoc::space())
            .append(self.for_.to_doc())
            .append(RcDoc::space())
            .append(self.left_curly.to_doc())
            .append(
                RcDoc::concat(self.items.iter().map(|i| RcDoc::line().append(i.to_doc()))).nest(2),
            )
            .append(RcDoc::line())
            .append(self.right_curly.to_doc())
    }
}

impl<'a, S: Copy> Spanned for Options<'a, S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        self.span
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub enum OptionsFor<S = Span> {
    RustyDilemma(Token<"rusty_dilemma", S>),
    KeymapDrawer(Token<"keymap_drawer", S>),
    Formatter(Token<"formatter", S>),
}

impl OptionsFor {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            OptionsFor::RustyDilemma(x) => x.to_doc(),
            OptionsFor::KeymapDrawer(x) => x.to_doc(),
            OptionsFor::Formatter(x) => x.to_doc(),
        }
    }
}

impl<S: Copy> Spanned for OptionsFor<S> {
    type Span = S;

    fn span(&self) -> Self::Span {
        match self {
            OptionsFor::RustyDilemma(t) => t.span(),
            OptionsFor::KeymapDrawer(t) => t.span(),
            OptionsFor::Formatter(t) => t.span(),
        }
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct OptionsItem<'a, S = Span> {
    pub name: Ident<'a, S>,
    pub colon: Token<":", S>,
    pub value: Text<'a, S>,
    pub semi: Token<";", S>,
    pub span: S,
}

impl<'a> OptionsItem<'a> {
    pub fn to_doc(&self) -> RcDoc {
        self.name
            .to_doc()
            .append(self.colon.to_doc())
            .append(RcDoc::space())
            .append(self.value.to_doc())
            .append(self.semi.to_doc())
    }
}

impl<'a, S: Copy> Spanned for OptionsItem<'a, S> {
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

impl<'a> CustomKey<'a> {
    pub fn to_doc(&self) -> RcDoc {
        self.key_token
            .to_doc()
            .append(RcDoc::space())
            .append(self.name.to_doc())
            .append(RcDoc::space())
            .append(self.left_curly.to_doc())
            .append(
                RcDoc::concat(
                    self.outputs
                        .iter()
                        .map(|i| RcDoc::line().append(i.to_doc())),
                )
                .nest(2),
            )
            .append(RcDoc::line())
            .append(self.right_curly.to_doc())
    }
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

impl<'a> CustomKeyOutput<'a> {
    pub fn to_doc(&self) -> RcDoc {
        self.out_token
            .to_doc()
            .append(RcDoc::space())
            .append(self.name.to_doc())
            .append(self.colon.to_doc())
            .append(RcDoc::space())
            .append(self.output.to_doc())
            .append(self.semi.to_doc())
    }
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

impl<'a> Text<'a> {
    pub fn to_doc(&self) -> RcDoc {
        RcDoc::text(format!("{:?}", self.text))
    }
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

impl Layout {
    pub fn to_doc(&self) -> RcDoc {
        self.layout_token
            .to_doc()
            .append(RcDoc::space())
            .append(self.left_curly.to_doc())
            .append(
                RcDoc::concat(self.rows.iter().map(|i| RcDoc::line().append(i.to_doc()))).nest(2),
            )
            .append(RcDoc::line())
            .append(self.right_curly.to_doc())
    }
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

impl LayoutRow {
    pub fn to_doc(&self) -> RcDoc {
        let doc = RcDoc::intersperse(self.items.iter().map(|i| i.to_doc()), RcDoc::softline());

        doc.append(self.semi.to_doc())
    }
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

impl LayoutDefn {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            LayoutDefn::Keys {
                count,
                k: _,
                span: _,
            } => RcDoc::text(format!("{}k", count)),
            LayoutDefn::RemappedKey {
                left_bracket,
                position,
                right_bracket,
                span: _,
            } => left_bracket
                .to_doc()
                .append(RcDoc::as_string(position))
                .append(right_bracket.to_doc()),
            LayoutDefn::Spaces {
                count,
                s: _,
                span: _,
            } => RcDoc::text(format!("{}s", count)),
        }
    }
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

impl<'a> Layer<'a> {
    pub fn to_doc(&self, spacing: &[KeySpacing], empties: &HashSet<(u8, u8)>) -> RcDoc {
        let mut doc = RcDoc::nil();
        for (y, row) in self.rows.iter().enumerate() {
            let empties = empties
                .iter()
                .copied()
                .filter_map(|(x, y_)| (y_ == y as u8).then_some(x))
                .collect::<HashSet<_>>();

            doc = doc.append(RcDoc::line());
            doc = doc.append(row.to_doc(spacing, &empties));
        }

        self.layer_token
            .to_doc()
            .append(RcDoc::space())
            .append(self.name.to_doc())
            .append(RcDoc::space())
            .append(self.left_curly.to_doc())
            .append(doc.nest(2))
            .append(RcDoc::line())
            .append(self.right_curly.to_doc())
    }
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

impl<'a> LayerRow<'a> {
    pub fn to_doc(&self, spacing: &[KeySpacing], empties: &HashSet<u8>) -> RcDoc {
        let mut doc = RcDoc::nil();

        let mut items_it = self.items.iter().peekable();
        let mut is_first = true;

        for (idx, s) in spacing.iter().enumerate() {
            let idx = idx as u8;

            if empties.contains(&idx) {
                if items_it.peek().is_some() {
                    if !is_first {
                        doc = doc.append(RcDoc::softline());
                    }
                    doc = doc.append(RcDoc::text(" ".repeat(s.key_width)));
                    doc = doc.append(RcDoc::softline());
                    doc = doc.append(RcDoc::text(" ".repeat(s.chord_width)));
                }
            } else {
                if !is_first {
                    doc = doc.append(RcDoc::softline());
                }
                let item = items_it.next().unwrap();
                let (key_width, chord_width) = if items_it.peek().is_some() {
                    (s.key_width, s.chord_width)
                } else {
                    (0, 0)
                };

                doc = doc.append(item.to_doc(key_width, chord_width));

                if let Some(KeyOrChord::Chord(_)) = items_it.peek() {
                    doc = doc.append(RcDoc::softline());
                    doc = doc.append(items_it.next().unwrap().to_doc(s.key_width, s.chord_width));
                } else if items_it.peek().is_some() {
                    doc = doc.append(RcDoc::softline());
                    doc = doc.append(RcDoc::text(" ".repeat(s.chord_width)));
                }
            }

            is_first = false;
        }

        doc.append(self.semi.to_doc())
    }
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

impl<'a> KeyOrChord<'a> {
    pub fn to_doc(&self, key_spacing: usize, chord_spacing: usize) -> RcDoc {
        match self {
            KeyOrChord::Key(k) => k.to_doc(Some(key_spacing)),
            KeyOrChord::Chord(c) => c.to_doc(chord_spacing),
        }
    }
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

impl<'a> Chord<'a> {
    pub fn to_doc(&self, spacing: usize) -> RcDoc {
        let d = self
            .right_angle
            .to_doc()
            .append(self.key.to_doc(None))
            .append(self.left_angle.to_doc());

        let plain = d.pretty(self.span().len()).to_string();

        let padded = plain.pad_to_width(spacing);

        RcDoc::text(padded)
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub enum ModTapType<S = Span> {
    Permissive(Token<"@", S>),
    OnOtherKey(Token<"@~", S>),
}

impl ModTapType {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            ModTapType::Permissive(t) => t.to_doc(),
            ModTapType::OnOtherKey(t) => t.to_doc(),
        }
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub struct ModTapTimeout<S = Span> {
    pub left_square: Token<"[", S>,
    pub timeout: u32,
    pub right_square: Token<"]", S>,
    pub span: S,
}

impl ModTapTimeout {
    pub fn to_doc(&self) -> RcDoc {
        self.left_square
            .to_doc()
            .append(RcDoc::text(self.timeout.to_string()))
            .append(self.right_square.to_doc())
    }
}

#[derive(Debug, debug3::Debug, Clone, PartialEq, Eq)]
pub enum Key<'a, S = Span> {
    Plain(PlainKey<'a, S>),
    ModTap {
        tap: PlainKey<'a, S>,
        at: ModTapType<S>,
        timeout: Option<ModTapTimeout<S>>,
        hold: PlainKey<'a, S>,
        span: S,
    },
}

impl<'a> Key<'a> {
    pub fn to_doc(&self, spacing: Option<usize>) -> RcDoc {
        let d = match self {
            Key::Plain(p) => p.to_doc(),
            Key::ModTap {
                tap,
                at,
                timeout,
                hold,
                span: _,
            } => tap
                .to_doc()
                .append(at.to_doc())
                .append(timeout.as_ref().map_or(RcDoc::nil(), ModTapTimeout::to_doc))
                .append(hold.to_doc()),
        };

        if let Some(spacing) = spacing {
            let plain = d.pretty(self.span().len()).to_string();

            let padded = plain.pad_to_width(spacing);

            RcDoc::text(padded)
        } else {
            d
        }
    }
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
        left_quote: Token<"'", S>,
        c: char,
        right_quote: Token<"'", S>,
        span: S,
    },
}

impl<'a> PlainKey<'a> {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            PlainKey::Named(name) => name.to_doc(),
            PlainKey::Layer {
                left_square,
                layer,
                right_square,
                span: _,
            } => left_square
                .to_doc()
                .append(layer.to_doc())
                .append(right_square.to_doc()),
            PlainKey::Char {
                left_quote,
                c,
                right_quote,
                span: _,
            } => left_quote
                .to_doc()
                .append(RcDoc::as_string(c))
                .append(right_quote.to_doc()),
        }
    }
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
