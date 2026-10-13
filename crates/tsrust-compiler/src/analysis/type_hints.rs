//! Type-hint continuations preserve metadata precedence and lazy child queries.
use super::*;
mod calls;
pub(super) enum Work<'a>{
    Expression(&'a Expr),Publish(&'a Expr),Call(calls::Frame<'a>),
    Pattern(&'a Expr),PatternResult(&'a Expr),
    Optional(&'a Expr),OptionalResult(&'a Expr),
    OptionalMissing{cursor:&'a Expr,body:&'a Expr,result:Type},
    OptionalMissingResult{cursor:&'a Expr,body:&'a Expr,result:Type},
    OptionalJoin(Type),BorrowedKnown{body:&'a Expr,hint:Option<Type>},
    Borrowed(&'a Expr),BorrowedResult(&'a Expr),BorrowedIndex(&'a Expr),
    Named(&'a Expr),NamedThen(&'a Expr),NamedMap,

    Conditional(&'a Expr,&'a Expr),ConditionalLeft(&'a Expr),ConditionalRight(Option<Type>,Span),
    Coalesce{left:&'a Expr,right:&'a Expr,span:Span,undefined_only:bool},
    CoalesceLeft{right:&'a Expr,span:Span,undefined_only:bool},CoalesceRight{lhs:Type,payload:Option<Type>,span:Span},
    Logical{left:&'a Expr,right:&'a Expr,op:BinaryOp,span:Span},
    LogicalLeft{right:&'a Expr,op:BinaryOp,span:Span},LogicalRight{lhs:Type,selected:Vec<Type>,span:Span},
    Unary(UnaryOp),LogicalAssignment,
    ArithmeticLeft{right:&'a Expr,op:BinaryOp},ArithmeticRight{left:Option<Type>,op:BinaryOp},
    Field{base:&'a Expr,name:&'a str,span:Span},Index{base:&'a Expr,index:&'a Expr,span:Span},
    ArrayNext{values:&'a [Expr],index:usize,first:Option<Type>,number:Option<Type>},
    ArrayValue{values:&'a [Expr],index:usize,first:Option<Type>,number:Option<Type>,spread:bool},
}
impl Analyzer{
    pub(super) fn type_hint_work(&self,context:&Context,root:Work<'_>)->Option<Type>{
        let mut work=vec![root];let mut value=None;let mut borrowed=false;
        // The source and context are immutable for this invocation. Discard the
        // memo when returning; ownership/flow changes require a fresh query.
        let mut memo:HashMap<*const Expr,Option<Type>>=HashMap::new();
        while let Some(next)=work.pop(){match next{
            Work::Call(frame)=>calls::step(self,context,frame,&mut work,&mut value),
            Work::Publish(expr)=>{memo.insert(expr as *const Expr,value);},
            Work::Expression(expr)=>{
                if let Some(ready)=memo.get(&(expr as *const Expr)){value=*ready;continue;}
                if matches!(expr.kind,ExprKind::Call(..)){work.push(Work::Publish(expr));work.push(Work::Call(calls::Frame::Start(expr)));continue;}
                if let Some(ready)=self.type_hint_ready(context,expr){value=ready;memo.insert(expr as *const Expr,value);continue;}
                work.push(Work::Publish(expr));
                match &expr.kind{
                    ExprKind::OptionalChain(body)=>work.push(Work::Optional(body)),
                    ExprKind::Sequence(values)=>{value=None;if let Some(last)=values.last(){work.push(Work::Expression(last));}},
                    ExprKind::TypeAssertion{value:child,satisfies:true,..}=>work.push(Work::Expression(child)),
                    ExprKind::Conditional{consequent,alternate,..}=>work.push(Work::Conditional(consequent,alternate)),
                    ExprKind::CoalesceAssign{target,value:child,..}=>work.push(Work::Coalesce{left:target,right:child,span:expr.span,undefined_only:false}),
                    ExprKind::LogicalAssign{target,..}=>{work.push(Work::LogicalAssignment);work.push(Work::Expression(target));},
                    ExprKind::Update{target,..}|ExprKind::Assign{target,..}|ExprKind::CompoundAssign{target,..}=>work.push(Work::Expression(target)),
                    ExprKind::Unary{op,value:child,..}=>match op{
                        UnaryOp::VariantIs(_)|UnaryOp::Not=>value=Some(Type::Boolean),
                        UnaryOp::Typeof=>value=Some(Type::String),UnaryOp::Void=>value=Some(Type::Undefined),
                        UnaryOp::BitNot=>value=Some(self.bitwise_or_power_type(context,child,None,None)),
                        UnaryOp::PatternSelection=>work.push(Work::Pattern(child)),
                        _=>{work.push(Work::Unary(*op));work.push(Work::Expression(child));},
                    },
                    ExprKind::Binary{op,left,right,..}=>match op{
                        BinaryOp::Coalesce|BinaryOp::PatternDefault=>work.push(Work::Coalesce{left,right,span:expr.span,undefined_only:*op==BinaryOp::PatternDefault}),
                        BinaryOp::And|BinaryOp::Or=>work.push(Work::Logical{left,right,op:*op,span:expr.span}),
                        op if op.bitwise_or_power()=>value=Some(self.bitwise_or_power_type(context,left,Some(right),None)),
                        BinaryOp::Add|BinaryOp::Subtract|BinaryOp::Multiply|BinaryOp::Divide|BinaryOp::Remainder=>{
                            work.push(Work::ArithmeticLeft{right,op:*op});work.push(Work::Expression(left));
                        },
                        _=>value=Some(Type::Boolean),
                    },
                    ExprKind::Field(base,name) if matches!(&base.kind,ExprKind::Name(namespace) if NumericConstant::lookup(namespace,name).is_some())=>value=Some(Type::Number),
                    ExprKind::Field(base,name)=>{work.push(Work::Field{base,name,span:expr.span});work.push(Work::Expression(base));},
                    ExprKind::Index(base,index)=>{work.push(Work::Index{base,index,span:expr.span});work.push(Work::Expression(base));},
                    ExprKind::Array(values) if values.is_empty() && self.builtin_literals=>value=Some(Type::Array(ArrayElement::Int(0))),
                    ExprKind::Array(values)|ExprKind::ArrayOf(values,None)=>work.push(Work::ArrayNext{values,index:0,first:None,number:None}),
                    _=>value=self.type_hint_leaf(context,expr),
                }
            },
            Work::Pattern(expr)=>match &expr.kind{
                ExprKind::Field(base,_)|ExprKind::Index(base,_)=>{work.push(Work::PatternResult(expr));work.push(Work::Expression(base));},
                _=>work.push(Work::Expression(expr)),
            },
            Work::PatternResult(expr)=>value=value.and_then(|ty|self.pattern_hint_result(expr,ty)),
            Work::Optional(body)=>{work.push(Work::OptionalResult(body));work.push(Work::Expression(body));},
            Work::OptionalResult(body)=>if let Some(result)=value{work.push(Work::OptionalMissing{cursor:body,body,result});},
            Work::OptionalMissing{mut cursor,body,result}=>{
                loop{match &cursor.kind{
                    ExprKind::Unary{op:UnaryOp::OptionalBoundary,value:child,..}=>{
                        work.push(Work::OptionalMissingResult{cursor:child,body,result});work.push(Work::Expression(child));break;
                    },
                    ExprKind::Field(base,_)|ExprKind::Index(base,_)|ExprKind::Call(base,_)=>cursor=base,
                    _=>{value=Some(result);break;},
                }}
            },
            Work::OptionalMissingResult{cursor,body,result}=>{
                let missing=value.is_some_and(|ty|ty.union().is_some_and(|id|self.unions.members(id).iter().any(|ty|matches!(ty,Type::Null|Type::Undefined))) || matches!(ty,Type::Null|Type::Undefined));
                if missing{
                    let joined=self.unions.intern([if result==Type::Void{Type::Undefined}else{result.owning()},Type::Undefined],body.span).ok();
                    value=joined;
                    if let Some(joined)=joined{work.push(Work::OptionalJoin(joined));work.push(Work::BorrowedKnown{body,hint:Some(result)});}
                }else{work.push(Work::OptionalMissing{cursor,body,result});}
            },
            Work::OptionalJoin(joined)=>value=if borrowed && !joined.is_copy(){joined.union().map(Type::BorrowedUnion)}else{Some(joined)},
            Work::Borrowed(body)=>{work.push(Work::BorrowedResult(body));work.push(Work::Expression(body));},
            Work::BorrowedResult(body)=>work.push(Work::BorrowedKnown{body,hint:value}),
            Work::BorrowedKnown{body,hint}=>{
                borrowed=false;
                if !hint.is_some_and(Type::is_copy){match &body.kind{
                    ExprKind::Index(base,_)=>{work.push(Work::BorrowedIndex(base));work.push(Work::Expression(base));},
                    ExprKind::Field(base,_)=>work.push(Work::Named(base)),_=>{},
                }}
            },
            Work::BorrowedIndex(base)=>{
                borrowed=false;
                if !value.is_some_and(|ty|self.unions.payload(ty).unwrap_or(ty)==Type::String){work.push(Work::Named(base));}
            },
            Work::Named(expr)=>{
                borrowed=false;
                match &expr.kind{
                    ExprKind::OptionalChain(body)=>work.push(Work::Borrowed(body)),
                    ExprKind::Conditional{consequent,alternate,..}=>{work.push(Work::NamedThen(alternate));work.push(Work::Named(consequent));},
                    ExprKind::Unary{op:UnaryOp::OptionalBoundary,value,..}=>work.push(Work::Named(value)),
                    ExprKind::Field(base,_)|ExprKind::Index(base,_)=>work.push(Work::Named(base)),
                    ExprKind::Call(callee,_)=>if let ExprKind::Field(base,name)=&callee.kind && name=="get" && Self::stable_at_receiver(base){work.push(Work::NamedMap);work.push(Work::Expression(base));},
                    ExprKind::Selected{owned,..}=>borrowed=!*owned,
                    ExprKind::Name(name)=>borrowed=context.names.contains_key(name),_=>{},
                }
            },
            Work::NamedThen(alternate)=>if borrowed{work.push(Work::Named(alternate));},
            Work::NamedMap=>borrowed=matches!(value,Some(Type::Map(_))),
            Work::Conditional(left,right)=>{work.push(Work::ConditionalLeft(right));work.push(Work::Expression(left));},
            Work::ConditionalLeft(right)=>{work.push(Work::ConditionalRight(value,right.span));work.push(Work::Expression(right));},
            Work::ConditionalRight(left,span)=>value=self.join_conditional_hints(left,value,span),
            Work::Coalesce{left,right,span,undefined_only}=>{work.push(Work::CoalesceLeft{right,span,undefined_only});work.push(Work::Expression(left));},
            Work::CoalesceLeft{right,span,undefined_only}=>{
                let Some(lhs)=value else{continue};
                if lhs==Type::Undefined || (!undefined_only && lhs==Type::Null){work.push(Work::Expression(right));continue;}
                let Some(union)=lhs.union() else{continue};
                let payload=if undefined_only{
                    let members=self.unions.members(union).into_iter().filter(|ty|*ty!=Type::Undefined).collect::<Vec<_>>();
                    if members.is_empty(){None}else{self.unions.intern(members,span).ok()}
                }else{self.unions.present(lhs.owning(),span).ok().flatten()};
                work.push(Work::CoalesceRight{lhs,payload,span});work.push(Work::Expression(right));
            },
            Work::CoalesceRight{lhs,payload,span}=>{
                let rhs=value;
                value=match(payload,rhs){
                    (None,rhs)=>rhs,
                    (Some(payload),rhs)=>{
                        let joined=match rhs{None=>Some(payload),Some(rhs)=>self.unions.intern([payload,rhs.owning()],span).ok()};
                        joined.map(|joined|if matches!(lhs,Type::BorrowedUnion(_))||matches!(rhs,Some(Type::BorrowedUnion(_))){joined.union().map(Type::BorrowedUnion).unwrap_or(joined)}else{joined})
                    },
                };
            },
            Work::Logical{left,right,op,span}=>{work.push(Work::LogicalLeft{right,op,span});work.push(Work::Expression(left));},
            Work::LogicalLeft{right,op,span}=>{
                let Some(lhs)=value else{continue};let(selected,reaches_right)=self.logical_selection(lhs,op);
                if reaches_right{work.push(Work::LogicalRight{lhs,selected,span});work.push(Work::Expression(right));}
                else{value=self.join_logical_hints(lhs,selected,span);}
            },
            Work::LogicalRight{lhs,mut selected,span}=>{
                if let Some(rhs)=value{selected.push(rhs.owning());value=self.join_logical_hints(lhs,selected,span);}
            },
            Work::Unary(op)=>value=value.and_then(|ty|match op{
                UnaryOp::NonNull=>self.unions.payload(ty.owning()).or_else(||(!matches!(ty,Type::Null|Type::Undefined|Type::Void)&&ty.union().is_none()).then_some(ty)),
                UnaryOp::OptionalBoundary=>self.unions.payload(ty).or(Some(ty)),
                UnaryOp::Positive if !matches!(ty,Type::Int|Type::Number)=>Some(Type::Number),
                _=>Some(ty),
            }),
            Work::LogicalAssignment=>value=value.or(Some(Type::Boolean)),
            Work::ArithmeticLeft{right,op}=>{work.push(Work::ArithmeticRight{left:value,op});work.push(Work::Expression(right));},
            Work::ArithmeticRight{left,op}=>value=Some(if op==BinaryOp::Add && (left==Some(Type::String)||value==Some(Type::String)){Type::String}else if left==Some(Type::Number)||value==Some(Type::Number){Type::Number}else{Type::Int}),
            Work::Field{base,name,span}=>value=value.and_then(|hint|self.field_type_hint_result(base,name,span,hint)),
            Work::Index{base,index,span}=>value=value.and_then(|hint|self.index_type_hint_result(base,index,span,hint)),
            Work::ArrayNext{values,index,first,number}=>{
                if let Some(expr)=values.get(index){
                    let(spread,child)=match &expr.kind{ExprKind::Unary{op:UnaryOp::Spread,value,..}=>(true,value.as_ref()),_=>(false,expr)};
                    work.push(Work::ArrayValue{values,index,first,number,spread});work.push(Work::Expression(child));
                }else{value=number.or(first).and_then(ArrayElement::from_value_type).map(Type::Array);}
            },
            Work::ArrayValue{values,index,first,number,spread}=>{
                let hint=if spread{match value{Some(Type::Array(element))=>Some(element.ty()),_=>None}}else{value};
                let number=number.or(hint.filter(|ty|matches!(ty,Type::Number|Type::Array(ArrayElement::Number(_)))));
                work.push(Work::ArrayNext{values,index:index+1,first:first.or(hint),number});
            },
        }}
        value
    }
    fn pattern_hint_result(&self,value:&Expr,hint:Type)->Option<Type>{
        match &value.kind{
            ExprKind::Index(_,index)=>match hint{
                Type::Array(element)=>{let ty=self.unions.intern([element.ty(),Type::Undefined],value.span).ok()?;Some(if !ty.is_copy(){Type::BorrowedUnion(ty.union()?)}else{ty})},
                Type::Record(id)=>Some(Self::flat_literal_depth(index).and_then(|position|self.fields[id].get(&position.to_string()).copied()).unwrap_or(Type::Undefined)),_=>None,
            },
            ExprKind::Field(_,name)=>match hint{Type::Record(id)=>Some(self.fields[id].get(name).copied().unwrap_or(Type::Undefined)),_=>None},
            _=>unreachable!("pattern hint completion"),
        }
    }
    fn join_logical_hints(&self,lhs:Type,selected:Vec<Type>,span:Span)->Option<Type>{
        let result=self.unions.intern(selected,span).ok()?;
        Some(if matches!(lhs,Type::BorrowedUnion(_)){result.union().map(Type::BorrowedUnion).unwrap_or(result)}else{result})
    }
    fn field_type_hint_result(&self,base:&Expr,name:&str,span:Span,hint:Type)->Option<Type>{
        let direct=matches!(base.kind,ExprKind::Unary{op:UnaryOp::OptionalBoundary,..});
        let optional=Self::optional_hint_boundary(base);
        let receiver=if direct{self.unions.payload(hint.owning()).unwrap_or(hint.owning())}
            else if optional && let Some(id)=hint.union(){self.unions.intern(self.unions.members(id).into_iter().filter(|ty|*ty!=Type::Undefined),base.span).ok()?}else{hint};
        let result=match receiver{
            Type::Set(_)|Type::Map(_) if name=="size"=>Some(Type::Int),
            Type::Record(id) if name=="length" && self.program.records[id].is_tuple()=>Some(Type::Int),
            Type::Record(id)=>self.fields[id].get(name).copied(),Type::Union(union)=>self.record_union_field(union,name),
            Type::Array(_)|Type::String if name=="length"=>Some(Type::Int),_=>None,
        }?;
        if optional{
            let result=self.unions.intern([result,Type::Undefined],span).ok()?;
            Some(if matches!(hint,Type::BorrowedUnion(_)) && !result.is_copy(){Type::BorrowedUnion(result.union()?)}else{result})
        }else{Some(result)}
    }
    fn index_type_hint_result(&self,base:&Expr,index:&Expr,span:Span,hint:Type)->Option<Type>{
        let optional=Self::optional_hint_boundary(base);
        let receiver=if optional && let Some(id)=hint.union(){self.unions.intern(self.unions.members(id).into_iter().filter(|ty|*ty!=Type::Undefined),base.span).ok()?}else{hint};
        let result=match receiver{
            Type::Record(id) if self.program.records[id].is_tuple()=>self.fields[id].get(&Self::flat_literal_depth(index)?.to_string()).copied(),
            Type::Array(element)=>Some(element.ty()),Type::String=>Some(Type::String),_=>None,
        }?;
        if optional{self.unions.intern([result,Type::Undefined],span).ok()}else{Some(result)}
    }
    fn join_conditional_hints(&self,left:Option<Type>,right:Option<Type>,span:Span)->Option<Type>{
        if let (Some(left), Some(right)) = (left, right)
            && (left.union().is_some()
                || right.union().is_some()
                || matches!(left, Type::Null | Type::Undefined)
                || matches!(right, Type::Null | Type::Undefined))
        {
            let joined = self
                .unions
                .intern([left.owning(), right.owning()], span)
                .ok()?;
            return Some(
                if matches!(left, Type::BorrowedUnion(_)) || matches!(right, Type::BorrowedUnion(_))
                {
                    Type::BorrowedUnion(joined.union()?)
                } else {
                    joined
                },
            );
        }
        if matches!(
            left,
            Some(Type::Number | Type::Array(ArrayElement::Number(_)))
        ) {
            left
        } else if matches!(
            right,
            Some(Type::Number | Type::Array(ArrayElement::Number(_)))
        ) {
            right
        } else {
            left.or(right)
        }
    }

}

#[cfg(test)]
mod tests{
    use super::*;
    fn int()->Expr{Expr::new(ExprKind::Integer("1".into()),Span::default())}
    fn binary(op:BinaryOp,left:Expr,right:Expr)->Expr{Expr::new(ExprKind::Binary{op,left:Box::new(left),right:Box::new(right),operator:Span::default()},Span::default())}
    #[test]
    fn mixed_structural_hints_share_one_worklist_and_query_memo(){
        let analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();let context=Context::new("main".into());
        let mut value=int();
        for layer in 0..16_384{
            value=match layer%7{
                0=>Expr::new(ExprKind::Sequence(vec![value]),Span::default()),
                1=>Expr::new(ExprKind::Unary{op:UnaryOp::NonNull,operator:Span::default(),value:Box::new(Expr::new(ExprKind::Conditional{predicate:Box::new(Expr::new(ExprKind::Boolean(true),Span::default())),consequent:Box::new(value),alternate:Box::new(Expr::new(ExprKind::Undefined,Span::default()))},Span::default()))},Span::default()),
                2=>Expr::new(ExprKind::Index(Box::new(Expr::new(ExprKind::Array(vec![value]),Span::default())),Box::new(int())),Span::default()),
                3=>binary(BinaryOp::Add,value,int()),
                4=>binary(BinaryOp::Or,Expr::new(ExprKind::Null,Span::default()),value),
                5=>binary(BinaryOp::Coalesce,Expr::new(ExprKind::Undefined,Span::default()),value),
                _=>Expr::new(ExprKind::OptionalChain(Box::new(Expr::new(ExprKind::Unary{op:UnaryOp::OptionalBoundary,value:Box::new(value),operator:Span::default()},Span::default()))),Span::default()),
            };
        }
        assert_eq!(analyzer.expression_type_hint(&context,&value),Some(Type::Int));drop(value);
    }
    #[test]
    fn lazy_hints_skip_unneeded_children_and_memos_do_not_escape_queries(){
        let analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();let mut context=Context::new("main".into());
        let id=Analyzer::bind(&mut context,&Token{text:"source".into(),kind:TokenKind::Name,span:Span::default()},Type::Int,false).unwrap();
        let name=Expr::new(ExprKind::Name("source".into()),Span::default());
        assert_eq!(analyzer.expression_type_hint(&context,&name),Some(Type::Int));
        context.bindings[id].ty=Type::String;
        assert_eq!(analyzer.expression_type_hint(&context,&name),Some(Type::String));
        // This invalid metadata must stay unobserved on a skipped hint branch.
        context.bindings[id].ty=Type::Record(usize::MAX);
        let bad=||Expr::new(ExprKind::Field(Box::new(name.clone()),"missing".into()),Span::default());
        assert_eq!(analyzer.expression_type_hint(&context,&binary(BinaryOp::Coalesce,int(),bad())),Some(Type::Int));
        assert_eq!(analyzer.expression_type_hint(&context,&binary(BinaryOp::And,Expr::new(ExprKind::Undefined,Span::default()),bad())),Some(Type::Undefined));
    }
}
