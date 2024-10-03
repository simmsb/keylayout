use std::{collections::HashMap, io::Write};

use indexmap::IndexMap;
use itertools::Itertools;
use ngrammatic::CorpusBuilder;

use crate::{
    errors::AppError,
    process::Metadata,
    syntax::{File, Key, KeyOrChord, PlainKey},
};

#[derive(Debug, serde::Serialize)]
struct Spec {
    layout: LayoutSpec,
    layers: LayersSpec,
    combos: CombosSpec,
}

#[derive(Debug, serde::Serialize)]
struct LayoutSpec {
    qmk_keyboard: String,
    qmk_layout: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct LayersSpec(IndexMap<String, LayerSpec>);

#[derive(Debug, serde::Serialize)]
struct LayerSpec(Vec<Vec<KeySpec>>);

#[derive(Debug, serde::Serialize)]
struct KeySpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    tap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hold: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct CombosSpec(Vec<ComboSpec>);

#[derive(Debug, serde::Serialize)]
struct ComboSpec {
    key_positions: (usize, usize),
    key: KeySpec,
    layers: Vec<String>,
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
                .filter(|d| d.name.s == "keymap_drawer")
                .next()
                .map(|d| (k.name.s.to_string(), Some(d.output.text.to_string())))
        })
        .collect::<HashMap<_, _>>();

    named_keys.extend(predefined_named_keys());

    let convert_plain_key = |k: &PlainKey<'a>| -> miette::Result<Option<String>> {
        match k {
            PlainKey::Named(name) => {
                if let Some(k) = named_keys.get(name.s) {
                    return Ok(k.clone());
                }

                let mut possible_names = CorpusBuilder::new().case_insensitive().finish();

                for name in named_keys.keys() {
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
            } => Ok(Some(layer.s.to_string())),
            PlainKey::Char {
                left_quote: _,
                c,
                right_quote: _,
                span: _,
            } => Ok(Some(format!("{} ", c))),
        }
    };

    let convert_key = |k: &Key<'a>| -> miette::Result<KeySpec> {
        match k {
            Key::Plain(k) => Ok(KeySpec {
                tap: convert_plain_key(k)?,
                hold: None,
            }),
            Key::ModTap {
                tap,
                at: _,
                timeout: _,
                hold,
                span: _,
            } => Ok(KeySpec {
                tap: convert_plain_key(tap)?,
                hold: convert_plain_key(hold)?,
            }),
        }
    };

    let mut combos = Vec::new();
    let mut layers = IndexMap::new();

    for layer in &file.layers {
        let mut layer_r = Vec::new();
        let mut idx = 0;
        for row in &layer.rows {
            let mut row_r = Vec::new();
            for key in &row.items {
                match key {
                    KeyOrChord::Key(k) => {
                        let key_r = convert_key(k)?;
                        row_r.push(key_r);

                        idx += 1;
                    }
                    KeyOrChord::Chord(c) => combos.push(ComboSpec {
                        key_positions: (idx - 1, idx),
                        key: convert_key(&c.key)?,
                        layers: vec![layer.name.s.to_string()],
                    }),
                };
            }
            layer_r.push(row_r);
        }
        layers.insert(layer.name.s.to_string(), LayerSpec(layer_r));
    }

    let get_option = |k| -> miette::Result<&str> {
        if let Some(r) = metadata.get_option(crate::process::OptionKey::KeymapDrawer, k) {
            Ok(r)
        } else {
            Err(AppError::OptionRequired {
                option_name: k.to_string(),
                backend: "keymap_drawer".to_string(),
            }
            .into())
        }
    };

    let layout_spec = LayoutSpec {
        qmk_keyboard: get_option("qmk_keyboard")?.to_string(),
        qmk_layout: get_option("qmk_layout").ok().map(|x| x.to_string()),
    };

    let spec = Spec {
        layout: layout_spec,
        layers: LayersSpec(layers),
        combos: CombosSpec(combos),
    };

    serde_yaml::to_writer(out, &spec).unwrap();

    Ok(())
}

fn predefined_named_keys() -> HashMap<String, Option<String>> {
    let mut keys: HashMap<_, _> = [
        ("esc", Some("Escape".to_string())),
        ("space", Some("Space".to_string())),
        ("bspace", Some("BSpace".to_string())),
        ("del", Some("Delete".to_string())),
        ("lshift", Some("LShift".to_string())),
        ("rshift", Some("RShift".to_string())),
        ("lctrl", Some("LCtrl".to_string())),
        ("rctrl", Some("RCtrl".to_string())),
        ("lalt", Some("LAlt".to_string())),
        ("ralt", Some("RAlt".to_string())),
        ("lgui", Some("LGui".to_string())),
        ("rgui", Some("RGui".to_string())),
        ("enter", Some("Enter".to_string())),
        ("tab", Some("Tab".to_string())),
        ("n", None),
        ("pgup", Some("PgUp".to_string())),
        ("pgdown", Some("PgDown".to_string())),
        ("volup", Some("VolUp".to_string())),
        ("voldown", Some("VolDown".to_string())),
        ("left", Some("Left".to_string())),
        ("up", Some("Up".to_string())),
        ("right", Some("Right".to_string())),
        ("down", Some("Down".to_string())),
        ("end", Some("End".to_string())),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();

    keys.extend((1..=10).map(|n| (format!("f{n}"), Some(format!("F{n}")))));

    keys
}
