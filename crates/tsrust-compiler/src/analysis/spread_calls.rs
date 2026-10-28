//! Dense call spread keeps source effects separate from selected native slots.
use super::*;

#[derive(Clone)]
pub(super) struct SpreadCallableContract {
    pub parameters: Vec<ArgumentContract>,
    pub invocation: Mode,
}

impl Analyzer {
    pub(super) fn native_spread_source(&self, ty:Type)->bool {
        matches!(ty,Type::Record(id) if matches!(self.program.records[id].native_type,Some(crate::native_signatures::NativeTypeId::Deque | crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet)))
    }
    pub(super) fn immutable_spread_keys(&self, ty:Type)->bool {
        matches!(ty,Type::Set(_) | Type::Map(_)) || matches!(ty,Type::Record(id) if matches!(self.program.records[id].native_type,Some(crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet)))
    }

    #[inline(never)]
    pub(super) fn array_spread_context(&mut self,value:&Expr,guessed:Option<Type>,element:Option<Type>)->Result<Option<Type>,Diagnostic>{
        if matches!(value.kind,ExprKind::SetConstructor(_,Some(_))|ExprKind::MapConstructor(_,Some(_))){return Ok(None);}
        let Some(element)=element else{return Ok(None);};
        // Spread converts selected elements, not the borrowed source array's storage.
        if matches!(element,Type::Union(_)) && matches!(guessed,Some(Type::Array(_))) && !matches!(value.kind,ExprKind::Array(_)) {return Ok(guessed);}

        let family=match &value.kind {ExprKind::SetConstructor(_,None)=>Some("Set"),ExprKind::MapConstructor(_,None)=>Some("Map"),_=>None};
        if let Some(family)=family {
            let arguments=if family=="Set" {vec![element]}else if let Type::Record(id)=element {
                let fields=&self.program.records[id].fields;
                if fields.len()!=2{return Ok(None);}
                vec![self.fields[id][&fields[0].name.text],self.fields[id][&fields[1].name.text]]
            }else{return Ok(None);};
            let types=crate::generic_inference::Types{ callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
            let mut annotation=crate::type_syntax::ValueAnnotation{span:value.span,kind:crate::type_syntax::ValueAnnotationKind::Applied{name:crate::type_syntax::TypeName{text:family.into(),span:value.span},arguments:arguments.into_iter().map(|ty|types.annotation(ty,value.span)).collect::<Result<Vec<_>,_>>()?}};
            self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
            append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
            return self.resolve_value_annotation(&annotation,false).map(Some);
        }
        if matches!(guessed,Some(Type::String|Type::Set(_)|Type::Map(_))) || guessed.is_some_and(|ty|self.native_spread_source(ty)){Ok(None)}else{self.constructor_array_type(element,value.span).map(Some)}
    }

    #[inline(never)]
    pub(super) fn spread_source_type_hint(&mut self,context:&Context,value:&Expr)->Result<Option<Type>,Diagnostic>{
        if let Some(ty)=self.expression_type_hint(context,value){return Ok(Some(ty));}
        let annotation=match &value.kind {ExprKind::SetConstructor(_,Some(annotation))|ExprKind::MapConstructor(_,Some(annotation))=>annotation,_=>return Ok(None)};
        let mut annotation=annotation.as_ref().clone();
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        self.resolve_value_annotation(&annotation,false).map(Some)
    }

    /// Generic preparation sees element slots, not an iterable as one argument.
    /// This pass infers types only; ordinary spread checking owns source effects.
    #[inline(never)]
    pub(super) fn generic_spread_prototype(&mut self,context:&Context,expr:&Expr)->Result<Option<Box<Expr>>,Diagnostic>{
        let ExprKind::Call(callee,args)=&expr.kind else{return Ok(None);};
        let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
        if context.names.contains_key(name)||context.pending_names.contains(name)||self.generic_functions.template(name).is_none(){return Ok(None);}
        let mut normalized=Vec::new();
        for arg in args {
            if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind {
                let source=self.spread_source_type_hint(context,value)?;
                let element=source.and_then(|ty|self.iterable_element(ty)).map(ArrayElement::ty);
                let count=if let ExprKind::Array(items)=&value.kind {if items.iter().any(|item|matches!(item.kind,ExprKind::Unary{op:UnaryOp::Spread,..})){1}else{items.len()}}else{1};
                for _ in 0..count {
                    let mut slot=self.spread_selected(context,value,expr.span.start,SpreadSelection::Argument(normalized.len()),element.unwrap_or(Type::Undefined),Mode::Consume);
                    slot.ty=element;normalized.push(slot);
                }
            }else{normalized.push(arg.clone());}
        }
        Ok(Some(Box::new(Expr::new(ExprKind::Call(callee.clone(),normalized),expr.span))))
    }

    pub(super) fn spread_selected(&self, context: &Context, source: &Expr, slot: usize, selection: SpreadSelection, ty: Type, access: Mode) -> Expr {
        let place=crate::flow::place(source,&context.names);
        let mut value=Expr::new(ExprKind::SpreadValue(Box::new(SpreadSelected {
            normalized_union:None,literal_integer:None, stable:matches!(source.kind,ExprKind::Name(_)|ExprKind::Selected{..}|ExprKind::Index(_,_)|ExprKind::Field(_,_)|ExprKind::Project{..}) || matches!(source.kind,ExprKind::Unary{op:UnaryOp::NonNull,..}) && Self::named_storage(source),
            carrier:matches!(selection,SpreadSelection::Argument(_)), slot, selection, source:Box::new(SelectedSource {
                readonly: access==Mode::Read && !ty.is_copy(),
                roots:Self::loan_roots(source,access),
                place:place.map(|place|(place.root,place.fields)), span:source.span,
            }), access, owned:access==Mode::Consume || ty.is_copy(),
        })),source.span);
        value.ty=Some(ty); value.binding=source.binding; value
    }

    pub(super) fn spread_argument(&self,context:&Context,segments:&[SpreadSegment],slot:usize,position:usize,ty:Type,normalized:Option<UnionId>,minimum:usize,maximum:Option<usize>)->Expr {
        let candidates=segments.iter().enumerate().filter(|(index,segment)| segment.length!=Some(0) && {let(lower,upper)=spread_position_range(segments,*index,minimum,maximum);position>=lower&&upper.is_none_or(|upper|position<=upper)}).map(|(_,segment)|segment).collect::<Vec<_>>();
        let access=candidates.iter().map(|segment|segment.access).max().expect("reachable spread argument");
        let mut argument=self.spread_selected(context,&candidates[0].value,slot,SpreadSelection::Argument(position),ty,access);
        if let ExprKind::SpreadValue(selected)=&mut argument.kind {
            selected.literal_integer=spread_literal_integer(segments,position);selected.normalized_union=normalized;
            selected.source.roots=candidates.iter().flat_map(|segment|Self::loan_roots(&segment.value,segment.access)).collect();
            selected.source.roots.sort_unstable();selected.source.roots.dedup();
            if candidates.iter().any(|segment|crate::flow::place(&segment.value,&context.names).map(|place|(place.root,place.fields))!=selected.source.place){selected.source.place=None;}
        }
        argument
    }

    pub(super) fn spread_value_expression(
        &mut self,
        context: &mut Context,
        expression: &mut Expr,
        mode: Mode,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::SpreadValue(selected) = &expression.kind else { unreachable!() };
        let ty = expression.ty.expect("checked spread slot");
        if selected.source.readonly && mode == Mode::Mutate {
            return Err(Diagnostic::new("borrowed-place", "A shared spread selection cannot be mutated.", expression.span));
        }
        if !selected.owned && mode == Mode::Consume && !ty.is_copy() {
            return Err(Diagnostic::new("borrowed-place", "A borrowed spread element cannot be transferred; use an explicit owned copy.", expression.span));
        }
        // Source evaluation/access was checked once when staging this plan.
        // Synthetic selected slots never reevaluate a source or acquire a new
        // permission from their native reference representation.
        let ty=self.rest_callable_adapter(context,expression,ty,expected,mode)?;
        self.require(ty, expected, expression.span)?;
        if let Some(Type::Union(id))=expected && ty!=Type::Union(id) && ty!=Type::BorrowedUnion(id) {
            expression.inject(id);
            let injected=if !id.copy && mode==Mode::Read {Type::BorrowedUnion(id)}else{Type::Union(id)};
            expression.ty=Some(injected);return Ok(injected);
        }
        Ok(ty)
    }

    pub(super) fn ordinary_spread_contract(
        &mut self,
        context: &Context,
        callee: &Expr,
    ) -> Result<Option<SpreadCallableContract>, Diagnostic> {
        if let ExprKind::Name(name) = &callee.kind
            && !context.names.contains_key(name)
            && let Some(&index) = self.function_names.get(name) {
            let parameters = if let Some(function) = &self.checked[index] {
                function.bindings[..function.parameter_count].iter().map(ArgumentContract::from).collect()
            } else {
                self.assumed_modes.as_ref().expect("recursive access summaries")[index]
                    .iter().copied().zip(self.signatures[index].params.iter().copied())
                    .zip(&self.program.functions[index].parameter_initializers)
                    .enumerate().map(|(position, ((mode, ty), initializer))| ArgumentContract {
                        ty, mode, omission: initializer.omission(),
                        rest: self.program.functions[index].rest_parameter
                            && position + 1 == self.signatures[index].params.len(),
                    }).collect()
            };
            return Ok(Some(SpreadCallableContract { parameters, invocation: Mode::Read }));
        }
        let hint = self.expression_type_hint(context, callee);
        let hint = hint.and_then(|ty| self.unions.payload(ty).or(Some(ty)));
        let contract = match hint {
            Some(Type::Closure(id)) => {
                let closure = &self.closures[id];
                // Preserve native borrowed string tails rather than materializing
                // the language callable's owned rest pack for a known native origin.
                if let Some(ClosureForward::Native(native))=closure.forward
                    && crate::native_bindings::BINDINGS[native].parameters.last().is_some_and(|parameter|parameter.rest) {
                    return Ok(None);
                }
                SpreadCallableContract {
                    parameters: closure.parameters.iter().map(|&id| ArgumentContract::from(&closure.function.bindings[id])).collect(),
                    invocation: closure.invocation,
                }
            }
            Some(Type::Callback(id)) => {
                let callback = &self.callables.callbacks[id];
                SpreadCallableContract { parameters: callback.params.clone(), invocation: callback.invocation(&self.callables.shapes) }
            }
            Some(Type::StoredCallable(id)) => {
                let declaration = self.callables.storage.declaration(id).ok_or_else(|| Diagnostic::new("callable-layout", "Unknown callable storage family.", callee.span))?;
                let callback = &self.callables.callbacks[declaration];
                SpreadCallableContract { parameters: callback.params.clone(), invocation: callback.invocation(&self.callables.shapes) }
            }
            Some(Type::Callable(id)) => {
                let callable = &self.callables.results[id];
                SpreadCallableContract { parameters: callable.params.clone(), invocation: self.callables.shapes.get(callable.structural).kind.unwrap_or(CallbackKind::Owned).invocation() }
            }
            _ => return Ok(None),
        };
        Ok(Some(contract))
    }
}

/// Reachable expanded positions respect the total arity guard. A trailing
/// ordinary argument of a required fixed call may therefore have one exact
/// position even when an earlier spread has dynamic cardinality.
pub(crate) fn spread_position_range(
    segments: &[SpreadSegment], index: usize, minimum: usize, maximum: Option<usize>,
) -> (usize, Option<usize>) {
    let known_before: usize = segments[..index].iter().map(|segment| if segment.spread { segment.length.unwrap_or(0) } else { 1 }).sum();
    let known_after: usize = segments[index + 1..].iter().map(|segment| if segment.spread { segment.length.unwrap_or(0) } else { 1 }).sum();
    let dynamic_before = segments[..index].iter().any(|segment| segment.spread && segment.length.is_none());
    let dynamic_after = segments[index + 1..].iter().any(|segment| segment.spread && segment.length.is_none());
    let current = &segments[index];
    let own = if current.spread { current.length } else { Some(1) };
    let lower = if !dynamic_after && own.is_some() && dynamic_before {
        known_before.max(minimum.saturating_sub(known_after + own.unwrap()))
    } else { known_before };
    let upper = if !dynamic_before && own.is_some() {
        Some(known_before + own.unwrap().saturating_sub(1))
    } else {
        maximum.map(|maximum| maximum.saturating_sub(known_after + 1))
    };
    (lower, upper)
}

pub(super) fn args_count(selection:&SpreadVariantSelection)->usize {usize::from(matches!(selection,SpreadVariantSelection::FirstSegment(_)))}

// Literal provenance is metadata only: the source array is still constructed
// and staged once. Dynamic positions never acquire a literal-depth proof.
fn spread_literal_integer(segments:&[SpreadSegment], position:usize)->Option<i64> {
    let mut offset=0;
    for segment in segments {
        let length=segment.length?;
        if position<offset+length {
            let source=if segment.spread {let ExprKind::Array(values)=&segment.value.kind else{return None;};values.get(position-offset)?}else{&segment.value};
            return Analyzer::flat_literal_depth(source);
        }
        offset+=length;
    }
    None
}
