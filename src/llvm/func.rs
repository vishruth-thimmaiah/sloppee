use inkwell::{
    types::{BasicMetadataTypeEnum, BasicType},
    values::{BasicValueEnum, FunctionValue},
};

use crate::{
    errors,
    lexer::types::DATATYPE,
    parser::nodes::{ExpressionParserNode, FunctionCallParserNode, FunctionParserNode, ReturnNode},
};

use super::{
    codegen::{CodeGen, FunctionStore},
    stdlib_defs::STDLIB_MODULES,
};

impl<'ctx> CodeGen<'ctx> {
    pub fn add_function(&self, node: &FunctionParserNode) {
        self.variables
            .borrow_mut()
            .push(FunctionStore::new(node.func_name.clone()));
        let args = self.def_func_args(&node.args);

        let ret_type = &node.return_type;

        let fn_type = if let Some(expr) = self.def_expr(&ret_type) {
            expr.fn_type(&args, false)
        } else {
            self.context.void_type().fn_type(&args, false)
        };

        let function = self.module.add_function(&node.func_name, fn_type, None);

        for (index, arg) in function.get_param_iter().enumerate() {
            arg.set_name(&node.args[index].0);
        }

        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        self.nested_codegen(&node.body, &node.func_name, &node.return_type);
    }

    pub fn add_return(&self, node: &ReturnNode, func_name: &str, ret_type: &DATATYPE) {
        let ret_expr = node
            .return_value
            .any()
            .downcast_ref::<ExpressionParserNode>()
            .unwrap();
        if ret_type == &DATATYPE::NONE {
            self.builder.build_return(None).unwrap();
        } else {
            let ret_val = self.add_expression(ret_expr, func_name, ret_type);
            self.builder.build_return(Some(&ret_val)).unwrap();
        }
    }

    pub fn def_extern(&self, func_name: &str, imported: &Vec<String>) -> FunctionValue<'ctx> {
        if let Some(e) = self.module.get_function(func_name) {
            if e.get_linkage() == inkwell::module::Linkage::External {
                return e;
            }
        }
        let import_path = self.check_if_imported(imported);
        if import_path.is_none() {
            errors::compiler_error(&format!("Module '{}' not imported", imported.join(":")));
        }

        let import_path = import_path.unwrap();

        // TODO: Add support for nested modules
        let func = if import_path.first().unwrap() == "std" {
            let req_module = STDLIB_MODULES
                .iter()
                .find(|x| x.name == import_path.last().unwrap())
                .expect("Module not found");

            let req_func = req_module
                .funcs
                .iter()
                .find(|x| x.name == func_name)
                .expect("Function not found");
            req_func
        } else {
            todo!("user defined functions are not supported yet");
        };
        let params = self.def_func_args(
            &func
                .args
                .to_vec()
                .iter()
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect::<Vec<_>>(),
        );

        let fn_type = if let Some(expr) = self.def_expr(&func.return_type) {
            expr.fn_type(&params, false)
        } else {
            self.context.void_type().fn_type(&params, false)
        };

        let func =
            self.module
                .add_function(func_name, fn_type, Some(inkwell::module::Linkage::External));

        self.add_stdlib_import(func_name, func);
        func
    }

    pub fn add_func_call(
        &self,
        func_node: &FunctionCallParserNode,
        func_name: &str,
    ) -> BasicValueEnum<'ctx> {
        let function = if let Some(imported) = &func_node.imported {
            self.def_extern(&func_node.func_name, imported)
        } else {
            self.module.get_function(&func_node.func_name).unwrap()
        };
        let mut args = Vec::new();
        let params = function.get_params();
        for (index, arg) in func_node.args.iter().enumerate() {
            let req_type = &self.get_datatype(params[index].get_type());
            let arg_val = self.add_expression(arg, func_name, req_type).into();
            args.push(arg_val);
        }

        self.builder
            .build_call(function, &args, "")
            .unwrap()
            .try_as_basic_value()
            .left()
            // FIXME: Temporary fix for functions that return nothing
            .unwrap_or(self.context.i32_type().const_zero().into())
    }

    pub fn def_func_args(
        &self,
        args: &Vec<(String, DATATYPE)>,
    ) -> Vec<BasicMetadataTypeEnum<'ctx>> {
        let mut result_arr: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();

        for arg in args {
            result_arr.push(self.def_expr(&arg.1).unwrap().into());
        }

        return result_arr;
    }

    fn check_if_imported(&self, import_path: &Vec<String>) -> Option<Vec<String>> {
        self.imports
            .borrow()
            .iter()
            .find(|x| x.last() == import_path.last())
            .map(|x| x.clone())
    }
}
