#[cfg(test)]
mod tests {
    use crate::llvm::tests::general::generate_result;

    #[test]
    fn check_if_else_cond() {
        let contents = r#"
        func main() u32 {
            if 5 > 2 {
                return 1
            } else {
                return 0
            }
        }
        "#;

        assert_eq!(1, generate_result(contents).unwrap());
    }

    #[test]
    fn check_if_else_cond2() {
        let contents = r#"
        func main() u32 {
            let u32 a = 2
            if a == 0 {
                return 1
            } else if a == 1 {
                return 2
            } else if a == 2 {
                return 3
            } else {
                return 0
            }
        }
        "#;

        assert_eq!(3, generate_result(contents).unwrap());
    }
}
