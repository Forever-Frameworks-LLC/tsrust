//! Concrete absent environments for local nullable callback-record storage.
use super::*;

impl Analyzer {
    pub(super) fn nullable_scoped_record_declaration(&self, ty: Type) -> Option<usize> {
        let Type::Record(record) = self.unions.payload(ty)? else { return None; };
        let declaration = self.scoped_records.declaration(record);
        self.aggregate_callable_facts(Type::Record(declaration)).scoped.then_some(declaration)
    }

    /// Absence carries no callback instance, but the tagged type still needs a
    /// concrete environment. Preserve the nullish variant and nominal contract.
    pub(super) fn absent_nullable_record_annotation(&mut self, context: &Context, expected: Option<Type>, value: &Expr) -> Result<Option<Type>, Diagnostic> {
        if !matches!(value.kind, ExprKind::Null | ExprKind::Undefined)
            && !matches!(&value.kind,ExprKind::Name(name) if name=="undefined" && !context.names.contains_key(name))
        { return Ok(expected); }
        let Some(ty) = expected else { return Ok(None); };
        let Some(record) = self.nullable_scoped_record_declaration(ty) else { return Ok(expected); };
        let Some(union) = ty.union() else { return Ok(expected); };
        let empty = self.empty_scoped_record_environment(record)?;
        let concrete = self.unions.intern(self.unions.members(union).into_iter().map(|member| {
            if matches!(member,Type::Record(_)) { empty } else { member }
        }), value.span)?;
        Ok(Some(concrete))
    }
}
