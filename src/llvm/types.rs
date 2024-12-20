use inkwell::{
    types::{BasicType, BasicTypeEnum},
    values::BasicValueEnum,
    AddressSpace,
};

use crate::lexer::types::{ArrayDetails, Types, DATATYPE};

use super::codegen::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    pub fn string_to_value(
        &self,
        value: &str,
        val_type: &Types,
        req_type: &DATATYPE,
    ) -> BasicValueEnum<'ctx> {
        if val_type == &Types::BOOL {
            self.context
                .bool_type()
                .const_int(value.parse::<u64>().unwrap(), false)
                .into()
        } else if val_type == &Types::NUMBER {
            let dt = self.def_expr(req_type).unwrap();
            if value.contains('.') {
                let f64_value = value.parse::<f64>().unwrap();
                if dt.is_float_type() {
                    dt.into_float_type().const_float(f64_value).into()
                } else {
                    self.context.f64_type().const_float(f64_value).into()
                }
            } else {
                let i64_value = value.parse::<u64>().unwrap();
                if dt.is_int_type() {
                    dt.into_int_type().const_int(i64_value, false).into()
                } else {
                    self.context.i64_type().const_int(i64_value, false).into()
                }
            }
        } else if let Types::DATATYPE(DATATYPE::STRING(_)) = val_type {
            let string = self.context.const_string(value.as_bytes(), false);
            let string_ptr = self.builder.build_alloca(string.get_type(), "").unwrap();
            self.builder.build_store(string_ptr, string).unwrap();

            let struct_ty = self.context.struct_type(
                &[
                    self.context.i64_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            );

            let struct_val = struct_ty.const_named_struct(&[
                self.context
                    .i64_type()
                    .const_int(value.len() as u64, false)
                    .into(),
                self.context
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into(),
            ]);

            let struct_ptr = self.builder.build_alloca(struct_ty, "").unwrap();
            self.builder.build_store(struct_ptr, struct_val).unwrap();

            let gep = unsafe {
                self.builder
                    .build_in_bounds_gep(
                        struct_ty,
                        struct_ptr,
                        &[
                            self.context.i32_type().const_zero().into(),
                            self.context.i32_type().const_int(1, false).into(),
                        ],
                        "",
                    )
                    .unwrap()
            };

            self.builder.build_store(gep, string_ptr).unwrap();

            struct_ptr.into()
        } else {
            unreachable!()
        }
    }

    pub fn get_datatype(&self, bt: BasicTypeEnum<'ctx>) -> DATATYPE {
        match bt {
            BasicTypeEnum::IntType(it) => match it.get_bit_width() {
                1 => DATATYPE::BOOL,
                8 => DATATYPE::U8,
                16 => DATATYPE::U16,

                32 => DATATYPE::U32,
                64 => DATATYPE::U64,
                _ => todo!(),
            },
            BasicTypeEnum::FloatType(_) => DATATYPE::F32,
            BasicTypeEnum::ArrayType(arr) => DATATYPE::ARRAY(Box::new(ArrayDetails {
                datatype: self.get_datatype(arr.get_element_type()).clone(),
                length: arr.len(),
            })),
            _ => todo!(),
        }
    }

    // TODO: Make U.. unsigned
    pub fn def_expr(&self, req_type: &DATATYPE) -> Option<BasicTypeEnum<'ctx>> {
        match req_type {
            DATATYPE::U8 => Some(self.context.i8_type().into()),
            DATATYPE::U16 => Some(self.context.i16_type().into()),
            DATATYPE::U32 => Some(self.context.i32_type().into()),
            DATATYPE::U64 => Some(self.context.i64_type().into()),
            DATATYPE::I8 => Some(self.context.i8_type().into()),
            DATATYPE::I16 => Some(self.context.i16_type().into()),
            DATATYPE::I32 => Some(self.context.i32_type().into()),
            DATATYPE::I64 => Some(self.context.i64_type().into()),
            DATATYPE::BOOL => Some(self.context.bool_type().into()),
            DATATYPE::F32 => Some(self.context.f32_type().into()),
            DATATYPE::F64 => Some(self.context.f64_type().into()),

            DATATYPE::STRING(len) => Some(
                self.context
                    .const_string(&vec![0; *len], true)
                    .get_type()
                    .into(),
            ),
            DATATYPE::ARRAY(inner) => Some(
                self.def_expr(&inner.datatype)
                    .unwrap()
                    .array_type(inner.length)
                    .into(),
            ),

            DATATYPE::NONE => None,

            DATATYPE::CUSTOM(_) => todo!(),
        }
    }
}
