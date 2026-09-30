#[deny(unused_must_use)]

mod compiler;
mod vm;

use std::fs::File;
use crate::compiler::CompiledData;

use clap::Parser;

const STACK_SIZE: usize = 1024;

#[derive(Clone, clap::ValueEnum)]
enum Mode {
    Compile,
    Execute,
}
#[derive(Parser)]
struct CliArgs {
    mode: Mode,
    file: std::path::PathBuf,
}
fn compile(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let extension = path.extension().and_then(|ext| ext.to_str());
    if extension != Some("tl") {
        println!("Unexpected file extension {}. Expected .tl.", extension.unwrap_or("none"));
        return Ok(());
    }

    let src: String = std::fs::read_to_string(path)?;
    let compiled_data: CompiledData = compiler::compile(&src)?;

    let output_path = path.with_extension("tb");
    let file: File = File::create(&output_path)?;
    let writer = std::io::BufWriter::new(file);

    bincode::serialize_into(writer, &compiled_data)?;

    println!("Compiled {} successfully", output_path.display());

    Ok(())
}
fn execute(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let extension = path.extension().and_then(|ext| ext.to_str());
    if extension != Some("tb") {
        println!("Unexpected file extension {}. Expected .tb.", extension.unwrap_or("none"));
        return Ok(());
    }

    let read_path = path.with_extension("tb");
    let file: File = File::open(&read_path)?;
    let reader: std::io::BufReader<File> = std::io::BufReader::new(file);

    let compiled_data: CompiledData = bincode::deserialize_from(reader)?;

    vm::execute(compiled_data)?;

    println!("executed {} successfully", read_path.display());

    Ok(())
}
fn main() {
    let args: CliArgs = CliArgs::parse();

    let result: Result<(), Box<dyn std::error::Error>> = match args.mode {
        Mode::Compile => compile(&args.file),
        Mode::Execute => execute(&args.file),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}