mod compiler;
mod error;
mod vm;
mod builtin;

use std::fs::File;
use std::path::{Path, PathBuf};
use crate::compiler::CompiledData;
use crate::error::Error;

use clap::{Parser, Args, Subcommand};

const DEFAULT_STACK_SIZE: u64 = 1 << 12;
const MAX_STACK_SIZE: u64 = 1 << 20;

#[derive(Args, Clone)]
struct Input {
    file: PathBuf,
}

#[derive(Clone, Subcommand)]
enum Mode {
    Compile {
        #[command(flatten)]
        input: Input,
    },
    Execute {
        #[command(flatten)]
        input: Input,

        #[arg(short, long, default_value_t = DEFAULT_STACK_SIZE, value_parser = clap::value_parser!(u64).range(2..=MAX_STACK_SIZE))]
        stack_size: u64,
    },
}

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    mode: Mode,
}

fn check_extension(path: &Path, expected: &'static str) -> Result<(), Error> {
    let found: Option<&str> = path.extension().and_then(|ext| ext.to_str());
    if found != Some(expected) {
        return Err(Error::UnexpectedExtension { expected, found: found.map(str::to_owned) });
    }

    Ok(())
}
fn compile(path: &Path) -> Result<(), Error> {
    check_extension(path, "tl")?;

    let src: String = std::fs::read_to_string(path)
        .map_err(|source| Error::Io { action: "read", path: path.to_path_buf(), source })?;
    let compiled_data: CompiledData = compiler::compile(&src)?;

    let output_path = path.with_extension("tb");
    let file: File = File::create(&output_path)
        .map_err(|source| Error::Io { action: "create", path: output_path.clone(), source })?;
    let writer = std::io::BufWriter::new(file);

    bincode::serialize_into(writer, &compiled_data)
        .map_err(|source| Error::Bytecode { action: "write", path: output_path.clone(), source })?;

    eprintln!("Compiled {} successfully", output_path.display());

    Ok(())
}
fn execute(path: &Path, stack_size: usize) -> Result<(), Error> {
    check_extension(path, "tb")?;

    let file: File = File::open(path)
        .map_err(|source| Error::Io { action: "open", path: path.to_path_buf(), source })?;
    let reader: std::io::BufReader<File> = std::io::BufReader::new(file);

    let compiled_data: CompiledData = bincode::deserialize_from(reader)
        .map_err(|source| Error::Bytecode { action: "read", path: path.to_path_buf(), source })?;
    vm::execute(compiled_data, stack_size, stack_size / 2)?;

    eprintln!("Executed {} successfully", path.display());

    Ok(())
}
fn main() {
    let args: CliArgs = CliArgs::parse();

    let result: Result<(), Error> = match args.mode {
        Mode::Compile { input } => compile(&input.file),
        Mode::Execute { input, stack_size } => execute(&input.file, stack_size as usize),
    };

    if let Err(error) = result {
        eprintln!("{}", error.report());
        std::process::exit(1);
    }
}
