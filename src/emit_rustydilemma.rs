use std::{collections::HashMap, io::Write};

use itertools::Itertools;
use ngrammatic::CorpusBuilder;
use once_cell::sync::Lazy;

use crate::{
    errors::AppError,
    process::{LayerMeta, MatrixPosition, Metadata, ResolvedChord},
    syntax::{File, Key, PlainKey, ModTapType},
};

#[derive(Clone, Debug)]
struct MatrixKey(String);

struct Emit<'a> {
    named_keys: HashMap<String, MatrixKey>,
    extra_allocated_rows: u8,
    extra_allocated_cols: u8,
    chord_table: HashMap<(MatrixPosition, MatrixPosition), MatrixPosition>,

    metadata: &'a Metadata<'a>,
}

impl<'a> Emit<'a> {
    fn option(&self, key: &str) -> Option<&'a str> {
        self.metadata
            .get_option(crate::process::OptionKey::RustyDilemma, key)
    }

    fn option_d<'d: 'a>(&self, key: &str, default: &'d str) -> &'a str {
        self.option(key).unwrap_or(default)
    }

    fn allocate_extra_key(
        &mut self,
        left: MatrixPosition,
        right: MatrixPosition,
    ) -> MatrixPosition {
        if self.extra_allocated_rows == 0 {
            self.extra_allocated_rows = 1;
        }

        let pos = MatrixPosition(
            self.extra_allocated_cols,
            self.extra_allocated_rows + self.metadata.layout.height - 1,
        );

        self.extra_allocated_cols += 1;

        if self.extra_allocated_cols >= self.metadata.layout.width {
            self.extra_allocated_cols = 0;
            self.extra_allocated_rows += 1;
        }

        self.chord_table.insert((left, right), pos);

        pos
    }

    fn process_chord(&mut self, chord: &ResolvedChord<'a>) -> MatrixPosition {
        self.chord_table
            .get(&(chord.left, chord.right))
            .copied()
            .unwrap_or_else(|| self.allocate_extra_key(chord.left, chord.right))
    }

    fn process_layer(&mut self, layer: &'a LayerMeta<'a>) -> HashMap<MatrixPosition, &'a Key<'a>> {
        let _layer_idx = *self.metadata.layers.layer_map.get(layer.name).unwrap() as u8;

        let mut matrix = HashMap::new();
        for chord in &layer.chords {
            let pos = self.process_chord(chord);
            matrix.insert(pos, &chord.chord.key);
        }

        for key in &layer.keys {
            matrix.insert(key.matrix_pos, &key.key);
        }

        matrix
    }

    fn map_key(&mut self, key: &'a Key<'a>) -> miette::Result<MatrixKey> {
        match key {
            Key::Plain(p) => self.map_plain_key(p),
            Key::ModTap {
                tap,
                at,
                hold,
                span: _,
            } => {
                let tap = self.map_plain_key(tap)?.0;
                let hold = self.map_plain_key(hold)?.0;

                let config = match at {
                    ModTapType::Permissive(_) => "PermissiveHold",
                    ModTapType::OnOtherKey(_) => "HoldOnOtherKeyPress",
                };

                let a = format!(
                    r#"::keyberon::action::Action::HoldTap(
    &::keyberon::action::HoldTapAction {{
        timeout: {},
        hold: {hold},
        tap: {tap},
        config: ::keyberon::action::HoldTapConfig::{},
        tap_hold_interval: {},
    }})"#,
                    self.option_d("hold_tap_timeout", "400"),
                    config,
                    self.option_d("hold_tap_interval", "200")
                );

                Ok(MatrixKey(a))
            }
        }
    }

    fn map_plain_key(&mut self, p: &PlainKey<'_>) -> miette::Result<MatrixKey> {
        match p {
            PlainKey::Named(name) => {
                if let Some(k) = self.named_keys.get(name.s) {
                    return Ok(k.clone());
                }

                let mut possible_names = CorpusBuilder::new().case_insensitive().finish();

                for name in self.named_keys.keys() {
                    possible_names.add_text(name);
                }

                let similar = possible_names
                    .search(name.s, 0.40)
                    .into_iter()
                    .map(|s| s.text)
                    .join(", ");

                return Err(AppError::UnknownNamedKey {
                    span: name.span,
                    key: name.s.to_string(),
                    similar,
                }
                .into());
            }
            PlainKey::Layer {
                left_square: _,
                layer,
                right_square: _,
                span: _,
            } => {
                if let Some(idx) = self.metadata.layers.layer_map.get(layer.s) {
                    let k = MatrixKey(format!("::keyberon::action::Action::Layer({idx})"));
                    return Ok(k);
                }

                let mut possible_names = CorpusBuilder::new().case_insensitive().finish();

                for name in self.metadata.layers.layer_map.keys() {
                    possible_names.add_text(name);
                }

                let similar = possible_names
                    .search(layer.s, 0.40)
                    .into_iter()
                    .map(|s| s.text)
                    .join(", ");

                return Err(AppError::UnknownNamedLayer {
                    span: layer.span,
                    layer: layer.s.to_string(),
                    similar,
                }
                .into());
            }
            PlainKey::Char {
                left_quote: _,
                c,
                right_quote: _,
                span,
            } => {
                if let Some(a) = CHAR_KEYS.get(c) {
                    return Ok(a.clone());
                }

                return Err(AppError::UnknownKey {
                    span: *span,
                    key: *c,
                }
                .into());
            }
        }
    }

    fn map_keys(
        &mut self,
        matrix: HashMap<MatrixPosition, &'a Key<'a>>,
    ) -> miette::Result<HashMap<MatrixPosition, MatrixKey>> {
        matrix
            .into_iter()
            .map(|(k, v)| Ok((k, self.map_key(v)?)))
            .collect()
    }

    fn render_chords(&self, out: &mut impl Write) {
        writeln!(out, "pub fn chorder() -> super::chord::Chorder {{").unwrap();
        writeln!(out, "    dilemma_macros::chords!(").unwrap();

        for (pos, map) in &self.chord_table {
            writeln!(
                out,
                "        [({}, {}), ({}, {})] => [({}, {})],",
                pos.0 .1, pos.0 .0, pos.1 .1, pos.1 .0, map.1, map.0
            )
            .unwrap();
        }

        writeln!(out, "    )").unwrap();
        writeln!(out, "}}").unwrap();
    }

    fn render_matrix(&self, matrix: HashMap<MatrixPosition, MatrixKey>, out: &mut impl Write) {
        writeln!(out, "  [").unwrap();
        for y in 0..(self.metadata.layout.height + self.extra_allocated_rows) {
            write!(out, "    [").unwrap();
            for x in 0..self.metadata.layout.width {
                if let Some(k) = matrix.get(&MatrixPosition(x, y)) {
                    write!(out, "{}, ", k.0).unwrap();
                } else {
                    write!(out, "::keyberon::action::Action::NoOp, ").unwrap();
                }
            }
            writeln!(out, "],").unwrap();
        }
        writeln!(out, "  ],").unwrap();
    }

    fn process(&mut self, out: &mut impl Write) -> miette::Result<()> {
        let mut layer_matrices = Vec::new();

        for layer in &self.metadata.layers.layers {
            let matrix = self.process_layer(layer);

            layer_matrices.push(matrix);
        }

        self.render_chords(out);

        let cols = self.metadata.layout.width;
        let rows = self.metadata.layout.height + self.extra_allocated_rows;
        let num_layers = layer_matrices.len();
        writeln!(
            out,
            "pub static LAYERS: ::keyberon::layout::Layers<{cols}, {rows}, {num_layers}, {}> = [",
            self.option_d("custom_event", "()")
        )
        .unwrap();

        for matrix in layer_matrices {
            let mapped_matrix = self.map_keys(matrix)?;
            self.render_matrix(mapped_matrix, out);
        }

        writeln!(out, "];").unwrap();

        Ok(())
    }
}

