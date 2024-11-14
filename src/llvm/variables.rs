use inkwell::{
    types::VectorType,
    values::{ArrayValue, BasicValue, BasicValueEnum, PointerValue},
};

use crate::{
    errors,
    lexer::types::{Types, DATATYPE},
    parser::{
        nodes::{
            AssignmentParserNode, ExpressionParserNode, ValueIterCallParserNode,
            ValueIterParserNode, ValueParserNode, VariableCallParserNode,
        },
        types::ParserTypes,
    },
};

use super::codegen::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    /// used to create a new variable with a name and value. stores a pointer at the corresponding
    /// func at self.variable.
    pub fn add_variable(&self, func_name: &str, node: &AssignmentParserNode) {
        let value = node
            .value
            .any()
            .downcast_ref::<ExpressionParserNode>()
            .unwrap();

        let possible_iter_node = value.left.any().downcast_ref::<ValueIterParserNode>();
        let is_iter = possible_iter_node.is_some();
        let expr = if node.is_mutable && is_iter {
            self.add_vec(possible_iter_node.unwrap(), func_name, &node.var_type)
        } else if is_iter {
            if let DATATYPE::CUSTOM(dt) = &node.var_type {
                self.create_struct(&dt, possible_iter_node.unwrap()).into()
            } else {
                self.add_array(possible_iter_node.unwrap(), func_name, &node.var_type)
            }
        } else {
            self.add_expression(value, func_name, &node.var_type)
        };

        if expr.is_pointer_value() {
            let ptr = expr.into_pointer_value();
            ptr.set_name(&node.var_name);
            self.store_ptr(
                func_name,
                &node.var_name,
                node.is_mutable,
                &node.var_type,
                ptr,
            );
        } else {
            let ptr =
                self.store_new_var(func_name, &node.var_name, &node.var_type, node.is_mutable);
            self.builder.build_store(ptr, expr).unwrap();
        };
    }

    /// used to update a variable.
    pub fn mod_variable(&self, func_name: &str, node: &VariableCallParserNode) {
        let variables = self.variables.borrow();
        let func = variables.iter().find(|x| x.name == func_name).unwrap();

        let var_name = if let Some(name) = node
            .var_name
            .any()
            .downcast_ref::<ValueIterCallParserNode>()
        {
            &name.value
        } else {
            &node
                .var_name
                .any()
                .downcast_ref::<ValueParserNode>()
                .unwrap()
                .value
        };
        let variable = func.vars.get(var_name).expect("Variable not found");

        if !variable.is_mutable {
            errors::compiler_error("Cannot modify immutable variable");
        }

        let (var_ptr, datatype) = if node.var_name.get_type() == ParserTypes::VALUE_ITER_CALL {
            let datatype = if let DATATYPE::ARRAY(array_type) = &variable.datatype {
                &array_type.datatype
            } else {
                unreachable!()
            };
            (
                self.get_array_val(
                    node.var_name
                        .any()
                        .downcast_ref::<ValueIterCallParserNode>()
                        .unwrap(),
                    func_name,
                    datatype,
                ),
                datatype,
            )
        } else {
            (variable.ptr, &variable.datatype)
        };

        let expr = self.add_expression(&node.rhs, func_name, datatype);

        self.builder.build_store(var_ptr, expr).unwrap();
    }

    /// used to create an array. does not assign variable name.
    pub fn add_array(
        &self,
        node: &ValueIterParserNode,
        func_name: &str,
        req_type: &DATATYPE,
    ) -> BasicValueEnum<'ctx> {
        let array_type = if let DATATYPE::ARRAY(array_type) = req_type {
            array_type
        } else {
            errors::compiler_error("Expected array type")
        };

        let mut array_val = vec![];

        for value in &node.value {
            let value = self.add_expression(&value, func_name, &array_type.datatype);
            array_val.push(value);
        }
        // Figure out how to do this without unsafe
        let array = unsafe {
            ArrayValue::new_const_array(&self.def_expr(&array_type.datatype).unwrap(), &array_val)
        };

        array.into()
    }

    /// used to create an vec, used when an array is declared as mut. does not assign variable name.
    pub fn add_vec(
        &self,
        node: &ValueIterParserNode,
        func_name: &str,
        req_type: &DATATYPE,
    ) -> BasicValueEnum<'ctx> {
        let vec_type = if let DATATYPE::ARRAY(array_type) = req_type {
            array_type
        } else {
            errors::compiler_error("Expected vec type")
        };

        let mut vec_val = vec![];
        for value in &node.value {
            let value = self.add_expression(&value, func_name, &vec_type.datatype);
            vec_val.push(value);
        }

        VectorType::const_vector(&vec_val).into()
    }

    /// used to get an array's index value.
    pub fn get_array_val(
        &self,
        node: &ValueIterCallParserNode,
        func_name: &str,
        req_type: &DATATYPE,
    ) -> PointerValue<'ctx> {
        let vars = self.variables.borrow();
        let array = vars
            .iter()
            .find(|x| x.name == func_name)
            .unwrap()
            .vars
            .get(&node.value)
            .unwrap();

        let array_index = self
            .add_expression(&node.index, func_name, req_type)
            .into_int_value();
        let array_type = self.def_expr(req_type);

        let array_size = if let DATATYPE::ARRAY(array_type) = &array.datatype {
            self.context
                .i32_type()
                .const_int(array_type.length.into(), false)
        } else {
            unreachable!()
        };

        let cmp = self
            .builder
            .build_int_compare(inkwell::IntPredicate::ULT, array_index, array_size, "")
            .unwrap();

        let val_at_index = unsafe {
            self.builder
                .build_in_bounds_gep(array_type.unwrap(), array.ptr, &[array_index], "")
                .unwrap()
        };

        // FIXME: Panic instead of returning 0.
        let zero = self
            .builder
            .build_alloca(self.context.i32_type(), "")
            .unwrap();

        self.builder
            .build_store(zero, self.context.i32_type().const_zero())
            .unwrap();
        self.builder
            .build_select(cmp, val_at_index, zero, "")
            .unwrap()
            .into_pointer_value()
    }

    /// Converts a string to a valid datatype. does not store, evaluate values. A raw value can be
    /// passed, or an identifier name.
    pub fn add_value(
        &self,
        node: &ValueParserNode,
        func_name: &str,
        req_type: &DATATYPE,
    ) -> BasicValueEnum<'ctx> {
        match node.r#type {
            Types::NUMBER => self.string_to_value(&node.value, req_type),
            Types::BOOL => self.string_to_value(&node.value, req_type),
            Types::DATATYPE(DATATYPE::STRING(str)) => {
                self.string_to_value(&node.value, &DATATYPE::STRING(str))
            }
            Types::IDENTIFIER => {
                let vars = self.variables.borrow();
                let var = vars.iter().find(|x| x.name == func_name).unwrap();
                let res = {
                    if let Some(var_name) = var.vars.get(node.value.as_str()) {
                        if let DATATYPE::ARRAY(_) = &var_name.datatype {
                            var_name.ptr.as_basic_value_enum()
                        } else if let DATATYPE::STRING(_) = var_name.datatype {
                            var_name.ptr.as_basic_value_enum()
                        } else {
                            self.builder
                                .build_load(
                                    self.def_expr(req_type).unwrap(),
                                    var_name.ptr,
                                    &node.value,
                                )
                                .unwrap()
                        }
                    } else if let Some(func) = self.module.get_function(func_name) {
                        func.get_params()
                            .iter()
                            .find(|x| x.get_name().to_str().unwrap() == node.value)
                            .unwrap()
                            .to_owned()
                    } else {
                        errors::compiler_error("Invalid type");
                    }
                };
                res
            }
            _ => errors::compiler_error("Invalid type"),
        }
    }
}
