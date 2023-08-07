#![feature(adt_const_params)]

mod emit_keyberon;
mod errors;
mod parse;
mod process;
mod syntax;

use std::path::PathBuf;

use chumsky::Parser as _;
use clap::Parser;
use miette::IntoDiagnostic;

use crate::process::{LayerMeta, LayersMeta, LayoutMeta};

#[derive(Parser, Debug)]
struct Args {
    file: PathBuf,
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    let file = std::fs::read_to_string(args.file).into_diagnostic()?;

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .context_lines(3)
                .rgb_colors(miette::RgbColors::Preferred)
                .graphical_theme(miette::GraphicalTheme {
                    characters: miette::ThemeCharacters::emoji(),
                    styles: miette::ThemeStyles::rgb(),
                })
                .build(),
        )
    }))?;

    if let Err(e) = main_inner(&file) {
        return Err(e.with_source_code(file));
    }

    Ok(())
}

fn main_inner(source: &str) -> miette::Result<()> {
    let r = match parse::file().parse(source).into_result() {
        Ok(r) => r,
        Err(e) => {
            for m in e {
                let e = miette::Error::new(parse::convert_error(m));
                return Err(e);
            }
            return Ok(());
        }
    };

    let layout_meta = LayoutMeta::process(&r.layout)?;
    let layers_meta = LayersMeta::process(&layout_meta, &r.layers)?;

    let s = emit_keyberon::emit(r, &layout_meta, &layers_meta)?;
    print!("{}", s);

    Ok(())
}
