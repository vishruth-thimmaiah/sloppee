use lexer::types::{Datatype, Operator, Types};

#[derive(Debug, PartialEq)]
pub enum ASTNodes {
    AssignStmt(AssignStmt),
    ArrayIndex(ArrayIndex),
    Attr(Attr),
    Block(Block),
    Conditional(Conditional),
    Expression(Expression),
    Function(Function),
    FunctionCall(FunctionCall),
    ImportDef(ImportDef),
    ImportCall(ImportCall),
    LetStmt(LetStmt),
    Literal(Literal),
    Loop(Loop),
    ForLoop(ForLoop),
    Method(Method),
    Return(Return),
    StructDef(StructDef),
    Token(Types),
    Variable(Variable),
    Break,
    Extern(Extern),
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, Datatype)>,
    pub return_type: Option<Datatype>,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub body: Vec<ASTNodes>,
}

#[derive(Debug, PartialEq)]
pub struct Return {
    pub value: Option<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Simple {
        left: Box<ASTNodes>,
        right: Option<Box<ASTNodes>>,
        operator: Option<Operator>,
    },
    Array(Vec<Expression>),
    String(String),
    Struct(Vec<(String, Expression)>),
    None,
}

impl Expression {
    pub fn is_none(&self) -> bool {
        if let Expression::None = self {
            return true;
        }
        false
    }
}

#[derive(Debug, PartialEq)]
pub struct Literal {
    pub value: String,
    pub r#type: Types,
}

#[derive(Debug, PartialEq)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Attr {
    pub name: Variable,
    pub parent: Box<ASTNodes>,
}

#[derive(Debug, PartialEq)]
pub struct Method {
    pub func: FunctionCall,
    pub parent: Box<ASTNodes>,
}

#[derive(Debug, PartialEq)]
pub struct LetStmt {
    pub name: String,
    pub value: Expression,
    pub datatype: Datatype,
    pub mutable: bool,
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Datatype)>,
}

#[derive(Debug, PartialEq)]
pub struct ImportDef {
    pub path: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct AssignStmt {
    pub name: Box<ASTNodes>,
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub enum Conditional {
    If {
        condition: Expression,
        body: Block,
        else_body: Option<Box<Conditional>>,
    },
    Else {
        body: Block,
    },
}

#[derive(Debug, PartialEq)]
pub struct Loop {
    pub condition: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub struct ForLoop {
    pub value: Variable,
    pub increment: Variable,
    pub iterator: Expression,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub struct ImportCall {
    pub path: Vec<String>,
    pub ident: Box<ASTNodes>,
}

#[derive(Debug, PartialEq)]
pub struct ArrayIndex {
    pub array_var: Box<ASTNodes>,
    pub index: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Extern {
    pub name: String,
    pub args: Vec<(String, Datatype)>,
    pub return_type: Option<Datatype>,
}
