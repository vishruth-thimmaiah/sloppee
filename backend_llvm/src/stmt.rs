use std::{cell::RefCell, collections::HashMap};

use inkwell::values::{InstructionValue, PointerValue};
use new_parser::nodes;

use crate::CodeGen;

#[derive(Debug, Default)]
pub struct Variables<'ctx> {
    vars: RefCell<HashMap<String, PointerValue<'ctx>>>,
}

impl<'ctx> Variables<'ctx> {
    pub(crate) fn get(&self, name: &str) -> Option<PointerValue<'ctx>> {
        self.vars.borrow().get(name).cloned()
    }

    pub(crate) fn insert(&self, name: &str, ptr: PointerValue<'ctx>) {
        self.vars.borrow_mut().insert(name.to_string(), ptr);
    }

    pub(crate) fn remove(&self, name: &str) {
        self.vars.borrow_mut().remove(name);
    }

    pub(crate) fn clear(&self) {
        self.vars.borrow_mut().clear();
    }
}

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn impl_let_stmt(
        &self,
        stmt: &nodes::LetStmt,
    ) -> Result<InstructionValue<'ctx>, ()> {
        let dt = self.parser_to_llvm_dt(&stmt.datatype);
        let expr = self.impl_expr(&stmt.value, dt)?;

        let ptr = self.builder.build_alloca(dt, &stmt.name).map_err(|_| ())?;
        self.var_ptrs.insert(&stmt.name, ptr);

        self.builder.build_store(ptr, expr).map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_codegen_let_stmt() {
        let data = "func main() { let u32 a = 5 }";
        let result = crate::get_codegen_for_string(data).unwrap();

        assert_eq!(
            result,
            r#"; ModuleID = 'main'
source_filename = "main"

define void @main() {
entry:
  %a = alloca i32, align 4
  store i32 5, ptr %a, align 4
  ret void
}
"#
        )
    }

    #[test]
    fn test_codegen_let_stmt_array() {
        let data = "func main() { let u32[] a = [1, 2, 3, 4, 5] }";
        let result = crate::get_codegen_for_string(data).unwrap();

        assert_eq!(
            result,
            r#"; ModuleID = 'main'
source_filename = "main"

define void @main() {
entry:
  %a = alloca [5 x i32], align 4
  store [5 x i32] [i32 1, i32 2, i32 3, i32 4, i32 5], ptr %a, align 4
  ret void
}
"#
        )
    }
}
