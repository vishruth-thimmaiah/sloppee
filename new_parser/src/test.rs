use lexer::{
    lexer::Lexer,
    types::{Datatype, Operator},
};

use crate::{
    Parser,
    nodes::{ASTNodes, Block, Expression, Function, LetStmt, Literal, Return, Variable},
};

#[test]
fn test_parse_full_1() {
    let mut lexer = Lexer::new(
        r#"
    func main() u32 {
        let u32 a = 1
        let u32 b = 4
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let ast = parser.parse().unwrap();
    assert_eq!(
        ast,
        vec![ASTNodes::Function(Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Datatype::U32,
            body: Block {
                body: vec![
                    ASTNodes::LetStmt(LetStmt {
                        name: "a".to_string(),
                        value: Expression::Simple {
                            left: Box::new(ASTNodes::Literal(Literal {
                                value: "1".to_string(),
                                r#type: lexer::types::Types::NUMBER
                            })),
                            right: None,
                            operator: None
                        },
                        datatype: Datatype::U32,
                        mutable: false
                    }),
                    ASTNodes::LetStmt(LetStmt {
                        name: "b".to_string(),
                        value: Expression::Simple {
                            left: Box::new(ASTNodes::Literal(Literal {
                                value: "4".to_string(),
                                r#type: lexer::types::Types::NUMBER
                            })),
                            right: None,
                            operator: None
                        },
                        datatype: Datatype::U32,
                        mutable: false
                    })
                ]
            },
        })]
    );
}

// #[ignore = "impl function calls"]
#[test]
fn test_parse_full_2() {
    let mut lexer = Lexer::new(
        r#"
    func num(e u32, f u32) u32 {
        return e + f
    }

    func main() u32 {
        let u32 a = 6
        let u32 b = num(5, 4)
        return a * b
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let ast = parser.parse().unwrap();
    assert_eq!(
        ast,
        vec![
            ASTNodes::Function(Function {
                name: "num".to_string(),
                args: vec![
                    ("e".to_string(), Datatype::U32),
                    ("f".to_string(), Datatype::U32)
                ],
                return_type: Datatype::U32,
                body: Block {
                    body: vec![ASTNodes::Return(Return {
                        value: Some(Expression::Simple {
                            left: Box::new(ASTNodes::Variable(Variable {
                                name: "e".to_string(),
                            })),
                            right: Some(Box::new(ASTNodes::Variable(Variable {
                                name: "f".to_string(),
                            }))),
                            operator: Some(Operator::PLUS)
                        })
                    })]
                }
            }),
            ASTNodes::Function(Function {
                name: "main".to_string(),
                args: vec![],
                return_type: Datatype::U32,
                body: Block {
                    body: vec![
                        ASTNodes::LetStmt(LetStmt {
                            name: "a".to_string(),
                            value: Expression::Simple {
                                left: Box::new(ASTNodes::Literal(Literal {
                                    value: "6".to_string(),
                                    r#type: lexer::types::Types::NUMBER
                                })),
                                right: None,
                                operator: None
                            },
                            datatype: Datatype::U32,
                            mutable: false
                        }),
                        ASTNodes::LetStmt(LetStmt {
                            name: "b".to_string(),
                            value: Expression::Simple {
                                left: Box::new(ASTNodes::FunctionCall(
                                    crate::nodes::FunctionCall {
                                        name: "num".to_string(),
                                        args: vec![
                                            Expression::Simple {
                                                left: Box::new(ASTNodes::Literal(Literal {
                                                    value: "5".to_string(),
                                                    r#type: lexer::types::Types::NUMBER
                                                })),
                                                right: None,
                                                operator: None
                                            },
                                            Expression::Simple {
                                                left: Box::new(ASTNodes::Literal(Literal {
                                                    value: "4".to_string(),
                                                    r#type: lexer::types::Types::NUMBER
                                                })),
                                                right: None,
                                                operator: None
                                            }
                                        ]
                                    }
                                )),
                                right: None,
                                operator: None,
                            },
                            datatype: Datatype::U32,
                            mutable: false
                        }),
                        ASTNodes::Return(Return {
                            value: Some(Expression::Simple {
                                left: Box::new(ASTNodes::Variable(Variable {
                                    name: "a".to_string(),
                                })),
                                right: Some(Box::new(ASTNodes::Variable(Variable {
                                    name: "b".to_string(),
                                }))),
                                operator: Some(Operator::MULTIPLY)
                            })
                        })
                    ]
                }
            })
        ]
    )
}

#[ignore = "impl conditionals"]
#[test]
fn test_parse_full_3() {
    let mut lexer = Lexer::new(
        r#"
    func main() u32 {
        if false {
            return 1
        }  

        if true {
            return 2
        }  
        return 0
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let _ast = parser.parse().unwrap();
    todo!();
}

#[ignore = "impl loops"]
#[test]
fn test_parse_full_4() {
    let mut lexer = Lexer::new(
        r#"
    func main() u32 {
        let u32! a = 0
        loop {
            a = a + 1
        }
        return a
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let _ast = parser.parse().unwrap();
    todo!();
}

#[ignore = "impl array datatypes, method calls"]
#[test]
fn test_parse_full_5() {
    let mut lexer = Lexer::new(
        r#"
    func main() u32 {
        let u32[] a = [2, 3, 2]
        let u32 b = a.len()
        return b
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let _ast = parser.parse().unwrap();
    todo!();
}

#[ignore = "impl imports, function calls"]
#[test]
fn test_parse_full_6() {
    let mut lexer = Lexer::new(
        r#"
    import std:io

    func main() i32 {
        io:println("Hello World")
        return 0
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let _ast = parser.parse().unwrap();
    todo!();
}

#[ignore = "impl structs"]
#[test]
fn test_parse_full_7() {
    let mut lexer = Lexer::new(
        r#"
    struct Test {
        a u32,
        b u32,
    }

    func main() u32 {
        let Test t = {
            b 28,
            a 1,
        }
        return t.b
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let _ast = parser.parse().unwrap();
    todo!();
}

#[ignore = "impl casts, imports, function calls"]
#[test]
fn test_parse_full_8() {
    let mut lexer = Lexer::new(
        r#"
    import std:io

    func main() u32 {
        let f32 a = 34.1
        let u32 b = a -> u32
        let f32 c = b -> f32
        
        io:printflt(a)
        io:printint(b)
        io:printflt(c)
        
        return 0
    }"#,
    );

    let mut parser = Parser::new(lexer.tokenize());
    let _ast = parser.parse().unwrap();
    todo!();
}
