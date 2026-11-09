//! Suspended contextual access and nullable conversion retain cleanup boundaries.
use super::super::*;
pub(in crate::analysis) enum Prelude{
    Ready(Type),
    Optional{mode:Mode,expected:Option<Type>},
    Child{mode:Mode,expected:Option<Type>,frame:Frame},
}
pub(in crate::analysis) enum Frame{
    Mutation,
    ArrayUnionWiden(Type),
    ScalarUnionWiden(Type),
    Stored(super::StoredValueFrame),
    StoredInjection{stored:super::StoredValueFrame,injection:Injection},
    Projection(Projection),
    BorrowedExpected(UnionId),
    BorrowedInjection{source:UnionId,union:UnionId},
    MutableInjection(UnionId),
    Injection(Injection),
}
pub(in crate::analysis) struct Projection{
    place:crate::flow::Place,keep_start:usize,physical:Type,union:UnionId,narrowed:Type,
    actual:Type,expected:Option<Type>,mode:Mode,
}
pub(in crate::analysis) struct Injection{
    union:UnionId,member_hint:Option<Type>,borrowing_record:bool,borrowing_callback:bool,mode:Mode,
}
impl Analyzer{
    pub(super) fn finish_prelude_work(&mut self,context:&mut Context,expr:&mut Expr,frame:Frame,result:Result<Type,Diagnostic>)->Result<Type,Diagnostic>{
        let result=match frame{
            Frame::Stored(frame)=>result.and_then(|actual|self.finish_stored_value(context,expr,frame,actual)),
            Frame::StoredInjection{stored,injection}=>result.and_then(|actual|self.finish_stored_value(context,expr,stored,actual)).and_then(|actual|self.finish_injection_work(context,expr,injection,actual)),
            Frame::ScalarUnionWiden(target)=>result.and_then(|actual|{
                if !actual.union().is_some_and(|id|self.unions.members(id).iter().all(|&member|target.accepts_scalar_value(member))){return Err(Diagnostic::new("type","Every union alternative must have the destination scalar type.",expr.span));}
                expr.wrap_record_carrier(target);Ok(target)
            }),
            Frame::ArrayUnionWiden(target)=>result.and_then(|actual|self.finish_array_union_widening(expr,target,actual)),
            Frame::Mutation=>{context.mutation_place=None;result},
            Frame::Projection(frame)=>self.finish_projection_work(context,expr,frame,result),
            Frame::BorrowedExpected(id)=>result.and_then(|actual|{
                self.require(actual,Some(Type::Union(id)),expr.span)?;
                if !matches!(actual,Type::BorrowedUnion(_)|Type::Null|Type::Undefined){
                    return Err(Diagnostic::new("borrowed-view","A borrowed view branch requires another view or an absent sentinel; clone the view for an owning join.",expr.span));
                }
                if actual!=Type::BorrowedUnion(id){expr.inject(id);expr.ty=Some(Type::BorrowedUnion(id));}
                Ok(Type::BorrowedUnion(id))
            }),
            Frame::BorrowedInjection{source,union}=>result.and_then(|actual|self.finish_borrowed_injection_work(expr,source,union,actual)),
            Frame::MutableInjection(union)=>result.and_then(|actual|{self.require(actual,Some(Type::Union(union)),expr.span)?;Ok(actual)}),
            Frame::Injection(frame)=>result.and_then(|actual|self.finish_injection_work(context,expr,frame,actual)),
        };
        if result.is_ok(){expr.json_string_view=self.json_string_proof(context,expr).is_some() || self.is_string_view(context,expr);}
        result
    }
    pub(in crate::analysis) fn prepare_projection_work(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        expected: Option<Type>,
        place: crate::flow::Place,
        union: UnionId,
        narrowed: Type,
    ) -> Result<Prelude, Diagnostic> {
        let physical = self
            .place_type(context, &place)
            .unwrap_or(Type::Union(union));
        // Same-tag transport and general residual observations keep the original
        // enum. Common string observations can use the existing borrowed subset
        // retagging without moving a payload or admitting an absent alternative.
        let actual = if expected == Some(physical)
            // An argument probe without a contract borrows the declared enum;
            // selecting a narrower enum would require replacing its tag.
            || (mode == Mode::Mutate && expected.is_none()
                && matches!(physical,Type::Union(_)) && matches!(narrowed,Type::Union(_)))
            || (mode == Mode::Mutate && expected.is_some_and(|expected| self.scoped_record_nullable_contract(physical,expected)))
            || context.observer_places.contains(&place)
            || (mode == Mode::Read
                && matches!(narrowed, Type::Union(id) if !id.copy)
                && expected.is_none()
                && !self.is_string_observation(narrowed))
        {
            physical
        } else {
            narrowed
        };
        if actual != physical
            && let Some(fact) = context.flow.get(&place).cloned()
        {
            for frame in &mut context.loops {
                if frame
                    .flow_entry
                    .get(&place)
                    .is_some_and(|entry| entry.origin == fact.origin)
                {
                    frame.flow_required.insert(place.clone(), fact.clone());
                }
            }
        }
        let retag = matches!(actual, Type::Union(id) if id != union && !id.copy);
        let injecting_owned =
            matches!(expected, Some(Type::Union(id)) if !id.copy && actual != physical);
        let access = if matches!(physical, Type::BorrowedUnion(_))
            && mode == Mode::Consume
            && !actual.is_copy()
        {
            return Err(Diagnostic::new(
                "borrowed-view",
                "A borrowed optional payload cannot be transferred; use clone() to own it.",
                expr.span,
            ));
        } else if actual.is_copy() && !union.copy && mode != Mode::Mutate {
            // Copying an absent variant does not transfer the owning enum.
            Mode::Read
        } else if (retag || injecting_owned) && !matches!(mode, Mode::Mutate | Mode::Read) {
            Mode::Consume
        } else {
            mode
        };
        if mode == Mode::Mutate && actual != physical && matches!(actual, Type::Union(id) if id != union) {
            #[cfg(test)]
            {
                eprintln!("MUTABLE_TAG_REJECT physical={physical:?} actual={actual:?} expected={expected:?}");
                for ty in [Some(physical),Some(actual),expected].into_iter().flatten(){
                    let members=ty.union().map(|id|self.unions.members(id)).unwrap_or_else(||vec![ty]);
                    eprintln!("MUTABLE_TAG_MEMBERS type={ty:?} members={members:?}");
                    for member in members{if let Type::Record(record)=member{eprintln!("MUTABLE_TAG_RECORD id={record} declaration={} parameter={} owned_parameter={} fields={:?}",self.scoped_records.declaration(record),self.scoped_records.parameter_contract(record),self.owned_record_parameter(member),self.program.records[record].fields);}}
                }
            }
            return Err(Diagnostic::new(
                "unsupported",
                "Replacing a narrowed tag requires assignment to its declared nullable storage.",
                expr.span,
            ));
        }
        context.suppressed_flow.insert(place.clone());
        let keep_start = context.keep_flow.len();
        if access == Mode::Mutate {
            context.keep_flow.push(place.clone());
        }
        Ok(Prelude::Child { mode: access, expected: None, frame: Frame::Projection(Projection {
            place, keep_start, physical, union, narrowed, actual, expected, mode,
        }) })
    }
    fn finish_projection_work(&mut self,context:&mut Context,expr:&mut Expr,frame:Projection,checked:Result<Type,Diagnostic>)->Result<Type,Diagnostic>{
        let Projection{place,keep_start,physical,union,narrowed,actual,expected,mode}=frame;
        context.keep_flow.truncate(keep_start);
        context.suppressed_flow.remove(&place);
        let checked = checked?;
        self.require(checked, Some(physical), expr.span)?;
        expr.project(union, narrowed, actual);
        if let Some(Type::Union(target)) = expected
            && actual != Type::Union(target)
        {
            if let Some(concrete) = self.scoped_record_injection(actual, target, expr.span)? {
                return self.finish_record_carrier_injection(expr,actual,concrete,mode);
            }
            self.require(actual, expected, expr.span)?;
            expr.inject(target);
            return Ok(Type::Union(target));
        }
        Ok(actual)
    }
    pub(in crate::analysis) fn prepare_injection_work(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        _loans: &Loans,
        union: UnionId,
    ) -> Result<Prelude, Diagnostic> {
        let expected = Type::Union(union);
        let hint = self.expression_type_hint(context, expr);
        if let Some(Type::BorrowedUnion(source)) = hint {
            if mode != Mode::Read {
                return Err(Diagnostic::new(
                    "borrowed-view",
                    "A borrowed optional element requires a readonly consumer; use clone() to own its value.",
                    expr.span,
                ));
            }
            return Ok(Prelude::Child{mode:Mode::Read,expected:None,frame:Frame::BorrowedInjection{source,union}});
        }
        if matches!(expr.kind, ExprKind::OptionalChain(_)) {
            // The present branch receives payload context inside optional_chain;
            // the complete expression still satisfies the nullable expectation.
            return Ok(Prelude::Optional{mode,expected:Some(expected)});
        }
        let member_hint = if hint.is_some_and(|ty| {
            matches!(
                ty,
                Type::Null | Type::Undefined | Type::Union(_) | Type::BorrowedUnion(_)
            )
        }) {
            hint
        } else {
            self.unions.payload(expected)
        };
        let member_hint = member_hint.filter(|ty| !matches!(ty, Type::Union(_)));
        // A named record supplies its own physical fields. Check its nominal
        // contract here; the injection completion joins/retargets that carrier.
        let member_hint = match (hint,member_hint) {
            (Some(Type::Record(actual)),Some(Type::Record(target)))
                if actual!=target
                    && self.scoped_records.declaration(actual)==self.scoped_records.declaration(target)
                    && self.physical_record_result_needed(Type::Record(target)) =>
                Some(Type::Record(self.scoped_records.declaration(target))),
            _=>member_hint,
        };
        let member_hint = match &expr.kind {
            ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(name) if name=="@native:std:ui-runtime#reactive") => Some(Type::Record(self.ui_reactive_prop_type(Some(expected),expr.span)?.0)),
            ExprKind::Record(fields) if member_hint.is_none() => {
                self.record_alternative(union, fields, expr.span)?
            }
            _ => member_hint,
        };
        // An integer literal is a number where the union has number and no int.
        let member_hint = if member_hint.is_none() && matches!(expr.kind, ExprKind::Integer(_)) {
            let members = self.unions.members(union);
            (members.contains(&Type::Number) && !members.contains(&Type::Int)).then_some(Type::Number)
        } else {
            member_hint
        };
        if mode == Mode::Mutate
            && self
                .unions
                .payload(expected)
                .is_some_and(|ty| matches!(ty, Type::Callback(_)))
            && hint.is_some_and(|ty| ty.union().is_some())
        {
            // A mutable callback borrow retains the existing tag and environment.
            // The concrete call specialization uses that physical tag identity;
            // abstract template bodies are never emitted as native operations.
            return Ok(Prelude::Child{mode,expected:None,frame:Frame::MutableInjection(union)});
        }
        let injecting_owned = hint != Some(expected) && !expected.is_copy();
        if injecting_owned && mode == Mode::Mutate {
            return Err(Diagnostic::new(
                "type",
                "A nullable mutable parameter requires existing tagged storage.",
                expr.span,
            ));
        }
        let borrowing_record = mode == Mode::Read
            && (self.nullable_scoped_record_declaration(expected).is_some()
                || self.unions.members(union).iter().any(|ty|matches!(ty,Type::Record(id) if self.aggregate_callable_facts(Type::Record(*id)).retains)));
        let borrowing_callback=mode==Mode::Read && self.unions.payload(expected).is_some_and(|ty|matches!(ty,Type::Callback(id) if self.callables.callbacks[id].kind==CallbackKind::Read || hint.is_some_and(|ty|ty.union().is_some())));
        let frame=Injection{union,member_hint,borrowing_record,borrowing_callback,mode};
        if let Some(Type::StoredCallable(family))=member_hint {
            let stored=self.prepare_stored_value(context,expr,family)?;
            return Ok(Prelude::Child{mode:Mode::Consume,expected:stored.expected,frame:Frame::StoredInjection{stored,injection:frame}});
        }
        Ok(Prelude::Child{
            mode:if injecting_owned && !borrowing_callback && !borrowing_record{Mode::Consume}else{mode},
            expected:member_hint,frame:Frame::Injection(frame),
        })
    }
    fn finish_injection_work(&mut self,context:&mut Context,expr:&mut Expr,frame:Injection,mut actual:Type)->Result<Type,Diagnostic>{
        let Injection{union,member_hint,borrowing_record,borrowing_callback,mode}=frame;
        let expected=Type::Union(union);
        if borrowing_record && matches!(actual, Type::Null | Type::Undefined)
            && let Some(concrete) = self.absent_nullable_record_annotation(context, Some(expected), expr)?
        {
            let concrete = concrete.union().expect("absent nullable record view");
            expr.inject(concrete);
            expr.ty = Some(Type::BorrowedUnion(concrete));
            return Ok(Type::BorrowedUnion(concrete));
        }
        if let Some(concrete) = self.native_view_storage_injection(actual, union, expr.span)? {
            expr.inject(concrete.union().expect("nullable scoped native storage"));
            return Ok(concrete);
        }
        if let Some(concrete) = self.scoped_record_injection(actual, union, expr.span)? {
            return self.finish_record_carrier_injection(expr,actual,concrete,mode);
        }
        if let Some(Type::Callback(callback)) = member_hint {
            self.require(actual, Some(Type::Callback(callback)), expr.span)?;
            if self.callables.callbacks[callback].kind.is_once()
                && let Type::Closure(origin) = actual
                && self.closures[origin].invocation != Mode::Consume
            {
                let wrapper = self.once_return(origin, context, expr.span);
                expr.wrap_once(wrapper);
                actual = Type::Closure(wrapper);
            }
        } else {
            self.require(actual, Some(expected), expr.span)?;
        }
        if actual == expected {
            return Ok(actual);
        }
        if context.returning_closure
            && matches!(self.unions.payload(expected),Some(Type::Callback(id)) if self.callables.callbacks[id].kind.is_owned())
            && let Some(payload) = self.unions.payload(actual)
            && (matches!(payload, Type::Closure(_)) || self.absent_callable_type(actual))
        {
            let concrete = self.unions.intern(
                self.unions.members(union).into_iter().map(|member| {
                    if matches!(member, Type::Callback(_)) {
                        payload
                    } else {
                        member
                    }
                }),
                expr.span,
            )?;
            if actual != concrete {
                expr.inject(concrete.union().expect("nullable result"));
            }
            return Ok(concrete);
        }
        if context.returning_closure
            && matches!(actual, Type::Null | Type::Undefined)
            && let Some(Type::Callback(id)) = self.unions.payload(expected)
            && self.callables.callbacks[id].kind.is_owned()
        {
            let empty = self.absent_callback(id);
            let concrete = self.unions.intern(
                self.unions.members(union).into_iter().map(|member| {
                    if matches!(member, Type::Callback(_)) {
                        empty
                    } else {
                        member
                    }
                }),
                expr.span,
            )?;
            expr.inject(concrete.union().expect("nullable absent result"));
            return Ok(concrete);
        }
        if context.returning_closure
            && matches!(self.unions.payload(expected),Some(Type::Callback(id)) if self.callables.callbacks[id].kind.is_owned())
            && let Type::Closure(_) = actual
        {
            let concrete = self.unions.intern(
                self.unions.members(union).into_iter().map(|member| {
                    if matches!(member, Type::Callback(_)) {
                        actual
                    } else {
                        member
                    }
                }),
                expr.span,
            )?;
            expr.inject(concrete.union().expect("nullable result"));
            return Ok(concrete);
        }
        expr.inject(union);
        if borrowing_callback || borrowing_record {
            expr.ty = Some(Type::BorrowedUnion(union));
            return Ok(Type::BorrowedUnion(union));
        }
        Ok(expected)
    }
    /// Borrowing preserves each selected physical record. Only an owning
    /// injection may move it into a wider finite environment carrier.
    fn finish_record_carrier_injection(&mut self,expr:&mut Expr,actual:Type,concrete:Type,mode:Mode)->Result<Type,Diagnostic>{
        let union=concrete.union().expect("record carrier injection");
        if matches!(mode,Mode::Read|Mode::Mutate){
            let source=actual.union().map(|id|self.unions.members(id)).unwrap_or_else(||vec![actual]);
            let mut members=Vec::new();
            for member in self.unions.members(union){
                if let Type::Record(record)=member{
                    let declaration=self.scoped_records.declaration(record);
                    let selected=source.iter().copied().filter(|ty|matches!(ty,Type::Record(id) if self.scoped_records.declaration(*id)==declaration)).collect::<Vec<_>>();
                    if !selected.is_empty(){members.extend(selected);continue;}
                }
                members.push(member);
            }
            let view=self.unions.intern(members,expr.span)?;
            if mode==Mode::Mutate && view!=concrete{return Err(Diagnostic::new("type","A nullable mutable parameter requires existing tagged storage.",expr.span));}
            let target=view.union().expect("record view injection");
            expr.inject(target);
            let result=if mode==Mode::Read{Type::BorrowedUnion(target)}else{view};expr.ty=Some(result);
            return Ok(result);
        }
        if actual.union().is_some(){self.retarget_nullable_store(expr,concrete)?;}
        else{
            if let Type::Record(record)=actual{
                let targets=self.unions.members(union).into_iter().filter(|ty|matches!(ty,Type::Record(id) if self.scoped_records.declaration(*id)==self.scoped_records.declaration(record))).collect::<Vec<_>>();
                let [target]=targets.as_slice() else{return Err(Diagnostic::new("type","Record injection requires one matching nominal alternative.",expr.span));};
                if *target!=actual{self.retarget_nullable_store(expr,*target)?;}
            }
            expr.inject(union);
        }
        Ok(concrete)
    }
    fn finish_borrowed_injection_work(&mut self,expr:&mut Expr,source:UnionId,union:UnionId,actual:Type)->Result<Type,Diagnostic>{
        let expected=Type::Union(union);

            if let Some(concrete) = self.scoped_record_injection(actual, union, expr.span)? {
                return self.finish_record_carrier_injection(expr,actual,concrete,Mode::Read);
            }
            self.require(actual, Some(expected), expr.span)?;
            if source != union {
                expr.inject(union);
                expr.ty = Some(Type::BorrowedUnion(union));
                return Ok(Type::BorrowedUnion(union));
            }
            return Ok(actual);
    }
}

