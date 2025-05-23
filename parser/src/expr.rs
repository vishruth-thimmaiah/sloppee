use lexer::types::{Datatype, Delimiter, Operator, Types};

use super::{
    Parser, Result,
    errors::ParserError,
    nodes::{ASTNodes, Expression, Literal},
};

impl Parser {
    pub(crate) fn parse_expression(&mut self, delim: Vec<Types>) -> Result<Expression> {
        if self
            .next_if_type(Types::DELIMITER(Delimiter::LBRACKET))
            .is_some()
        {
            return self.parse_array();
        } else if let Types::DATATYPE(Datatype::STRING(_)) = self.peek().unwrap().r#type {
            return Ok(Expression::String(self.next().unwrap().value.unwrap()));
        } else if self
            .next_if_type(Types::DELIMITER(Delimiter::LBRACE))
            .is_some()
        {
            return self.parse_struct();
        }

        let mut operands: Vec<ASTNodes> = Vec::new();
        let mut operators: Vec<Types> = Vec::new();

        'outer: loop {
            let token = self.next().ok_or(ParserError::unexpected_eof(None))?;
            match token.r#type {
                Types::NUMBER | Types::BOOL => operands.push(ASTNodes::Literal(Literal {
                    value: token.value.unwrap(),
                    r#type: token.r#type,
                })),
                Types::IDENTIFIER => {
                    operands.push(self.parse_complex_variable()?);
                }
                Types::OPERATOR(Operator::CAST) => {
                    operands.push(ASTNodes::Token(Types::DATATYPE(self.parse_cast()?)));
                    operators.push(token.r#type);
                }
                Types::OPERATOR(ref op) => {
                    while !operators.is_empty() {
                        let pop_op = operators.last().unwrap();
                        if self.get_precedence(&token.r#type) > self.get_precedence(pop_op) {
                            break;
                        }
                        let pop = operators.pop().unwrap();
                        operands.push(ASTNodes::Token(pop));
                    }
                    operators.push(Types::OPERATOR(op.clone()));
                }
                Types::DELIMITER(Delimiter::LPAREN) => {
                    operators.push(token.r#type);
                }
                Types::DELIMITER(Delimiter::RPAREN) => loop {
                    let pop_op = &operators.pop();
                    if let Some(op) = pop_op {
                        if op == &Types::DELIMITER(Delimiter::LPAREN) {
                            break;
                        }
                        operands.push(ASTNodes::Token(op.clone()));
                    } else {
                        if delim.contains(&Types::DELIMITER(Delimiter::RPAREN)) {
                            self.prev();
                            break 'outer;
                        } else {
                            return Err(ParserError::new("Unexpected LPAREN", token));
                        }
                    }
                },
                Types::IDENTIFIER_FUNC => {
                    operands.push(ASTNodes::FunctionCall(self.parse_function_call()?));
                }
                ty if delim.contains(&ty) => {
                    self.prev();
                    break;
                }
                _ => return Err(ParserError::unexpected_token_err(token)),
            }
        }
        while !operators.is_empty() {
            let value = operators.pop().unwrap();
            if value == Types::DELIMITER(Delimiter::LPAREN) {}
            operands.push(ASTNodes::Token(value));
        }

        if operands.is_empty() {
            return Ok(Expression::None);
        }

