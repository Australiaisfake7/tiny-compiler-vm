use parser::Statement;
use lexer::Token;
pub use ast_flattener::CompiledData;

mod lexer;
mod parser;
mod ast_flattener;

pub fn compile(source: &str) -> Result<CompiledData, Box<dyn std::error::Error>> {
    let tokens: Vec<Token> = lexer::lex_chars(source.chars())?;
    let statements: Vec<Statement> = parser::parse_tokens(tokens)?;
    let compiled_data: CompiledData = ast_flattener::flatten_ast(&statements)?;
    Ok(compiled_data)
}
