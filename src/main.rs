mod compiler;

use std::{ffi::OsStr, fs::File};

use clap::Parser;

#[derive(Clone, clap::ValueEnum)]
enum Mode {
    Compile,
    Run,
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
    let compiled_data: compiler::CompiledData = compiler::compile(&src)?;

    let output_path = path.with_extension("tb");
    let file: File = File::create(&output_path)?;
    let writer = std::io::BufWriter::new(file);

    bincode::serialize_into(writer, &compiled_data)?;

    println!("Compiled {} successfully", output_path.display());

    Ok(())
}
fn main() {
}