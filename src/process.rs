use std::collections::{HashMap, HashSet};

use locspan::Spanned;

use crate::{
    errors::AppError,
    syntax::{Layer, Layout, LayoutDefn},
};

#[derive(Debug)]
pub struct LayoutMeta {
    phys_to_matrix: HashMap<(u8, u8), (u8, u8)>,
    spaces: HashSet<(u8, u8)>,
}

impl LayoutMeta {
    pub fn process(layout: &Layout) -> miette::Result<Self> {
        let mut spaces = HashSet::new();
        let mut phys_to_matrix = HashMap::new();
        let mut matrix_to_key: HashMap<(u8, u8), &LayoutDefn> = HashMap::new();

        for (y, row) in layout.rows.iter().enumerate() {
            let mut x = 0;
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

                            phys_to_matrix.insert(pos, pos);
                        }

                        x += count;
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

                        phys_to_matrix.insert(phys_pos, matr_pos);

                        x += 1;
                    }
                    crate::syntax::LayoutDefn::Spaces { count, s: _, span } => {
                        for n in 0..*count {
                            let pos = (x + n, y as u8);
                            phys_to_matrix.insert(pos, pos);
                            spaces.insert(pos);
                        }

                        x += count;
                    }
                }
            }
        }

        Ok(LayoutMeta {
            phys_to_matrix,
            spaces,
        })
    }
}

#[derive(Debug)]
pub struct LayersMeta {
    layer_map: HashMap<String, usize>,
    layers: Vec<LayerMeta>,
}

impl LayersMeta {
    pub fn process<'a>(layers: &[Layer<'a>]) -> miette::Result<Self> {
        let mut layer_map = HashMap::new();
        let mut processed_layers = Vec::new();

        for layer in layers {
            layer_map.insert(layer.name.s.to_string(), layer_map.len());
        }

        for layer in layers {
            processed_layers.push(LayerMeta::process(&layer_map, layer)?);
        }

        Ok(LayersMeta {
            layer_map,
            layers: processed_layers,
        })
    }
}

#[derive(Debug)]
pub struct LayerMeta {}

impl LayerMeta {
    pub fn process<'a>(
        layer_map: &HashMap<String, usize>,
        layer: &Layer<'a>,
    ) -> miette::Result<Self> {
        todo!()
    }
}