        self.postfix_to_tree(&mut operands)
    }

    fn postfix_to_tree(&self, operands: &mut Vec<ASTNodes>) -> Result<Expression> {
        let op = if operands.len() > 1 {
            let value = operands.pop().unwrap();
            self.value_to_operator(value).unwrap()
        } else if operands.len() == 0 {
            todo!()
            // errors::parser_error(self, "Invalid postfix expression");
        } else {
            let token = operands.pop().unwrap();
            return Ok(Expression::Simple {
                left: Box::new(token),
                right: None,
                operator: None,
            });
        };

        let right = {
            let last_op = operands.last().unwrap();
            if let ASTNodes::Token(Types::OPERATOR(_)) = last_op {
                ASTNodes::Expression(self.postfix_to_tree(operands)?)
            } else {
                operands.pop().unwrap()
            }
        };

        let left = {
            let last_op = operands.last().unwrap();
            if let ASTNodes::Token(Types::OPERATOR(_)) = last_op {
                ASTNodes::Expression(self.postfix_to_tree(operands)?)
            } else {
                operands.pop().unwrap()
            }
        };

        Ok(Expression::Simple {
            left: Box::new(left),
            right: Some(Box::new(right)),
            operator: Some(op),
        })
    }

    fn get_precedence(&self, operator: &Types) -> usize {
        use Operator::*;
        match operator {
            Types::OPERATOR(CAST) => 10,
            Types::OPERATOR(MULTIPLY | DIVIDE | MODULO) => 8,
            Types::OPERATOR(PLUS | MINUS) => 7,
            Types::OPERATOR(LSHIFT | RSHIFT) => 6,
            Types::OPERATOR(GREATER | GREATER_EQUAL | LESSER | LESSER_EQUAL) => 5,
            Types::OPERATOR(EQUAL | NOT_EQUAL) => 4,
            Types::OPERATOR(BITWISE_AND) => 3,
            Types::OPERATOR(BITWISE_XOR) => 2,
            Types::OPERATOR(BITWISE_OR) => 1,
            Types::DELIMITER(Delimiter::LPAREN) => 0,
            _ => unreachable!(),
        }
    }

    fn value_to_operator(&self, value: ASTNodes) -> Option<Operator> {
        if let ASTNodes::Token(Types::OPERATOR(op)) = &value {
            return Some(op.clone());
        }
        None
    }

    // FIXME: Support trailing commas
    pub(crate) fn parse_array(&mut self) -> Result<Expression> {
        let mut array = Vec::new();
        loop {
            array.push(self.parse_expression(vec![
                Types::DELIMITER(Delimiter::COMMA),
                Types::DELIMITER(Delimiter::RBRACKET),
            ])?);
            if self
                .next_if_type(Types::DELIMITER(Delimiter::RBRACKET))
                .is_some()
            {
                break;
            }
            self.next_with_type(Types::DELIMITER(Delimiter::COMMA))?;
        }
        return Ok(Expression::Array(array));
    }

    // FIXME: Support trailing commas
    pub(crate) fn parse_struct(&mut self) -> Result<Expression> {
        let mut fields = vec![];

        loop {
            let name = self.next_with_type(Types::IDENTIFIER)?;
            let expr = self.parse_expression(vec![
                Types::DELIMITER(Delimiter::COMMA),
                Types::DELIMITER(Delimiter::RBRACE),
            ])?;
            fields.push((name.value.unwrap(), expr));

            if self
                .next_if_type(Types::DELIMITER(Delimiter::RBRACE))
                .is_some()
            {
                break;
            }
            self.next_with_type(Types::DELIMITER(Delimiter::COMMA))?;
        }

        return Ok(Expression::Struct(fields));
    }

    fn parse_cast(&mut self) -> Result<Datatype> {
        self.current_with_type(Types::OPERATOR(Operator::CAST))?;
        return self.parse_datatype();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::lexer::Lexer;

    #[test]
    fn test_parse_expression() {
        let mut lexer = Lexer::new("1 + 2 * 3 - 4 / 5 ");
        let mut parser = Parser::new(lexer.tokenize());
        let ast = parser.parse_expression(vec![Types::EOF]).unwrap();
        assert_eq!(
            ast,
            Expression::Simple {
                left: Box::new(ASTNodes::Expression(Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "1".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: Some(Box::new(ASTNodes::Expression(Expression::Simple {
                        left: Box::new(ASTNodes::Literal(Literal {
                            value: "2".to_string(),
                            r#type: Types::NUMBER
                        })),
                        right: Some(Box::new(ASTNodes::Literal(Literal {
                            value: "3".to_string(),
                            r#type: Types::NUMBER
                        }))),
                        operator: Some(Operator::MULTIPLY)
                    }))),
                    operator: Some(Operator::PLUS)
                })),
                right: Some(Box::new(ASTNodes::Expression(Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "4".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: Some(Box::new(ASTNodes::Literal(Literal {
                        value: "5".to_string(),
                        r#type: Types::NUMBER
                    }))),
                    operator: Some(Operator::DIVIDE)
                }))),
                operator: Some(Operator::MINUS)
            }
        );
    }

    #[test]
    fn test_parse_array() {
        let mut lexer = Lexer::new("[1, 2, 3, 4, 5]");
        let mut parser = Parser::new(lexer.tokenize());
        let ast = parser.parse_expression(vec![Types::EOF]).unwrap();
        assert_eq!(
            ast,
            Expression::Array(vec![
                Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "1".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: None,
                    operator: None
                },
                Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "2".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: None,
                    operator: None
                },
                Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "3".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: None,
                    operator: None
                },
                Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "4".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: None,
                    operator: None
                },
                Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "5".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: None,
                    operator: None
                }
            ])
        );
    }

    #[test]
    fn test_parse_string() {
        let mut lexer = Lexer::new("\"Hello World\"");
        let mut parser = Parser::new(lexer.tokenize());
        let ast = parser.parse_expression(vec![Types::EOF]).unwrap();
        assert_eq!(ast, Expression::String("Hello World".to_string()));
    }

    #[test]
    fn test_parse_struct() {
        let mut lexer = Lexer::new(" { a 4, b 7 }");
        let mut parser = Parser::new(lexer.tokenize());
        let ast = parser.parse_expression(vec![Types::EOF]).unwrap();
        assert_eq!(
            ast,
            Expression::Struct(vec![
                (
                    "a".to_string(),
                    Expression::Simple {
                        left: Box::new(ASTNodes::Literal(Literal {
                            value: "4".to_string(),
                            r#type: Types::NUMBER
                        })),
                        right: None,
                        operator: None
                    }
                ),
                (
                    "b".to_string(),
                    Expression::Simple {
                        left: Box::new(ASTNodes::Literal(Literal {
                            value: "7".to_string(),
                            r#type: Types::NUMBER
                        })),
                        right: None,
                        operator: None
                    }
                )
            ])
        );
    }

    #[test]
    fn test_parse_cast() {
        let mut lexer = Lexer::new("(23 + 43 * 3) -> f32 ");
        let mut parser = Parser::new(lexer.tokenize());
        let ast = parser.parse_expression(vec![Types::EOF]).unwrap();
        assert_eq!(
            ast,
            Expression::Simple {
                left: Box::new(ASTNodes::Expression(Expression::Simple {
                    left: Box::new(ASTNodes::Literal(Literal {
                        value: "23".to_string(),
                        r#type: Types::NUMBER
                    })),
                    right: Some(Box::new(ASTNodes::Expression(Expression::Simple {
                        left: Box::new(ASTNodes::Literal(Literal {
                            value: "43".to_string(),
                            r#type: Types::NUMBER
                        })),
                        right: Some(Box::new(ASTNodes::Literal(Literal {
                            value: "3".to_string(),
                            r#type: Types::NUMBER
                        }))),
                        operator: Some(Operator::MULTIPLY)
                    }))),
                    operator: Some(Operator::PLUS)
                })),
                right: Some(Box::new(ASTNodes::Token(Types::DATATYPE(Datatype::F32)))),
                operator: Some(Operator::CAST)
            }
        );
    }
}
