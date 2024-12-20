use inkwell::values::{BasicValueEnum, IntValue};

use crate::lexer::types::{DATATYPE, OPERATOR};

use super::codegen::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    pub fn add_binary_operation(
        &self,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        if left.is_int_value() && right.is_int_value() {
            let left_int = left.into_int_value();
            let right_int = right.into_int_value();
            self.builder
                .build_int_add(left_int, right_int, "")
                .unwrap()
                .into()
        } else {
            let left_float = left.into_float_value();
            let right_float = right.into_float_value();
            self.builder
                .build_float_add(left_float, right_float, "")
                .unwrap()
                .into()
        }
    }

    pub fn sub_binary_operation(
        &self,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        if left.is_int_value() && right.is_int_value() {
            let left_int = left.into_int_value();
            let right_int = right.into_int_value();
            self.builder
                .build_int_sub(left_int, right_int, "")
                .unwrap()
                .into()
        } else {
            let left_float = left.into_float_value();
            let right_float = right.into_float_value();
            self.builder
                .build_float_sub(left_float, right_float, "")
                .unwrap()
                .into()
        }
    }

    pub fn mul_binary_operation(
        &self,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        if left.is_int_value() && right.is_int_value() {
            let left_int = left.into_int_value();
            let right_int = right.into_int_value();
            self.builder
                .build_int_mul(left_int, right_int, "")
                .unwrap()
                .into()
        } else {
            let left_float = left.into_float_value();
            let right_float = right.into_float_value();
            self.builder
                .build_float_mul(left_float, right_float, "")
                .unwrap()
                .into()
        }
    }

    pub fn div_binary_operation(
        &self,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        if left.is_int_value() && right.is_int_value() {
            let left_int = left.into_int_value();
            let right_int = right.into_int_value();
            self.builder
                .build_int_signed_div(left_int, right_int, "")
                .unwrap()
                .into()
        } else {
            let left_float = left.into_float_value();
            let right_float = right.into_float_value();
            self.builder
                .build_float_div(left_float, right_float, "")
                .unwrap()
                .into()
        }
    }

    pub fn comp_binary_operation(
        &self,
        op: OPERATOR,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        let (ip, fp) = self.get_predicate(op);
        if left.is_int_value() && right.is_int_value() {
            let left_int = left.into_int_value();
            let right_int = right.into_int_value();
            self.builder
                .build_int_compare(ip, left_int, right_int, "")
                .unwrap()
                .into()
        } else {
            let left_float = left.into_float_value();
            let right_float = right.into_float_value();
            self.builder
                .build_float_compare(fp, left_float, right_float, "")
                .unwrap()
                .into()
        }
    }

    pub fn to_bool(&self, expr: &BasicValueEnum<'ctx>) -> IntValue<'ctx> {
        let datatype = self.get_datatype(expr.get_type());
        if expr.is_int_value() {
            let val = self
                .def_expr(&datatype)
                .unwrap()
                .const_zero()
                .into_int_value();
            if datatype == DATATYPE::BOOL {
                return expr.into_int_value();
            }
            self.builder
                .build_int_compare(inkwell::IntPredicate::NE, expr.into_int_value(), val, "")
                .unwrap()
                .into()
        } else {
            let val = self
                .def_expr(&datatype)
                .unwrap()
                .const_zero()
                .into_float_value();
            self.builder
                .build_float_compare(
                    inkwell::FloatPredicate::ONE,
                    expr.into_float_value(),
                    val,
                    "",
                )
                .unwrap()
        }
    }

    fn get_predicate(&self, op: OPERATOR) -> (inkwell::IntPredicate, inkwell::FloatPredicate) {
        match op {
            OPERATOR::EQUAL => (inkwell::IntPredicate::EQ, inkwell::FloatPredicate::OEQ),
            OPERATOR::NOT_EQUAL => (inkwell::IntPredicate::NE, inkwell::FloatPredicate::ONE),
            OPERATOR::GREATER => (inkwell::IntPredicate::SGT, inkwell::FloatPredicate::OGT),
            OPERATOR::LESSER => (inkwell::IntPredicate::SLT, inkwell::FloatPredicate::OLT),
            OPERATOR::GREATER_EQUAL => (inkwell::IntPredicate::SGE, inkwell::FloatPredicate::OGE),
            OPERATOR::LESSER_EQUAL => (inkwell::IntPredicate::SLE, inkwell::FloatPredicate::UEQ),
            _ => todo!(),
        }
    }
}