pub fn emit<'a>(
    file: &'a File<'a>,
    metadata: &'a Metadata<'a>,
    out: &mut impl Write,
) -> miette::Result<()> {
    let mut named_keys = file
        .custom_keys
        .iter()
        .filter_map(|k| {
            k.outputs
                .iter()
                .filter(|d| d.name.s == "keyberon")
                .next()
                .map(|d| (k.name.s.to_string(), MatrixKey(d.output.text.to_string())))
        })
        .collect::<HashMap<_, _>>();

    named_keys.extend(predefined_named_keys());

    let mut e = Emit {
        metadata,

        named_keys,
        extra_allocated_rows: 0,
        extra_allocated_cols: 0,
        chord_table: HashMap::new(),
    };

    e.process(out)?;

    Ok(())
}

fn kc(name: &str) -> String {
    format!("::keyberon::key_code::KeyCode::{name}")
}

fn pl(key: String) -> MatrixKey {
    MatrixKey(format!("::keyberon::action::Action::KeyCode({})", key))
}

fn plkc(name: &str) -> MatrixKey {
    pl(kc(name))
}

fn sh(code: &str) -> MatrixKey {
    MatrixKey(format!(
        "::keyberon::action::Action::MultipleKeyCodes(&[{}, {}].as_slice())",
        kc("LShift"),
        kc(code),
    ))
}

