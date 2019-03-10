#[derive(Parser)]
#[grammar = "dsl/dsl.pest"]
struct DSLParser;

mod tests;
mod ast;