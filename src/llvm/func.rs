use inkwell::{types::BasicType, values::BasicValueEnum};

use crate::{
    lexer::types::DATATYPE,
    parser::nodes::{ExpressionParserNode, FunctionCallParserNode, FunctionParserNode, ReturnNode},
};

use super::codegen::{CodeGen, FunctionStore};

impl<'ctx> CodeGen<'ctx> {
    pub fn add_function(&self, node: &FunctionParserNode) {
        self.variables
            .borrow_mut()
            .push(FunctionStore::new(node.func_name.clone()));
        let args = self.def_func_args(&node.args);

        let ret_type = node.return_type.as_ref().unwrap();
        let fn_type = self.def_expr(ret_type).fn_type(&args, false);
        let function = self.module.add_function(&node.func_name, fn_type, None);

        for (index, arg) in function.get_param_iter().enumerate() {
            arg.set_name(&node.args[index].0);
        }

        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        self.nested_codegen(
            &node.body,
            &node.func_name,
            node.return_type.as_ref().unwrap(),
        );
    }

    pub fn add_return(&self, node: &ReturnNode, func_name: &str, ret_type: &DATATYPE) {
        let ret_expr = node
            .return_value
            .any()
            .downcast_ref::<ExpressionParserNode>()
            .unwrap();
        let ret_val = self.add_expression(ret_expr, func_name, ret_type);

        self.builder.build_return(Some(&ret_val)).unwrap();
    }

    pub fn add_func_call(
        &self,
        func_node: &FunctionCallParserNode,
        func_name: &str,
    ) -> BasicValueEnum<'ctx> {
        let function = self.module.get_function(&func_node.func_name).unwrap();
        let mut args = Vec::new();
        let params = function.get_params();
        for (index, arg) in func_node.args.iter().enumerate() {
            args.push(
                self.add_expression(arg, func_name, self.get_datatype(params[index]))
                    .into(),
            );
        }

        self.builder
            .build_call(function, &args, &func_node.func_name)
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
    }
}
