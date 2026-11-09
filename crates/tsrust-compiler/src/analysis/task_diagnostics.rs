//! Task abandonment follows owners through checked control flow, independently of suppression.
use super::*;
use std::collections::BTreeSet;
type Tokens = BTreeSet<(usize, usize, bool)>;
type Place = (usize, Vec<String>);
#[derive(Clone,Debug,PartialEq,Eq)]
struct Poll { cursors:Vec<Place>, selected:Vec<Place> }
struct CursorPollSources{cursors:Vec<Place>,mapped:bool}
#[derive(Clone,Debug,Default,PartialEq,Eq)]
struct State { owners:BTreeMap<Place,Tokens>, polls:BTreeMap<Place,Poll> }
impl State { fn new()->Self{Self::default()} }
impl<const N:usize> From<[(Place,Tokens);N]> for State{fn from(values:[(Place,Tokens);N])->Self{Self{owners:BTreeMap::from(values),polls:BTreeMap::new()}}}
impl std::ops::Deref for State{type Target=BTreeMap<Place,Tokens>;fn deref(&self)->&Self::Target{&self.owners}}
impl std::ops::DerefMut for State{fn deref_mut(&mut self)->&mut Self::Target{&mut self.owners}}
impl IntoIterator for State{type Item=(Place,Tokens);type IntoIter=std::collections::btree_map::IntoIter<Place,Tokens>;fn into_iter(self)->Self::IntoIter{self.owners.into_iter()}}
fn within(place:&Place,parent:&Place)->bool{place.0==parent.0&&place.1.starts_with(&parent.1)}
fn overlaps(left:&Place,right:&Place)->bool{left.0==right.0&&left.1.iter().zip(&right.1).all(|(left,right)|left==right||left=="*"||right=="*")}
fn rebased(place:&Place,source:&Place,destination:&Place)->Place{let mut path=destination.1.clone();path.extend_from_slice(&place.1[source.1.len()..]);(destination.0,path)}
fn invalidate(state:&mut State,owner:&Place){state.polls.retain(|result,poll|!overlaps(result,owner)&&!poll.cursors.iter().any(|cursor|overlaps(cursor,owner)));}
fn widen(state:&mut State,function:&CheckedFunction,program:&CheckedProgram){
 let owners=std::mem::take(&mut state.owners);
 for((root,mut path),tokens)in owners{
  if tokens.is_empty(){continue;}
  let patterns=function.bindings.get(root).map(|binding|program.task_paths(binding.ty));
  let original=path.clone();
  for(index,part)in path.iter_mut().enumerate(){
   if part.parse::<usize>().is_ok()&&patterns.as_ref().is_none_or(|patterns|patterns.iter().any(|(pattern,_)|pattern.get(index).is_some_and(|part|part=="*")&&pattern.iter().take(index).zip(&original).all(|(pattern,actual)|pattern==actual||pattern=="*"))){*part="*".into();}
  }
  state.entry((root,path)).or_default().extend(tokens);
 }
 // A backedge can repeat writes/polls. Only a freshly evaluated observation
 // may refine the next iteration's source; owner tokens remain conservative.
 state.polls.clear();
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Exit { Next, Return, Throw, Break, Continue }
struct Path { state: State, exit: Exit }
struct Tracker<'a> { program: &'a CheckedProgram, warnings: BTreeMap<(usize,usize,usize),Diagnostic>, failures: Vec<State>, projections:usize }
impl CheckedProgram {
    pub(crate) fn task_type(&self, ty: Type) -> bool {
        match ty {
            Type::Closure(id) => self.task_origins.contains(&id),
            Type::Callback(id) => self.callback_contracts[id].kind==CallbackKind::Task,
            Type::StoredCallable(id)|Type::BorrowedCallable(id) => self.stored_contracts.declaration(id).is_some_and(|id| self.callback_contracts[id].kind==CallbackKind::Task),
            _=>false,
        }
    }
    pub(crate) fn contains_started_job(&self,root:Type)->bool {
        let mut pending=vec![root];let mut seen=HashSet::new();
        while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}match ty {
            Type::Record(id)=>{let record=&self.records[id];if record.native_type==Some(crate::native_signatures::NativeTypeId::Job){return true;}pending.extend(record.fields.iter().filter_map(|field|field.ty));if let Some(application)=&record.application{pending.extend(&application.resolved_arguments);}},
            Type::Closure(id)=>pending.extend(self.closures[id].captures.iter().filter(|capture|capture.storage==Mode::Consume).map(|capture|self.closures[id].function.bindings[capture.inner].ty)),
            Type::Array(element)=>pending.push(element.ty()),
            Type::Union(id)|Type::BorrowedUnion(id)=>pending.extend(&self.unions[id.index()].members),
            Type::StoredCallable(id)=>if let Some((_,_,origins))=self.stored_contracts.populated().find(|(family,_,_)|*family==id){pending.extend(origins.iter().copied().map(Type::Closure));},
            _=>{}
        }}false
    }
    fn task_paths(&self,root:Type)->Vec<(Vec<String>,bool)> {
        let mut found=Vec::new();let mut pending=vec![(root,Vec::new(),HashSet::new())];
        while let Some((ty,path,mut seen))=pending.pop(){
            if self.task_type(ty) || matches!(ty,Type::Record(id) if self.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Job)){found.push((path,self.contains_started_job(ty)));continue;}
            if !seen.insert(ty){continue;}
            match ty {
                Type::Record(id)=>for field in &self.records[id].fields {if let Some(ty)=field.ty{let mut child=path.clone();child.push(field.name.text.clone());pending.push((ty,child,seen.clone()));}},
                Type::Union(id)|Type::BorrowedUnion(id)=>for &ty in &self.unions[id.index()].members{pending.push((ty,path.clone(),seen.clone()));},
                Type::Array(element)=>{let mut child=path;child.push("*".into());pending.push((element.ty(),child,seen));},
                Type::Closure(id)=>for capture in &self.closures[id].captures{if capture.storage==Mode::Consume{let mut child=path.clone();child.push(format!("capture{}",capture.inner));pending.push((self.closures[id].function.bindings[capture.inner].ty,child,seen.clone()));}},
                Type::StoredCallable(family)=>if let Some(layout)=self.stored_contracts.scoped_native_family(family).filter(|layout|layout.lifetimes()==0){
                    for alternative in &layout.alternatives{
                        pending.push((Type::Closure(alternative.origin),path.clone(),seen.clone()));
                    }
                },
                _=>{}
            }
        }found
    }
    pub(crate) fn task_warnings(&self)->Vec<Diagnostic> {
        if self.task_origins.is_empty() && !self.callback_contracts.iter().any(|contract|contract.kind==CallbackKind::Task) && !self.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Job)) {return Vec::new();}
        let mut tracker=Tracker{program:self,warnings:BTreeMap::new(),failures:Vec::new(),projections:0};
        for function in &self.functions { if !function.template {tracker.function(function, &[],None,Mode::Consume);} }
        for closure in &self.closures {
            if closure.forward.is_some(){continue;}
            if !closure.polymorphic {
                if !closure.function.template{tracker.function(&closure.function,&closure.captures,Some(&closure.parameters),closure.invocation);}
            }else{
                // A polymorphic declaration retains an empty symbolic body.
                // Responsibilities belong to its actual checked invocation bodies.
                for method in &closure.methods {
                    if method.function.template{continue;}
                    let captures=closure.captures.iter().zip(&method.captures).map(|(capture,&inner)|Capture{restored:capture.restored,outer:capture.outer,inner,storage:capture.storage,access:capture.access,span:capture.span}).collect::<Vec<_>>();
                    tracker.function(&method.function,&captures,Some(&method.parameters),method.invocation);
                }
            }
        }
        tracker.warnings.into_values().collect()
    }
}
fn place(value:&Expr)->Option<Place> {
    match &value.kind {
        ExprKind::Name(_)=>value.binding.map(|id|(id,Vec::new())),
        ExprKind::Field(base,name)=>{let(id,mut path)=place(base)?;path.push(name.clone());Some((id,std::mem::take(&mut path)))},
        ExprKind::Index(base,index)=>{let(id,mut path)=place(base)?;if let ExprKind::Integer(text)=&index.kind {path.push(text.clone());}else{path.push("*".into());}Some((id,path))},
        // Await evaluates and consumes its operand; its result is not storage
        // selected from the task binding (including through field/index syntax).
        ExprKind::Unary{op:UnaryOp::Await,..}=>None,
        ExprKind::Unary{value,..}|ExprKind::Project{value,..}|ExprKind::TypeAssertion{value,..}=>place(value),
        _=>None,
    }
}
// Resolve only checked single-return forwarding. An arbitrary iterator whose
// callback can return done before polling its owner is deliberately not a leaf.
impl CheckedProgram{
 fn call_argument_mode(&self,value:&Expr,index:usize)->Option<Mode>{
  if let Some(plan)=value.generic_invocation{return self.polymorphic_invocations.checked(plan,&self.closures,value.span).ok()?.argument_modes.get(index).copied();}
  match value.target{
   Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=>return self.functions[id].bindings.get(index).map(|binding|binding.mode),
   Some(CallTarget::NativeBinding(id))=>return crate::native_bindings::BINDINGS[id].parameters.get(index).map(|parameter|parameter.access),
   Some(CallTarget::IteratorCursor)=>return Some(Mode::Consume),Some(CallTarget::IteratorNext)=>return Some(Mode::Mutate),_=>{}
  }
  let ExprKind::Call(callee,args)=&value.kind else{return None;};
  let callable=callee.ty.or_else(||match value.target{
   Some(CallTarget::Closure(_))=>value.closure_origin().map(Type::Closure),
   Some(CallTarget::ConditionalClosure(origin))=>Some(Type::Closure(origin)),
   _=>None,
  })?;
  match callable{
   Type::Callback(id)=>self.callback_contracts[id].params.get(index).map(|parameter|parameter.mode),
   Type::StoredCallable(id)|Type::BorrowedCallable(id)=>self.stored_contracts.declaration(id).and_then(|id|self.callback_contracts[id].params.get(index)).map(|parameter|parameter.mode),
   Type::Closure(id)=>{let closure=&self.closures[id];match closure.forward{
    Some(ClosureForward::Function(id))=>self.functions[id].bindings.get(index).map(|binding|binding.mode),
    Some(ClosureForward::Native(id))=>crate::native_bindings::BINDINGS[id].parameters.get(index).map(|parameter|parameter.access),
    _=>if closure.methods.is_empty(){closure.parameters.get(index).map(|&slot|closure.function.bindings[slot].mode)}else{
     let mut modes=closure.methods.iter().filter(|method|method.parameters.len()==args.len()).filter_map(|method|method.parameters.get(index).map(|&slot|method.function.bindings[slot].mode));let first=modes.next()?;modes.all(|mode|mode==first).then_some(first)
    }
   }},_=>None,
  }
 }
 fn cursor_poll_body<'a>(&self,function:&'a CheckedFunction)->Option<(&'a Expr,bool)>{
  fn transparent(mut value:&Expr)->&Expr{loop{match &value.kind{ExprKind::Inject{value:inner,..}|ExprKind::Project{value:inner,..}|ExprKind::TypeAssertion{value:inner,..}=>value=inner,ExprKind::Unary{op:UnaryOp::RecordCarrier|UnaryOp::CallableStore,value:inner,..}=>value=inner,_=>return value}}}
  fn record(value:&Expr,done:bool)->Option<&[(Token,Expr)]>{let ExprKind::Record(fields)=&transparent(value).kind else{return None;};fields.iter().any(|(name,value)|name.text=="done"&&matches!(transparent(value).kind,ExprKind::Boolean(actual)if actual==done)).then_some(fields.as_slice())}
  if let [statement]=function.body.as_slice(){let StmtKind::Return(Some(value))=&statement.kind else{return None;};return Some((value,false));}
  // A mapped producer can transform the yielded value, but can end ONLY on
  // the actual upstream poll's end branch. No other return path is admitted.
  let [first,guard,last]=function.body.as_slice()else{return None;};
  let StmtKind::Let{binding:Some(step),value:poll,..}=&first.kind else{return None;};
  let StmtKind::If{condition,then_body,else_body}=&guard.kind else{return None;};if !else_body.is_empty(){return None;}
  let(base,field,truth)=boolean_field_guard(condition)?;if field!="done"||!truth||place(base)!=Some((*step,vec![])){return None;}
  let [end]=then_body.as_slice()else{return None;};let StmtKind::Return(Some(end))=&end.kind else{return None;};if record(end,true)?.len()!=1{return None;}
  let StmtKind::Return(Some(some))=&last.kind else{return None;};let fields=record(some,false)?;if fields.len()!=2{return None;}
  let value=transparent(&fields.iter().find(|(name,_)|name.text=="value")?.1);let ExprKind::Call(_,args)=&value.kind else{return None;};
  if args.len()!=1||place(&args[0])!=Some((*step,vec!["value".into()])){return None;}
  let consumes=self.call_argument_mode(value,0).is_some_and(|mode|matches!(mode,Mode::Copy|Mode::Consume));
  consumes.then_some((poll,true))
 }
 fn cursor_poll_sources(&self,function:&CheckedFunction,expression:&Expr)->Option<CursorPollSources>{
  type Mapping=BTreeMap<usize,Place>;
  fn mapped(value:&Expr,mapping:&Mapping)->Option<Place>{let source=place(value)?;let root=mapping.get(&source.0)?;if source.1.iter().any(|part|part=="*"){return None;}let mut path=root.1.clone();path.extend(source.1);Some((root.0,path))}
  let identity=(0..function.bindings.len()).map(|id|(id,(id,Vec::new()))).collect::<Mapping>();
  let mut pending=vec![(expression,identity,HashSet::<(bool,usize)>::new())];let mut cursors=Vec::new();let mut transformed_any=false;
  while let Some((mut value,mapping,mut seen))=pending.pop(){
   loop{match &value.kind{ExprKind::Project{value:inner,..}|ExprKind::TypeAssertion{value:inner,..}|ExprKind::Inject{value:inner,..}=>value=inner,ExprKind::Unary{op:UnaryOp::RecordCarrier|UnaryOp::CallableStore|UnaryOp::NonNull,value:inner,..}=>value=inner,_=>break}}
   let ExprKind::Call(callee,args)=&value.kind else{return None;};
   if value.target==Some(CallTarget::IteratorNext){
    let [argument]=args.as_slice()else{return None;};
    if !matches!(argument.ty,Some(Type::Record(id))if self.records[id].native_type==Some(crate::native_signatures::NativeTypeId::IteratorCursor)){return None;}
    let mut source=mapped(argument,&mapping)?;source.1.push("_items".into());cursors.push(source);continue;
   }
   if let Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=value.target{
    if !seen.insert((false,id)){return None;}let callee=&self.functions[id];let(body,transformed)=self.cursor_poll_body(callee)?;transformed_any|=transformed;
    let mut inputs=Mapping::new();for(index,arg)in args.iter().enumerate(){if let Some(actual)=mapped(arg,&mapping){inputs.insert(index,actual);}}
    pending.push((body,inputs,seen));continue;
   }
   let origins=match callee.ty?{Type::Closure(id)=>vec![id],Type::StoredCallable(family)|Type::BorrowedCallable(family)=>self.stored_contracts.populated().find(|(id,_,_)|*id==family)?.2.to_vec(),_=>return None};
   if origins.is_empty(){return None;}let owner=mapped(callee,&mapping)?;
   for origin in origins{
    let mut seen=seen.clone();if !seen.insert((true,origin)){return None;}let closure=&self.closures[origin];
    if let Some(ClosureForward::Function(id))=closure.forward{
     if !seen.insert((false,id)){return None;}let(body,transformed)=self.cursor_poll_body(&self.functions[id])?;transformed_any|=transformed;let mut inputs=Mapping::new();for(index,arg)in args.iter().enumerate(){if let Some(actual)=mapped(arg,&mapping){inputs.insert(index,actual);}}pending.push((body,inputs,seen));continue;
    }
    if closure.forward.is_some(){return None;}
    if closure.methods.is_empty(){
     let mut inputs=Mapping::new();for capture in &closure.captures{if capture.storage==Mode::Consume{let mut path=owner.1.clone();path.push(format!("capture{}",capture.inner));inputs.insert(capture.inner,(owner.0,path));}}
     for(index,&slot)in closure.parameters.iter().enumerate(){if let Some(arg)=args.get(index).and_then(|arg|mapped(arg,&mapping)){inputs.insert(slot,arg);}}
     let(body,transformed)=self.cursor_poll_body(&closure.function)?;transformed_any|=transformed;pending.push((body,inputs,seen));
    }else{
     let mut found=false;for method in &closure.methods{if method.parameters.len()!=args.len(){continue;}found=true;
      let mut inputs=Mapping::new();for(capture,&slot)in closure.captures.iter().zip(&method.captures){if capture.storage==Mode::Consume{let mut path=owner.1.clone();path.push(format!("capture{}",capture.inner));inputs.insert(slot,(owner.0,path));}}
      for(index,&slot)in method.parameters.iter().enumerate(){if let Some(arg)=args.get(index).and_then(|arg|mapped(arg,&mapping)){inputs.insert(slot,arg);}}
      let(body,transformed)=self.cursor_poll_body(&method.function)?;transformed_any|=transformed;pending.push((body,inputs,seen.clone()));
     }if !found{return None;}
    }
   }
  }
  cursors.sort();cursors.dedup();(!cursors.is_empty()).then_some(CursorPollSources{cursors,mapped:transformed_any})
 }
}
fn boolean_field_guard(value:&Expr)->Option<(&Expr,&str,bool)>{match &value.kind{
 ExprKind::Field(base,name)=>Some((base,name,true)),
 ExprKind::Unary{op:UnaryOp::Not,value,..}=>boolean_field_guard(value).map(|(base,name,yes)|(base,name,!yes)),
 ExprKind::Binary{op:BinaryOp::Equal|BinaryOp::NotEqual,left,right,..}=>{
  let(observed,literal)=if let ExprKind::Boolean(value)=right.kind{(left.as_ref(),value)}else if let ExprKind::Boolean(value)=left.kind{(right.as_ref(),value)}else{return None;};
  let(base,name,yes)=boolean_field_guard(observed)?;Some((base,name,if matches!(value.kind,ExprKind::Binary{op:BinaryOp::Equal,..})==literal{yes}else{!yes}))
 },_=>None,
}}
impl Tracker<'_> {
    fn warn(&mut self,tokens:Tokens,at:Span) {
        for(start,end,started)in tokens {
            self.warnings.entry((start,end,at.start)).or_insert_with(||Diagnostic::new(if started{"unsettled-job"}else{"unused-task"},if started{"Started work must be awaited or transferred to an owner that settles it. A cancellation request alone does not account for cleanup."}else{"This lazy task can be discarded without running. Await it, return it, or transfer it to an owner that handles it."},at).related(Span::new(start,end),"Task responsibility begins here."));
        }
    }
    fn take(state:&mut State,selected:&Place)->Tokens {
        invalidate(state,selected);
        let keys=state.keys().filter(|(root,path)|*root==selected.0 && path.starts_with(&selected.1)).cloned().collect::<Vec<_>>();
        let mut tokens=Tokens::new();for key in keys {tokens.extend(state.remove(&key).unwrap());}tokens
    }
    fn transfer(state:&mut State,source:&Place,destination:&Place) {
        let polls=std::mem::take(&mut state.polls);
        for(result,mut poll)in polls{
            if overlaps(&result,destination)||poll.cursors.iter().any(|cursor|overlaps(cursor,destination)){continue;}
            let result=if within(&result,source){rebased(&result,source,destination)}else{result};
            for cursor in &mut poll.cursors{if within(cursor,source){*cursor=rebased(cursor,source,destination);}}
            for item in &mut poll.selected{if within(item,source){*item=rebased(item,source,destination);}}
            state.polls.insert(result,poll);
        }
        let selected=state.keys().filter(|(root,path)|*root==source.0 && path.starts_with(&source.1)).cloned().collect::<Vec<_>>();
        for place in selected {let tokens=state.remove(&place).unwrap();let mut path=destination.1.clone();path.extend_from_slice(&place.1[source.1.len()..]);state.entry((destination.0,path)).or_default().extend(tokens);}
    }
    fn evaluate_place(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State){
        enum Work<'a>{Selection(&'a Expr),Index(&'a Expr),Failure(&'a Expr)}
        let mut pending=vec![Work::Selection(value)];
        while let Some(work)=pending.pop(){match work{
            Work::Selection(value)=>{
                pending.push(Work::Failure(value));
                match &value.kind{
                    ExprKind::Field(base,_)=>pending.push(Work::Selection(base)),
                    ExprKind::Index(base,index)=>{pending.push(Work::Index(index));pending.push(Work::Selection(base));},
                    ExprKind::Unary{value,..}|ExprKind::Project{value,..}|ExprKind::TypeAssertion{value,..}=>pending.push(Work::Selection(value)),
                    _=>{},
                }
            },
            Work::Index(index)=>{let tokens=self.value(function,index,state,false);self.warn(tokens,index.span);},
            Work::Failure(value)=>if value.own_may_fail{self.failures.push(state.clone());},
        }}
    }
    fn absence_branches(condition:&Expr,state:State)->(State,State){
        let mut yes=state.clone();let mut no=state;
        if let ExprKind::Binary{op:BinaryOp::Equal|BinaryOp::NotEqual,left,right,..}=&condition.kind {
            // Only literal sentinels and exact stable storage are selected.
            // A dynamic array index must never erase other entries' owners.
            let exact=|value:&Expr|{
                let mut observed=value;
                loop{match &observed.kind{
                    ExprKind::Name(_)=>break,
                    ExprKind::Field(base,_)|ExprKind::Index(base,_)=>observed=base,
                    ExprKind::Project{value,..}|ExprKind::TypeAssertion{value,..}|ExprKind::Unary{op:UnaryOp::NonNull,value,..}=>observed=value,
                    _=>return None,
                }}
                place(value)
            };
            let selected=if matches!(right.kind,ExprKind::Null|ExprKind::Undefined){exact(left)}else if matches!(left.kind,ExprKind::Null|ExprKind::Undefined){exact(right)}else{None};
            if let Some(selected)=selected.filter(|(_,path)|!path.iter().any(|part|part=="*")){
                let absent=if matches!(condition.kind,ExprKind::Binary{op:BinaryOp::Equal,..}){&mut yes}else{&mut no};
                let _=Self::take(absent,&selected);
            }
        }
        (yes,no)
    }
    fn branches(condition:&Expr,state:State)->(State,State){
        let relation=boolean_field_guard(condition).filter(|(_,name,_)|*name=="done").and_then(|(base,_,done)|place(base).and_then(|result|state.polls.get(&result).cloned().map(|poll|(result,done,poll))));
        let(mut yes,mut no)=Self::absence_branches(condition,state);
        if let Some((result,done,poll))=relation{
            let(absent,present)=if done{(&mut yes,&mut no)}else{(&mut no,&mut yes)};
            let mut payload=result.clone();payload.1.push("value".into());let _=Self::take(absent,&payload);
            for cursor in &poll.cursors{let _=Self::take(absent,cursor);}
            for selected in &poll.selected{let _=Self::take(present,selected);}
            absent.polls.remove(&result);present.polls.remove(&result);
        }
        (yes,no)
    }
    fn checked_branches(&self,condition:&Expr,state:State)->(State,State){
        // Absence of a payload follows the actual closed union's discriminant,
        // independently of whether its producer exhausts an upstream owner.
        let refinement=boolean_field_guard(condition).and_then(|(base,field,yes)|{
            let owner=place(base)?;if owner.1.iter().any(|part|part=="*"){return None;}
            let members=match base.ty?{Type::Union(id)|Type::BorrowedUnion(id)=>self.program.unions[id.index()].members.clone(),Type::Record(id)=>vec![Type::Record(id)],_=>return None};
            let mut alternatives=Vec::new();for member in members{
                let Type::Record(record)=member else{return None;};let tag=self.program.records[record].fields.iter().find(|item|item.name.text==field)?.ty?;
                let Type::Literal(literal,crate::literal_types::LiteralKind::Boolean)=tag else{return None;};
                let crate::literal_types::LiteralValue::Boolean(tag)=self.program.records[literal].literal_definition.as_ref()? else{return None;};
                alternatives.push((*tag,self.program.task_paths(member)));
            }
            Some((owner,yes,alternatives))
        });
        let(mut yes,mut no)=Self::branches(condition,state);
        if let Some((owner,truth,alternatives))=refinement{for(branch,wanted)in [(&mut yes,truth),(&mut no,!truth)]{
            let selected=alternatives.iter().filter(|(tag,_)|*tag==wanted).collect::<Vec<_>>();if selected.is_empty(){continue;}
            let absent=branch.keys().filter(|place|within(place,&owner)).filter(|place|{
                let relative=&place.1[owner.1.len()..];!selected.iter().any(|(_,paths)|paths.iter().any(|(path,_)|path.iter().zip(relative).all(|(expected,actual)|expected==actual||expected=="*"||actual=="*")))
            }).cloned().collect::<Vec<_>>();
            for place in absent{let _=Self::take(branch,&place);}
        }}
        (yes,no)
    }
    fn poll(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State,plan:CursorPollSources)->(BTreeMap<Vec<String>,Tokens>,Poll){
        let CursorPollSources{cursors,mapped}=plan;
        let ExprKind::Call(callee,args)=&value.kind else{unreachable!("checked cursor poll")};
        self.evaluate_place(function,callee,state);
        for(index,arg)in args.iter().enumerate(){
            let access=self.program.call_argument_mode(value,index);
            let tokens=self.value(function,arg,state,access.is_none_or(|mode|mode==Mode::Consume));self.warn(tokens,arg.span);
        }
        let mut payload=BTreeMap::<Vec<String>,Tokens>::new();let mut selected=Vec::new();
        for cursor in &cursors{
            invalidate(state,cursor);
            let first=state.keys().filter(|place|within(place,cursor)).filter_map(|place|place.1.get(cursor.1.len())?.parse::<usize>().ok().map(|index|(index,place))).min_by_key(|(index,_)|*index).map(|(_,place)|place.1[cursor.1.len()].clone());
            if let Some(first)=first{let mut item=cursor.clone();item.1.push(first);for(place,owned)in &state.owners{if within(place,&item){payload.entry(place.1[item.1.len()..].to_vec()).or_default().extend(owned.iter().copied());}}selected.push(item);}
            // A wildcard denotes remaining elements of unknown cardinality. Each
            // Some yields one obligation; only this leaf's None proves no residual.
            for(place,owned)in &state.owners{if within(place,cursor)&&place.1.get(cursor.1.len()).is_some_and(|part|part=="*"){payload.entry(place.1[cursor.1.len()+1..].to_vec()).or_default().extend(owned.iter().copied());}}
        }
        if mapped && let Some(result)=value.ty{
            payload.clear();
            for(path,started)in self.program.task_paths(result){if path.first().is_some_and(|field|field=="value"){payload.entry(path[1..].to_vec()).or_default().insert((value.span.start,value.span.end,started));}}
        }
        (payload,Poll{cursors,selected})
    }
    fn store(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State,destination:Place) {
        #[cfg(test)]
        if std::thread::current().name().is_some_and(|name|name.ends_with("public_array_iterator_drains_transfer_lazy_and_started_owners_to_all")) && function.name.contains("toArray") {
            let callee=if let ExprKind::Call(callee,_)=&value.kind{Some((callee.ty,callee.generic_invocation))}else{None};
            let sources=self.program.cursor_poll_sources(function,value).map(|plan|(plan.cursors,plan.mapped));
            eprintln!("ITERATOR_DRAIN function={} span={:?} target={:?} ty={:?} callee={:?} own_fail={} sources={:?} owners={:?}",function.name,value.span,value.target,value.ty,callee,value.own_may_fail,sources,state.owners);
        }
        if value.target==Some(CallTarget::IteratorCursor){
            let ExprKind::Call(_,args)=&value.kind else{unreachable!("checked cursor creation")};let mut items=destination;items.1.push("_items".into());self.store(function,&args[0],state,items);if value.own_may_fail{self.failures.push(state.clone());}return;
        }
        if let Some(cursors)=self.program.cursor_poll_sources(function,value){
            let(payload,poll)=self.poll(function,value,state,cursors);if value.own_may_fail{self.failures.push(state.clone());}for(path,tokens)in payload{let mut item=destination.clone();item.1.push("value".into());item.1.extend(path);if !tokens.is_empty(){state.entry(item).or_default().extend(tokens);}}state.polls.insert(destination,poll);return;
        }
        if let Some(source)=place(value){self.evaluate_place(function,value,state);Self::transfer(state,&source,&destination);return;}
        match &value.kind {
            ExprKind::Record(fields)=>for(field,value)in fields{let mut child=destination.clone();child.1.push(field.text.clone());self.store(function,value,state,child);},
            ExprKind::Array(values)=>for(index,value)in values.iter().enumerate(){let mut child=destination.clone();child.1.push(index.to_string());self.store(function,value,state,child);},
            ExprKind::Inject{value,..}|ExprKind::TypeAssertion{value,..}=>self.store(function,value,state,destination),
            ExprKind::Unary{op,value:inner,..} if *op!=UnaryOp::Await && !value.ty.is_some_and(|ty|self.program.task_type(ty))=>self.store(function,inner,state,destination),
            _=>{let tokens=self.value(function,value,state,true);let paths=value.ty.map(|ty|self.program.task_paths(ty)).unwrap_or_default();
                if paths.is_empty(){state.entry(destination).or_default().extend(tokens);}else{for (path,_) in paths {let mut child=destination.clone();child.1.extend(path);state.entry(child).or_default().extend(tokens.iter().copied());}}}

        }
    }
    fn replace(&mut self,function:&CheckedFunction,target:&Expr,value:&Expr,state:&mut State) {
        let Some(destination)=place(target)else{let _=self.value(function,value,state,true);return;};
        self.evaluate_place(function,target,state);
        let temporary=(usize::MAX,Vec::new());self.store(function,value,state,temporary.clone());
        let old=Self::take(state,&destination);self.warn(old,target.span);Self::transfer(state,&temporary,&destination);
    }
    fn function(&mut self,function:&CheckedFunction,captures:&[Capture],parameters:Option<&[usize]>,invocation:Mode) {
        if function.bindings.iter().all(|binding|self.program.task_paths(binding.ty).is_empty()) && crate::error_effects::checked_expressions(function).iter().all(|value|value.ty.is_none_or(|ty|self.program.task_paths(ty).is_empty())) {return;}
        let mut state=State::new();
        for(id,binding)in function.bindings.iter().enumerate() {
            // Closure binding vectors also contain unused lexical projections, not parameters.
            let parameter=parameters.map_or(id<function.parameter_count,|slots|slots.contains(&id));
            // Ordinary Task parameters remain owned even when a physical
            // inferred signature has no callback marker. Only a stated readonly
            // Task value is borrowed; its caller keeps the responsibility.
            let borrowed_task=binding.mode==Mode::Read&&binding.callback_kind==Some(CallbackKind::Task);
            if (parameter && (binding.mode==Mode::Consume || self.program.task_type(binding.ty)&&!borrowed_task)) || captures.iter().any(|capture|capture.inner==id && capture.storage==Mode::Consume) {
                let at=binding.declaration_span.or(binding.used).unwrap_or_default();
                for (path,started) in self.program.task_paths(binding.ty){state.insert((id,path),Tokens::from([(at.start,at.end,started)]));}
            }
        }
        for mut path in self.body(function,&function.body,state) {
            // A borrowed invocation returns its environment to its caller. Its
            // remaining captured owners are retained, never implicitly settled.
            if invocation!=Mode::Consume{for capture in captures{let _=Self::take(&mut path.state,&(capture.inner,Vec::new()));}}
            for(_,tokens)in path.state {self.warn(tokens,function.body.last().map_or(Span::default(),|stmt|stmt.span));}
        }
    }
    fn projection(&mut self,function:&CheckedFunction,value:&Expr,base:&Expr,index:Option<&Expr>,member:String,state:&mut State,consume:bool)->Tokens {
        // A projection of an owned rvalue still evaluates its base (including
        // await) before its index, and can abandon unselected task-bearing data.
        let temporary=(usize::MAX-2,vec![self.projections.to_string()]);self.projections+=1;
        self.store(function,base,state,temporary.clone());
        if let Some(index)=index{let _=self.value(function,index,state,false);}
        // Bounds/own projection failure occurs while the temporary still owns
        // its contents; preserve that path before selecting the successful value.
        if value.own_may_fail{self.failures.push(state.clone());}
        let mut selected=temporary.clone();selected.1.push(member);
        let tokens=Self::take(state,&selected);let remaining=Self::take(state,&temporary);self.warn(remaining,value.span);
        if consume{tokens}else{self.warn(tokens,value.span);Tokens::new()}
    }
    fn value(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State,consume:bool)->Tokens {
        let tokens=self.value_inner(function,value,state,consume);
        if value.own_may_fail && place(value).is_none() {self.failures.push(state.clone());}
        tokens
    }
    fn value_inner(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State,consume:bool)->Tokens {
        if let ExprKind::Unary{op:UnaryOp::Void|UnaryOp::Typeof,value,..}=&value.kind {let _=self.value(function,value,state,false);return Tokens::new();}
        if let Some(selected)=place(value) {self.evaluate_place(function,value,state);return if consume {Self::take(state,&selected)}else{Tokens::new()};}
        match &value.kind {
            ExprKind::Field(base,member)=>self.projection(function,value,base,None,member.clone(),state,consume),
            ExprKind::Index(base,index)=>{
                let member=if let ExprKind::Integer(text)=&index.kind{text.clone()}else{"*".into()};
                self.projection(function,value,base,Some(index),member,state,consume)
            },
            ExprKind::Unary{op:UnaryOp::Await,value:operand,..}=>{
                let _=self.value(function,operand,state,true);
                if value.ty.is_some_and(|ty|!self.program.task_paths(ty).is_empty()) {Tokens::from([(value.span.start,value.span.end,value.ty.is_some_and(|ty|self.program.contains_started_job(ty)))])}else{Tokens::new()}
            },
            ExprKind::Unary{op:UnaryOp::TaskCreate,value:inner,..}=>{
                let mut tokens=self.value(function,inner,state,true);tokens.insert((value.span.start,value.span.end,value.ty.is_some_and(|ty|self.program.contains_started_job(ty))));tokens
            }
            ExprKind::Closure{..}=>{
                let mut tokens=Tokens::new();
                if let Some(Type::Closure(origin))=value.ty && let Some(closure)=self.program.closures.get(origin) {for capture in &closure.captures {if capture.storage==Mode::Consume {tokens.extend(Self::take(state,&(capture.outer,Vec::new())));}}}
                tokens
            }
            ExprKind::Call(callee,args)=>{
                if let Some(cursors)=self.program.cursor_poll_sources(function,value){let(payload,_)=self.poll(function,value,state,cursors);return payload.into_values().flatten().collect();}
                if let ExprKind::Field(base,method)=&callee.kind && let Some(owner)=place(base) {
                    if matches!(method.as_str(),"pop"|"shift"|"push"|"unshift"){self.evaluate_place(function,base,state);}
                    if matches!(method.as_str(),"pop"|"shift") {
                        let selected=state.keys().filter(|(root,path)|*root==owner.0 && path.starts_with(&owner.1) && path.len()>owner.1.len()).filter_map(|place|place.1[owner.1.len()].parse::<usize>().ok().map(|index|(index,place.clone())));
                        let selected=if method=="pop"{selected.max_by_key(|(index,_)|*index)}else{selected.min_by_key(|(index,_)|*index)};
                        return selected.map_or_else(|| {
                            if value.ty.is_some_and(|ty|self.program.contains_started_job(ty)) {Tokens::from([(value.span.start,value.span.end,true)])}else{Tokens::new()}
                        },|(_,selected)|Self::take(state,&selected));
                    }
                    if method=="splice" && (base.ty.is_some_and(|ty|self.program.contains_started_job(ty)) || args.iter().any(|arg|arg.ty.is_some_and(|ty|self.program.contains_started_job(ty)))) {
                        // shortcut: splice can discard or replace jobs, support only after tracking each removed owner.
                        self.warn(Tokens::from([(value.span.start,value.span.end,true)]),value.span);
                    }
                    if matches!(method.as_str(),"push"|"unshift") {
                        if method=="unshift" {
                            let selected=state.keys().filter(|(root,path)|*root==owner.0 && path.starts_with(&owner.1) && path.get(owner.1.len()).is_some_and(|part|part.parse::<usize>().is_ok())).cloned().collect::<Vec<_>>();
                            let moved=selected.into_iter().map(|key|{let tokens=state.remove(&key).unwrap();let mut next=key;let index=next.1[owner.1.len()].parse::<usize>().unwrap();next.1[owner.1.len()]=(index+args.len()).to_string();(next,tokens)}).collect::<Vec<_>>();
                            state.extend(moved);
                        }
                        let mut index=state.keys().filter(|(root,path)|*root==owner.0 && path.starts_with(&owner.1)).filter_map(|(_,path)|path.get(owner.1.len()).and_then(|part|part.parse::<usize>().ok())).max().map_or(0,|index|index+1);
                        if method=="unshift" {index=0;}
                        for arg in args {let tokens=self.value(function,arg,state,true);let mut path=owner.1.clone();path.push(index.to_string());state.insert((owner.0,path),tokens);index+=1;}return Tokens::new();
                    }
                }
                let awaited=matches!(callee.kind,ExprKind::Unary{op:UnaryOp::AwaitCallee,..});
                let consumes=awaited || callee.ty.is_some_and(|ty|match ty {Type::Closure(id)=>self.program.closures[id].invocation==Mode::Consume,Type::Callback(id)=>self.program.callback_contracts[id].kind.is_once(),_=>false});
                if let Some(owner)=place(callee){invalidate(state,&owner);}
                let mut tokens=self.value(function,callee,state,consumes);
                for(index,arg)in args.iter().enumerate() {
                    let access=self.program.call_argument_mode(value,index);
                    if access!=Some(Mode::Read){if let Some(owner)=place(arg){invalidate(state,&owner);}}
                    let first_failure=self.failures.len();
                    let argument=self.value(function,arg,state,access.is_none_or(|mode|mode==Mode::Consume));
                    // Earlier moved arguments are still temporaries if a later argument throws.
                    for failure in &mut self.failures[first_failure..] {failure.entry((usize::MAX-1,Vec::new())).or_default().extend(tokens.iter().copied());}
                    tokens.extend(argument);
                }
                if awaited {return Tokens::new();}
                if value.ty.is_some_and(|ty|!self.program.task_paths(ty).is_empty()) {if tokens.is_empty() || value.ty.is_some_and(|ty|self.program.task_type(ty) || self.program.contains_started_job(ty)){tokens.insert((value.span.start,value.span.end,value.ty.is_some_and(|ty|self.program.contains_started_job(ty))));}tokens}else{Tokens::new()}
            }
            ExprKind::Assign{target,value,..}=>{self.replace(function,target,value,state);Tokens::new()}
            ExprKind::Conditional{predicate,consequent,alternate}=>{
                let _=self.value(function,predicate,state,false);
                let mut other=state.clone();let mut tokens=self.value(function,consequent,state,consume);tokens.extend(self.value(function,alternate,&mut other,consume));merge(state,other);tokens
            }
            ExprKind::Record(fields)=>{let mut tokens=Tokens::new();for(_,value)in fields {tokens.extend(self.value(function,value,state,consume));}tokens},
            ExprKind::Sequence(values)=>{
                let mut result=Tokens::new();for(index,value)in values.iter().enumerate(){let tokens=self.value(function,value,state,consume);if index+1==values.len(){result=tokens;}else{self.warn(tokens,value.span);}}result
            }
            ExprKind::Array(values)|ExprKind::Template(values)=>{
                let mut tokens=Tokens::new();for value in values {tokens.extend(self.value(function,value,state,consume));}tokens
            }
            ExprKind::Unary{value,..}|ExprKind::Inject{value,..}|ExprKind::Project{value,..}|ExprKind::TypeAssertion{value,..}=>self.value(function,value,state,consume),
            ExprKind::Binary{left,right,..}=>{let _=self.value(function,left,state,false);let _=self.value(function,right,state,false);Tokens::new()},
            ExprKind::Match{body,..}=>{let mut paths=self.body(function,body,state.clone()).into_iter();let mut result=paths.next().map_or_else(State::new,|path|path.state);for path in paths{merge(&mut result,path.state);}*state=result;if value.ty.is_some_and(|ty|!self.program.task_paths(ty).is_empty()){Tokens::from([(value.span.start,value.span.end,value.ty.is_some_and(|ty|self.program.contains_started_job(ty)))])}else{Tokens::new()}},
            _=>Tokens::new(),
        }
    }
    fn body(&mut self,function:&CheckedFunction,body:&[Stmt],state:State)->Vec<Path> {
        let locals=body.iter().filter_map(|stmt|if let StmtKind::Let{binding:Some(id),..}=stmt.kind {Some(id)}else{None}).collect::<Vec<_>>();
        let mut paths=vec![Path{state,exit:Exit::Next}];
        for stmt in body {
            let mut next=Vec::new();
            for path in paths {if path.exit!=Exit::Next {next.push(path);}else{next.extend(self.statement(function,stmt,path.state));}}
            paths=coalesce(next);
        }
        for path in &mut paths {for &id in &locals {let tokens=Self::take(&mut path.state,&(id,Vec::new()));self.warn(tokens,body.last().map_or(Span::default(),|stmt|stmt.span));}}
        paths
    }
    fn statement(&mut self,function:&CheckedFunction,stmt:&Stmt,state:State)->Vec<Path> {
        let outer=std::mem::take(&mut self.failures);
        let mut paths=self.statement_inner(function,stmt,state);
        paths.extend(std::mem::replace(&mut self.failures,outer).into_iter().map(|state|Path{state,exit:Exit::Throw}));
        coalesce(paths)
    }
    fn while_paths(&mut self,function:&CheckedFunction,condition:&Expr,body:&[Stmt],entry:State)->Vec<Path>{
        let mut head=entry;let mut exits=Vec::new();
        loop{
            let mut observed=head.clone();let _=self.value(function,condition,&mut observed,false);
            let(yes,no)=self.checked_branches(condition,observed);
            if !matches!(condition.kind,ExprKind::Boolean(true)){exits.push(Path{state:no,exit:Exit::Next});}
            if matches!(condition.kind,ExprKind::Boolean(false)){break;}
            let mut back=None;
            for mut path in self.body(function,body,yes){match path.exit{
                Exit::Break=>{path.exit=Exit::Next;exits.push(path);},
                Exit::Next|Exit::Continue=>{widen(&mut path.state,function,self.program);if let Some(state)=&mut back{merge(state,path.state);}else{back=Some(path.state);}},
                Exit::Return|Exit::Throw=>exits.push(path),
            }}
            let Some(back)=back else{break;};let mut next=head.clone();merge(&mut next,back);widen(&mut next,function,self.program);if next==head{break;}head=next;
        }
        coalesce(exits)
    }
    fn statement_inner(&mut self,function:&CheckedFunction,stmt:&Stmt,mut state:State)->Vec<Path> {
        let exit=match &stmt.kind {
            StmtKind::Let{binding:Some(id),value,..}=>{self.store(function,value,&mut state,(*id,Vec::new()));Exit::Next}
            StmtKind::Expr(value)=>{let tokens=self.value(function,value,&mut state,true);self.warn(tokens,stmt.span);Exit::Next},
            StmtKind::Assign{target,value,..}=>{self.replace(function,target,value,&mut state);Exit::Next}
            StmtKind::Return(value)=>{if let Some(value)=value {let _=self.value(function,value,&mut state,true);}Exit::Return},
            StmtKind::Throw(value)=>{let _=self.value(function,value,&mut state,true);Exit::Throw},
            StmtKind::Break=>Exit::Break,
            StmtKind::Continue=>Exit::Continue,
            StmtKind::Block(body)=>return self.body(function,body,state),
            StmtKind::If{condition,then_body,else_body}=>{
                let _=self.value(function,condition,&mut state,false);
                if let ExprKind::Boolean(value)=condition.kind {return self.body(function,if value {then_body}else{else_body},state);}
                let (yes,no)=self.checked_branches(condition,state);
                let mut paths=self.body(function,then_body,yes);paths.extend(self.body(function,else_body,no));return coalesce(paths);
            }
            StmtKind::While{condition,body}=>return self.while_paths(function,condition,body,state),
            StmtKind::Loop{body,..}|StmtKind::ForOf{body,..}=>{
                let mut paths=self.body(function,body,state.clone());for path in &mut paths {if matches!(path.exit,Exit::Break|Exit::Continue){path.exit=Exit::Next;}}
                paths.push(Path{state,exit:Exit::Next});return coalesce(paths);
            }
            StmtKind::Try(region)=>{
                let mut paths=self.body(function,&region.body,state);
                if let Some(handler)=&region.catch {
                    let mut caught=None;paths.retain(|path|{if path.exit==Exit::Throw{if let Some(caught)=&mut caught{merge(caught,path.state.clone());}else{caught=Some(path.state.clone());}false}else{true}});
                    if let Some(caught)=caught {paths.extend(self.body(function,&handler.body,caught));}
                }
                if let Some(cleanup)=&region.finally {
                    let mut completed=Vec::new();for path in paths {for mut after in self.body(function,cleanup,path.state){if after.exit==Exit::Next {after.exit=path.exit;}completed.push(after);}}paths=completed;
                }
                return coalesce(paths);
            }
            _=>Exit::Next,
        };
        vec![Path{state,exit}]
    }
}
fn merge(target:&mut State,other:State){target.polls.retain(|result,poll|other.polls.get(result)==Some(poll));for(place,tokens)in other{target.entry(place).or_default().extend(tokens);}}
fn coalesce(paths:Vec<Path>)->Vec<Path>{let mut result:Vec<Path>=Vec::new();for path in paths{if let Some(existing)=result.iter_mut().find(|existing|existing.exit==path.exit){merge(&mut existing.state,path.state);}else{result.push(path);}}result}


