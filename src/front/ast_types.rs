use std::collections::HashMap;
use std::rc::Rc;
use crate::middle::format::types::GlobalName;

pub type RawName = String;
pub type ResolvedName = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Reference {
    pub raw: RawName,
    pub module_resolved: Option<Rc<ResolvedName>>,
    pub global_resolved: Option<Rc<GlobalName>>,
}

impl Reference {
    pub fn new(raw: RawName) -> Reference {
        Reference {
            raw,
            module_resolved: None,
            global_resolved: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Void,
    Bool,
    Int,
    Float,
    Double,
    String,
    Struct(Reference),
}

#[derive(Debug, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
    Deref,
    Ref,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NamePath {
    pub name: Reference,
    pub path: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum VarMod {
    Const,
}

#[derive(Debug, PartialEq)]
pub struct VarDecl {
    pub var_def: VarDef,
    pub expr: Option<Box<Expression>>,
}

#[derive(Debug, PartialEq)]
pub struct VarAssign {
    pub name_path: NamePath,
    pub expr: Box<Expression>,
}

pub type Compound = HashMap<String, CompoundValue>;

#[derive(Debug, PartialEq)]
pub enum CompoundValue {
    Expression(Box<Expression>),
    Compound(Box<Compound>),
}

#[derive(Debug, PartialEq)]
pub enum FnMod {
    Rec,
    Inline,
}

#[derive(Debug, PartialEq)]
pub enum StructMod {}

#[derive(Debug, PartialEq)]
pub struct StructDef {
    pub mods: Rc<Vec<StructMod>>,
    pub type_name: Reference,
    pub map: HashMap<String, Type>,
}

#[derive(Debug, PartialEq)]
pub struct VarDef {
    pub mods: Rc<Vec<VarMod>>,
    pub name: Reference,
    pub type_: Type,
}

#[derive(Debug, PartialEq)]
pub struct FnDef {
    pub return_type: Type,
    pub mods: Rc<Vec<FnMod>>,
    pub name: Reference,
    pub args: Vec<VarDef>,
    pub body: Option<Block>, // only None for imported
}

#[derive(Debug, PartialEq)]
pub struct FnCall {
    pub name: Reference,
    pub args: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Null,
    Bool(bool),
    Int(i32),
    Decimal(f64),
    String(String),
    Compound(Compound),
}

#[derive(Debug, PartialEq)]
pub enum AtomicExpression {
    Literal(LiteralValue),
    Variable(NamePath),
    FnCall(Box<FnCall>),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    AtomicExpression(AtomicExpression),
    Unary(UnOp, Box<Expression>),
    Binary(Box<Expression>, BinOp, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub struct If {
    pub cond: Box<Expression>,
    pub body: Box<Block>,
    pub else_: Option<Box<If>>,
}

#[derive(Debug, PartialEq)]
pub struct While {
    pub cond: Box<Expression>,
    pub body: Box<Block>,
}

#[derive(Debug, PartialEq)]
pub struct For {
    pub init: Option<Box<Statement>>,
    pub cond: Option<Box<Expression>>,
    pub step: Option<Box<Statement>>,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    VarDecl(VarDecl),
    VarAssign(VarAssign),
    If(If),
    While(While),
    For(For),
    Return(Box<Expression>),
    Break,
    Continue,
    Expression(Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum StatementBlock {
    Statement(Statement),
    Block(Block),
}

#[derive(Debug, PartialEq)]
pub enum Definition {
    VarDecl(VarDecl),
    StructDef(StructDef),
    FnDef(FnDef),
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub definitions: Vec<Definition>,
    pub statements: Vec<StatementBlock>,
}