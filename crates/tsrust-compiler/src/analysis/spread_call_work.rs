//! Spread contracts and selected-slot construction never evaluate source operands.
use super::*;
use std::rc::Rc;
pub(super) struct Work {
    pub callee: Box<Expr>, pub segments: Vec<SpreadSegment>, pub span: Span, pub slot: usize,
    pub fresh: bool, pub boundary: Option<crate::capture_loan_identity::Boundary>,
    pub ordinary: Option<spread_calls::SpreadCallableContract>, pub builtin: Option<spread_builtins::SpreadBuiltinContract>,
    pub parameters: Vec<ArgumentContract>, pub minimum: usize, pub maximum: Option<usize>, pub fixed: usize,
    pub known: usize, pub dynamic: bool, pub staged: Option<SpreadCallee>, pub active: Rc<Loans>,
    pub selected_types: Vec<Type>, pub selected_unions: Vec<Option<UnionId>>, pub variants: Vec<SpreadCallVariant>,
    pub counts: Vec<usize>, pub next_count: usize, pub result: Option<Type>, pub observe: bool, pub prefix_variadic: bool,
    source_method: Option<String>,
}
pub(super) enum Stage { Ready, Callee { mode: Mode, expected: Option<Type> }, Receiver(Mode) }
pub(super) struct Segment {
    pub index: usize, pub mode: Mode, pub expected: Option<Type>,
    access: Mode, contracts: Vec<(ArgumentContract,bool)>,
}
pub(super) struct Variant { pub call: Box<Expr>, pub selection: SpreadVariantSelection, pub count: usize }
impl Work {
    pub fn receiver(&mut self) -> &mut Box<Expr> {
        let ExprKind::Field(base, _) = &mut self.callee.kind else { unreachable!("spread builtin receiver") }; base
    }
}
impl Analyzer {
    pub(super) fn start_spread_work(&mut self, context: &Context, expr: &Expr, loans: &Loans) -> Result<Work, Diagnostic> {
        let span=expr.span; let slot=span.start;
        let ExprKind::Call(source_callee,source_args)=&expr.kind else {unreachable!()};
        let callee=source_callee.clone();
        let segments=source_args.iter().map(|arg| {
            let (value,spread)=if let ExprKind::Unary {op:UnaryOp::Spread,value,..}=&arg.kind {(value.as_ref().clone(),true)} else {(arg.clone(),false)};
            let length=if spread {if let ExprKind::Array(values)=&value.kind { if values.iter().any(|value|matches!(value.kind,ExprKind::Unary{op:UnaryOp::Spread,..})) {None} else {Some(values.len())} }else{None}}else{Some(1)};
            let hint=self.spread_source_type_hint(context,&value)?;
            let element=if spread {hint.and_then(|ty|self.iterable_element(ty)).map(ArrayElement::ty).unwrap_or(Type::Undefined)}else{hint.unwrap_or(Type::Undefined)};
            Ok(SpreadSegment {value:Box::new(value),spread,element,access:Mode::Read,length})
        }).collect::<Result<Vec<_>,Diagnostic>>()?;
        let source_method=if let ExprKind::Field(_,name)=&source_callee.kind {Some(name.clone())}else{None};
        Ok(Work { callee,segments,span,slot,source_method,fresh:false,boundary:None,ordinary:None,builtin:None,parameters:Vec::new(),minimum:0,maximum:None,fixed:0,known:0,dynamic:false,staged:None,active:Rc::new(loans.clone()),selected_types:Vec::new(),selected_unions:Vec::new(),variants:Vec::new(),counts:Vec::new(),next_count:0,result:None,observe:false,prefix_variadic:false })
    }
    pub(super) fn spread_work_prototype(&self, context:&Context, work:&Work) -> Option<Box<Expr>> {
        if !matches!(&work.callee.kind,ExprKind::Name(name) if self.generic_functions.template(name).is_some() && !context.names.contains_key(name)) {return None;}
        let mut args=Vec::new();
        for segment in &work.segments {
            for _ in 0..segment.length.unwrap_or(1) {args.push(self.spread_selected(context,&segment.value,work.slot,SpreadSelection::Argument(args.len()),segment.element,Mode::Consume));}
        }
        Some(Box::new(Expr::new(ExprKind::Call(work.callee.clone(),args),work.span)))
    }
    pub(super) fn begin_spread_access(&mut self, context:&Context, work:&mut Work) -> Result<bool,Diagnostic> {
        for segment in &mut work.segments {
            if !segment.spread && let Some(ty)=self.callback_value_hint(context,&segment.value)? {segment.element=ty;}
        }
        work.boundary=Some(crate::capture_loan_identity::Boundary::capture(&context.capture_loans));
        work.fresh=matches!(work.callee.kind,ExprKind::Closure{..}|ExprKind::Call(_, _));
        if work.fresh {self.contextual_spread_callee(&mut work.callee,&work.segments)?;}
        Ok(work.fresh)
    }
    pub(super) fn spread_contract_request(&mut self, context:&Context, work:&Work) -> Result<Option<usize>,Diagnostic> {
        if let ExprKind::Name(name)=&work.callee.kind && !context.names.contains_key(name) && let Some(&index)=self.function_names.get(name) {
            self.named_captures.prepare(index,context,&self.program)?;
            return Ok(Some(index));
        }
        Ok(None)
    }
    pub(super) fn prepare_spread_contracts(&mut self,context:&Context,work:&mut Work)->Result<(),Diagnostic>{
        work.ordinary=self.ordinary_spread_contract(context,&work.callee)?;
        work.builtin=if work.ordinary.is_none(){self.builtin_spread_contract(context,&work.callee,&work.segments,work.span)?}else{None};
        let (parameters,minimum,maximum)=if let Some(contract)=&work.ordinary {
            let rest=contract.parameters.last().is_some_and(|parameter|parameter.rest);
            let fixed=contract.parameters.len()-usize::from(rest);
            (contract.parameters.clone(),crate::parameter_arity::minimum_arguments(&contract.parameters[..fixed],|parameter|parameter.omission.is_omittable()),if rest {None}else{Some(fixed)})
        } else if let Some(contract)=&work.builtin {
            let parameters=if matches!(contract.kind,SpreadBuiltinKind::ArraySplice|SpreadBuiltinKind::ArrayToSpliced) {
                let mut parameters=contract.arguments.iter().map(|argument|argument.contract(Type::Int)).collect::<Vec<_>>();
                let tail=contract.tail.expect("checked variadic array tail");
                let mut rest=tail.contract(Type::Undefined);rest.ty=self.constructor_array_type(rest.ty,work.span)?;rest.rest=true;parameters.push(rest);parameters
            }else{Vec::new()};
            (parameters,contract.minimum,contract.maximum)
        } else {return Err(Diagnostic::new("unsupported","Spread requires a resolved supported callable target.",work.callee.span));};
        work.parameters=parameters;work.minimum=minimum;work.maximum=maximum;
        work.known=work.segments.iter().map(|segment|segment.length.unwrap_or(0)).sum();
        work.dynamic=work.segments.iter().any(|segment|segment.length.is_none());
        if (!work.dynamic && work.known<minimum) || maximum.is_some_and(|maximum|work.known>maximum) {
            return Err(Diagnostic::new("arity",format!("Expanded call requires at least {minimum} arguments within its accepted maximum; statically supplied {}.",work.known),work.span));
        }
        work.fixed=work.parameters.len()-usize::from(work.parameters.last().is_some_and(|parameter|parameter.rest));
        Ok(())
    }
    pub(super) fn prepare_spread_stage(&mut self,context:&mut Context,work:&mut Work)->Result<Stage,Diagnostic>{
        if work.ordinary.is_some() && !matches!(&work.callee.kind,ExprKind::Name(name) if !context.names.contains_key(name) && self.function_names.contains_key(name)) {
            let access=work.ordinary.as_ref().unwrap().invocation;
            let hint=self.expression_type_hint(context,&work.callee);
            let ty=if work.fresh {work.callee.ty.expect("checked expression callee")} else if let ExprKind::Name(name)=&work.callee.kind && let Some(&id)=context.names.get(name)
                && !matches!(context.bindings[id].ty,Type::Union(_)|Type::BorrowedUnion(_)) {
                if id<context.capture_boundary && context.bindings[id].callback_kind.is_some_and(|kind|!kind.is_owned()) {
                    return Err(Diagnostic::new("callback-escape","Capturing a callback requires an owned parameter contract.",work.callee.span));
                }
                Self::access(context,id,access,work.callee.span,&work.active)?;
                work.callee.binding=Some(id);work.callee.ty=Some(context.bindings[id].ty);context.bindings[id].ty
            }else{return Ok(Stage::Callee{mode:access,expected:hint});};
            self.finish_spread_callee(context,work,ty)?;
        }else if let Some(contract)=&work.builtin && let ExprKind::Field(base,_) = &work.callee.kind
            && !matches!(&base.kind,ExprKind::Name(name) if !context.names.contains_key(name) && matches!(name.as_str(),"Math"|"Array"|"Number"|"console")) {
            return Ok(Stage::Receiver(contract.receiver_access));
        }
        Ok(Stage::Ready)
    }
    pub(super) fn finish_spread_callee(&self,context:&Context,work:&mut Work,ty:Type)->Result<(),Diagnostic>{
        let access=work.ordinary.as_ref().unwrap().invocation;
        for root in Self::loan_roots(&work.callee,access) {Rc::make_mut(&mut work.active).entry(root).or_default().push((access,work.callee.span));}
        let selected=self.spread_selected(context,&work.callee,work.slot,SpreadSelection::Callee,ty,access);
        let value=std::mem::replace(&mut work.callee,Box::new(selected));
        work.staged=Some(SpreadCallee{value,access,selection:SpreadSelection::Callee});Ok(())
    }
    pub(super) fn finish_spread_receiver(&mut self,context:&mut Context,work:&mut Work,ty:Type,mut access:Mode)->Result<(),Diagnostic>{
        if let Some(method)=&work.source_method {
            let checked=self.spread_builtin_traversal_access(context,ty,method,&work.segments)?;
            if checked>access {let ExprKind::Field(base,_) = &work.callee.kind else {unreachable!()};Self::storage_access(context,base,checked,base.span,&work.active)?;access=checked;}
        }
        let ExprKind::Field(base,_) = &mut work.callee.kind else {unreachable!()};
        for root in Self::loan_roots(base,access) {Rc::make_mut(&mut work.active).entry(root).or_default().push((access,base.span));}
        let selected=self.spread_selected(context,base,work.slot,SpreadSelection::Receiver,ty,access);
        let value=std::mem::replace(base,Box::new(selected));
        work.staged=Some(SpreadCallee{value,access,selection:SpreadSelection::Receiver});Ok(())
    }
    pub(super) fn prepare_spread_segment(&mut self,context:&Context,work:&mut Work,index:usize)->Result<Segment,Diagnostic>{
            let (lower,upper)=spread_position_range(&work.segments,index,work.minimum,work.maximum);
            let known_hint=self.callback_value_hint(context,&work.segments[index].value)?.is_some();
            if work.segments[index].spread && self.expression_type_hint(context,&work.segments[index].value).is_some_and(|ty| !self.native_spread_source(ty) && !matches!(ty,Type::Array(_) | Type::String | Type::Set(_) | Type::Map(_) | Type::Union(_) | Type::BorrowedUnion(_))) {return Err(Diagnostic::new("spread-source","Call spread requires an array, string, Set, Map, Deque, Heap or OrderedSet.",work.segments[index].value.span));}
            let mut contracts=Vec::new();
            let last=upper.unwrap_or(work.fixed.max(lower));
            for position in lower..=last {
                let contract=if let Some(builtin)=&work.builtin {builtin.argument(position).map(|argument|argument.contract(work.segments[index].element))}
                    else {work.parameters.get(position).copied().or_else(||work.parameters.last().copied().filter(|parameter|parameter.rest)).map(|mut parameter|{if parameter.rest {if let Type::Array(element)=parameter.ty {parameter.ty=element.ty();}} parameter})};
                if let Some(contract)=contract {let typed=work.builtin.as_ref().is_none_or(|builtin|builtin.argument(position).is_some_and(|argument|argument.ty.is_some()) || known_hint);contracts.push((contract,typed));}
            }
            let mut access=contracts.iter().map(|(contract,_)|contract.mode).max().unwrap_or(Mode::Read);
            let hints=contracts.iter().filter(|(_,typed)|*typed).map(|(contract,_)|if work.segments[index].spread {self.unions.payload(contract.ty).unwrap_or(contract.ty)}else{contract.ty}).collect::<HashSet<_>>();
            let expected_element=if matches!(work.segments[index].element,Type::Null|Type::Undefined) && contracts.iter().all(|(contract,typed)|!*typed || self.require(work.segments[index].element,Some(contract.ty),work.segments[index].value.span).is_ok()) && self.expression_type_hint(context,&work.segments[index].value).is_some() {Some(work.segments[index].element)}else if hints.len()==1 {hints.iter().copied().next()}else{None};
            if work.segments[index].element.is_copy() && access!=Mode::Mutate {access=Mode::Copy;}
            let segment=&mut work.segments[index];
            let guessed=self.spread_source_type_hint(context,&segment.value)?;
            let expected_source=if segment.spread {self.array_spread_context(&segment.value,guessed,expected_element)?.or_else(|| (segment.length==Some(0) && matches!(segment.value.kind,ExprKind::Array(_))).then_some(Type::Array(ArrayElement::Undefined(0))))}else{expected_element};
            if segment.spread && guessed.is_some_and(|ty|self.immutable_spread_keys(ty)) && access==Mode::Mutate {
                return Err(Diagnostic::new("collection-key","Call spread cannot lend mutable stored Set keys, Map entry tuples or ordered collection elements; explicitly copy to an array first.",segment.value.span));
            }
            let source_access=if segment.spread && (access==Mode::Copy || guessed==Some(Type::String)) {Mode::Read}else{access};

        Ok(Segment { index,mode:source_access,expected:expected_source,access,contracts })
    }
    pub(super) fn finish_spread_segment(&mut self,work:&mut Work,frame:Segment,actual:Type)->Result<(),Diagnostic>{
        let Segment { index,access,contracts,.. }=frame;
        let segment=&mut work.segments[index];
            if segment.spread && self.immutable_spread_keys(actual) && access==Mode::Mutate {return Err(Diagnostic::new("collection-key","Call spread cannot lend mutable stored Set keys, Map entry tuples or ordered collection elements; explicitly copy to an array first.",segment.value.span));}
            segment.element=if segment.spread {self.iterable_element(actual).ok_or_else(||Diagnostic::new("spread-source","Call spread requires an array, string, Set, Map, Deque, Heap or OrderedSet.",segment.value.span))?.ty()}else{actual};
            // Character strings are new owners, not references into the source.
            segment.access=if segment.spread && actual==Type::String {Mode::Consume}else if segment.element.is_copy()&&access!=Mode::Mutate {Mode::Copy}else{access};
            if segment.access==Mode::Read && let Type::Union(id)=segment.element && !id.copy {segment.element=Type::BorrowedUnion(id);}
            for (contract,typed) in contracts {if typed {self.require(segment.element,Some(contract.ty),segment.value.span)?;}}
            if !segment.element.is_copy() && matches!(segment.access,Mode::Read|Mode::Mutate) {
                for root in Self::loan_roots(&segment.value,segment.access) {std::rc::Rc::make_mut(&mut work.active).entry(root).or_default().push((segment.access,segment.value.span));}
            }

        Ok(())
    }
    pub(super) fn prepare_spread_variants(&mut self,work:&mut Work)->Result<(),Diagnostic>{
        let span=work.span;let maximum=work.maximum;let minimum=work.minimum;let fixed=work.fixed;let known=work.known;let dynamic=work.dynamic;
        let segments=&work.segments;let builtin=&work.builtin;
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
        let prefix_variadic=builtin.as_ref().is_some_and(|contract|matches!(contract.kind,SpreadBuiltinKind::ArraySplice|SpreadBuiltinKind::ArrayToSpliced));
        let counts=if prefix_variadic {if dynamic {(minimum..=fixed).collect::<Vec<_>>()}else{vec![known.min(fixed)]}}else if maximum.is_some() {if dynamic {(minimum..=maximum.unwrap()).collect::<Vec<_>>()}else{vec![known]}}
            else if builtin.as_ref().is_some_and(|contract|matches!(contract.kind,SpreadBuiltinKind::MathVariadic|SpreadBuiltinKind::StringConcat|SpreadBuiltinKind::ConsoleVariadic)) {vec![segments.iter().filter(|segment|segment.length!=Some(0)).count()]}
            else if builtin.is_some() {vec![segments.iter().filter(|segment|segment.length!=Some(0)).count()]}
            else {if dynamic {(minimum..=fixed).collect::<Vec<_>>()}else{vec![known.min(fixed)]}};
        let observe=builtin.as_ref().is_some_and(|contract|contract.kind==SpreadBuiltinKind::ObserveVariadic);
        let counts=if observe {(0..=segments.len()).collect::<Vec<_>>()}else{counts};

        work.selected_types=selected_types;work.selected_unions=selected_unions;work.counts=counts;work.observe=observe;work.prefix_variadic=prefix_variadic;
        Ok(())
    }
    pub(super) fn next_spread_variant(&self,context:&Context,work:&mut Work)->Option<Variant>{
        while let Some(&count)=work.counts.get(work.next_count) {
            work.next_count+=1;
            let span=work.span;let slot=work.slot;let minimum=work.minimum;let maximum=work.maximum;let fixed=work.fixed;
            let segments=&work.segments;let builtin=&work.builtin;let parameters=&work.parameters;
            let selected_types=&work.selected_types;let selected_unions=&work.selected_unions;let observe=work.observe;let prefix_variadic=work.prefix_variadic;
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

            return Some(Variant { call:Box::new(Expr::new(ExprKind::Call(work.callee.clone(),args),span)),selection,count:if observe{spread_calls::args_count(&selection)}else{count} });
        }
        None
    }
    pub(super) fn finish_spread_variant(&self,work:&mut Work,mut variant:Variant,ty:Type)->Result<(),Diagnostic>{
        if work.result.is_some_and(|prior|prior!=ty) {return Err(Diagnostic::new("type","Reachable expanded arities must share one checked result type.",work.span));}
        work.result=Some(ty);variant.call.supplied_count=Some(CallArgumentCount::Runtime{slot:work.slot});
        work.variants.push(SpreadCallVariant{selection:variant.selection,count:variant.count,call:variant.call});Ok(())
    }
    pub(super) fn finish_spread_work(&self,context:&mut Context,expr:&mut Expr,work:Work)->Type {
        let result=work.result.expect("checked spread result");
        let call=work.variants.first().expect("checked spread call variant").call.clone();
        if work.fresh && !self.expression_result_retains_captures(result){work.boundary.expect("spread capture boundary").unwind(&mut context.capture_loans);}
        expr.kind=ExprKind::SpreadCall(Box::new(CheckedSpreadCall{callee:work.staged,segments:work.segments,call,parameters:work.parameters,selected_types:work.selected_types,selected_unions:work.selected_unions,variants:work.variants,slot:work.slot,minimum:work.minimum,maximum:work.maximum,builtin:work.builtin.map(|contract|contract.kind)}));
        expr.ty=Some(result);result
    }
}