#[cfg(test)]
mod tests {
    use super::*;
    fn indexed(index:Expr)->Expr{let mut owner=Expr::new(ExprKind::Name("owners".into()),Span::default());owner.binding=Some(0);Expr::new(ExprKind::Index(Box::new(owner),Box::new(index)),Span::default())}
    fn condition(value:Expr,op:BinaryOp,reversed:bool)->Expr{let absent=Expr::new(ExprKind::Undefined,Span::default());let(left,right)=if reversed{(absent,value)}else{(value,absent)};Expr::new(ExprKind::Binary{op,operator:Span::default(),left:Box::new(left),right:Box::new(right)},Span::default())}
    #[test]
    fn absent_constant_index_removes_only_that_branch_entry(){
        let state=State::from([((0,vec!["0".into()]),Tokens::from([(1,2,true)])),((0,vec!["1".into()]),Tokens::from([(3,4,true)]))]);
        for op in [BinaryOp::Equal,BinaryOp::NotEqual]{for reversed in [false,true]{
            let guard=condition(indexed(Expr::new(ExprKind::Integer("0".into()),Span::default())),op,reversed);let(yes,no)=Tracker::absence_branches(&guard,state.clone());let(absent,present)=if op==BinaryOp::Equal{(yes,no)}else{(no,yes)};
            assert!(!absent.contains_key(&(0,vec!["0".into()])));assert_eq!(absent.get(&(0,vec!["1".into()])),state.get(&(0,vec!["1".into()])));assert_eq!(present,state);
        }}
    }
    #[test]
    fn absent_dynamic_index_and_nonliteral_sentinel_preserve_all_entries(){
        let state=State::from([((0,vec!["0".into()]),Tokens::from([(1,2,true)])),((0,vec!["1".into()]),Tokens::from([(3,4,true)]))]);
        let dynamic=condition(indexed(Expr::new(ExprKind::Name("index".into()),Span::default())),BinaryOp::Equal,false);
        let mut nonliteral=condition(indexed(Expr::new(ExprKind::Integer("0".into()),Span::default())),BinaryOp::Equal,false);if let ExprKind::Binary{right,..}=&mut nonliteral.kind{**right=Expr::new(ExprKind::Sequence(vec![Expr::new(ExprKind::Undefined,Span::default())]),Span::default());}
        let observed=condition(Expr::new(ExprKind::Unary{op:UnaryOp::Void,operator:Span::default(),value:Box::new(indexed(Expr::new(ExprKind::Integer("0".into()),Span::default())))},Span::default()),BinaryOp::Equal,false);
        for guard in [dynamic,nonliteral,observed]{assert_eq!(Tracker::absence_branches(&guard,state.clone()),(state.clone(),state.clone()));}
    }
}

