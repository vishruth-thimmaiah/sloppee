use inkwell::{basic_block::BasicBlock, values::FunctionValue};
use new_parser::nodes::{ASTNodes, Block};

use crate::{CodeGen, CodeGenError};

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn codegen_block(
        &self,
        block: &Block,
        built_func: FunctionValue<'ctx>,
        block_name: &str,
    ) -> Result<BasicBlock, CodeGenError> {
        let basic_block = self.context.append_basic_block(built_func, block_name);
        self.builder.position_at_end(basic_block);

        for node in &block.body {
            match node {
                ASTNodes::LetStmt(let_stmt) => {
                    self.impl_let_stmt(built_func, let_stmt)?;
                }
                ASTNodes::Conditional(cond) => {
                    self.impl_if_stmt(built_func, cond)?;
                }
                ASTNodes::Return(ret) => {
                    self.impl_function_return(built_func, ret)?;
                }
                ASTNodes::AssignStmt(stmt) => {
                    self.impl_assign_stmt(built_func, stmt)?;
                }
                ASTNodes::FunctionCall(call) => {
                    self.impl_function_call(built_func, call)?;
                }
                _ => todo!(),
            };
        }
        if basic_block.get_terminator().is_none() {
            if built_func.get_type().get_return_type().is_none() {
                self.builder.build_return(None).unwrap();
            } else {
                return Err(CodeGenError::new("Missing return statement"));
            }
        }
        Ok(basic_block)
    }

    pub(crate) fn codegen_function_block(
        &self,
        block: &Block,
        built_func: FunctionValue<'ctx>,
    ) -> Result<(), CodeGenError> {
        self.codegen_block(block, built_func, "entry")?;
        self.var_ptrs.clear();
        Ok(())
    }
}
