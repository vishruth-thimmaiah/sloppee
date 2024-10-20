use std::cell::RefCell;
use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::values::BasicValueEnum;
use inkwell::OptimizationLevel;

use crate::lexer::types::{Types, DATATYPE, OPERATOR};
use crate::llvm::builder;
use crate::parser::nodes::{
    AssignmentParserNode, ExpressionParserNode, FunctionCallParserNode, FunctionParserNode,
    ParserType, ReturnNode, ValueParserNode,
};
use crate::parser::types::ParserTypes;

type MainFunc = unsafe extern "C" fn() -> u32;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    module: Module<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    tokens: Vec<Box<dyn ParserType>>,
    variables: RefCell<HashMap<String, BasicValueEnum<'ctx>>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, tokens: Vec<Box<dyn ParserType>>) -> Self {
        let module = context.create_module("main");
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("failed to create execution engine");
        Self {
            context: &context,
            module,
            builder: context.create_builder(),
            execution_engine,
            tokens,
            variables: RefCell::new(HashMap::new()),
        }
    }

    pub fn jit_compile(&self, build: bool) -> Option<u32> {
        for node in &self.tokens {
            // functions should be the only type of node at the top level
            match node.get_type() {
                ParserTypes::FUNCTION => {
                    let downcast_node = node.any().downcast_ref::<FunctionParserNode>().unwrap();

                    self.add_function(downcast_node);
                }
                _ => todo!(),
            }
        }
        if build {
            builder::build_ir(&self.module);
            None
        } else {
            unsafe {
                let exec: JitFunction<MainFunc> =
                    self.execution_engine.get_function("main").unwrap();
                Some(exec.call())
            }
        }
    }

    fn nested_codegen(&self, body: &Vec<Box<dyn ParserType>>, ret_type: &DATATYPE) {
        for node in body {
            match node.get_type() {
                ParserTypes::VARIABLE => {
                    let downcast_node = node.any().downcast_ref::<AssignmentParserNode>().unwrap();
                    self.add_variable(downcast_node);
                }
                ParserTypes::RETURN => {
                    let downcast_node = node.any().downcast_ref::<ReturnNode>().unwrap();
                    self.add_return(downcast_node, ret_type);
                }
                _ => todo!(),
            }
        }
    }

    fn add_variable(&self, node: &AssignmentParserNode) {
        let alloc = self.new_ptr(node);
        let build_alloc = self.builder.build_load(self.def_expr(&node.var_type), alloc, &node.var_name).unwrap();
        self.variables
            .borrow_mut()
            .insert(node.var_name.clone(), build_alloc);

        let value = node
            .value
            .any()
            .downcast_ref::<ExpressionParserNode>()
            .unwrap();

        self.builder
            .build_store(alloc, self.add_expression(value, &node.var_type))
            .unwrap();
    }

    fn add_function(&self, node: &FunctionParserNode) {
        let args = self.def_func_args(&node.args);

        let ret_type = node.return_type.as_ref().unwrap();
        let fn_type = self.def_expr(ret_type).fn_type(&args, false);
        let function = self.module.add_function(&node.func_name, fn_type, None);

        for (index, arg) in function.get_param_iter().enumerate() {
            arg.set_name(&node.args[index].0);
            self.variables
                .borrow_mut()
                .insert(node.args[index].0.clone(), arg);
        }

        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        self.nested_codegen(&node.body, node.return_type.as_ref().unwrap());
    }

    fn add_return(&self, node: &ReturnNode, ret_type: &DATATYPE) {
        let ret_expr = node
            .return_value
            .any()
            .downcast_ref::<ExpressionParserNode>()
            .unwrap();
        let ret_val = self.add_expression(ret_expr, ret_type);

        self.builder.build_return(Some(&ret_val)).unwrap();
    }

    fn add_expression(
        &self,
        node: &ExpressionParserNode,
        req_type: &DATATYPE,
    ) -> BasicValueEnum<'ctx> {
        let left_val = match node.left.get_type() {
            ParserTypes::VALUE => {
                let value_parser_node = node.left.any().downcast_ref::<ValueParserNode>().unwrap();
                match value_parser_node.r#type {
                    Types::NUMBER => self.string_to_value(&value_parser_node.value, req_type),

                    Types::IDENTIFIER => *self
                        .variables
                        .borrow()
                        .get(value_parser_node.value.as_str())
                        .expect("unknown variable."),
                    _ => panic!("Invalid type"),
                }
            }
            ParserTypes::FUNCTION_CALL => {
                let downcast_node = node
                    .left
                    .any()
                    .downcast_ref::<FunctionCallParserNode>()
                    .unwrap();

                let function = self.module.get_function(&downcast_node.func_name).unwrap();
                let mut args = Vec::new();
                let params = function.get_params();
                for (index, arg) in downcast_node.args.iter().enumerate() {
                    args.push(
                        self.add_expression(arg, self.get_datatype(params[index]))
                            .into(),
                    );
                }

                self.builder
                    .build_call(function, &args, &downcast_node.func_name)
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            _ => panic!("Invalid type"),
        };

        let right_val = {
            if let Some(right) = &node.right {
                let right_expr = right.any().downcast_ref::<ExpressionParserNode>().unwrap();
                self.add_expression(right_expr, req_type)
            } else {
                return left_val;
            }
        };

        match node.operator.as_ref().unwrap() {
            OPERATOR::PLUS => self.add_binary_operation(&left_val, &right_val),
            OPERATOR::MINUS => self.sub_binary_operation(&left_val, &right_val),
            OPERATOR::MULTIPLY => self.mul_binary_operation(&left_val, &right_val),
            OPERATOR::DIVIDE => self.div_binary_operation(&left_val, &right_val),
            _ => unreachable!(),
        }
    }
}
