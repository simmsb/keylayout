use std::collections::{BTreeMap, HashMap};

use locspan::Spanned;

use crate::{
    errors::AppError,
    syntax::{Chord, File, Key, KeyOrChord, Layer, Layout, LayoutDefn, Options, OptionsFor},
};

#[derive(Debug, debug3::Debug, Clone, Copy)]
pub enum KeyAt {
    Space,

    Located(MatrixPosition),
}

#[derive(Debug, debug3::Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct MatrixPosition(pub u8, pub u8);

#[derive(Debug, debug3::Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum OptionKey {
    RustyDilemma,
    KeymapDrawer,
    Formatter,
}

#[derive(Debug, debug3::Debug)]
pub struct Metadata<'a> {
    pub options: OptionsMeta<'a>,
    pub layout: LayoutMeta,
    pub layers: LayersMeta<'a>,
}

impl<'a> Metadata<'a> {
    pub fn process(file: &'a File<'a>) -> miette::Result<Self> {
        let options = OptionsMeta::process(&file.options);
        let layout = LayoutMeta::process(&file.layout)?;
        let layers = LayersMeta::process(&layout, &file.layers)?;

        Ok(Self {
            options,
            layout,
            layers,
        })
    }

    pub fn get_option(&self, emitter: OptionKey, key: &str) -> Option<&'a str> {
        self.options.options.get(&(emitter, key)).map(|x| *x)
    }
}

#[derive(Debug, debug3::Debug)]
pub struct OptionsMeta<'a> {
    pub options: HashMap<(OptionKey, &'a str), &'a str>,
}

impl<'a> OptionsMeta<'a> {
    pub fn process(options: &'a [Options<'a>]) -> Self {
        let mut resolved_options = HashMap::new();

        for option in options {
            let for_ = match option.for_ {
                OptionsFor::RustyDilemma(_) => OptionKey::RustyDilemma,
                OptionsFor::KeymapDrawer(_) => OptionKey::KeymapDrawer,
                OptionsFor::Formatter(_) => OptionKey::Formatter,
            };

            for item in &option.items {
                resolved_options.insert(
                    (for_, item.name.s),
                    item.value.text.as_ref(),
                );
            }
        }

        Self {
            options: resolved_options,
        }
    }
}

#[derive(Debug, debug3::Debug)]
pub struct LayoutMeta {
    pub phys_to_matrix: BTreeMap<(u8, u8), KeyAt>,
    pub layout_to_matrix: BTreeMap<(u8, u8), KeyAt>,
    pub layout_to_phys: BTreeMap<(u8, u8), (u8, u8)>,
    pub width: u8,
    pub height: u8,
}

impl LayoutMeta {
    pub fn process(layout: &Layout) -> miette::Result<Self> {
        let mut phys_to_matrix = BTreeMap::new();
        let mut layout_to_matrix = BTreeMap::new();
        let mut layout_to_phys = BTreeMap::new();
        let mut matrix_to_key: BTreeMap<(u8, u8), &LayoutDefn> = BTreeMap::new();
        let mut width = None;
        let height = layout.rows.len() as u8;

        for (y, row) in layout.rows.iter().enumerate() {
            let mut x = 0;
            let mut x_l = 0;
            for defn in &row.items {
                match defn {
                    crate::syntax::LayoutDefn::Keys { count, k: _, span } => {
                        for n in 0..*count {
                            let pos = (x + n, y as u8);
                            if let Some(k) = matrix_to_key.insert(pos, defn) {
                                return Err(AppError::OverlappingKeys {
                                    span: k.span(),
                                    other_span: *span,
                                }
                                .into());
                            }

                            phys_to_matrix
                                .insert(pos, KeyAt::Located(MatrixPosition(pos.0, pos.1)));
                            let pos_l = (x_l + n, y as u8);
                            layout_to_matrix
                                .insert(pos_l, KeyAt::Located(MatrixPosition(pos.0, pos.1)));
                            layout_to_phys.insert(pos_l, pos);
                        }

                        x += count;
                        x_l += count;
                    }
                    crate::syntax::LayoutDefn::RemappedKey {
                        left_bracket: _,
                        position,
                        right_bracket: _,
                        span,
                    } => {
                        let phys_pos = (x, y as u8);
                        let matr_pos = (*position, y as u8);

                        if let Some(k) = matrix_to_key.insert(matr_pos, defn) {
                            return Err(AppError::OverlappingKeys {
                                span: k.span(),
                                other_span: *span,
                            }
                            .into());
                        }

                        phys_to_matrix.insert(
                            phys_pos,
                            KeyAt::Located(MatrixPosition(matr_pos.0, matr_pos.1)),
                        );
                        let pos_l = (x_l, y as u8);
                        layout_to_matrix.insert(
                            pos_l,
                            KeyAt::Located(MatrixPosition(matr_pos.0, matr_pos.1)),
                        );
                        layout_to_phys.insert(pos_l, phys_pos);

                        x += 1;
                        x_l += 1;
                    }
                    crate::syntax::LayoutDefn::Spaces {
                        count,
                        s: _,
                        span: _,
                    } => {
                        for n in 0..*count {
                            let pos = (x + n, y as u8);
                            phys_to_matrix.insert(pos, KeyAt::Space);
                        }

                        x += count;
                    }
                }
            }

            if let Some(expected_width) = width {
                if expected_width != x {
                    return Err(AppError::InconsistentMatrixWidth {
                        bad_row: row.span(),
                        got: x,
                        expected: expected_width,
                    }
                    .into());
                }
            } else {
                width = Some(x);
            }
        }

        Ok(LayoutMeta {
            phys_to_matrix,
            layout_to_matrix,
            layout_to_phys,
            width: width.unwrap(),
            height,
        })
    }
}