#[cfg(test)]
mod injection_context_tests {
 use super::*;
 #[test]
 fn named_nullable_record_replacements_keep_their_actual_tagged_carrier(){
  let source=crate::format("type Holder={pull:Owned<Function<int>>;};function main(){let current:Holder|undefined=undefined;const inner:Holder={pull:():int=>7};current=inner;if(current!==undefined){console.log(current.pull());}}").unwrap();assert_eq!(crate::format(&source).unwrap(),source);
  let checked=crate::check(&source).unwrap();let mut stores=0;
  for function in &checked.program.functions{crate::fixed_storage::visit_function_expressions(function,&mut |value|{if matches!(&value.kind,ExprKind::Inject{value,..} if matches!(&value.kind,ExprKind::Name(name) if name=="inner")){assert!(matches!(value.ty,Some(Type::Union(_))));stores+=1;}});}assert_eq!(stores,1);let emitted=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||checked.emit_rust()));assert!(emitted.is_ok(),"records={:#?}\nunions={:#?}\nfunctions={:#?}",checked.program.records,checked.program.unions,checked.program.functions);
 }
}

#[cfg(test)]
#[test]
fn mutable_mixed_union_default_keeps_declared_tagged_storage(){
 let source="type Reactive<T>={evaluate:Owned<Function<T>>;};type Value<T>=Reactive<T>|T;function snapshot(value:Mutable<Value<string|undefined>|undefined>):string{if(value===undefined){return ''; }if(typeof value==='string'){return clone(value);}return value.evaluate()??'';}function invoke(value:Value<string|undefined>|undefined=''):string{return snapshot(value);}function main(){console.log(invoke());}";
 let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);let checked=crate::check(&source).unwrap();drop(checked.emit_rust());
}

#[cfg(test)]
#[test]
fn mutable_mixed_union_probes_preserve_readonly_and_declared_tag_limits(){
 for source in [
  "type Row={text:string;};function replace(value:Mutable<Row|string>):void{value='changed';}function inspect(value:Mutable<Row|string|undefined>):void{if(value!==undefined){replace(value);}}function main(){}",
  "type Row={text:string;};function replace(value:Mutable<Row|string|undefined>):void{value=undefined;}function inspect(value:ReadOnly<Row|string|undefined>):void{if(value!==undefined){replace(value);}}function main(){}",
 ]{let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);assert!(crate::check(&source).is_err(),"Mutable argument probes cannot retag or acquire readonly storage: {source}");}
}