#[cfg(test)]
mod cursor_tests{
 use super::*;
 fn done(binding:usize)->Expr{let mut owner=Expr::new(ExprKind::Name("step".into()),Span::default());owner.binding=Some(binding);Expr::new(ExprKind::Field(Box::new(owner),"done".into()),Span::default())}
 fn observed()->State{
  let mut state=State::from([((0,vec!["_items".into(),"*".into()]),Tokens::from([(1,2,true)])),((1,vec!["value".into()]),Tokens::from([(1,2,true)])),((9,vec!["_items".into(),"*".into()]),Tokens::from([(3,4,true)]))]);
  state.polls.insert((1,vec![]),Poll{cursors:vec![(0,vec!["_items".into()])],selected:vec![]});state
 }
 #[test]
 fn cursor_done_refines_only_its_observed_residual_and_payload(){
  let state=observed();let(yes,no)=Tracker::branches(&done(1),state.clone());
  assert!(!yes.contains_key(&(0,vec!["_items".into(),"*".into()])));assert!(!yes.contains_key(&(1,vec!["value".into()])));
  assert_eq!(yes.get(&(9,vec!["_items".into(),"*".into()])),state.get(&(9,vec!["_items".into(),"*".into()])));
  assert_eq!(no.owners,state.owners,"Some retains unknown remaining elements and yields its payload");
  let(unrelated,_)=Tracker::branches(&done(2),state.clone());assert_eq!(unrelated,state,"An unrelated done flag cannot settle a cursor");
 }
 #[test]
 fn cursor_poll_relations_follow_moves_and_are_invalidated_by_mutation_and_joins(){
  for owner in [(0,vec![]),(1,vec![])]{let mut state=observed();invalidate(&mut state,&owner);let(yes,_)=Tracker::branches(&done(1),state.clone());assert_eq!(yes.owners,state.owners);assert!(state.polls.is_empty());}
  let mut joined=observed();let mut unobserved=joined.clone();unobserved.polls.clear();merge(&mut joined,unobserved);assert!(joined.polls.is_empty(),"Only common branch provenance may refine a later read");
  let mut moved=observed();Tracker::transfer(&mut moved,&(1,vec![]),&(2,vec![]));Tracker::transfer(&mut moved,&(0,vec![]),&(3,vec![]));let(yes,_)=Tracker::branches(&done(2),moved);assert!(!yes.contains_key(&(3,vec!["_items".into(),"*".into()])));assert!(yes.contains_key(&(9,vec!["_items".into(),"*".into()])));
  let mut dynamic=observed();dynamic.polls.get_mut(&(1,vec![])).unwrap().cursors=vec![(0,vec!["3".into(),"_items".into()])];invalidate(&mut dynamic,&(0,vec!["*".into()]));assert!(dynamic.polls.is_empty(),"An unknown index can replace the observed cursor");
 }
 fn native_loop(body:&str,argument_fails:bool)->(CheckedProgram,Stmt){
  let mut program=analyze(crate::syntax::parse("type Cursor={_items:int[];};function drain(cursor:Owned<Cursor>){}function main(){}").unwrap()).unwrap();
  let cursor=program.records.iter().position(|record|record.name.text=="Cursor").unwrap();program.records[cursor].native_type=Some(crate::native_signatures::NativeTypeId::IteratorCursor);
  let mut parsed=crate::syntax::parse(&format!("function raw(){{while(true){{{body}}}}}")).unwrap();let mut statements=std::mem::take(&mut parsed.functions[0].body);
  crate::source_walk::body(&mut statements,&mut |_|Ok(()),&mut |value|{
   if let ExprKind::Name(name)=&value.kind{if name=="cursor"{value.binding=Some(0);value.ty=Some(Type::Record(cursor));value.own_may_fail=argument_fails;}else if name=="step"{value.binding=Some(1);}}
   if let ExprKind::Call(callee,args)=&mut value.kind{if matches!(&callee.kind,ExprKind::Name(name)if name=="poll"){value.target=Some(CallTarget::IteratorNext);args[0].binding=Some(0);args[0].ty=Some(Type::Record(cursor));args[0].own_may_fail=argument_fails;}else if matches!(&callee.kind,ExprKind::Name(name)if name=="settle"){let inner=args.remove(0);value.kind=ExprKind::Unary{op:UnaryOp::Await,operator:value.span,value:Box::new(inner)};}}
   Ok(())
  }).unwrap();
  if let StmtKind::While{body,..}=&mut statements[0].kind{for statement in body{if let StmtKind::Let{binding,..}=&mut statement.kind{*binding=Some(1);}}}
  (program,statements.remove(0))
 }
 #[test]
 fn native_cursor_drain_converges_and_keeps_early_and_failure_owners(){
  for argument_fails in [false,true]{
   let(program,statement)=native_loop("const step=poll(cursor);if(step.done){return;}settle(step.value);",argument_fails);let function=&program.functions[0];
   let state=State::from([((0,vec!["_items".into(),"*".into()]),Tokens::from([(1,2,true)]))]);
   let mut tracker=Tracker{program:&program,warnings:BTreeMap::new(),failures:Vec::new(),projections:0};let paths=tracker.statement(function,&statement,state);
   assert!(paths.iter().any(|path|path.exit==Exit::Return&&path.state.owners.is_empty()),"Actual None edge has no residual cursor owners");
   assert!(tracker.warnings.is_empty(),"Every Some is settled before the next poll");
   assert_eq!(paths.iter().any(|path|path.exit==Exit::Throw&&!path.state.owners.is_empty()),argument_fails,"Failures retain the pre-poll owner snapshot");
  }
  for body in ["const step=poll(cursor);return;","const step=poll(cursor);throw 'early';"]{
   let(program,statement)=native_loop(body,false);let mut tracker=Tracker{program:&program,warnings:BTreeMap::new(),failures:Vec::new(),projections:0};let paths=tracker.statement(&program.functions[0],&statement,State::from([((0,vec!["_items".into(),"*".into()]),Tokens::from([(1,2,true)]))]));
   assert!(paths.iter().any(|path|!path.state.owners.is_empty()),"A successful next alone does not exhaust its owner");assert!(!tracker.warnings.is_empty(),"An abandoned yielded owner is still diagnosed");
  }
 }
 #[test]
 fn checked_discriminants_refine_result_absence_without_claiming_source_exhaustion(){
  use crate::project::{SourceId,SourceProvider,check_graph,load_project};struct Input;
  impl SourceProvider for Input{fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}fn read(&self,id:&SourceId)->Result<String,String>{if id.0!="main.tsr"{return Err(format!("Missing source: {}",id.0));}Ok("type End={done:true;};type Value={done:false;value:Task<int>;};type Both={done:true;value:Task<int>;};type Loose={done:boolean;value:Task<int>;};function observe(step:ReadOnly<End|Value>){if(step.done){return;}}function present(step:ReadOnly<Both|Value>){if(step.done){return;}}function arbitrary(step:ReadOnly<Loose>){if(step.done){return;}}function main(){}".into())}}
  let checked=check_graph(load_project(&Input,"main.tsr").unwrap()).unwrap_or_else(|error|panic!("{error:?}"));let program=&checked.analysis.program;
  for(name,absent)in [("observe",true),("present",false),("arbitrary",false)]{
   let function=program.functions.iter().find(|function|function.name.ends_with(name)).unwrap();let StmtKind::If{condition,..}=&function.body[0].kind else{panic!("checked discriminant")};
   let state=State::from([((0,vec!["value".into()]),Tokens::from([(1,2,false)])),((9,vec!["_items".into(),"*".into()]),Tokens::from([(3,4,true)]))]);
   let tracker=Tracker{program,warnings:BTreeMap::new(),failures:Vec::new(),projections:0};let(yes,no)=tracker.checked_branches(condition,state.clone());
   assert_eq!(yes.contains_key(&(0,vec!["value".into()])),!absent,"{name}");assert!(no.contains_key(&(0,vec!["value".into()])));assert_eq!(yes.get(&(9,vec!["_items".into(),"*".into()])),state.get(&(9,vec!["_items".into(),"*".into()])),"Result absence does not exhaust an upstream owner");
  }
 }
 fn public_warnings(body:&str)->Result<Vec<String>,crate::project::ProjectDiagnostic>{
  use crate::project::{SourceId,SourceProvider,check_graph,load_project};struct Input(String);
  impl SourceProvider for Input{fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}fn read(&self,id:&SourceId)->Result<String,String>{if id.0=="main.tsr"{Ok(self.0.clone())}else{Err(format!("Missing source: {}",id.0))}}}
  let source=format!("import {{Iterator}} from 'std:iterator';import {{all,Job}} from 'std:futures';async function value():Task<int>{{return 3;}}async function main():Task<void>{{{body}}}");
  check_graph(load_project(&Input(source),"main.tsr").unwrap_or_else(|error|panic!("{body}: {error:?}"))).map(|checked|checked.warnings().into_iter().map(|warning|warning.code.to_string()).collect())
 }
 #[test]
 fn loop_local_started_jobs_are_diagnosed_without_a_function_exit(){
  let error=public_warnings("while(true){const job=value().start();}").unwrap_err();assert_eq!(error.code,"unsettled-job","Each iteration abandons its local Job even though the function never exits: {error:?}");
 }
 #[test]
 fn readonly_task_predicates_observe_without_awaiting_or_consuming_the_value(){
  assert_eq!(public_warnings("async function observe(text:ReadOnly<string>):Task<int>{return text.length;}const text='borrowed async data';console.log(await observe(text));console.log(text);").unwrap_or_else(|error|panic!("Readonly data reads remain legal: {error:?}")),Vec::<String>::new());
  assert_eq!(public_warnings("const pending=value();const observe:Owned<Function<boolean,[work:ReadOnly<Task<int>>]>>=(work:ReadOnly<Task<int>>):boolean=>typeof work==='function';observe(pending);await pending;").unwrap_or_else(|error|panic!("{error:?}")),Vec::<String>::new());
  assert_eq!(public_warnings("function observe(work:ReadOnly<Task<int>>):boolean{return typeof work==='function';}const pending=value();observe(pending);await pending;").unwrap_or_else(|error|panic!("{error:?}")),Vec::<String>::new());
  assert!(public_warnings("const pending=value();const observe:Owned<Function<boolean,[work:ReadOnly<Task<int>>]>>=(work:ReadOnly<Task<int>>):boolean=>typeof work==='function';observe(pending);").unwrap_or_else(|error|panic!("{error:?}")).iter().any(|warning|warning=="unused-task"),"Readonly invocation leaves its caller's obligation intact");
  assert!(public_warnings("function lose(work:Task<int>){}lose(value());").unwrap_or_else(|error|panic!("{error:?}")).iter().any(|warning|warning=="unused-task"),"An implicit Task parameter is still an owned input");
  let dropped=public_warnings("function lose(work:Owned<Job<int,undefined>>){}lose(value().start());").unwrap_err();assert_eq!(dropped.code,"unsettled-job","{dropped:?}");
  for body in ["const observe:Owned<Function<Task<int>,[work:ReadOnly<Task<int>>]>>=(work:ReadOnly<Task<int>>):Task<int>=>work;","const observe:Owned<Function<Task<void>,[work:ReadOnly<Task<int>>]>>=async(work:ReadOnly<Task<int>>):Task<void>=>{await work;};","const observe:Owned<Function<Task<void>,[work:ReadOnly<Task<int>>]>>=async(work:ReadOnly<Task<int>>):Task<void>=>{const job=work.start();await job;};"]{
   let error=public_warnings(body).expect_err(&format!("A borrowed Task cannot transfer, await or start its owner: {body}"));assert_ne!(error.code,"callback-access","The stated readonly Task contract is valid; reject the consuming operation instead: {error:?}");
  }
 }
 #[test]
 fn direct_callable_argument_modes_retain_checked_origin_without_callee_type(){
  let source="type Item={value:string;};function main(){const identity=(value:Owned<Item>):Item=>value;const item=identity({value:'kept'});console.log(item.value);}";
  let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
  let checked=crate::check(&source).unwrap();
  let main=checked.program.functions.iter().find(|function|function.name=="main").unwrap();
  let call=crate::error_effects::checked_expressions(main).into_iter().find(|value|matches!(value.target,Some(CallTarget::Closure(_)))).unwrap();
  let ExprKind::Call(callee,_)=&call.kind else{unreachable!()};assert!(callee.ty.is_none());assert!(call.closure_origin().is_some());
  assert_eq!(checked.program.call_argument_mode(call,0),Some(Mode::Consume));
 }
 #[test]
 fn public_array_iterator_drains_transfer_lazy_and_started_owners_to_all(){
  for body in ["const tasks=[value(),value()];await all(Iterator.from(tasks).map((work:Task<int>):Task<int>=>work).toArray());","await all(Iterator.from([value(),value()]).toArray());","await all(Iterator.from([value().start(),value().start()]).toArray());","await all(Iterator.from([1,2]).map((_item:int):Task<int>=>value()).toArray());","const jobs=[value().start(),value().start()];await all(Iterator.from(jobs).map((work:Job<int,undefined>):Job<int,undefined>=>work).toArray());"]{assert_eq!(public_warnings(body).unwrap_or_else(|error|panic!("{body}: {error:?}")),Vec::<String>::new(),"{body}");}
 }
 #[test]
 fn public_iterator_early_end_discards_and_abandonment_do_not_settle_owners(){
  for body in [
   "const pending=Iterator.from([value()]);",
   "const pending=Iterator.from([value()]);const step=pending.next();",
   "await all(Iterator.from([value()]).take(0).toArray());",
   "await all(Iterator.from([value()]).drop(1).toArray());",
   "await all(Iterator.from([value()]).filter((_value)=>false).toArray());",
   "const tasks=[value()];console.log(Iterator.from(tasks).map((_work:Task<int>):int=>1).toArray().length);",
  ]{assert!(public_warnings(body).unwrap_or_else(|error|panic!("{body}: {error:?}")).iter().any(|warning|warning=="unused-task"),"{body}");}
  for body in [
   "const pending=Iterator.from([value().start()]);",
   "await all(Iterator.from([value().start()]).take(0).toArray());",
   "const pending=Iterator.from([value().start()]);const step=pending.next();if(!step.done){step.value.cancel();}",
   "const pending=Iterator.from([value().start()]);const step=pending.next();throw 'early';",
  ]{let error=public_warnings(body).unwrap_err();assert_eq!(error.code,"unsettled-job","{body}: {error:?}");}
 }
}
