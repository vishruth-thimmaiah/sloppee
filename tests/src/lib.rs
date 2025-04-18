use inkwell::context::Context;

use lexer::lexer::Lexer;
use llvm::codegen::CodeGen;
use parser::Parser;

mod conditionals;
mod general;
mod loops;

#[allow(dead_code)]
pub fn generate_result(contents: &str) -> Option<i32> {
    let lexer = Lexer::new(&contents).tokenize();

    let parser = Parser::new(lexer.clone()).parse();

    let context = Context::create();
    let codegen = CodeGen::new(&context, parser, true);
    codegen.compile(false, false)
}
