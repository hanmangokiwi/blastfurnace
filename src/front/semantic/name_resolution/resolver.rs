use crate::front::semantic::name_resolution::scope_table::{name_format, ScopeTable, SymbolType};
use crate::front::syntax::ast_types::{
    AtomicExpression, Block, Compound, CompoundValue, Expression, FnCall, FnDef, For, If,
    LiteralValue, NamePath, Statement, StatementBlock, StructDef, Type, VarAssign, VarDecl, VarDef,
    While,
};

pub trait Resolvable {
    fn resolve(&mut self, _scope_table: &mut ScopeTable) -> ResolveResult<()> {
        Ok(())
    }
}

pub trait Registrable {
    fn register(&mut self, _scope_table: &mut ScopeTable) -> ResolveResult<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ResolverError {
    UndefinedVariable(String),
    Redefinition(String),
}

pub type ResolveResult<T> = Result<T, ResolverError>;

impl Resolvable for Block {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        scope_table.scope_enter();
        for statement in &mut self.statements {
            statement.resolve(scope_table)?;
        }
        scope_table.scope_exit();
        Ok(())
    }
}

impl Resolvable for StatementBlock {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        match self {
            StatementBlock::Statement(statement) => statement.resolve(scope_table)?,
            StatementBlock::Block(block) => block.resolve(scope_table)?,
        }
        Ok(())
    }
}

impl Resolvable for Statement {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        match self {
            Statement::VarDecl(statement) => statement.resolve(scope_table)?,
            Statement::VarAssign(statement) => statement.resolve(scope_table)?,
            Statement::StructDef(statement) => statement.resolve(scope_table)?,
            Statement::FnDef(statement) => statement.resolve(scope_table)?,
            Statement::If(statement) => statement.resolve(scope_table)?,
            Statement::While(statement) => statement.resolve(scope_table)?,
            Statement::For(statement) => statement.resolve(scope_table)?,
            Statement::Return(statement) => statement.resolve(scope_table)?,
            Statement::Expression(statement) => statement.resolve(scope_table)?,
            _ => {}
        };
        Ok(())
    }
}

impl Resolvable for VarDef {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        if let Type::Struct(struct_name) = &mut self.type_ {
            struct_name.resolved =
                match scope_table.scope_lookup(&struct_name.raw, SymbolType::Struct) {
                    Some(name) => Some(name.clone()),
                    None => Some(name_format(&struct_name.raw, 0)),
                }
        }
        self.register(scope_table)?;
        Ok(())
    }
}

impl Resolvable for VarDecl {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        if let Some(ref mut expr) = self.expr {
            expr.resolve(scope_table)?
        }
        self.var_def.resolve(scope_table)?;

        Ok(())
    }
}

impl Resolvable for VarAssign {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        self.name_path.resolve(scope_table)?;
        self.expr.resolve(scope_table)?;

        Ok(())
    }
}

impl Resolvable for StructDef {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        self.register(scope_table)?;

        Ok(())
    }
}

impl Resolvable for FnDef {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        scope_table.scope_enter();
        self.register(scope_table)?;
        self.body.resolve(scope_table)?;
        scope_table.scope_exit();

        Ok(())
    }
}

impl Resolvable for Expression {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        match self {
            Expression::AtomicExpression(atomic) => {
                atomic.resolve(scope_table)?;
            }
            Expression::Unary(_, expression) => {
                expression.resolve(scope_table)?;
            }
            Expression::Binary(e0, _, e1) => {
                e0.resolve(scope_table)?;
                e1.resolve(scope_table)?;
            }
        }

        Ok(())
    }
}

impl Resolvable for AtomicExpression {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        match self {
            AtomicExpression::Literal(lit) => {
                if let LiteralValue::Compound(compound) = lit {
                    compound.resolve(scope_table)?;
                }
            }
            AtomicExpression::Variable(var) => {
                var.resolve(scope_table)?;
            }
            AtomicExpression::FnCall(fn_call) => {
                fn_call.resolve(scope_table)?;
            }
        }
        Ok(())
    }
}

impl Resolvable for NamePath {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        match scope_table.scope_lookup(&self.name.raw, SymbolType::Var) {
            Some(name) => {
                self.name.resolved = Some(name.clone());
                Ok(())
            }
            None => Err(ResolverError::UndefinedVariable(self.name.raw.clone())),
        }
    }
}

impl Resolvable for Compound {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        for (_, value) in self.iter_mut() {
            match value {
                CompoundValue::Expression(expr) => expr.resolve(scope_table)?,
                CompoundValue::Compound(compound) => compound.resolve(scope_table)?,
            }
        }
        Ok(())
    }
}

impl Resolvable for FnCall {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        match scope_table.scope_lookup(&self.name.raw, SymbolType::Fn) {
            Some(name) => {
                self.name.resolved = Some(name.clone());
            }
            None => return Err(ResolverError::UndefinedVariable(self.name.raw.clone())),
        }

        for arg in &mut self.args {
            arg.resolve(scope_table)?;
        }
        Ok(())
    }
}

impl Resolvable for If {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        self.cond.resolve(scope_table)?;
        self.body.resolve(scope_table)?;
        if let Some(ref mut else_body) = self.else_ {
            else_body.resolve(scope_table)?;
        }
        Ok(())
    }
}

impl Resolvable for While {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        self.cond.resolve(scope_table)?;
        self.body.resolve(scope_table)?;
        Ok(())
    }
}

impl Resolvable for For {
    fn resolve(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        if let Some(ref mut init) = self.init {
            init.resolve(scope_table)?;
        }
        if let Some(ref mut cond) = self.cond {
            cond.resolve(scope_table)?;
        }
        if let Some(ref mut step) = self.step {
            step.resolve(scope_table)?;
        }
        self.body.resolve(scope_table)?;
        Ok(())
    }
}
impl Registrable for StructDef {
    fn register(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        self.type_name.resolved =
            Some(scope_table.scope_bind(&self.type_name.raw, SymbolType::Struct)?);
        Ok(())
    }
}
impl Registrable for VarDef {
    fn register(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        self.name.resolved = Some(scope_table.scope_bind(&self.name.raw, SymbolType::Var)?);
        Ok(())
    }
}
impl Registrable for FnDef {
    fn register(&mut self, scope_table: &mut ScopeTable) -> ResolveResult<()> {
        self.name.resolved = Some(scope_table.scope_bind(&self.name.raw, SymbolType::Fn)?);
        for arg in &mut self.args {
            arg.register(scope_table)?;
        }
        Ok(())
    }
}
