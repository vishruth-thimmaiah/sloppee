use std::{env::args, fs};

use inkwell::context::Context;
use lexer::lexer::Lexer;
use llvm::codegen::CodeGen;
use parser::Parser;

mod args;

fn main() {
    let args: Vec<String> = args().collect();
    let parsed_args = args::parse_args(&args);

    if args.len() > 1 {
        let contents = fs::read_to_string(parsed_args.path.unwrap()).unwrap();

        let lexer = Lexer::new(&contents).tokenize();
        if parsed_args.print_lexer_ouput {
            println!("{:#?}", lexer);
        }

        let context = Context::create();

        if parsed_args.new_impl {
            let parser = new_parser::Parser::new(lexer.clone()).parse().unwrap();

            if parsed_args.print_ast_output {
                println!("{:#?}", parser);
            }

            let codegen = backend_llvm::CodeGen::new(&context, parser);
            codegen.codegen().unwrap();
        } else {
            let parser = Parser::new(lexer.clone()).parse();
            if parsed_args.print_ast_output {
                println!("{:#?}", parser);
            }

            if parsed_args.dry_run {
                return;
            }

            let codegen = CodeGen::new(&context, parser, parsed_args.jit);

            if parsed_args.jit {
                let output = codegen.compile(false, false);
                println!("Exit Code: {}", output.unwrap());
            } else {
                codegen.compile(true, parsed_args.run);
            }
        }
    }
}
