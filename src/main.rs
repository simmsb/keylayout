#![feature(adt_const_params)]

mod emit_keyberon;
mod errors;
mod parse;
mod process;
mod syntax;

use std::path::PathBuf;

use chumsky::Parser as _;
use clap::{CommandFactory, Parser};
use miette::{IntoDiagnostic, NamedSource};
use patharg::OutputArg;

use crate::{
    errors::AppError,
    process::{LayersMeta, LayoutMeta},
};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Your layout file
    #[arg(global = true)]
    file: Option<PathBuf>,

    /// Where to place output, can be '-' for stdout
    #[arg(short, long, global = true, default_value = "-")]
    output: Option<OutputArg>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    Emit(Emit),
    Format(Format),

    /// Generate completions for your shell
    GenCompletions,
}

/// Process a keyboard layout and emit for a specified backend
#[derive(clap::Args, Debug)]
struct Emit {
    /// Which generator to use
    #[arg(value_enum)]
    mode: EmitBackend,

    #[arg(from_global)]
    file: PathBuf,

    #[arg(from_global)]
    output: OutputArg,
}

impl Emit {
    fn run(self) -> miette::Result<()> {
        let source = std::fs::read_to_string(&self.file).map_err(AppError::IOError)?;
        let r = match parse::file().parse(&source).into_result() {
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

        let mut output = self.output.create().map_err(AppError::IOError)?;
        match self.mode {
            EmitBackend::RustyDilemma => {
                emit_keyberon::emit(r, &layout_meta, &layers_meta, &mut output)?;
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
enum EmitBackend {
    /// Generate a layout file for the rusty dilemma firmware
    RustyDilemma,
}

/// Format the layout definition
#[derive(clap::Args, Debug)]
struct Format {
    #[arg(from_global)]
    file: PathBuf,

    #[arg(from_global)]
    output: OutputArg,
}

impl Format {
    fn run(&self) -> miette::Result<()> {
        Ok(())
    }
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

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

    let r = match args.command {
        Command::Emit(cmd) => cmd.run(),
        Command::Format(cmd) => cmd.run(),
        Command::GenCompletions => {
            let shell = clap_complete::Shell::from_env().unwrap();
            let bin_name = Args::command().get_name().to_string();

            clap_complete::generate(
                shell,
                &mut Args::command(),
                bin_name,
                &mut std::io::stdout(),
            );

            Ok(())
        }
    };

    if let Err(e) = r {
        if let Some((name, source)) = args
            .file
            .as_ref()
            .and_then(|name| Some((name, std::fs::read_to_string(name).ok()?)))
        {
            return Err(e.with_source_code(NamedSource::new(name.to_string_lossy(), source)));
        }

        return Err(e);
    }

    Ok(())
}
