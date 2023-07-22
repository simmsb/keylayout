#![feature(adt_const_params)]

mod parse;
mod syntax;

use std::path::PathBuf;

use chumsky::Parser as _;
use clap::Parser;
use miette::IntoDiagnostic;

#[derive(Parser, Debug)]
struct Args {
    file: PathBuf,
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    let file = std::fs::read_to_string(args.file).into_diagnostic()?;

    let r = match parse::layer().parse(&file).into_result() {
        Ok(r) => r,
        Err(e) => {
            for m in e {
                let e =
                    miette::Error::new(parse::convert_error(m)).with_source_code(file.to_owned());

                println!("{:?}", e);
            }
            return Ok(());
        }
    };

    println!("{:#?}", r);

    Ok(())
}
