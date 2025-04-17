use inkwell::values::FunctionValue;
use new_parser::nodes::{ASTNodes, Block};

use crate::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn codegen_block(
        &self,
        block: &Block,
        built_func: FunctionValue<'ctx>,
    ) -> Result<(), ()> {
        let basic_block = self.context.append_basic_block(built_func, "entry");
        self.builder.position_at_end(basic_block);

        for node in &block.body {
            match node {
                ASTNodes::LetStmt(let_stmt) => self.impl_let_stmt(let_stmt)?,
                ASTNodes::Return(ret) => self.impl_function_return(built_func, ret)?,
                _ => todo!(),
            };
        }

        // TODO: check if the last instruction is a terminator; if it is not, then we
        // need to add a return instruction
        Ok(())
    }

    pub(crate) fn codegen_function_block(
        &self,
        block: &Block,
        built_func: FunctionValue<'ctx>,
    ) -> Result<(), ()> {
        self.codegen_block(block, built_func)?;
        self.var_ptrs.clear();
        Ok(())
    }
}
