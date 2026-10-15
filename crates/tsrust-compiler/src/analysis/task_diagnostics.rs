//! Task abandonment follows owners through checked control flow, independently of suppression.
use super::*;
use std::collections::BTreeSet;
type Tokens = BTreeSet<(usize, usize)>;
type Place = (usize, Vec<String>);
type State = BTreeMap<Place, Tokens>;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Exit { Next, Return, Throw, Break, Continue }
struct Path { state: State, exit: Exit }
struct Tracker<'a> { program: &'a CheckedProgram, warnings: BTreeMap<(usize,usize,usize),Diagnostic>, failures: Vec<State> }
impl CheckedProgram {
    pub(crate) fn task_type(&self, ty: Type) -> bool {
        match ty {
            Type::Closure(id) => self.task_origins.contains(&id),
            Type::Callback(id) => self.callback_contracts[id].kind==CallbackKind::Task,
            Type::StoredCallable(id)|Type::BorrowedCallable(id) => self.stored_contracts.declaration(id).is_some_and(|id| self.callback_contracts[id].kind==CallbackKind::Task),
            _=>false,
        }
    }
    fn task_paths(&self,root:Type)->Vec<Vec<String>> {
        let mut found=Vec::new();let mut pending=vec![(root,Vec::new(),HashSet::new())];
        while let Some((ty,path,mut seen))=pending.pop(){
            if self.task_type(ty){found.push(path);continue;}
            if !seen.insert(ty){continue;}
            match ty {
                Type::Record(id)=>for field in &self.records[id].fields {if let Some(ty)=field.ty{let mut child=path.clone();child.push(field.name.text.clone());pending.push((ty,child,seen.clone()));}},
                Type::Union(id)|Type::BorrowedUnion(id)=>for &ty in &self.unions[id.index()].members{pending.push((ty,path.clone(),seen.clone()));},
                Type::Array(element)=>{let mut child=path;child.push("*".into());pending.push((element.ty(),child,seen));},
                Type::Closure(id)=>for capture in &self.closures[id].captures{if capture.storage==Mode::Consume{let mut child=path.clone();child.push(format!("capture{}",capture.inner));pending.push((self.closures[id].function.bindings[capture.inner].ty,child,seen.clone()));}},
                _=>{}
            }
        }found
    }
    pub(crate) fn task_warnings(&self)->Vec<Diagnostic> {
        if self.task_origins.is_empty() && !self.callback_contracts.iter().any(|contract|contract.kind==CallbackKind::Task) {return Vec::new();}
        let mut tracker=Tracker{program:self,warnings:BTreeMap::new(),failures:Vec::new()};
        for function in &self.functions { if !function.template {tracker.function(function, &[]);} }
        for closure in &self.closures {if !closure.function.template && closure.forward.is_none() {tracker.function(&closure.function,&closure.captures);} }
        tracker.warnings.into_values().collect()
    }
}
fn place(value:&Expr)->Option<Place> {
    match &value.kind {
        ExprKind::Name(_)=>value.binding.map(|id|(id,Vec::new())),
        ExprKind::Field(base,name)=>{let(id,mut path)=place(base)?;path.push(name.clone());Some((id,std::mem::take(&mut path)))},
        ExprKind::Index(base,index)=>{let(id,mut path)=place(base)?;if let ExprKind::Integer(text)=&index.kind {path.push(text.clone());}else{path.push("*".into());}Some((id,path))},
        ExprKind::Unary{value,..}|ExprKind::Project{value,..}|ExprKind::TypeAssertion{value,..}=>place(value),
        _=>None,
    }
}
impl Tracker<'_> {
    fn warn(&mut self,tokens:Tokens,at:Span) {
        for(start,end)in tokens {
            self.warnings.entry((start,end,at.start)).or_insert_with(||Diagnostic::new("unused-task","This lazy task can be discarded without running. Await it, return it, or transfer it to an owner that handles it.",at).related(Span::new(start,end),"Task responsibility begins here."));
        }
    }
    fn take(state:&mut State,selected:&Place)->Tokens {
        let keys=state.keys().filter(|(root,path)|*root==selected.0 && path.starts_with(&selected.1)).cloned().collect::<Vec<_>>();
        let mut tokens=Tokens::new();for key in keys {tokens.extend(state.remove(&key).unwrap());}tokens
    }
    fn transfer(state:&mut State,source:&Place,destination:&Place) {
        let selected=state.keys().filter(|(root,path)|*root==source.0 && path.starts_with(&source.1)).cloned().collect::<Vec<_>>();
        for place in selected {let tokens=state.remove(&place).unwrap();let mut path=destination.1.clone();path.extend_from_slice(&place.1[source.1.len()..]);state.entry((destination.0,path)).or_default().extend(tokens);}
    }
    fn store(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State,destination:Place) {
        if let Some(source)=place(value){Self::transfer(state,&source,&destination);return;}
        match &value.kind {
            ExprKind::Record(fields)=>for(field,value)in fields{let mut child=destination.clone();child.1.push(field.text.clone());self.store(function,value,state,child);},
            ExprKind::Array(values)=>for(index,value)in values.iter().enumerate(){let mut child=destination.clone();child.1.push(index.to_string());self.store(function,value,state,child);},
            ExprKind::Inject{value,..}|ExprKind::TypeAssertion{value,..}=>self.store(function,value,state,destination),
            ExprKind::Unary{value:inner,..} if !value.ty.is_some_and(|ty|self.program.task_type(ty))=>self.store(function,inner,state,destination),
            _=>{let tokens=self.value(function,value,state,true);let paths=value.ty.map(|ty|self.program.task_paths(ty)).unwrap_or_default();
                if paths.is_empty(){state.entry(destination).or_default().extend(tokens);}else{for path in paths {let mut child=destination.clone();child.1.extend(path);state.entry(child).or_default().extend(tokens.iter().copied());}}}

        }
    }
    fn replace(&mut self,function:&CheckedFunction,target:&Expr,value:&Expr,state:&mut State) {
        let Some(destination)=place(target)else{let _=self.value(function,value,state,true);return;};
        let temporary=(usize::MAX,Vec::new());self.store(function,value,state,temporary.clone());
        let old=Self::take(state,&destination);self.warn(old,target.span);Self::transfer(state,&temporary,&destination);
    }
    fn function(&mut self,function:&CheckedFunction,captures:&[Capture]) {
        if function.bindings.iter().all(|binding|self.program.task_paths(binding.ty).is_empty()) && crate::error_effects::checked_expressions(function).iter().all(|value|value.ty.is_none_or(|ty|self.program.task_paths(ty).is_empty())) {return;}
        let mut state=State::new();
        for(id,binding)in function.bindings.iter().enumerate() {
            if (binding.parameter && (binding.mode==Mode::Consume || self.program.task_type(binding.ty))) || captures.iter().any(|capture|capture.inner==id && capture.storage==Mode::Consume) {
                let at=binding.declaration_span.or(binding.used).unwrap_or_default();
                for path in self.program.task_paths(binding.ty){state.insert((id,path),Tokens::from([(at.start,at.end)]));}
            }
        }
        for path in self.body(function,&function.body,state) {for(_,tokens)in path.state {self.warn(tokens,function.body.last().map_or(Span::default(),|stmt|stmt.span));}}
    }
    fn value(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State,consume:bool)->Tokens {
        let tokens=self.value_inner(function,value,state,consume);
        if value.own_may_fail {self.failures.push(state.clone());}
        tokens
    }
    fn value_inner(&mut self,function:&CheckedFunction,value:&Expr,state:&mut State,consume:bool)->Tokens {
        if let ExprKind::Unary{op:UnaryOp::Void|UnaryOp::Typeof,value,..}=&value.kind {let _=self.value(function,value,state,false);return Tokens::new();}
        if let Some(selected)=place(value) {return if consume {Self::take(state,&selected)}else{Tokens::new()};}
        match &value.kind {
            ExprKind::Unary{op:UnaryOp::Await,value,..}=>{let _=self.value(function,value,state,true);Tokens::new()},
            ExprKind::Unary{op:UnaryOp::TaskCreate,value:inner,..}=>{
                let mut tokens=self.value(function,inner,state,true);tokens.insert((value.span.start,value.span.end));tokens
            }
            ExprKind::Closure{..}=>{
                let mut tokens=Tokens::new();
                if let Some(Type::Closure(origin))=value.ty && let Some(closure)=self.program.closures.get(origin) {for capture in &closure.captures {if capture.storage==Mode::Consume {tokens.extend(Self::take(state,&(capture.outer,Vec::new())));}}}
                tokens
            }
            ExprKind::Call(callee,args)=>{
                if let ExprKind::Field(base,method)=&callee.kind && let Some(owner)=place(base) {
                    if matches!(method.as_str(),"pop"|"shift") {
                        let selected=state.keys().filter(|(root,path)|*root==owner.0 && path.starts_with(&owner.1) && path.len()>owner.1.len()).filter_map(|place|place.1[owner.1.len()].parse::<usize>().ok().map(|index|(index,place.clone())));
                        let selected=if method=="pop"{selected.max_by_key(|(index,_)|*index)}else{selected.min_by_key(|(index,_)|*index)};
                        return selected.map_or_else(Tokens::new,|(_,selected)|Self::take(state,&selected));
                    }
                    if method=="push" {
                        let mut index=state.keys().filter(|(root,path)|*root==owner.0 && path.starts_with(&owner.1)).filter_map(|(_,path)|path.get(owner.1.len()).and_then(|part|part.parse::<usize>().ok())).max().map_or(0,|index|index+1);
                        for arg in args {let tokens=self.value(function,arg,state,true);let mut path=owner.1.clone();path.push(index.to_string());state.insert((owner.0,path),tokens);index+=1;}return Tokens::new();
                    }
                }
                let awaited=matches!(callee.kind,ExprKind::Unary{op:UnaryOp::AwaitCallee,..});
                let consumes=awaited || callee.ty.is_some_and(|ty|match ty {Type::Closure(id)=>self.program.closures[id].invocation==Mode::Consume,Type::Callback(id)=>self.program.callback_contracts[id].kind.is_once(),_=>false});
                let mut tokens=if consumes {self.value(function,callee,state,true)}else{Tokens::new()};
                for(index,arg)in args.iter().enumerate() {
                    let access=match value.target {
                        Some(CallTarget::Function(id))=>self.program.functions[id].bindings.get(index).map(|binding|binding.mode),
                        _=>None,
                    };
                    let first_failure=self.failures.len();
                    let argument=self.value(function,arg,state,access.is_none_or(|mode|mode==Mode::Consume));
                    // Earlier moved arguments are still temporaries if a later argument throws.
                    for failure in &mut self.failures[first_failure..] {failure.entry((usize::MAX-1,Vec::new())).or_default().extend(tokens.iter().copied());}
                    tokens.extend(argument);
                }
                if awaited {return Tokens::new();}
                if value.ty.is_some_and(|ty|!self.program.task_paths(ty).is_empty()) {if tokens.is_empty() || value.ty.is_some_and(|ty|self.program.task_type(ty)){tokens.insert((value.span.start,value.span.end));}tokens}else{Tokens::new()}
            }
            ExprKind::Assign{target,value,..}=>{self.replace(function,target,value,state);Tokens::new()}
            ExprKind::Conditional{predicate,consequent,alternate}=>{
                let _=self.value(function,predicate,state,false);
                let mut other=state.clone();let mut tokens=self.value(function,consequent,state,consume);tokens.extend(self.value(function,alternate,&mut other,consume));merge(state,other);tokens
            }
            ExprKind::Record(fields)=>{let mut tokens=Tokens::new();for(_,value)in fields {tokens.extend(self.value(function,value,state,consume));}tokens},
            ExprKind::Array(values)|ExprKind::Template(values)|ExprKind::Sequence(values)=>{
                let mut tokens=Tokens::new();for value in values {tokens.extend(self.value(function,value,state,consume));}tokens
            }
            ExprKind::Unary{value,..}|ExprKind::Inject{value,..}|ExprKind::Project{value,..}|ExprKind::TypeAssertion{value,..}=>self.value(function,value,state,consume),
            ExprKind::Binary{left,right,..}=>{let _=self.value(function,left,state,false);let _=self.value(function,right,state,false);Tokens::new()},
            ExprKind::Match{body,..}=>{let mut result=State::new();for path in self.body(function,body,state.clone()){merge(&mut result,path.state);}*state=result;Tokens::new()},
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
                let mut paths=self.body(function,then_body,state.clone());paths.extend(self.body(function,else_body,state));return coalesce(paths);
            }
            StmtKind::While{condition,body}=>{
                let _=self.value(function,condition,&mut state,false);
                let mut paths=self.body(function,body,state.clone());for path in &mut paths {if matches!(path.exit,Exit::Break|Exit::Continue){path.exit=Exit::Next;}}paths.push(Path{state,exit:Exit::Next});return coalesce(paths);
            }
            StmtKind::Loop{body,..}|StmtKind::ForOf{body,..}=>{
                let mut paths=self.body(function,body,state.clone());for path in &mut paths {if matches!(path.exit,Exit::Break|Exit::Continue){path.exit=Exit::Next;}}
                paths.push(Path{state,exit:Exit::Next});return coalesce(paths);
            }
            StmtKind::Try(region)=>{
                let mut paths=self.body(function,&region.body,state);
                if let Some(handler)=&region.catch {
                    let mut caught=State::new();let mut any=false;paths.retain(|path|{if path.exit==Exit::Throw{merge(&mut caught,path.state.clone());any=true;false}else{true}});
                    if any {paths.extend(self.body(function,&handler.body,caught));}
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
fn merge(target:&mut State,other:State){for(place,tokens)in other{target.entry(place).or_default().extend(tokens);}}
fn coalesce(paths:Vec<Path>)->Vec<Path>{let mut result:Vec<Path>=Vec::new();for path in paths{if let Some(existing)=result.iter_mut().find(|existing|existing.exit==path.exit){merge(&mut existing.state,path.state);}else{result.push(path);}}result}
