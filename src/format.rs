use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use itertools::Itertools;
use locspan::Spanned;

use crate::{process::Metadata, syntax::File};

#[derive(Default, Debug, debug3::Debug, Clone)]
pub struct KeySpacing {
    pub key_width: usize,
    pub chord_width: usize,
}

struct Format<'a> {
    column_widths: Vec<KeySpacing>,
    empties: HashSet<(u8, u8)>,

    file: &'a File<'a>,
}

impl<'a> Format<'a> {
    fn new(file: &'a File<'a>, meta: &'a Metadata<'a>) -> Self {
        let mut column_widths = vec![KeySpacing::default(); meta.layout.width as usize];
        let mut empties = HashSet::new();

        let phys_to_layout = meta
            .layout
            .layout_to_phys
            .iter()
            .map(|(p, l)| (*l, *p))
            .collect::<HashMap<_, _>>();

        for layer in &meta.layers.layers {
            let layout_to_key = layer
                .keys
                .iter()
                .map(|k| (k.layout_pos, k))
                .collect::<HashMap<_, _>>();

            let layout_to_chord = layer
                .chords
                .iter()
                .map(|c| (c.left_layout, c))
                .collect::<HashMap<_, _>>();

            for (x, y) in (0..meta.layout.width).cartesian_product(0..meta.layout.height) {
                if let Some(layout_pos) = phys_to_layout.get(&(x, y)) {
                    let Some(key_node) = layout_to_key.get(layout_pos) else {
                        panic!("Key does not exist on layout: {:?} of layer {}", layout_pos, layer.name);
                    };

                    let spacing = &mut column_widths[x as usize];

                    spacing.key_width = spacing.key_width.max(key_node.key.span().len());

                    if let Some(chord_node) = layout_to_chord.get(&(x, y)) {
                        spacing.chord_width =
                            spacing.chord_width.max(chord_node.chord.span().len());
                    }
                } else {
                    empties.insert((x, y));
                }
            }
        }

        Self {
            column_widths,
            empties,
            file,
        }
    }

    fn format(&self, out: &mut impl Write) {
        self.file
            .to_doc(&self.column_widths, &self.empties)
            .render(usize::MAX, out)
            .unwrap()
    }
}

pub fn format<'a>(file: &'a File<'a>, meta: &'a Metadata<'a>, out: &mut impl Write) {
    Format::new(file, meta).format(out);
}
