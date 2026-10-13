//! Optional postfix chains keep each selected source subtree exactly once.
use super::*;
use crate::syntax::SelectedSource;

impl Analyzer {
    pub(in crate::analysis) fn optional_static(&self, context: &Context, mut expr: &mut Expr) {
        loop { match &mut expr.kind {
            ExprKind::Call(callee, _) => {
                if matches!(&callee.kind, ExprKind::Unary { op: UnaryOp::OptionalBoundary, value, .. } if matches!(value.kind, ExprKind::Name(_) | ExprKind::Field(_, _)) && !self.expression_type_hint(context,value).is_some_and(|ty|ty.union().is_some() || matches!(ty,Type::Null|Type::Undefined)))
                {
                    let placeholder = Expr::checked(ExprKind::Null, callee.span, None);
                    let mut old = std::mem::replace(callee.as_mut(), placeholder);
                    let ExprKind::Unary { value, .. } = old.take_kind() else {
                        unreachable!()
                    };
                    **callee = *value;
                }
                expr=callee;
            }
            ExprKind::Field(base, _) => {
                if matches!(&base.kind, ExprKind::Unary { op: UnaryOp::OptionalBoundary, value, .. } if matches!(&value.kind, ExprKind::Name(name) if matches!(name.as_str(), "Array"|"Math"|"Number"|"console"|"Boolean"|"String")))
                {
                    let placeholder = Expr::checked(ExprKind::Null, base.span, None);
                    let mut old = std::mem::replace(base.as_mut(), placeholder);
                    let ExprKind::Unary { value, .. } = old.take_kind() else {
                        unreachable!()
                    };
                    **base = *value;
                }
                expr=base;
            }
            ExprKind::Index(base, _)
            | ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value: base,
                ..
            } => expr=base,
            _ => break
        }
    }
    }

    /// Find only the receiver spine, never argument/index subexpressions or grouped chains.
    pub(in crate::analysis) fn optional_extract(expr:&mut Expr)->Option<(Expr,Span)>{
        let mut cursor=&*expr;let mut depth=0;let mut boundary=None;
        loop{match &cursor.kind{
            ExprKind::Field(base,_)|ExprKind::Index(base,_)|ExprKind::Call(base,_)=>cursor=base,
            ExprKind::Unary{op:UnaryOp::OptionalBoundary,value,..}=>{boundary=Some(depth);cursor=value;},
            _=>break,
        }depth+=1;}
        let depth=boundary?;let mut cursor=expr;
        for _ in 0..depth{cursor=match &mut cursor.kind{
            ExprKind::Field(base,_)|ExprKind::Index(base,_)|ExprKind::Call(base,_)|ExprKind::Unary{op:UnaryOp::OptionalBoundary,value:base,..}=>base,
            _=>unreachable!("optional receiver spine"),
        };}
        let ExprKind::Unary{value,operator,..}=&mut cursor.kind else{unreachable!()};
        let span=*operator;let source_span=value.span;
        let base=std::mem::replace(value.as_mut(),Expr::checked(ExprKind::Null,source_span,None));
        *cursor=Expr::checked(ExprKind::Selected{source:Box::new(SelectedSource{readonly:false,roots:Vec::new(),place:None,span:source_span}),slot:span.start,owned:false,access:Mode::Read},source_span,None);
        Some((base,span))
    }

    pub(in crate::analysis) fn optional_selected(
        mut expr: &mut Expr,
        slot: usize,
        ty: Type,
        source: &SelectedSource,
        owned: bool,
    ) {
        loop { match &mut expr.kind {
            ExprKind::Selected {
                source: current,
                slot: id,
                owned: own,
                ..
            } if *id == slot => {
                **current = source.clone();
                *own = owned;
                expr.ty = Some(ty);
                break;
            }
            ExprKind::Field(base, _)
            | ExprKind::Index(base, _)
            | ExprKind::Call(base, _)
            | ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value: base,
                ..
            } => expr=base,
            _ => break
        }
    }
    }

    #[inline(never)]
    pub(super) fn optional_hint_boundary(mut value:&Expr)->bool {
        loop {match &value.kind {
            ExprKind::Unary{op:UnaryOp::OptionalBoundary,..}=>return true,
            ExprKind::Field(base,_)|ExprKind::Index(base,_)|ExprKind::Call(base,_)=>value=base,
            _=>return false,
        }}
    }

    pub(super) fn optional_borrowed_hint(&self,context:&Context,body:&Expr)->bool{
        self.optional_named_work(context,body,true)
    }
    pub(in crate::analysis) fn optional_source_named(&self,context:&Context,expr:&Expr)->bool{
        self.optional_named_work(context,expr,false)
    }
    fn optional_named_work(&self,context:&Context,expr:&Expr,borrowed:bool)->bool{
        let mut pending=vec![(expr,borrowed)];
        while let Some((expr,borrowed))=pending.pop(){
            if borrowed{
                if self.expression_type_hint(context,expr).is_some_and(Type::is_copy) || self.string_position(context,expr){return false;}
                match &expr.kind{
                    ExprKind::Field(base,_)|ExprKind::Index(base,_)=>pending.push((base,false)),
                    _=>return false,
                }
                continue;
            }
            match &expr.kind{
                ExprKind::OptionalChain(value)=>pending.push((value,true)),
                ExprKind::Conditional{consequent,alternate,..}=>{pending.push((alternate,false));pending.push((consequent,false));},
                ExprKind::Unary{op:UnaryOp::OptionalBoundary,value,..}=>pending.push((value,false)),
                ExprKind::Field(base,_)|ExprKind::Index(base,_)=>pending.push((base,false)),
                ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Field(base,name) if name=="get" && Self::stable_at_receiver(base) && matches!(self.expression_type_hint(context,base),Some(Type::Map(_))))=>{},
                ExprKind::Selected{owned,..} if !owned=>{},
                ExprKind::Name(name) if context.names.contains_key(name)=>{},
                _=>return false,
            }
        }
        true
    }

    pub(super) fn selected_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ty = expr.ty.expect("checked selected payload");
        let ExprKind::Selected {
            source,
            slot,
            owned,
            access,
        } = &mut expr.kind
        else {
            unreachable!()
        };
        if source.readonly && mode == Mode::Mutate {
            return Err(Diagnostic::new(
                "borrowed-view",
                "An optional readonly view cannot be mutated; use clone() for independent ownership.",
                expr.span,
            ));
        }
        if !*owned && mode == Mode::Consume && !ty.is_copy() {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An optional selection borrows its source; use clone() for independent ownership.",
                expr.span,
            ));
        }
        let mode = if ty.is_copy() && mode != Mode::Mutate {
            Mode::Read
        } else {
            mode
        };
        *access = (*access).max(mode);
        context
            .selected_access
            .entry(*slot)
            .and_modify(|prior| *prior = (*prior).max(mode))
            .or_insert(mode);
        if !*owned {
            for root in &source.roots {
                let prior = context.projection.take();
                if let Some((id, path)) = &source.place
                    && id == root
                    && !path.is_empty()
                {
                    context.projection = Some(Projection {
                        root: context.bindings[*root].name.clone(),
                        path: path.clone(),
                        replacing: false,
                        span: expr.span,
                    });
                }
                let keep = context.keep_flow.len();
                if let Some((root, fields)) = &source.place {
                    context.keep_flow.push(crate::flow::Place {
                        root: *root,
                        fields: fields.clone(),
                    });
                }
                let result = Self::access(context, *root, mode, expr.span, loans);
                context.keep_flow.truncate(keep);
                context.projection = prior;
                result?;
            }
        }
        self.require(ty, expected, expr.span)?;
        Ok(ty)
    }

    // A stable, physically non-null storage reference has no optional branch.
    // Keep the ordinary helper's argument evaluation and receiver reacquisition.
    pub(in crate::analysis) fn optional_present_storage(&self,context:&Context,expr:&mut Expr){
        let span=expr.span;let mut current=std::mem::replace(expr,Expr::checked(ExprKind::Null,span,None));
        let mut parents=Vec::new();
        loop{
            let child=match &mut current.kind{
                ExprKind::Field(base,_)|ExprKind::Index(base,_)|ExprKind::Call(base,_)|ExprKind::Unary{op:UnaryOp::OptionalBoundary,value:base,..}=>Some(base),
                _=>None,
            };
            let Some(child)=child else{break};
            let span=child.span;let next=std::mem::replace(child.as_mut(),Expr::checked(ExprKind::Null,span,None));
            parents.push(current);current=next;
        }
        while let Some(mut parent)=parents.pop(){
            if matches!(parent.kind,ExprKind::Unary{op:UnaryOp::OptionalBoundary,..}){
                let physical=crate::flow::place(&current,&context.names).and_then(|place|self.place_type(context,&place));
                if Self::named_storage(&current) && physical.is_some_and(|ty|ty.union().is_none() && !matches!(ty,Type::Null|Type::Undefined)){continue;}
            }
            let child=match &mut parent.kind{
                ExprKind::Field(base,_)|ExprKind::Index(base,_)|ExprKind::Call(base,_)|ExprKind::Unary{op:UnaryOp::OptionalBoundary,value:base,..}=>base,
                _=>unreachable!("optional storage parent"),
            };
            **child=current;current=parent;
        }
        *expr=current;
    }

    pub(super) fn optional_chain(&mut self,context:&mut Context,expr:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        continuations::driver::check_optional(self,context,expr,mode,loans,expected)
    }
}
