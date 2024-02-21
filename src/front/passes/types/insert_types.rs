use crate::front::ast_types::visitor::{ASTNodeEnum, GenericResolveResult, Visitable, Visitor};
use crate::front::ast_types::{AtomicExpression, ExpressionEnum, GlobalResolvedName, NamePath, Type};
use crate::front::exporter::export::FrontProgram;
use crate::front::passes::types::type_expression::{
    binop_type_resolver, literal_types, unop_type_resolver,
};
use std::collections::HashMap;
use std::rc::Rc;
use crate::front::passes::types::TypeError;

fn get_type_from_name_path(name_path: &NamePath, program: &FrontProgram) -> Type {
    let mut return_type = Type::Struct(name_path.name.clone());
    let mut struct_def = program.definitions.struct_definitions.get(name_path.name.global_resolved.as_ref().unwrap());

    for name_path_path in name_path.path.iter() {
        if let Some(struct_def_unwrap) = struct_def {
            return_type = struct_def_unwrap.fields.get(name_path_path).unwrap().clone();

            match &return_type {
                Type::Struct(name_path) => {
                    struct_def = program.definitions.struct_definitions.get(name_path.global_resolved.as_ref().unwrap());
                }
                _ => {}
            }
        } else {
            panic!("Tried to get field of non-struct type")
        }
    }

    return_type
}


#[derive(Debug, PartialEq)]
pub enum ResolverError {
    TypeError(TypeError),
}

pub type ResolveResult<T> = GenericResolveResult<T, ResolverError>;

impl Visitor<Type, ResolverError> for ResolvedVarDefTable<'_> {
    fn apply(&mut self, ast_node: &mut ASTNodeEnum) -> ResolveResult<Type> {
        match ast_node {
            ASTNodeEnum::VarDef(&mut ref mut x) => {
                x.type_ = Some(
                    self.var_types
                        .get(x.name.global_resolved.as_ref().unwrap())
                        .unwrap()
                        .clone(),
                );
            }

            ASTNodeEnum::VarDecl(&mut ref mut x) => {
                x.var_def.visit(self)?;
                if let Some(expr) = &mut x.expr {
                    if &expr.visit(self)?.unwrap() != x.var_def.type_.as_ref().unwrap() {
                        return Err(ResolverError::TypeError(TypeError::MultipleTypes));
                    }
                }
            }

            ASTNodeEnum::VarAssign(&mut ref mut x) => {
                // get_type_from_name_path(&x.name_path, self.program);


                if self
                    .var_types
                    .get(x.name_path.name.global_resolved.as_ref().unwrap())
                    .unwrap()
                    .clone()
                    != x.expr.visit(self)?.unwrap()
                {
                    return Err(ResolverError::TypeError(TypeError::MultipleTypes));
                }
            }

            ASTNodeEnum::Expression(&mut ref mut x) => {
                x.type_ = Some(match &mut x.expr {
                    ExpressionEnum::AtomicExpression(atomic) => match atomic {
                        AtomicExpression::Variable(name_path) => self
                            .var_types
                            .get(name_path.name.global_resolved.as_ref().unwrap())
                            .unwrap()
                            .clone(),
                        AtomicExpression::FnCall(fn_call) => self
                            .var_types
                            .get(fn_call.name.global_resolved.as_ref().unwrap())
                            .unwrap()
                            .clone(),
                        AtomicExpression::Literal(literal) => literal_types(literal),
                        AtomicExpression::StructInit(struct_init) => {
                            Type::Struct(struct_init.type_.clone())
                        }
                    },
                    ExpressionEnum::Unary(unop, x) => {
                        match unop_type_resolver(unop, &x.visit(self)?.unwrap()) {
                            Ok(type_) => type_,
                            Err(type_error) => {
                                return Err(ResolverError::TypeError(type_error));
                            }
                        }
                    }
                    ExpressionEnum::Binary(e0, binop, e1) => {
                        let t0 = e0.visit(self)?.unwrap();
                        let t1 = e1.visit(self)?.unwrap();

                        match binop_type_resolver(binop, &t0, &t1) {
                            Ok(type_) => type_,
                            Err(type_error) => {
                                return Err(ResolverError::TypeError(type_error));
                            }
                        }
                    }
                });
                return Ok((false, x.type_.clone()));
            }

            ASTNodeEnum::If(_)
            | ASTNodeEnum::Else(_)
            | ASTNodeEnum::While(_)
            | ASTNodeEnum::For(_)
            | ASTNodeEnum::Statement(_)
            | ASTNodeEnum::Block(_)
            | ASTNodeEnum::FnDef(_)
            | ASTNodeEnum::FnCall(_)
            | ASTNodeEnum::AtomicExpression(_)
            | ASTNodeEnum::StructInit(_) => return Ok((true, None)),

            ASTNodeEnum::NamePath(_)
            | ASTNodeEnum::Reference(_)
            | ASTNodeEnum::StructDef(_)
            | ASTNodeEnum::LiteralValue(_)
            | ASTNodeEnum::Definition(_)
            | ASTNodeEnum::Module(_)
            | ASTNodeEnum::Use(_) => return Ok((false, None)),
        };
        return Ok((false, None));
    }
}

pub struct ResolvedVarDefTable<'a> {
    pub program: &'a mut FrontProgram,
    pub var_types: HashMap<Rc<GlobalResolvedName>, Type>,
}

pub fn insert_types(program: &mut FrontProgram, var_types: HashMap<Rc<GlobalResolvedName>, Type>) -> Result<(), TypeError> {
    let mut var_types = var_types;



    for fn_name in program.definitions.function_definitions.keys().map(|x| x.clone()).collect::<Vec<_>>() {
        let fn_body = program.definitions.function_definitions.get_mut(&fn_name).unwrap();
        let mut statements = fn_body.body.statements.drain(..).collect::<Vec<_>>();

        let mut table = ResolvedVarDefTable {
            program,
            var_types,
        };

        for statement in &mut statements {
            if let Err(e) = statement.visit(&mut table) {
                return match e {
                    ResolverError::TypeError(type_error) => {
                        Err(type_error)
                    }
                };
            }
        }

        var_types = table.var_types;
        let fn_body = program.definitions.function_definitions.get_mut(&fn_name).unwrap();
        fn_body.body.statements = statements;
    }
    Ok(())
}
