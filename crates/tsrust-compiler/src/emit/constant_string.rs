//! Static concat results retain the ordinary operand evaluations.
use super::*;
use crate::fixed_storage::visit_statements;

#[derive(Default)]
pub(super) struct Index<'a>(HashMap<usize,HashMap<usize,&'a str>>);

fn value<'a>(expr:&'a Expr,known:Option<&HashMap<usize,&'a str>>)->Option<&'a str> {
    match &expr.kind {
        ExprKind::String(text)=>Some(text),
        ExprKind::Name(_)=>known?.get(&expr.binding?).copied(),
        ExprKind::Unary{op:UnaryOp::Typeof,value,..} if !matches!(value.kind,ExprKind::Name(_) if value.binding.is_none())=>match value.ty {
            Some(Type::Null)=>Some("object"),
            Some(Type::Undefined|Type::Void)=>Some("undefined"),
            _=>None,
        },
        ExprKind::Call(_,args) if expr.target==Some(CallTarget::StringConversion)=>match args.first().and_then(|arg|arg.ty) {
            Some(Type::Null)=>Some("null"),
            Some(Type::Undefined|Type::Void)=>Some("undefined"),
            _=>None,
        },
        _=>None,
    }
}

impl<'a> Index<'a> {
    pub(super) fn new(program:&'a CheckedProgram)->Self {
        let mut index=Self::default();
        for fun in program.functions.iter().chain(program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
            let mut known=HashMap::new();
            visit_statements(&fun.body,&mut |stmt| {
                let StmtKind::Let{binding:Some(id),value:initializer,..}=&stmt.kind else {return;};
                let binding=&fun.bindings[*id];
                if binding.parameter || binding.rebindable || binding.mutable || binding.capture_reference || binding.callback_reference
                    || binding.borrowed_place.is_some() || binding.borrowed_element.is_some() || binding.mutable_place {return;}
                if binding.ty.scalar_base()==Type::String && let Some(text)=value(initializer,Some(&known)) {known.insert(*id,text);}
            });
            if !known.is_empty(){index.0.insert(fun as *const CheckedFunction as usize,known);}
        }
        index
    }

    pub(super) fn concat(&self,fun:&CheckedFunction,base:&Expr,args:&[Expr])->Option<String> {
        let known=self.0.get(&(fun as *const CheckedFunction as usize));
        let mut result=String::new();
        for operand in std::iter::once(base).chain(args) {
            let text=value(operand,known)?;
            // Larger results retain runtime concat; bound emitted-literal duplication.
            if result.len().checked_add(text.len())?>4096{return None;}
            result.push_str(text);
        }
        Some(result)
    }
}