fn predefined_named_keys() -> HashMap<String, MatrixKey> {
    let mut keys: HashMap<_, _> = [
        ("esc", plkc("Escape")),
        ("space", plkc("Space")),
        ("bspace", plkc("BSpace")),
        ("del", plkc("Delete")),
        ("lshift", plkc("LShift")),
        ("rshift", plkc("RShift")),
        ("lctrl", plkc("LCtrl")),
        ("rctrl", plkc("RCtrl")),
        ("lalt", plkc("LAlt")),
        ("ralt", plkc("RAlt")),
        ("lgui", plkc("LGui")),
        ("rgui", plkc("RGui")),
        ("enter", plkc("Enter")),
        ("tab", plkc("Tab")),
        (
            "n",
            MatrixKey("::keyberon::action::Action::NoOp".to_owned()),
        ),
        ("pgup", plkc("PgUp")),
        ("pgdown", plkc("PgDown")),
        ("volup", plkc("VolUp")),
        ("voldown", plkc("VolDown")),
        ("left", plkc("Left")),
        ("up", plkc("Up")),
        ("right", plkc("Right")),
        ("down", plkc("Down")),
        ("end", plkc("End")),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();

    keys.extend((1..=10).map(|n| (format!("f{n}"), plkc(&format!("F{n}")))));

    keys
}

static CHAR_KEYS: Lazy<HashMap<char, MatrixKey>> = Lazy::new(char_keys);

fn char_keys() -> HashMap<char, MatrixKey> {
    let mut keys = HashMap::new();

    for k in 'a'..='z' {
        keys.insert(k, pl(kc(&k.to_ascii_uppercase().to_string())));
    }

    for k in '0'..='9' {
        keys.insert(k, pl(kc(&format!("Kb{k}"))));
    }

    keys.extend([
        ('!', sh("Kb1")),
        ('@', sh("Kb2")),
        ('#', sh("Kb3")),
        ('$', sh("Kb4")),
        ('%', sh("Kb5")),
        ('^', sh("Kb6")),
        ('&', sh("Kb7")),
        ('*', sh("Kb8")),
        ('(', sh("Kb9")),
        (')', sh("Kb0")),
        ('-', plkc("Minus")),
        ('_', sh("Minus")),
        ('=', plkc("Equal")),
        ('+', sh("Equal")),
        ('[', plkc("LBracket")),
        ('{', sh("LBracket")),
        (']', plkc("RBracket")),
        ('}', sh("RBracket")),
        ('\\', plkc("Bslash")),
        ('|', sh("Bslash")),
        (';', plkc("SColon")),
        (':', sh("SColon")),
        ('\'', plkc("Quote")),
        ('"', sh("Quote")),
        ('`', plkc("Grave")),
        ('~', sh("Grave")),
        (',', plkc("Comma")),
        ('<', sh("Comma")),
        ('.', plkc("Dot")),
        ('>', sh("Dot")),
        ('/', plkc("Slash")),
        ('?', sh("Slash")),
    ]);

    keys
}
