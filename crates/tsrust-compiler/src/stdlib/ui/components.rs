//! State owners stay in the component tree; expressions and closures get weak bindings.
use crate::{Diagnostic,Span,syntax::{Expr,ExprKind,Program,Stmt,StmtKind,Token,TokenKind}};
use std::collections::BTreeMap;

type Scope=BTreeMap<String,String>;
fn name(value:&str,span:Span)->Expr {Expr::new(ExprKind::Name(value.into()),span)}
fn call(target:&str,args:Vec<Expr>,span:Span)->Expr {Expr::new(ExprKind::Call(Box::new(name(target,span)),args),span)}
fn statement(kind:StmtKind,span:Span)->Stmt {Stmt{kind,span,labels:vec![],control_target:None}}
struct Lower<'a> {primitives:std::collections::BTreeSet<String>,class_binding:Option<&'a String>,variant_binding:Option<&'a String>,wrapped:Vec<Span>,computed:Option<&'a String>,reactive:Option<&'a String>,branch:&'a str,child_result:crate::type_syntax::ValueAnnotation,scalar_functions:std::collections::BTreeSet<String>,shadowed:std::collections::BTreeSet<String>,text_binding:&'a str,reactive_props:std::collections::BTreeMap<(usize,usize),std::collections::BTreeSet<String>>,condition:&'a str,value_binding:&'a str,next_closure:usize,signal:&'a str,location:Option<&'a String>,bind:&'a str,retain:&'a str,next:usize,constructors:Vec<Span>}
fn scalar_annotation(value:&crate::type_syntax::ValueAnnotation)->bool {
 use crate::type_syntax::ValueAnnotationKind as K;
 match &value.kind {K::Named(name)=>matches!(name.text.as_str(),"string"|"int"|"number"|"boolean"|"null"|"undefined"),K::Literal(_)=>true,K::Group(value)=>scalar_annotation(value),K::Union(values)=>values.iter().all(scalar_annotation),_=>false}
}
fn scalar_text(value:&Expr,scope:&Scope,functions:&std::collections::BTreeSet<String>,shadowed:&std::collections::BTreeSet<String>)->bool {
 // shortcut: only declared scalar forms lower before checking; widen with checked reactive prop plans.
 match &value.kind {
  ExprKind::Integer(_)|ExprKind::Number(_)|ExprKind::String(_)|ExprKind::Boolean(_)|ExprKind::Null|ExprKind::Undefined|ExprKind::Template(_)=>true,
  ExprKind::Name(name)=>matches!(name.as_str(),"NaN"|"Infinity"|"undefined") && !shadowed.contains(name),
  ExprKind::Call(callee,_)=>match &callee.kind {
   ExprKind::Name(name)=>functions.contains(name)&&!shadowed.contains(name),
   ExprKind::Field(base,method)=>matches!(&base.kind,ExprKind::Name(name) if method=="get" && scope.contains_key(name)) || scalar_text(base,scope,functions,shadowed) || matches!(&base.kind,ExprKind::Name(name) if !shadowed.contains(name) && (name=="Math" || name=="JSON" && method=="stringify")),
   _=>false,
  },
  ExprKind::Conditional{consequent,alternate,..}=>scalar_text(consequent,scope,functions,shadowed)&&scalar_text(alternate,scope,functions,shadowed),
  ExprKind::Binary{op,left,right,..}=>!matches!(op,crate::syntax::BinaryOp::And|crate::syntax::BinaryOp::Or|crate::syntax::BinaryOp::Coalesce|crate::syntax::BinaryOp::PatternDefault) || scalar_text(left,scope,functions,shadowed)&&scalar_text(right,scope,functions,shadowed),
  ExprKind::Unary{op,..}=>matches!(op,crate::syntax::UnaryOp::Not|crate::syntax::UnaryOp::Positive|crate::syntax::UnaryOp::Negate|crate::syntax::UnaryOp::BitNot|crate::syntax::UnaryOp::Typeof),
  ExprKind::OptionalChain(value)=>scalar_text(value,scope,functions,shadowed),
  ExprKind::Field(base,field)=>field=="length"&&scalar_text(base,scope,functions,shadowed),
  _=>false,
 }
}
impl Lower<'_> {
    fn attributes(&mut self,value:&mut Expr,scope:&Scope)->Result<(),Diagnostic>{
        if scope.is_empty()||self.wrapped.contains(&value.span){return Ok(());}
        let ExprKind::Call(callee,args)=&mut value.kind else{return Ok(())};let ExprKind::Name(component)=&callee.kind else{return Ok(())};if !self.primitives.contains(component)||args.len()!=1{return Ok(());}
        let ExprKind::Record(fields)=&mut args[0].kind else{return Ok(())};let mut prefix=vec![];let mut bindings=vec![];
        for(key,expression)in fields{
            if key.kind!=TokenKind::JsxOpen||!matches!(key.text.as_str(),"className"|"data-variant"){continue;}
            let mut reactive=false;crate::source_walk::expression(expression,&mut|value|{if let ExprKind::Name(name)=&value.kind{reactive|=scope.contains_key(name);}Ok(())})?;if !reactive{continue;}
            let span=expression.span;let getter=format!("@ui:attribute:{}",self.next);self.next+=1;let original=std::mem::replace(expression,Expr::new(ExprKind::Undefined,span));
            let result=crate::syntax::ResultAnnotation::Named(Box::new(crate::type_syntax::parse("string")?));
            let empty=Expr::new(ExprKind::Closure{id:self.next_closure,params:vec![],result:result.clone(),body:vec![statement(StmtKind::Return(Some(Expr::new(ExprKind::String(String::new()),span))),span)]},span);self.next_closure+=1;
            prefix.push(statement(StmtKind::Let{rebindable:true,name:Token{text:getter.clone(),span,kind:TokenKind::Name},annotation:Some(crate::type_syntax::parse("Owned<Function<string>>")?),value:empty,binding:None},span));
            let actual=Expr::new(ExprKind::Closure{id:self.next_closure,params:vec![],result,body:vec![statement(StmtKind::Return(Some(original)),span)]},span);self.next_closure+=1;
            let body=vec![statement(StmtKind::Assign{target:name(&getter,span),value:actual,operator:None},span),statement(StmtKind::Expr(call("@match:yield",vec![call(&getter,vec![],span)],span)),span)];
            *expression=Expr::new(ExprKind::Match{id:usize::MAX-self.next,body},span);self.next+=1;
            bindings.push((if key.text=="className"{self.class_binding}else{self.variant_binding}.expect("attribute binder"),getter));
        }
        if prefix.is_empty(){return Ok(());}
        let span=value.span;self.wrapped.push(span);let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));let mut result=original;
        for(binder,getter)in bindings{result=call(binder,vec![result,name(&getter,span)],span);}
        prefix.push(statement(StmtKind::Expr(call("@match:yield",vec![result],span)),span));
        *value=Expr::new(ExprKind::Match{id:usize::MAX-self.next,body:prefix},span);self.next+=1;Ok(())
    }
    fn branch_expression(&mut self,value:&mut Expr,scope:&Scope)->Result<bool,Diagnostic> {
        if scalar_text(value,scope,&self.scalar_functions,&self.shadowed){return Ok(false);}
        let ExprKind::Conditional{predicate,..}=&value.kind else{return Ok(false)};
        let mut reactive=false;let mut test=(**predicate).clone();crate::source_walk::expression(&mut test,&mut |expression|{if let ExprKind::Name(name)=&expression.kind{reactive|=scope.contains_key(name);}Ok(())})?;
        if !reactive{return Ok(false);}
        let at=value.span;let original=std::mem::replace(&mut value.kind,ExprKind::Undefined);let ExprKind::Conditional{predicate,mut consequent,mut alternate}=original else{unreachable!()};
        self.branch_expression(&mut consequent,scope)?;self.branch_expression(&mut alternate,scope)?;
        let mut getters=vec![];
        for (value,result) in [(*predicate,crate::syntax::ResultAnnotation::Named(Box::new(crate::type_syntax::parse("boolean")?))),(*consequent,crate::syntax::ResultAnnotation::Named(Box::new(self.child_result.clone()))),(*alternate,crate::syntax::ResultAnnotation::Named(Box::new(self.child_result.clone())))] {
            getters.push(Expr::new(ExprKind::Closure{id:self.next_closure,params:vec![],result,body:vec![statement(StmtKind::Return(Some(value)),at)]},at));self.next_closure+=1;
        }
        *value=call(self.branch,getters,at);Ok(true)
    }
    fn expression(&mut self,value:&mut Expr,scope:&Scope)->Result<(),Diagnostic> {
        crate::source_walk::expression_pruned(value,&mut |value| {
            self.attributes(value,scope)?;
            let span=value.span;
            if let ExprKind::Record(fields)=&mut value.kind {
                for (key,value) in fields {
                    if key.text=="children" && key.span==value.span && self.reactive_props.get(&(span.start,span.end)).is_some_and(|props|props.contains("children")) {
                        if let ExprKind::Array(children)=&mut value.kind {for child in children {
                            if self.reactive_props.get(&(span.start,span.end)).is_some_and(|props|props.contains("branch")) && self.branch_expression(child,scope)?{continue;}
                            if matches!(&child.kind,ExprKind::Name(name) if scope.contains_key(name)) || !scalar_text(child,scope,&self.scalar_functions,&self.shadowed){continue;}
                            let mut reactive=false;crate::source_walk::expression(child,&mut |expression|{if let ExprKind::Name(name)=&expression.kind{reactive|=scope.contains_key(name);}Ok(())})?;
                            if !reactive{continue;}
                            let span=child.span;let original=std::mem::replace(child,Expr::new(ExprKind::Undefined,span));let result=crate::syntax::ResultAnnotation::Named(Box::new(crate::type_syntax::parse("string | int | number | boolean | null | undefined")?));
                            let getter=Expr::new(ExprKind::Closure{id:self.next_closure,params:vec![],result,body:vec![statement(StmtKind::Return(Some(original)),span)]},span);self.next_closure+=1;
                            *child=call(self.text_binding,vec![getter],span);
                        }}
                    }
                    if key.kind!=TokenKind::JsxOpen || !matches!(key.text.as_str(),"disabled"|"readOnly"|"value"|"hidden") && !self.reactive_props.get(&(span.start,span.end)).is_some_and(|props|props.contains(&format!("generic:{}",key.text))) {continue;}
                    let generic=self.reactive_props.get(&(span.start,span.end)).is_some_and(|props|props.contains(&format!("generic:{}",key.text)));
                    if !generic && key.text!="disabled" && !self.reactive_props.get(&(span.start,span.end)).is_some_and(|props|props.contains(&key.text)){continue;}
                    if key.text=="value" && matches!(&value.kind,ExprKind::Name(name) if scope.contains_key(name)){continue;}
                    let mut reactive=false;
                    crate::source_walk::expression(value,&mut |expression| {if let ExprKind::Name(name)=&expression.kind {reactive|=scope.contains_key(name);}Ok(())})?;
                    if reactive {
                        let span=value.span;let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));
                        // Retained getters carry the declared scalar union through owning callback storage.
                        let result=if key.text=="value"{crate::syntax::ResultAnnotation::Named(Box::new(crate::type_syntax::parse("string | int | number")?))}else{crate::syntax::ResultAnnotation::Inferred(span)};
                        let closure=Expr::new(ExprKind::Closure{id:self.next_closure,params:vec![],result,body:vec![statement(StmtKind::Return(Some(original)),span)]},span);self.next_closure+=1;
                        *value=call(if generic{self.reactive.expect("reactive prop binder")}else if key.text=="value"{self.value_binding}else{self.condition},vec![closure],span);
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
                    let state=matches!(&value.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(n) if n==self.signal || self.location.is_some_and(|location|n==location) || self.computed.is_some_and(|computed|n==computed)));
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
                    if !owners.is_empty(){self.branch_expression(value,&scope)?;}
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
    let (Some(signal),Some(bind),Some(retain),Some(condition),Some(value_binding),Some(text_binding),Some(branch))=(names.get("signal"),names.get("bindState"),names.get("retainComponentState"),names.get("bindCondition"),names.get("bindValue"),names.get("bindText"),names.get("bindBranch")) else{return Ok(());};
    // Only declared reactive prop contracts receive retained getters; unrelated props stay ordinary values.
    let mut reactive_props=std::collections::BTreeMap::<(usize,usize),std::collections::BTreeSet<String>>::new();
    let text_marker=crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::UiTextBinding);
    let node_marker=crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::UiNode);
    for (property,binding) in [("hidden",names.get("ReactiveBoolean")),("value",names.get("ReactiveInputValue")),("readOnly",names.get("ReactiveBoolean")),("children",Some(&text_marker)),("branch",Some(&node_marker))] {
      if let Some(binding)=binding {
        fn includes(annotation:&crate::type_syntax::ValueAnnotation,binding:&str,arrays:bool)->bool {
            use crate::type_syntax::ValueAnnotationKind as K;
            let mut pending=vec![annotation];while let Some(value)=pending.pop(){match &value.kind{K::Named(name) if name.text==binding=>return true,K::Group(value)=>pending.push(value),K::Array(value) if arrays=>pending.push(value),K::Union(values)=>pending.extend(values),_=>{}}}false
        }
        fn record(annotation:&crate::type_syntax::ValueAnnotation)->Option<&str>{use crate::type_syntax::ValueAnnotationKind as K;match &annotation.kind{K::Named(name)|K::Applied{name,..}=>Some(&name.text),K::Group(value)=>record(value),_=>None}}
        let records=program.records.iter().filter(|record|record.fields.iter().any(|field|field.name.text==if property=="branch"{"children"}else{property} && includes(&field.annotation,binding,matches!(property,"children"|"branch")))).map(|record|record.name.text.as_str()).collect::<std::collections::BTreeSet<_>>();
        let components=program.functions.iter().filter(|function|function.params.first().and_then(|(_,annotation,_)|record(annotation)).is_some_and(|name|records.contains(name))).map(|function|function.name.text.clone()).collect::<std::collections::BTreeSet<_>>();
        for function in &mut program.functions {crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Call(callee,args)=&expression.kind{if matches!(&callee.kind,ExprKind::Name(name) if components.contains(name)){if let Some(props)=args.first(){reactive_props.entry((props.span.start,props.span.end)).or_default().insert(property.into());}}}Ok(())})?;}
    }
    }
    if names.get("bindReactive").is_some(){
     let aliases=[names.get("Reactive"),names.get("ReactiveValue")].into_iter().flatten().collect::<std::collections::BTreeSet<_>>();
     fn reactive(annotation:&crate::type_syntax::ValueAnnotation,names:&std::collections::BTreeSet<&String>)->bool {use crate::type_syntax::ValueAnnotationKind as K;match &annotation.kind{K::Applied{name,..}=>names.contains(&name.text),K::Group(value)=>reactive(value,names),K::Union(values)=>values.iter().any(|value|reactive(value,names)),_=>false}}
     let fields=program.records.iter().map(|record|(record.name.text.clone(),record.fields.iter().filter(|field|reactive(&field.annotation,&aliases)).map(|field|format!("generic:{}",field.name.text)).collect::<std::collections::BTreeSet<_>>())).filter(|(_,fields)|!fields.is_empty()).collect::<BTreeMap<_,_>>();
     let components=program.functions.iter().filter_map(|function|{let (_,annotation,_)=function.params.first()?;use crate::type_syntax::ValueAnnotationKind as K;let name=match &annotation.kind{K::Named(name)|K::Applied{name,..}=>&name.text,_=>return None};Some((function.name.text.clone(),fields.get(name)?.clone()))}).collect::<BTreeMap<_,_>>();
     for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Call(callee,args)=&expression.kind{if let ExprKind::Name(name)=&callee.kind{if let Some(fields)=components.get(name){if let Some(props)=args.first(){reactive_props.entry((props.span.start,props.span.end)).or_default().extend(fields.clone());}}}}Ok(())})?;}
    }
    let mut next_closure=0;
    for function in &mut program.functions {crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expr|{if let ExprKind::Closure{id,..}=&expr.kind {next_closure=next_closure.max(id+1);}Ok(())})?;}
    let scalar_functions=program.functions.iter().filter(|function|matches!(&function.result,Some(crate::syntax::ResultAnnotation::Named(value)) if scalar_annotation(value))).map(|function|function.name.text.clone()).chain(["String","Number","Boolean","parseInt","parseFloat","isNaN","isFinite"].into_iter().map(str::to_owned)).collect();
    let child_result=crate::native_type_resolution::project(&mut program.records,&super::CHILD_VALUE,Span::default())?;
    let mut lower=Lower{primitives:["View","Text","Button","Input","Textarea","TextField","Link","CopyButton"].into_iter().filter_map(|name|names.get(name).cloned()).collect(),class_binding:names.get("bindClasses"),variant_binding:names.get("bindVariant"),wrapped:vec![],computed:names.get("computed"),reactive:names.get("bindReactive"),branch,child_result,scalar_functions,shadowed:Default::default(),text_binding,reactive_props,condition,value_binding,next_closure,signal,location:names.get("location"),bind,retain,next:0,constructors:vec![]};
    for function in &mut program.functions {
        if names.values().any(|name|name==&function.name.text) {continue;}
        fn locals(body:&[Stmt],names:&mut std::collections::BTreeSet<String>) {
            let mut pending=body.iter().collect::<Vec<_>>();while let Some(statement)=pending.pop(){match &statement.kind {
                StmtKind::Let{name,..}=>{names.insert(name.text.clone());},
                StmtKind::LocalFunction(function)=>{names.insert(function.name.text.clone());pending.extend(&function.body);},
                StmtKind::Block(body)|StmtKind::While{body,..}=>pending.extend(body),
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
                StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);},
                StmtKind::ForOf{name,body,..}=>{names.insert(name.text.clone());pending.extend(body);},
                StmtKind::Try(region)=>{if let Some(catch)=&region.catch {if let Some(name)=&catch.name {names.insert(name.text.clone());}}pending.extend(region.statements());},
                _=>{}
            }}
        }
        let mut shadowed=function.params.iter().map(|(name,_,_)|name.text.clone()).collect::<std::collections::BTreeSet<_>>();locals(&function.body,&mut shadowed);
        crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Closure{params,body,..}=&expression.kind{for parameter in params{shadowed.insert(parameter.name.text.clone());}locals(body,&mut shadowed);}Ok(())})?;
        lower.shadowed=shadowed;
        lower.body(&mut function.body,&Scope::new(),&[],true)?;
        crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression| {
            if matches!(&expression.kind,ExprKind::Name(name) if name==signal || names.get("location").is_some_and(|location|name==location)) && !lower.constructors.contains(&expression.span) {
                return Err(Diagnostic::new("ui-state-owner","Create component state with a direct const declaration; signal constructor aliases are not supported.",expression.span));
            }
            Ok(())
        })?;
    }
    Ok(())
}
