#![feature(adt_const_params)]

mod emit_keymap_drawer;
mod emit_rustydilemma;
mod errors;
mod format;
mod parse;
mod process;
mod syntax;

use std::path::PathBuf;

use chumsky::Parser as _;
use clap::{CommandFactory, Parser};
use miette::NamedSource;
use patharg::OutputArg;
use process::Metadata;

use crate::errors::AppError;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Your layout file
    #[arg(global = true, value_hint = clap::ValueHint::FilePath)]
    file: Option<PathBuf>,

    /// Where to place output, can be '-' for stdout
    #[arg(short, long, global = true, default_value = "-")]
    output: Option<OutputArg>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    Emit(Emit),
    Format(Format),
    GenCompletions(GenCompletions),
}

/// Process a keyboard layout and emit for a specified backend
#[derive(clap::Args, Debug)]
struct Emit {
    /// Which generator to use
    #[arg(short, long, value_enum)]
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

        let metadata = Metadata::process(&r)?;

        let mut output = self.output.create().map_err(AppError::IOError)?;
        match self.mode {
            EmitBackend::RustyDilemma => {
                emit_rustydilemma::emit(&r, &metadata, &mut output)?;
            }
            EmitBackend::KeymapDrawer => {
                emit_keymap_drawer::emit(&r, &metadata, &mut output)?;
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
enum EmitBackend {
    /// Generate a layout file for the rusty dilemma firmware
    RustyDilemma,
    /// Generate a layout file for https://github.com/caksoylar/keymap-drawer
    KeymapDrawer,
}

/// Format the layout definition
#[derive(clap::Args, Debug)]
struct Format {
    #[arg(from_global)]
    file: PathBuf,

    /// Format the file in-place
    #[arg(short, long)]
    inplace: bool,

    #[arg(from_global)]
    output: OutputArg,
}

impl Format {
    fn run(&self) -> miette::Result<()> {
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

        let metadata = Metadata::process(&r)?;

        if self.inplace {
            let mut output = std::fs::File::create(&self.file).map_err(AppError::IOError)?;
            format::format(&r, &metadata, &mut output);
        } else {
            let mut output = self.output.create().map_err(AppError::IOError)?;
            format::format(&r, &metadata, &mut output);
        }

        Ok(())
    }
}

/// Generate completions for your shell
#[derive(clap::Args, Debug)]
struct GenCompletions {
    #[arg(short, long, default_value = "false")]
    nu: bool,
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
        Command::GenCompletions(cmd) => {
            if cmd.nu {
                let shell = clap_complete_nushell::Nushell;
                let bin_name = Args::command().get_name().to_string();

                clap_complete::generate(
                    shell,
                    &mut Args::command(),
                    bin_name,
                    &mut std::io::stdout(),
                );
            } else {
                let shell = clap_complete::Shell::from_env().unwrap();
                let bin_name = Args::command().get_name().to_string();

                clap_complete::generate(
                    shell,
                    &mut Args::command(),
                    bin_name,
                    &mut std::io::stdout(),
                );
            }

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
