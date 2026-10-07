use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn contextual_expression_callee(&self, context:&Context, callee:&mut Expr, args:&[Expr])->Result<(),Diagnostic> {
        let ExprKind::Closure {params,..}=&callee.kind else{return Ok(());};
        let mut hints=Vec::with_capacity(args.len());
        for (position,argument) in args.iter().enumerate() {
            let hint=self.expression_type_hint(context,argument);
            if hint.is_none() && params.get(position).is_some_and(|parameter|matches!(parameter.ty,ParameterAnnotation::Inferred(_)) && matches!(parameter.initializer,ParameterInitializer::Required)) {
                return Err(Diagnostic::new("parameter-inference","This argument needs an explicit parameter type in the invoked function expression.",argument.span));
            }
            hints.push(hint.unwrap_or(Type::Void));
        }
        self.contextual_arrow_parameters(callee,&hints)
    }

    #[inline(never)]
    pub(super) fn contextual_spread_callee(&self, callee:&mut Expr, segments:&[SpreadSegment])->Result<(),Diagnostic> {
        let ExprKind::Closure {params,..}=&callee.kind else{return Ok(());};
        let mut hints=Vec::with_capacity(params.len());
        for (position,param) in params.iter().enumerate() {
            let mut candidates=segments.iter().enumerate().filter(|(index,segment)| {
                let (lower,upper)=super::spread_calls::spread_position_range(segments,*index,0,None);
                segment.length!=Some(0) && position>=lower && upper.is_none_or(|upper|position<=upper)
            }).map(|(_,segment)|segment.element);
            let first=candidates.next();
            let unique=first.filter(|first|candidates.all(|next|next==*first));
            if unique.is_none() && matches!(param.ty,ParameterAnnotation::Inferred(_)) && matches!(param.initializer,ParameterInitializer::Required) {
                return Err(Diagnostic::new("parameter-inference","This spread argument position needs an explicit parameter type.",param.name.span));
            }
            hints.push(unique.unwrap_or(Type::Undefined));
        }
        self.contextual_arrow_parameters(callee,&hints)
    }

    /// A returned callable can extend a fresh callee's captures. Traverse
    /// aggregate results too, so releasing a temporary never hides an escape.
    #[inline(never)]
    pub(super) fn expression_result_retains_captures(&self, ty:Type)->bool {
        let mut pending=vec![ty];let mut seen=HashSet::new();
        while let Some(ty)=pending.pop(){
            if !seen.insert(ty){continue;}
            match ty {
                Type::Closure(_)|Type::Callback(_)|Type::Callable(_)|Type::StoredCallable(_)|Type::BorrowedCallable(_)=>return true,
                Type::Array(element)=>pending.push(element.ty()),
                Type::Record(id)=>{
                    for field in &self.program.records[id].fields {
                        let Some(ty)=field.ty else{return true;};pending.push(ty);
                    }
                }
                _=>{if let Some(union)=ty.union(){pending.extend(self.unions.members(union));}}
            }
        }
        false
    }
}
