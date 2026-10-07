//! Dense call spread keeps source effects separate from selected native slots.
use super::*;

#[derive(Clone)]
pub(super) struct SpreadCallableContract {
    pub parameters: Vec<ArgumentContract>,
    pub invocation: Mode,
}

impl Analyzer {
    #[inline(never)]
    pub(super) fn spread_array_construction(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic> {
        let (args,annotation)=match &expr.kind { ExprKind::ArrayOf(args,annotation) => (args,annotation.as_ref()), ExprKind::Array(args) => (args,None), _ => unreachable!() };
        let span=expr.span;let slot=span.start;
        let annotated=annotation.map(|annotation|self.resolve_value_annotation(annotation,false)).transpose()?;
        let hint=annotated.or_else(||if let Some(Type::Array(element))=expected {Some(element.ty())}else{None}).or_else(|| {
            args.iter().filter_map(|arg| {
                if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind {
                    match self.expression_type_hint(context,value)? { Type::Array(element)=>Some(element.ty()), _=>None }
                } else {self.expression_type_hint(context,arg)}
            }).find(|ty|matches!(ty,Type::Number|Type::Array(ArrayElement::Number(_))))
        });
        let mut element=hint;
        let mut segments=Vec::new();let mut active=loans.clone();
        for arg in args {
            let (mut value,spread)=if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind {(*value.clone(),true)}else{(arg.clone(),false)};
            let length=if spread {if let ExprKind::Array(values)=&value.kind { if values.iter().any(|value|matches!(value.kind,ExprKind::Unary{op:UnaryOp::Spread,..})) {None} else {Some(values.len())} }else{None}}else{Some(1)};
            let guessed=self.expression_type_hint(context,&value);
            let expected=if spread {self.array_spread_context(&value,guessed,element)?}else{element};
            let copied=if spread {match guessed {Some(Type::Array(element))=>element.ty().is_copy(),Some(Type::Set(id))=>self.program.records[id].set_definition.as_ref().unwrap().key.is_copy(),_=>false}}else{guessed.is_some_and(Type::is_copy)};
            let access=if spread && guessed==Some(Type::String) {Mode::Read}else if copied {Mode::Copy}else{Mode::Consume};
            let actual=self.expression(context,&mut value,if spread&&copied {Mode::Read}else{access},&active,expected)?;
            let actual=if spread {match actual {Type::Array(element)=>element.ty(),Type::String=>Type::String,Type::Set(id)=>self.program.records[id].set_definition.as_ref().unwrap().key,Type::Map(id)=>Type::Record(self.map_iteration_entry(id,value.span)?),_=>return Err(Diagnostic::new("spread-source","Array spread requires an array, string, Set or Map.",value.span))}}else{actual};
            if let Some(element)=element {self.require(actual,Some(element),value.span)?;}else{element=Some(actual);}
            segments.push(SpreadSegment{value:Box::new(value),spread,element:actual,access,length});
            let _=&mut active;
        }
        let element=element.ok_or_else(||Diagnostic::new("array-type","An empty array construction needs an explicit element type or array context.",span))?;
        let ty=self.constructor_array_type(element,span)?;self.require(ty,expected,span)?;
        let source=segments.first().expect("spread array source").value.as_ref();
        let mut call=self.spread_selected(context,source,slot,SpreadSelection::Rest,ty,Mode::Consume);call.span=span;
        let parameters=vec![ArgumentContract{ty,mode:Mode::Consume,omission:crate::ParameterOmission::Required,rest:true}];
        let variant=SpreadCallVariant{selection:SpreadVariantSelection::Count(0),count:0,call:Box::new(call.clone())};
        expr.kind=ExprKind::SpreadCall(Box::new(CheckedSpreadCall{callee:None,segments,call:Box::new(call),parameters,selected_types:Vec::new(),selected_unions:Vec::new(),variants:vec![variant],slot,minimum:0,maximum:None,builtin:None}));
        expr.ty=Some(ty);expr.target=Some(CallTarget::ArrayOf);Ok(ty)
    }

    #[inline(never)]
    fn array_spread_context(&mut self,value:&Expr,guessed:Option<Type>,element:Option<Type>)->Result<Option<Type>,Diagnostic>{
        if matches!(value.kind,ExprKind::SetConstructor(_,Some(_))|ExprKind::MapConstructor(_,Some(_))){return Ok(None);}
        let Some(element)=element else{return Ok(None);};
        let family=match &value.kind {ExprKind::SetConstructor(_,None)=>Some("Set"),ExprKind::MapConstructor(_,None)=>Some("Map"),_=>None};
        if let Some(family)=family {
            let arguments=if family=="Set" {vec![element]}else if let Type::Record(id)=element {
                let fields=&self.program.records[id].fields;
                if fields.len()!=2{return Ok(None);}
                vec![self.fields[id][&fields[0].name.text],self.fields[id][&fields[1].name.text]]
            }else{return Ok(None);};
            let types=crate::generic_inference::Types{ callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
            let mut annotation=crate::type_syntax::ValueAnnotation{span:value.span,kind:crate::type_syntax::ValueAnnotationKind::Applied{name:crate::type_syntax::TypeName{text:family.into(),span:value.span},arguments:arguments.into_iter().map(|ty|types.annotation(ty,value.span)).collect::<Result<Vec<_>,_>>()?}};
            self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions)?;
            append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
            return self.resolve_value_annotation(&annotation,false).map(Some);
        }
        if matches!(guessed,Some(Type::String|Type::Set(_)|Type::Map(_))){Ok(None)}else{self.constructor_array_type(element,value.span).map(Some)}
    }

    #[inline(never)]
    fn spread_source_type_hint(&mut self,context:&Context,value:&Expr)->Result<Option<Type>,Diagnostic>{
        if let Some(ty)=self.expression_type_hint(context,value){return Ok(Some(ty));}
        let annotation=match &value.kind {ExprKind::SetConstructor(_,Some(annotation))|ExprKind::MapConstructor(_,Some(annotation))=>annotation,_=>return Ok(None)};
        let mut annotation=annotation.as_ref().clone();
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        self.resolve_value_annotation(&annotation,false).map(Some)
    }

    /// Generic preparation sees element slots, not an iterable as one argument.
    /// This pass infers types only; ordinary spread checking owns source effects.
    #[inline(never)]
    pub(super) fn prepare_generic_spread(&mut self,context:&Context,expr:&mut Expr,expected:Option<Type>)->Result<(),Diagnostic>{
        let ExprKind::Call(callee,args)=&expr.kind else{return Ok(());};
        let ExprKind::Name(name)=&callee.kind else{return Ok(());};
        if context.names.contains_key(name)||context.pending_names.contains(name)||self.generic_functions.template(name).is_none(){return Ok(());}
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
        let mut prototype=Expr::new(ExprKind::Call(callee.clone(),normalized),expr.span);
        self.prepare_generic_call(context,&mut prototype,expected)?;
        let ExprKind::Call(prepared,_)=prototype.kind else{unreachable!()};
        let ExprKind::Call(callee,_)=&mut expr.kind else{unreachable!()};*callee=prepared;
        Ok(())
    }

    fn spread_selected(&self, context: &Context, source: &Expr, slot: usize, selection: SpreadSelection, ty: Type, access: Mode) -> Expr {
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

    fn spread_argument(&self,context:&Context,segments:&[SpreadSegment],slot:usize,position:usize,ty:Type,normalized:Option<UnionId>,minimum:usize,maximum:Option<usize>)->Expr {
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

    pub(super) fn spread_call(&mut self, context:&mut Context, expr:&mut Expr, loans:&Loans, expected:Option<Type>) -> Result<Type,Diagnostic> {
        let span=expr.span; let slot=span.start;
        let ExprKind::Call(source_callee,source_args)=&expr.kind else {unreachable!()};
        let mut callee=source_callee.as_ref().clone();
        let mut segments=source_args.iter().map(|arg| {
            let (value,spread)=if let ExprKind::Unary {op:UnaryOp::Spread,value,..}=&arg.kind {(value.as_ref().clone(),true)} else {(arg.clone(),false)};
            let length=if spread {if let ExprKind::Array(values)=&value.kind { if values.iter().any(|value|matches!(value.kind,ExprKind::Unary{op:UnaryOp::Spread,..})) {None} else {Some(values.len())} }else{None}}else{Some(1)};
            let hint=self.spread_source_type_hint(context,&value)?;
            let element=if spread {hint.and_then(|ty|self.iterable_element(ty)).map(ArrayElement::ty).unwrap_or(Type::Undefined)}else{hint.unwrap_or(Type::Undefined)};
            Ok(SpreadSegment {value:Box::new(value),spread,element,access:Mode::Read,length})
        }).collect::<Result<Vec<_>,Diagnostic>>()?;
        // Generic inference uses checked-type hints, never evaluation of array
        // elements in place of constructing the source array.
        if matches!(&callee.kind,ExprKind::Name(name) if self.generic_functions.template(name).is_some() && !context.names.contains_key(name)) {
            let mut args=Vec::new();
            for segment in &segments {
                let count=segment.length.unwrap_or(1);
                for _ in 0..count { args.push(self.spread_selected(context,&segment.value,slot,SpreadSelection::Argument(args.len()),segment.element,Mode::Consume)); }
            }
            let mut prototype=Expr::new(ExprKind::Call(Box::new(callee.clone()),args),span);
            self.prepare_generic_call(context,&mut prototype,expected)?;
            let ExprKind::Call(prepared,_) = prototype.kind else {unreachable!()}; callee=*prepared;
        }
        for segment in &mut segments {
            if !segment.spread && let Some(ty)=self.callback_value_hint(context,&segment.value)? {segment.element=ty;}
        }
        let first_capture=context.capture_loans.len();
        let fresh_callee=matches!(callee.kind,ExprKind::Closure{..}|ExprKind::Call(_, _));
        if fresh_callee {self.contextual_spread_callee(&mut callee,&segments)?;self.expression(context,&mut callee,Mode::Consume,loans,None)?;}
        let ordinary=self.ordinary_spread_contract(context,&callee)?;
        let builtin=if ordinary.is_none() {self.builtin_spread_contract(context,&callee,&segments,span)?}else{None};
        let (parameters,minimum,maximum)=if let Some(contract)=&ordinary {
            let rest=contract.parameters.last().is_some_and(|parameter|parameter.rest);
            let fixed=contract.parameters.len()-usize::from(rest);
            (contract.parameters.clone(),crate::parameter_arity::minimum_arguments(&contract.parameters[..fixed],|parameter|parameter.omission.is_omittable()),if rest {None}else{Some(fixed)})
        } else if let Some(contract)=&builtin {
            let parameters=if matches!(contract.kind,SpreadBuiltinKind::ArraySplice|SpreadBuiltinKind::ArrayToSpliced) {
                let mut parameters=contract.arguments.iter().map(|argument|argument.contract(Type::Int)).collect::<Vec<_>>();
                let tail=contract.tail.expect("checked variadic array tail");
                let mut rest=tail.contract(Type::Undefined);rest.ty=self.constructor_array_type(rest.ty,span)?;rest.rest=true;parameters.push(rest);parameters
            }else{Vec::new()};
            (parameters,contract.minimum,contract.maximum)
        } else {return Err(Diagnostic::new("unsupported","Spread requires a resolved supported callable target.",callee.span));};
        let known:usize=segments.iter().map(|segment|segment.length.unwrap_or(0)).sum();
        let dynamic=segments.iter().any(|segment|segment.length.is_none());
        if (!dynamic && known<minimum) || maximum.is_some_and(|maximum|known>maximum) {
            return Err(Diagnostic::new("arity",format!("Expanded call requires at least {minimum} arguments within its accepted maximum; statically supplied {known}."),span));
        }
        let fixed=parameters.len()-usize::from(parameters.last().is_some_and(|parameter|parameter.rest));
        let mut staged_callee=None; let mut active=loans.clone();
        if ordinary.is_some() && !matches!(&callee.kind,ExprKind::Name(name) if !context.names.contains_key(name) && self.function_names.contains_key(name)) {
            let access=ordinary.as_ref().unwrap().invocation;
            let hint=self.expression_type_hint(context,&callee);
            // A narrowed nullable local is checked as an ordinary expression so
            // the staged callee is its present payload, not the tagged slot.
            let ty=if fresh_callee {callee.ty.expect("checked expression callee")} else if let ExprKind::Name(name)=&callee.kind && let Some(&id)=context.names.get(name)
                && !matches!(context.bindings[id].ty,Type::Union(_)|Type::BorrowedUnion(_)) {
                if id<context.capture_boundary && context.bindings[id].callback_kind.is_some_and(|kind|!kind.is_owned()) {
                    return Err(Diagnostic::new("callback-escape","Capturing a callback requires an owned parameter contract.",callee.span));
                }
                Self::access(context,id,access,callee.span,&active)?;
                callee.binding=Some(id);callee.ty=Some(context.bindings[id].ty);context.bindings[id].ty
            }else{self.expression(context,&mut callee,access,&active,hint)?};
            for root in Self::loan_roots(&callee,access) {active.entry(root).or_default().push((access,callee.span));}
            let selected=self.spread_selected(context,&callee,slot,SpreadSelection::Callee,ty,access);
            staged_callee=Some(SpreadCallee {value:Box::new(callee),access,selection:SpreadSelection::Callee}); callee=selected;
        } else if let Some(contract)=&builtin && let ExprKind::Field(base,_) = &mut callee.kind
            && !matches!(&base.kind,ExprKind::Name(name) if !context.names.contains_key(name) && matches!(name.as_str(),"Math"|"Array"|"Number"|"console")) {
            let mut access=contract.receiver_access;
            let ty=self.expression(context,base,access,&active,None)?;
            if let ExprKind::Field(_,method)=&source_callee.kind {
                let checked=self.spread_builtin_traversal_access(context,ty,method,&segments)?;
                if checked>access {Self::storage_access(context,base,checked,base.span,&active)?;access=checked;}
            }
            for root in Self::loan_roots(base,access) {active.entry(root).or_default().push((access,base.span));}
            let selected=self.spread_selected(context,base,slot,SpreadSelection::Receiver,ty,access);
            staged_callee=Some(SpreadCallee {value:base.clone(),access,selection:SpreadSelection::Receiver}); **base=selected;
        }
        for index in 0..segments.len() {
            let (lower,upper)=spread_position_range(&segments,index,minimum,maximum);
            let known_hint=self.callback_value_hint(context,&segments[index].value)?.is_some();
            if segments[index].spread && self.expression_type_hint(context,&segments[index].value).is_some_and(|ty| !matches!(ty,Type::Array(_) | Type::String | Type::Set(_) | Type::Map(_) | Type::Union(_) | Type::BorrowedUnion(_))) {return Err(Diagnostic::new("spread-source","Call spread requires an array, string, Set or Map.",segments[index].value.span));}
            let mut contracts=Vec::new();
            let last=upper.unwrap_or(fixed.max(lower));
            for position in lower..=last {
                let contract=if let Some(builtin)=&builtin {builtin.argument(position).map(|argument|argument.contract(segments[index].element))}
                    else {parameters.get(position).copied().or_else(||parameters.last().copied().filter(|parameter|parameter.rest)).map(|mut parameter|{if parameter.rest {if let Type::Array(element)=parameter.ty {parameter.ty=element.ty();}} parameter})};
                if let Some(contract)=contract {let typed=builtin.as_ref().is_none_or(|builtin|builtin.argument(position).is_some_and(|argument|argument.ty.is_some()) || known_hint);contracts.push((contract,typed));}
            }
            let mut access=contracts.iter().map(|(contract,_)|contract.mode).max().unwrap_or(Mode::Read);
            let hints=contracts.iter().filter(|(_,typed)|*typed).map(|(contract,_)|if segments[index].spread {self.unions.payload(contract.ty).unwrap_or(contract.ty)}else{contract.ty}).collect::<HashSet<_>>();
            let expected_element=if matches!(segments[index].element,Type::Null|Type::Undefined) && contracts.iter().all(|(contract,typed)|!*typed || self.require(segments[index].element,Some(contract.ty),segments[index].value.span).is_ok()) && self.expression_type_hint(context,&segments[index].value).is_some() {Some(segments[index].element)}else if hints.len()==1 {hints.iter().copied().next()}else{None};
            if segments[index].element.is_copy() && access!=Mode::Mutate {access=Mode::Copy;}
            let segment=&mut segments[index];
            let guessed=self.spread_source_type_hint(context,&segment.value)?;
            let expected_source=if segment.spread {self.array_spread_context(&segment.value,guessed,expected_element)?.or_else(|| (segment.length==Some(0) && matches!(segment.value.kind,ExprKind::Array(_))).then_some(Type::Array(ArrayElement::Undefined(0))))}else{expected_element};
            if segment.spread && matches!(guessed,Some(Type::Set(_)|Type::Map(_))) && access==Mode::Mutate {
                return Err(Diagnostic::new("collection-key","Call spread cannot lend mutable stored Set keys or Map entry tuples; explicitly copy to an array first.",segment.value.span));
            }
            let source_access=if segment.spread && (access==Mode::Copy || guessed==Some(Type::String)) {Mode::Read}else{access};
            let actual=self.expression(context,&mut segment.value,source_access,&active,expected_source)?;
            if segment.spread && matches!(actual,Type::Set(_)|Type::Map(_)) && access==Mode::Mutate {return Err(Diagnostic::new("collection-key","Call spread cannot lend mutable stored Set keys or Map entry tuples; explicitly copy to an array first.",segment.value.span));}
            segment.element=if segment.spread {self.iterable_element(actual).ok_or_else(||Diagnostic::new("spread-source","Call spread requires an array, string, Set or Map.",segment.value.span))?.ty()}else{actual};
            // Character strings are new owners, not references into the source.
            segment.access=if segment.spread && actual==Type::String {Mode::Consume}else if segment.element.is_copy()&&access!=Mode::Mutate {Mode::Copy}else{access};
            if segment.access==Mode::Read && let Type::Union(id)=segment.element && !id.copy {segment.element=Type::BorrowedUnion(id);}
            for (contract,typed) in contracts {if typed {self.require(segment.element,Some(contract.ty),segment.value.span)?;}}
            if !segment.element.is_copy() && matches!(segment.access,Mode::Read|Mode::Mutate) {
                for root in Self::loan_roots(&segment.value,segment.access) {active.entry(root).or_default().push((segment.access,segment.value.span));}
            }
        }
        let slot_limit=maximum.unwrap_or(fixed);
        let mut selected_types=Vec::new();let mut selected_unions=Vec::new();
        for position in 0..slot_limit {
            let candidates=segments.iter().enumerate().filter(|(index,segment)|{
                if segment.length==Some(0) {return false;} let (lower,upper)=spread_position_range(&segments,*index,minimum,maximum); position>=lower&&upper.is_none_or(|upper|position<=upper)
            }).map(|(_,segment)|segment.element).collect::<HashSet<_>>();
            let ty=if candidates.len()==1 {*candidates.iter().next().unwrap()} else if candidates.is_empty() {Type::Undefined} else {
                self.unions.intern(candidates.iter().copied().map(Type::owning),span)?
            }; selected_unions.push((candidates.len()>1).then(||ty.union().expect("normalized nullable spread slot")));selected_types.push(ty);
        }
        let mut variants=Vec::new();
        let prefix_variadic=builtin.as_ref().is_some_and(|contract|matches!(contract.kind,SpreadBuiltinKind::ArraySplice|SpreadBuiltinKind::ArrayToSpliced));
        let counts=if prefix_variadic {if dynamic {(minimum..=fixed).collect::<Vec<_>>()}else{vec![known.min(fixed)]}}else if maximum.is_some() {if dynamic {(minimum..=maximum.unwrap()).collect::<Vec<_>>()}else{vec![known]}}
            else if builtin.as_ref().is_some_and(|contract|matches!(contract.kind,SpreadBuiltinKind::MathVariadic|SpreadBuiltinKind::StringConcat|SpreadBuiltinKind::ConsoleVariadic)) {vec![segments.iter().filter(|segment|segment.length!=Some(0)).count()]}
            else if builtin.is_some() {vec![segments.iter().filter(|segment|segment.length!=Some(0)).count()]}
            else {if dynamic {(minimum..=fixed).collect::<Vec<_>>()}else{vec![known.min(fixed)]}};
        let mut result=None;
        let observe=builtin.as_ref().is_some_and(|contract|contract.kind==SpreadBuiltinKind::ObserveVariadic);
        let counts=if observe {(0..=segments.len()).collect::<Vec<_>>()}else{counts};
        for count in counts {
            let mut args=Vec::new();
            let selection=if observe {
                if count==segments.len() {
                    if segments.iter().any(|segment|segment.length.is_some_and(|length|length>0)) {continue;}
                    SpreadVariantSelection::Empty
                }else{
                    if segments[count].length==Some(0) || segments[..count].iter().any(|segment|segment.length.is_some_and(|length|length>0)) {continue;}
                    let segment=&segments[count];
                    let mut argument=self.spread_selected(context,&segment.value,slot,SpreadSelection::Argument(0),segment.element,segment.access);
                    if let ExprKind::SpreadValue(selected)=&mut argument.kind {selected.carrier=false;}
                    args.push(argument);
                    SpreadVariantSelection::FirstSegment(count)
                }
            }else{SpreadVariantSelection::Count(count)};
            if observe {} else if prefix_variadic {
                for position in 0..count {
                    args.push(self.spread_argument(context,&segments,slot,position,selected_types[position],selected_unions[position],minimum,maximum));
                }
                if count==fixed {
                    for (index,segment) in segments.iter().enumerate().filter(|(_,segment)|segment.length!=Some(0)) {
                        let (_,upper)=spread_position_range(&segments,index,minimum,maximum);
                        if upper.is_none_or(|upper|upper>=fixed) {args.push(self.spread_selected(context,&segment.value,slot,SpreadSelection::Argument(args.len()),segment.element,segment.access));}
                    }
                }
            }else if maximum.is_none() && builtin.is_some() {
                for segment in segments.iter().filter(|segment|segment.length!=Some(0)) {args.push(self.spread_selected(context,&segment.value,slot,SpreadSelection::Argument(args.len()),segment.element,segment.access));}
            }else {
                for position in 0..count {
                    args.push(self.spread_argument(context,&segments,slot,position,selected_types[position],selected_unions[position],minimum,maximum));
                }
                if maximum.is_none() {
                    while args.len()<fixed {args.push(Expr::new(ExprKind::Undefined,span));}
                    let rest=parameters.last().expect("rest contract");
                    let source=segments.last().expect("spread source");
                    let value=self.spread_selected(context,&source.value,slot,SpreadSelection::Rest,rest.ty,Mode::Consume);
                    args.push(Expr::new(ExprKind::Unary{op:UnaryOp::RestPack,value:Box::new(value),operator:span},span));
                }
            }
            let mut call=Expr::new(ExprKind::Call(Box::new(callee.clone()),args),span);
            let ty=self.expression(context,&mut call,Mode::Consume,loans,expected)?;
            if result.is_some_and(|prior|prior!=ty) {return Err(Diagnostic::new("type","Reachable expanded arities must share one checked result type.",span));} result=Some(ty);
            call.supplied_count=Some(CallArgumentCount::Runtime{slot});
            variants.push(SpreadCallVariant{selection,count:if observe {args_count(&selection)}else{count},call:Box::new(call)});
        }
        let call=variants.first().expect("checked spread call variant").call.clone();
        expr.kind=ExprKind::SpreadCall(Box::new(CheckedSpreadCall {callee:staged_callee,segments,call,parameters,selected_types,selected_unions,variants,slot,minimum,maximum,builtin:builtin.map(|contract|contract.kind)}));
        let result=result.unwrap();
        if fresh_callee && !self.expression_result_retains_captures(result){context.capture_loans.truncate(first_capture);}
        expr.ty=Some(result); Ok(result)
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
            self.named_captures.prepare(index, context, &self.program)?;
            self.function(index)?;
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

fn args_count(selection:&SpreadVariantSelection)->usize {usize::from(matches!(selection,SpreadVariantSelection::FirstSegment(_)))}

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
