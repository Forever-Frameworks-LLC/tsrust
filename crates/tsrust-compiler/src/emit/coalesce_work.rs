//! Coalescing operand modes and branch output without recursive emission.
use super::*;

pub(super) struct Plan {
    pub(super) absent_only: bool,
    pub(super) undefined_only: bool,
    pub(super) left_mode: Mode,
    pub(super) right_mode: Mode,
    logical_union: Option<UnionId>,
    union: Option<UnionId>,
    result: Type,
    borrowed: bool,
    target_view: bool,
}

impl Emitter<'_> {
    /// A required field of the compiler's empty callback enum makes the whole
    /// ordinary record uninhabited. Optional/array/opaque fields do not prove it.
    pub(super) fn absent_record_field(&self,ty:Type)->Option<usize>{
        let Type::Record(record)=ty else{return None;};let record=&self.program.records[record];
        if record.native_type.is_some(){return None;}
        record.fields.iter().position(|field|!field.optional && matches!(field.ty,Some(Type::Callback(id))if self.program.absent_callback_payloads.contains(&id)))
    }

    fn coalesce_physical_record_tag(&self,target:UnionId,actual:Type)->Option<usize>{
        let Type::Record(source)=actual else{return None;};
        let mut matches=self.program.unions[target.index()].members.iter().enumerate().filter_map(|(tag,ty)|{
            let Type::Record(destination)=*ty else{return None;};
            (self.native_record_declaration(source)==self.native_record_declaration(destination)
                && self.native_type(actual)==self.native_type(*ty)
                && self.union_stored_type(target,actual,self.native_type(actual))==self.union_stored_type(target,*ty,self.native_type(*ty))).then_some(tag)
        });
        let tag=matches.next()?;if matches.next().is_some(){return None;}Some(tag)
    }

    pub(super) fn prepare_coalesce_emission<'a>(&self, expr: &Expr, left: &'a Expr, mode: Mode) -> (Plan, &'a Expr) {
        let undefined_only=matches!(expr.kind,ExprKind::Binary{op:BinaryOp::PatternDefault,..});
        let absent_only = left.ty.is_some_and(|ty| {
            (ty==Type::Undefined || (!undefined_only && ty==Type::Null))
                || ty.union().is_some_and(|id| {
                    self.program.unions[id.index()]
                        .members
                        .iter()
                        .all(|ty| (*ty==Type::Undefined || (!undefined_only && *ty==Type::Null)) || matches!(ty,Type::Callback(id) if self.program.absent_callback_payloads.contains(id)))
                })
        });
        if absent_only {
            return (Plan { absent_only, undefined_only, left_mode: Mode::Copy, right_mode: mode,
                logical_union: None, union: None, result: expr.ty.unwrap(), borrowed: false, target_view: false }, left);
        }
        let logical_union = left.ty.and_then(Type::union).expect("dynamic nullable lhs");
        // Match the physical tag directly: retagging an owning narrowed value
        // first would obscure the absent arm's ability to borrow its source.
        let (left, union) = if let ExprKind::Project { union, value, .. } = &left.kind {
            (value.as_ref(), *union)
        } else {
            (left, logical_union)
        };
        let result = expr.ty.expect("coalescing result");
        let borrowed =
            matches!(left.ty, Some(Type::BorrowedUnion(_))) || (!union.copy && mode == Mode::Read);
        let access = if borrowed {
            Mode::Read
        } else if union.copy {
            Mode::Copy
        } else {
            mode
        };
        let target_view = matches!(result, Type::BorrowedUnion(_))
            || (result.union().is_some_and(|id| !id.copy) && mode == Mode::Read);
        (Plan { absent_only, undefined_only, left_mode: access, right_mode: if target_view { Mode::Read } else { mode },
            logical_union: Some(logical_union), union: Some(union), result, borrowed, target_view }, left)
    }

    pub(super) fn finish_coalesce_emission(&self, plan: Plan, right: &Expr, source: String, fallback: String) -> String {
        if plan.absent_only { return format!("{{ let _nullish_left={source}; {fallback} }}"); }
        let Plan { undefined_only, left_mode: access, logical_union, union, result, borrowed, target_view, .. } = plan;
        let logical_union = logical_union.expect("dynamic logical union");
        let union = union.expect("dynamic physical union");
        let fallback = if let Some(target) = result.union() {
            let actual = right.ty.expect("coalescing fallback");
            let prefix = if target_view { "ViewUnion" } else { "Union" };
            if let Some(field)=self.absent_record_field(actual){
                format!("match ({fallback}).field{field} {{}}")
            }else if let Some(from) = actual.union() {
                if from == target {
                    fallback
                } else {
                    let source_view =
                        matches!(actual, Type::BorrowedUnion(_)) || (!from.copy && target_view);
                    let arms = self.program.unions[from.index()]
                        .members
                        .iter()
                        .enumerate()
                        .map(|(index, member)| {
                            if let Some(field)=self.absent_record_field(*member){
                                return format!("{}{}::V{index}(payload)=>match payload.field{field} {{}}",if source_view{"ViewUnion"}else{"Union"},from.index());
                            }
                            let to = self.program.unions[target.index()]
                                .members
                                .iter()
                                .position(|ty| {
                                    ty.accepts_scalar_value(*member)
                                        || matches!(ty, Type::Callback(_))
                                            && matches!(
                                                member,
                                                Type::Closure(_) | Type::Callback(_)
                                            )
                                })
                                .or_else(||self.coalesce_physical_record_tag(target,*member))
                                .unwrap_or_else(||{
                                    #[cfg(test)]
                                    eprintln!("COALESCE_FALLBACK_REJECT span={:?} source={actual:?} target={result:?} member={member:?} source_members={:?} target_members={:?}",right.span,self.program.unions[from.index()].members,self.program.unions[target.index()].members);
                                    #[cfg(test)]
                                    for ty in std::iter::once(member).chain(self.program.unions[target.index()].members.iter()){if let Type::Record(id)=*ty{eprintln!("COALESCE_RECORD id={id} native={} declaration={} fields={:?}",self.native_type(*ty),self.native_record_declaration(id),self.program.records[id].fields.iter().map(|field|(field.name.text.as_str(),field.ty)).collect::<Vec<_>>());}}
                                    panic!("fallback subset")
                                });
                            format!(
                                "{}{}::V{index}(payload)=>{prefix}{}::V{to}({})",
                                if source_view { "ViewUnion" } else { "Union" },
                                from.index(),target.index(),if target_view{self.data_view_payload(*member,"payload",source_view)}else if !source_view{self.union_transfer_payload(from,target,*member,"payload")}else{"payload".into()}
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    // A scalar constructor leaves the other physical payloads
                    // unconstrained, even when their arms are uninhabited.
                    let physical = self.native_argument_type(ArgumentContract {
                        ty: actual, mode: if source_view { Mode::Read } else { Mode::Consume },
                        omission: crate::ParameterOmission::Required, rest: false,
                    }, "");
                    format!("match std::convert::identity::<{physical}>({fallback}) {{{arms}}}")
                }
            } else {
                let index = self.program.unions[target.index()]
                    .members
                    .iter()
                    .position(|ty| ty.accepts_scalar_value(actual))
                    .or_else(||self.coalesce_physical_record_tag(target,actual))
                    .expect("fallback member");
                let code = if target_view && actual.is_copy() {
                    format!("&({fallback})")
                } else {
                    fallback
                };
                format!("{prefix}{}::V{index}({})", target.index(),if target_view{code}else{self.union_store_payload(target,actual,&code)})
            }
        } else {
            fallback
        };
        let arms = self.program.unions[union.index()]
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let selected = if !self.program.unions[logical_union.index()]
                    .members
                    .contains(member)
                {
                    "unreachable!(\"checked nullable guard\")".to_owned()
                } else if let Some(field)=self.absent_record_field(*member){
                    format!("match payload.field{field} {{}}")
                } else if *member==Type::Undefined || (!undefined_only && *member==Type::Null) {
                    fallback.clone()
                } else if let Some(target) = result.union() {
                    let to = self.program.unions[target.index()]
                        .members
                        .iter()
                        .position(|ty| {
                            ty.accepts_scalar_value(*member)
                                || matches!(ty, Type::Callback(_))
                                    && matches!(member, Type::Closure(_) | Type::Callback(_))
                        })
                        .or_else(||self.coalesce_physical_record_tag(target,*member))
                        .expect("retained payload variant");
                    let payload = if target_view { self.data_view_payload(*member,"payload",borrowed) } else if borrowed && target.copy { "*payload".to_owned() } else { self.union_transfer_payload(union,target,*member,"payload") };
                    format!("{}{}::V{to}({payload})",if target_view { "ViewUnion" } else { "Union" },target.index())
                } else if !borrowed && access==Mode::Consume {
                    self.union_move_payload(union,*member,"payload")
                } else if access==Mode::Mutate && self.boxed_union_member(union,*member) {
                    "&mut **payload".to_owned()
                } else { "payload".to_owned() };
                format!(
                    "{}{}::V{index}(payload)=>{selected}",
                    if borrowed { "ViewUnion" } else { "Union" },
                    union.index()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("match {source} {{{arms}}}")
    }

    pub(super) fn prepare_pattern_fallback_work(&self, right: &Expr, mode: Mode) -> (Mode, Option<usize>) {
        if mode == Mode::Read && Self::temporary_storage(right) && !right.ty.unwrap().is_copy()
            && !matches!(right.ty, Some(Type::BorrowedUnion(_))) {
            let slot = self.conditional_next.get(); self.conditional_next.set(slot + 1);
            let annotation = if right.ty.unwrap().union().is_some() { format!(": {}", self.native_type(right.ty.unwrap())) } else { String::new() };
            self.conditional_temporaries.borrow_mut().push(format!("let pattern_default_owner_{slot}{annotation};"));
            (Mode::Consume, Some(slot))
        } else { (mode, None) }
    }

    pub(super) fn finish_pattern_fallback_work(&self, right: &Expr, slot: Option<usize>, owner: String) -> String {
        let Some(slot) = slot else { return owner; };
        let view = if right.ty.unwrap().union().is_some() {
            format!("pattern_default_owner_{slot}.{}()", if self.callback_payload(right.ty.unwrap()).is_some() { "tsr_view" } else { "tsr_data_view" })
        } else { format!("&pattern_default_owner_{slot}") };
        format!("{{pattern_default_owner_{slot}={owner};{view}}}")
    }
}

#[cfg(test)]
#[test]
fn coalescing_literal_subset_preserves_borrowed_and_owned_scalar_payloads(){
 let source=include_str!("../../tests/fixtures/coalesce-literal-subsets/main.tsr");let formatted=crate::format(source).unwrap();assert_eq!(crate::format(&formatted).unwrap(),formatted);let checked=crate::check(&formatted).unwrap_or_else(|error|panic!("{}",error.render("coalesce-literal-subsets.tsr",&formatted)));drop(checked.emit_rust());
}

#[cfg(test)]
#[test]
fn coalescing_empty_callback_fallback_has_its_checked_physical_union_context(){
 let source="interface Reactive<T>{evaluate:Owned<Function<T>>;}type Value=Reactive<string>|string;function observe(value:ReadOnly<Value>):void{if(typeof value==='string'){console.log(value);}else{console.log('reactive');}}function main(){const value:Value|undefined=undefined;observe(value??'');}";
 let formatted=crate::format(source).unwrap();assert_eq!(crate::format(&formatted).unwrap(),formatted);
 let checked=crate::check(&formatted).unwrap();let emitter=super::emitter_for(&checked.program,"");
 let find=|absent:bool|checked.program.unions.iter().enumerate().find_map(|(index,union)|{
  (union.members.len()==2 && union.members.contains(&Type::String) && union.members.iter().any(|member|matches!(member,Type::Record(_))) && union.members.iter().any(|member|emitter.absent_record_field(*member).is_some())==absent).then_some(UnionId{index:index.try_into().unwrap(),copy:false,scalar_members:false})
 }).unwrap();
 let from=find(true);let target=find(false);assert_ne!(from,target);
 let tag=checked.program.unions[from.index()].members.iter().position(|member|*member==Type::String).unwrap();
 let mut right=Expr::new(ExprKind::Name("fallback".into()),crate::Span::default());right.ty=Some(Type::Union(from));
 for view in [false,true]{
  let mode=if view{Mode::Read}else{Mode::Consume};
  let physical=emitter.native_argument_type(ArgumentContract{ty:Type::Union(from),mode,omission:crate::ParameterOmission::Required,rest:false},"");
  let plan=Plan{absent_only:false,undefined_only:false,left_mode:mode,right_mode:mode,logical_union:Some(from),union:Some(from),result:Type::Union(target),borrowed:view,target_view:view};
  let code=emitter.finish_coalesce_emission(plan,&right,"left".into(),format!("{}{}::V{tag}(scalar)",if view{"ViewUnion"}else{"Union"},from.index()));
  assert!(code.contains(&format!("match std::convert::identity::<{physical}>(")),"{code}");assert_eq!(code.matches("(scalar)").count(),1);assert!(code.contains("=>match payload.field0 {}"));
 }
}
