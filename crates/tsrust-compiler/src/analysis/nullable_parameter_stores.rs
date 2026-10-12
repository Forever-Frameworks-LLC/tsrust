//! Owned environment obligations for nullable records stored through parameters.
use super::*;
impl Analyzer {
    pub(super) fn nullable_parameter_store_type(&self,value:&Expr)->Option<Type> {
        if matches!(&value.kind,ExprKind::Inject{value,..} if matches!(value.kind,ExprKind::Null|ExprKind::Undefined)) {return None;}
        let ty=value.ty?;
        self.nullable_scoped_record_declaration(ty).map(|_|ty)
    }
}
