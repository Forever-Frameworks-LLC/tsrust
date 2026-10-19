//! State owners stay in the component tree; expressions and closures get weak bindings.
use crate::{Diagnostic,Span,syntax::{Expr,ExprKind,Program,Stmt,StmtKind,Token,TokenKind}};
use std::collections::BTreeMap;

type Scope=BTreeMap<String,String>;
fn name(value:&str,span:Span)->Expr {Expr::new(ExprKind::Name(value.into()),span)}
fn call(target:&str,args:Vec<Expr>,span:Span)->Expr {Expr::new(ExprKind::Call(Box::new(name(target,span)),args),span)}
fn statement(kind:StmtKind,span:Span)->Stmt {Stmt{kind,span,labels:vec![],control_target:None}}
struct Lower<'a> {reactive_props:std::collections::BTreeMap<(usize,usize),std::collections::BTreeSet<String>>,condition:&'a str,value_binding:&'a str,next_closure:usize,signal:&'a str,bind:&'a str,retain:&'a str,next:usize,constructors:Vec<Span>}
impl Lower<'_> {
    fn expression(&mut self,value:&mut Expr,scope:&Scope)->Result<(),Diagnostic> {
        crate::source_walk::expression_pruned(value,&mut |value| {
            let span=value.span;
            if let ExprKind::Record(fields)=&mut value.kind {
                for (key,value) in fields {
                    if key.kind!=TokenKind::JsxOpen || !matches!(key.text.as_str(),"disabled"|"readOnly"|"value") {continue;}
                    if key.text!="disabled" && !self.reactive_props.get(&(span.start,span.end)).is_some_and(|props|props.contains(&key.text)){continue;}
                    if key.text=="value" && matches!(&value.kind,ExprKind::Name(name) if scope.contains_key(name)){continue;}
                    let mut reactive=false;
                    crate::source_walk::expression(value,&mut |expression| {if let ExprKind::Name(name)=&expression.kind {reactive|=scope.contains_key(name);}Ok(())})?;
                    if reactive {
                        let span=value.span;let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));
                        // Retained getters carry the declared scalar union through owning callback storage.
                        let result=if key.text=="value"{crate::syntax::ResultAnnotation::Named(Box::new(crate::type_syntax::parse("string | int | number")?))}else{crate::syntax::ResultAnnotation::Inferred(span)};
                        let closure=Expr::new(ExprKind::Closure{id:self.next_closure,params:vec![],result,body:vec![statement(StmtKind::Return(Some(original)),span)]},span);self.next_closure+=1;
                        *value=call(if key.text=="value"{self.value_binding}else{self.condition},vec![closure],span);
                    }
                }
            }

            if let ExprKind::Name(source)=&value.kind {
                if let Some(target)=scope.get(source) {*value=call(self.bind,vec![name(target,span)],span);return Ok(false);}
            }
            if let ExprKind::Match{body,..}=&mut value.kind {
                let owners=scope.iter().filter(|(source,target)|source==target).map(|(source,_)|Token{text:source.clone(),span,kind:TokenKind::Name}).collect::<Vec<_>>();
                self.body(body,scope,&owners,false)?;return Ok(false);
            }
            if let ExprKind::Closure{params,body,..}=&mut value.kind {
                let mut captures=scope.clone();
                for parameter in params {captures.remove(&parameter.name.text);}
                let mut bindings=vec![];
                for (source,target) in &mut captures {
                    let outer=target.clone();let hidden=format!("@ui:capture:{}",self.next);self.next+=1;
                    *target=hidden.clone();bindings.push((source.clone(),hidden,outer));
                }
                self.body(body,&captures,&[],false)?;
                // Only materialize references actually used by this closure or a nested closure.
                let mut used=std::collections::BTreeSet::new();
                crate::source_walk::body(body,&mut |_|Ok(()),&mut |expr| {if let ExprKind::Name(n)=&expr.kind {used.insert(n.clone());}Ok(())})?;
                let mut prefix=vec![];
                for (_,hidden,outer) in bindings {
                    if !used.contains(&hidden) {continue;}
                    prefix.push(statement(StmtKind::Let{rebindable:false,name:Token{text:hidden,span,kind:TokenKind::Name},annotation:None,value:call(self.bind,vec![name(&outer,span)],span),binding:None},span));
                }
                if !prefix.is_empty() {
                    let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));
                    prefix.push(statement(StmtKind::Expr(call("@match:yield",vec![original],span)),span));
                    let id=usize::MAX-self.next;self.next+=1;
                    *value=Expr::new(ExprKind::Match{id,body:prefix},span);
                }
                return Ok(false);
            }
            Ok(true)
        })
    }
    fn body(&mut self,statements:&mut [Stmt],inherited:&Scope,retained:&[Token],allow_state:bool)->Result<(),Diagnostic> {
        let mut scope=inherited.clone();let mut owners=retained.to_vec();
        for statement in statements {
            match &mut statement.kind {
                StmtKind::Let{rebindable,name,value,..}=>{
                    let state=matches!(&value.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(n) if n==self.signal));
                    self.expression(value,&scope)?;
                    if state {
                        if !allow_state || *rebindable {return Err(Diagnostic::new("ui-state-owner","Create component state with a direct const declaration outside anonymous callbacks.",name.span));}
                        if scope.contains_key(&name.text) {return Err(Diagnostic::new("ui-state-owner","Component state owner names cannot be shadowed in this scope.",name.span));}
                        if let ExprKind::Call(callee,_) = &value.kind {self.constructors.push(callee.span);}
                        owners.push(name.clone());scope.insert(name.text.clone(),name.text.clone());
                    } else {
                        if owners.iter().any(|owner|owner.text==name.text) {return Err(Diagnostic::new("ui-state-owner","Component state owner names cannot be shadowed in this scope.",name.span));}
                        scope.remove(&name.text);
                    }
                }
                StmtKind::Return(Some(value))=>{
                    self.expression(value,&scope)?;
                    if !owners.is_empty() && !matches!(value.kind,ExprKind::Null|ExprKind::Undefined) {
                        let span=value.span;let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));
                        let local=format!("@ui:return:{}",self.next);let id=usize::MAX-self.next;self.next+=1;
                        let result=name(&local,span);let mut retained=result.clone();
                        for owner in &owners {retained=call(self.retain,vec![retained,name(&owner.text,owner.span)],span);}
                        let absent=|kind|Expr::new(ExprKind::Binary{op:crate::syntax::BinaryOp::Equal,operator:span,left:Box::new(result.clone()),right:Box::new(Expr::new(kind,span))},span);
                        let inner=Expr::new(ExprKind::Conditional{predicate:Box::new(absent(ExprKind::Undefined)),consequent:Box::new(result.clone()),alternate:Box::new(retained)},span);
                        let output=Expr::new(ExprKind::Conditional{predicate:Box::new(absent(ExprKind::Null)),consequent:Box::new(result),alternate:Box::new(inner)},span);
                        // Absence leaves owners in lexical scope so local callbacks drop first.
                        *value=Expr::new(ExprKind::Match{id,body:vec![self::statement(StmtKind::Let{rebindable:false,name:Token{text:local,span,kind:TokenKind::Name},annotation:None,value:original,binding:None},span),self::statement(StmtKind::Expr(call("@match:yield",vec![output],span)),span)]},span);
                    }
                }
                StmtKind::Block(nested)=>self.body(nested,&scope,&owners,allow_state)?,
                StmtKind::If{condition,then_body,else_body}=>{self.expression(condition,&scope)?;self.body(then_body,&scope,&owners,allow_state)?;self.body(else_body,&scope,&owners,allow_state)?;},
                StmtKind::While{condition,body}=>{self.expression(condition,&scope)?;self.body(body,&scope,&owners,allow_state)?;},
                StmtKind::Loop{condition,body,update,..}=>{if let Some(value)=condition {self.expression(value,&scope)?;}self.body(body,&scope,&owners,allow_state)?;self.body(update,&scope,&owners,allow_state)?;},
                StmtKind::ForOf{name,iterable,body,..}=>{self.expression(iterable,&scope)?;let mut inner=scope.clone();if owners.iter().any(|owner|owner.text==name.text) {return Err(Diagnostic::new("ui-state-owner","Loop variables cannot shadow component state owners.",name.span));}inner.remove(&name.text);self.body(body,&inner,&owners,allow_state)?;},
                StmtKind::Try(region)=>{
                    self.body(&mut region.body,&scope,&owners,allow_state)?;
                    if let Some(catch)=&mut region.catch {
                        let mut inner=scope.clone();if let Some(name)=&catch.name {if owners.iter().any(|owner|owner.text==name.text) {return Err(Diagnostic::new("ui-state-owner","Catch variables cannot shadow component state owners.",name.span));}inner.remove(&name.text);}
                        self.body(&mut catch.body,&inner,&owners,allow_state)?;
                    }
                    if let Some(finally)=&mut region.finally {self.body(finally,&scope,&owners,allow_state)?;}
                }
                StmtKind::LocalFunction(function)=>{
                    // Named local functions retain their ordinary checked capture semantics.
                    self.body(&mut function.body,&Scope::new(),&[],allow_state)?;
                }
                StmtKind::Expr(value)|StmtKind::Throw(value)=>self.expression(value,&scope)?,
                StmtKind::Assign{target,value,..}=>{self.expression(target,&scope)?;self.expression(value,&scope)?;},
                _=>{}
            }
        }
        Ok(())
    }
}
pub(crate) fn retain_state(program:&mut Program,names:&BTreeMap<String,String>)->Result<(),Diagnostic> {
    let (Some(signal),Some(bind),Some(retain),Some(condition),Some(value_binding))=(names.get("signal"),names.get("bindState"),names.get("retainComponentState"),names.get("bindCondition"),names.get("bindValue")) else{return Ok(());};
    // Only declared text-field value contracts receive a retained getter. Other components keep ordinary scalar props.
    let mut reactive_props=std::collections::BTreeMap::<(usize,usize),std::collections::BTreeSet<String>>::new();
    for (property,marker) in [("value","ReactiveInputValue"),("readOnly","ReactiveBoolean")] {
      if let Some(binding)=names.get(marker) {
        fn includes(annotation:&crate::type_syntax::ValueAnnotation,binding:&str)->bool {
            use crate::type_syntax::ValueAnnotationKind as K;
            let mut pending=vec![annotation];while let Some(value)=pending.pop(){match &value.kind{K::Named(name) if name.text==binding=>return true,K::Group(value)=>pending.push(value),K::Union(values)=>pending.extend(values),_=>{}}}false
        }
        fn record(annotation:&crate::type_syntax::ValueAnnotation)->Option<&str>{use crate::type_syntax::ValueAnnotationKind as K;match &annotation.kind{K::Named(name)|K::Applied{name,..}=>Some(&name.text),K::Group(value)=>record(value),_=>None}}
        let records=program.records.iter().filter(|record|record.fields.iter().any(|field|field.name.text==property && includes(&field.annotation,binding))).map(|record|record.name.text.as_str()).collect::<std::collections::BTreeSet<_>>();
        let components=program.functions.iter().filter(|function|function.params.first().and_then(|(_,annotation,_)|record(annotation)).is_some_and(|name|records.contains(name))).map(|function|function.name.text.clone()).collect::<std::collections::BTreeSet<_>>();
        for function in &mut program.functions {crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Call(callee,args)=&expression.kind{if matches!(&callee.kind,ExprKind::Name(name) if components.contains(name)){if let Some(props)=args.first(){reactive_props.entry((props.span.start,props.span.end)).or_default().insert(property.into());}}}Ok(())})?;}
    }
    }
    let mut next_closure=0;
    for function in &mut program.functions {crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expr|{if let ExprKind::Closure{id,..}=&expr.kind {next_closure=next_closure.max(id+1);}Ok(())})?;}
    let mut lower=Lower{reactive_props,condition,value_binding,next_closure,signal,bind,retain,next:0,constructors:vec![]};
    for function in &mut program.functions {
        if names.values().any(|name|name==&function.name.text) {continue;}
        lower.body(&mut function.body,&Scope::new(),&[],true)?;
        crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression| {
            if matches!(&expression.kind,ExprKind::Name(name) if name==signal) && !lower.constructors.contains(&expression.span) {
                return Err(Diagnostic::new("ui-state-owner","Create component state with a direct const declaration; signal constructor aliases are not supported.",expression.span));
            }
            Ok(())
        })?;
    }
    Ok(())
}
