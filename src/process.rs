use std::collections::BTreeMap;

use locspan::Spanned;

use crate::{
    errors::AppError,
    syntax::{Chord, Key, KeyOrChord, Layer, Layout, LayoutDefn},
};

#[derive(Debug, debug3::Debug, Clone, Copy)]
pub enum KeyAt {
    Space,

    Located(MatrixPosition),
}

#[derive(Debug, debug3::Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct MatrixPosition(pub u8, pub u8);

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
                        let physical_pos = *layout_meta.layout_to_phys.get(&(x, y)).unwrap();
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
                                panic!("Tried to get {:?}", (x, y));
                            };
                            let Some(KeyAt::Located(right)) =
                                layout_meta.layout_to_matrix.get(&(x, y)).copied()
                            else {
                                panic!("Tried to get {:?}", (x, y));
                            };

                            chords.push(ResolvedChord {
                                chord: chord.clone(),
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