#[derive(Debug, debug3::Debug)]
pub struct LayersMeta<'a> {
    pub layer_map: BTreeMap<String, usize>,
    pub layers: Vec<LayerMeta<'a>>,
}

impl<'a> LayersMeta<'a> {
    pub fn process(layout_meta: &LayoutMeta, layers: &[Layer<'a>]) -> miette::Result<Self> {
        let mut layer_map = BTreeMap::new();
        let mut processed_layers = Vec::new();

        for layer in layers {
            layer_map.insert(layer.name.s.to_string(), layer_map.len());
        }

        for layer in layers {
            processed_layers.push(LayerMeta::process(&layout_meta, &layer_map, layer)?);
        }

        Ok(LayersMeta {
            layer_map,
            layers: processed_layers,
        })
    }
}

#[derive(Debug, debug3::Debug)]
pub struct ResolvedChord<'a> {
    pub chord: Chord<'a>,
    pub left_layout: (u8, u8),
    pub left: MatrixPosition,
    pub right: MatrixPosition,
}

#[derive(Debug, debug3::Debug)]
pub struct ResolvedKey<'a> {
    pub key: Key<'a>,
    pub layout_pos: (u8, u8),
    pub physical_pos: (u8, u8),
    pub matrix_pos: MatrixPosition,
}

#[derive(Debug, debug3::Debug)]
pub struct LayerMeta<'a> {
    pub name: &'a str,
    pub chords: Vec<ResolvedChord<'a>>,
    pub keys: Vec<ResolvedKey<'a>>,
}

impl<'a> LayerMeta<'a> {
    pub fn process(
        layout_meta: &LayoutMeta,
        _layer_map: &BTreeMap<String, usize>,
        layer: &Layer<'a>,
    ) -> miette::Result<Self> {
        let mut keys = Vec::new();
        let mut chords = Vec::new();

        for (y, row) in layer.rows.iter().enumerate() {
            let y = y as u8;
            let mut x = 0;
            let mut last_item = None;
            let mut item_iter = row.items.iter().peekable();

            while let Some(item) = item_iter.next() {
                match item {
                    crate::syntax::KeyOrChord::Key(key) => {
                        let Some(&physical_pos) = layout_meta.layout_to_phys.get(&(x, y)) else {
                            return Err(AppError::ImpossibleKeyLocation { key: item.span() }.into());
                        };
                        let KeyAt::Located(matrix_pos) =
                            *layout_meta.layout_to_matrix.get(&(x, y)).unwrap()
                        else {
                            panic!("Huh");
                        };
                        let resolved_key = ResolvedKey {
                            key: key.clone(),
                            layout_pos: (x, y),
                            physical_pos,
                            matrix_pos,
                        };
                        keys.push(resolved_key);
                        x += 1;
                    }
                    crate::syntax::KeyOrChord::Chord(chord) => {
                        if matches!(last_item, Some(&KeyOrChord::Key(_)))
                            && matches!(item_iter.peek(), Some(KeyOrChord::Key(_)))
                        {
                            let Some(KeyAt::Located(left)) =
                                layout_meta.layout_to_matrix.get(&(x - 1, y)).copied()
                            else {
                                return Err(AppError::ImpossibleKeyLocation { key: item.span() }.into());
                            };
                            let Some(KeyAt::Located(right)) =
                                layout_meta.layout_to_matrix.get(&(x, y)).copied()
                            else {
                                return Err(AppError::ImpossibleKeyLocation { key: item.span() }.into());
                            };

                            let left_layout = (x - 1, y);

                            chords.push(ResolvedChord {
                                chord: chord.clone(),
                                left_layout,
                                left,
                                right,
                            });
                        } else {
                            let prev_item =
                                last_item.map_or(row.span.start_singleton(), |c| c.span());
                            let next_item = item_iter
                                .peek()
                                .map_or(row.semi.span().start_singleton(), |c| c.span());

                            return Err(AppError::BadChordPositions {
                                bad_chord: chord.span(),
                                prev_item,
                                next_item,
                            }
                            .into());
                        }
                    }
                }

                last_item = Some(item);
            }
        }

        let name = layer.name.s;
        Ok(Self { name, keys, chords })
    }
}
