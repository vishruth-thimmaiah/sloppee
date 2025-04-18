use inkwell::IntPredicate;

use lexer::types::Datatype;
use parser::nodes::{ConditionalIfParserNode, ForLoopParserNode, LoopParserNode};

use super::codegen::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    pub fn add_conditional_if(
        &self,
        func_name: &str,
        node: &ConditionalIfParserNode,
        ret_type: &Datatype,
    ) {
        let function = self.module.get_function(func_name).unwrap();
        let if_block = self.context.append_basic_block(function, "if");

        let cont = self.context.append_basic_block(function, "if_cont");

        let mut prev_block = (if_block, &node.condition);
        let mut else_if_blocks = Vec::new();

        for (index, else_if_cond) in node.else_if_body.iter().enumerate() {
            let c_name = &("cond_".to_string() + &index.to_string());
            let b_name = &("else_if_".to_string() + &index.to_string());
            let cond_eval_block = self.context.append_basic_block(function, c_name);

            let expr = self.add_expression(prev_block.1, func_name, &Datatype::U32);

            self.builder
                .build_conditional_branch(self.to_bool(&expr), prev_block.0, cond_eval_block)
                .unwrap();

            let cond_block = self.context.append_basic_block(function, b_name);
            else_if_blocks.push(cond_block);
            self.builder.position_at_end(cond_block);
            self.nested_codegen(&else_if_cond.body, func_name, ret_type);

            self.add_unconditional(cont);

            self.builder.position_at_end(cond_eval_block);

            prev_block = (cond_block, &else_if_cond.condition);
        }

        let cont_eval_block = self.builder.get_insert_block().unwrap();

        let last_block = if let Some(else_body) = &node.else_body {
            let else_block = self.context.append_basic_block(function, "else");
            self.builder.position_at_end(else_block);
            self.nested_codegen(&else_body.body, func_name, ret_type);

            self.add_unconditional(cont);

            else_block
        } else if prev_block.0 == if_block {
            cont
        } else {
            prev_block.0
        };

        self.builder.position_at_end(cont_eval_block);

        let expr = self.add_expression(prev_block.1, func_name, &Datatype::U32);

        self.builder
            .build_conditional_branch(self.to_bool(&expr), prev_block.0, last_block)
            .unwrap();

        self.builder.position_at_end(if_block);
        self.nested_codegen(&node.body, func_name, ret_type);

        self.add_unconditional(cont);

        cont.move_after(last_block).unwrap();
        self.builder.position_at_end(cont);
    }

    pub fn add_unconditional(&self, move_to: inkwell::basic_block::BasicBlock) {
        let last_instruction = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_last_instruction();

        if let Some(last) = last_instruction {
            if last.get_opcode() == inkwell::values::InstructionOpcode::Return {
                return;
            } else if last.get_opcode() == inkwell::values::InstructionOpcode::Br {
                return;
            }
        }
        self.builder.build_unconditional_branch(move_to).unwrap();
    }

    pub fn add_for_loop(&self, func_name: &str, node: &ForLoopParserNode, ret_type: &Datatype) {
        let function = self.module.get_function(func_name).unwrap();

        let index = self.context.i32_type().const_zero();
        let index_ptr = self.store_new_var(func_name, &node.index, &Datatype::U32, true);

        self.builder.build_store(index_ptr, index).unwrap();

        let vars = self.variables.borrow();
        let var = vars.iter().find(|x| x.name == func_name).unwrap();

        let loop_block = self.context.append_basic_block(function, "for_loop");
        let cont = self.context.append_basic_block(function, "loop_cont");

        let expr_ptr = self
            .builder
            .build_alloca(self.context.bool_type(), "")
            .unwrap();

        let expr = self
            .builder
            .build_int_compare(
                IntPredicate::SLT,
                index,
                self.context.i32_type().const_int(5, false),
                "",
            )
            .unwrap();

        self.builder.build_store(expr_ptr, expr).unwrap();

        self.builder
            .build_conditional_branch(expr, loop_block, cont)
            .unwrap();

        self.builder.position_at_end(loop_block);

        self.nested_codegen(&node.body, func_name, ret_type);

        let index = self
            .builder
            .build_load(self.context.i32_type(), index_ptr, "")
            .unwrap();

        let new_index = self
            .builder
            .build_int_add(
                index.into_int_value(),
                self.context.i32_type().const_int(1, false),
                "",
            )
            .unwrap();
        self.builder.build_store(index_ptr, new_index).unwrap();
        let expr = self
            .builder
            .build_int_compare(
                IntPredicate::SLT,
                index.into_int_value(),
                self.context
                    .i32_type()
                    .const_int(var.vars.len() as u64 + 1, false),
                "",
            )
            .unwrap();

        self.builder
            .build_conditional_branch(expr, loop_block, cont)
            .unwrap();
        self.builder.position_at_end(cont);
    }

    pub fn add_loop(&self, func_name: &str, node: &LoopParserNode, ret_type: &Datatype) {
        let function = self.module.get_function(func_name).unwrap();

        let loop_block = self.context.append_basic_block(function, "loop");
        let cont = self.context.append_basic_block(function, "loop_cont");

        let expr = self.add_expression(&node.condition, func_name, &Datatype::U32);

        self.builder
            .build_conditional_branch(self.to_bool(&expr), loop_block, cont)
            .unwrap();

        self.builder.position_at_end(loop_block);
        self.nested_codegen(&node.body, func_name, ret_type);

        let expr = self.add_expression(&node.condition, func_name, &Datatype::U32);
        self.builder
            .build_conditional_branch(self.to_bool(&expr), loop_block, cont)
            .unwrap();
        self.builder.position_at_end(cont);
    }

    pub fn add_break(&self, func_name: &str) {
        let function = self.module.get_function(func_name).unwrap();
        let block = function
            .get_basic_block_iter()
            .find(|x| x.get_name().to_str().unwrap() == "loop_cont")
            .unwrap();

        self.builder.build_unconditional_branch(block).unwrap();
        self.add_unconditional(block);
    }
}
