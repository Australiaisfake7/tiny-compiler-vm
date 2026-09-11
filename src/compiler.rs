use parser::Statement;
use lexer::Token;
use ast_flattener::CompiledData;

mod lexer;
mod parser;
mod ast_flattener;

pub fn compile(source: &'static str) -> CompiledData {
    let tokens: Vec<Token> = lexer::lex_chars(source.chars()).unwrap();
    let statements: Vec<Statement> = parser::parse_tokens(tokens).unwrap();
    let compiled_data: CompiledData = ast_flattener::flatten_ast(&statements).unwrap();
    compiled_data
}
