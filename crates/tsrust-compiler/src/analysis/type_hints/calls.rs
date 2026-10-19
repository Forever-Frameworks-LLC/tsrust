//! Call hints suspend receiver, schema-branch and callback-result dependencies.
use super::*;
#[derive(Clone,Copy,PartialEq,Eq)]
pub(in crate::analysis) enum Flavor{Named,Wrapped,General,Record,Callback}
#[derive(Clone,Copy)]
pub(in crate::analysis) enum SchemaReceiver{Export,Constraint}
pub(in crate::analysis) enum Frame<'a>{
    Start(&'a Expr),
    SchemaReceiver{expr:&'a Expr,kind:SchemaReceiver},
    SchemaBranch{expr:&'a Expr,branches:&'a [Expr],index:usize,first:Option<(usize,Type)>,same:bool},
    Result(Flavor),Own,Unwrap,
    Receiver(&'a Expr),RecordField{expr:&'a Expr,receiver:Option<Type>},
    Callback{name:&'a str,span:Span},SetArgument(usize),
}
fn query<'a>(work:&mut Vec<Work<'a>>,frame:Frame<'a>,child:&'a Expr){work.push(Work::Call(frame));work.push(Work::Expression(child));}
pub(super) fn step<'a>(analyzer:&Analyzer,context:&Context,frame:Frame<'a>,work:&mut Vec<Work<'a>>,value:&mut Option<Type>){
    match frame{
        Frame::Start(expr)=>{
            let ExprKind::Call(callee,args)=&expr.kind else{unreachable!("call hint")};
            if let ExprKind::Field(base,name)=&callee.kind{
                if name=="toJsonSchema" && args.is_empty(){query(work,Frame::SchemaReceiver{expr,kind:SchemaReceiver::Export},base);return;}
                if matches!(name.as_str(),"min"|"max"|"gt"|"lt"|"default"){query(work,Frame::SchemaReceiver{expr,kind:SchemaReceiver::Constraint},base);return;}
            }
            if matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:union")
                && let [argument]=args.as_slice() && let ExprKind::Array(branches)=&argument.kind && let Some(first)=branches.first(){
                query(work,Frame::SchemaBranch{expr,branches,index:0,first:None,same:true},first);return;
            }
            regular(analyzer,context,expr,work,value);
        },
        Frame::SchemaReceiver{expr,kind}=>{
            if let Some(Type::Record(id))=*value{
                match kind{
                    SchemaReceiver::Export if analyzer.schema_pipelines.types(id).is_some() || analyzer.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema)=>{
                        *value=analyzer.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)).map(Type::Record);return;
                    },
                    SchemaReceiver::Constraint if analyzer.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema)=>{
                        let ExprKind::Call(callee,_)=&expr.kind else{unreachable!()};let ExprKind::Field(_,name)=&callee.kind else{unreachable!()};
                        if name=="default" || matches!(analyzer.program.records[id].application.as_ref().and_then(|application|application.resolved_arguments.first()),Some(Type::String|Type::Int|Type::Number)){return;}
                    },_=>{},
                }
            }
            regular(analyzer,context,expr,work,value);
        },
        Frame::SchemaBranch{expr,branches,index,mut first,mut same}=>{
            let current=value.and_then(|ty|{
                let Type::Record(id)=ty else{return None};let record=&analyzer.program.records[id];
                if record.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return None;}
                Some((id,*record.application.as_ref()?.resolved_arguments.first()?))
            });
            let Some(current)=current else{regular(analyzer,context,expr,work,value);return};
            if let Some((_,output))=first{same &= output==current.1;}else{first=Some(current);}
            if let Some(next)=branches.get(index+1){query(work,Frame::SchemaBranch{expr,branches,index:index+1,first,same},next);return;}
            if let Some((id,output))=first && same && matches!(output,Type::String|Type::Int|Type::Number){*value=Some(Type::Record(id));return;}
            regular(analyzer,context,expr,work,value);
        },
        Frame::Result(flavor)=>*value=value.and_then(|ty|analyzer.call_hint_result(ty,flavor)),
        Frame::Own=>*value=value.map(Type::owning),
        Frame::Unwrap=>*value=value.and_then(|ty|unwrapped(analyzer,ty)),
        Frame::Receiver(expr)=>{
            let receiver=*value;
            if matches!(receiver,Some(Type::Record(_))){let ExprKind::Call(callee,_)=&expr.kind else{unreachable!()};query(work,Frame::RecordField{expr,receiver},callee);}
            else{method(analyzer,context,expr,receiver,work,value);}
        },
        Frame::RecordField{expr,receiver}=>{
            let result=value.and_then(|ty|analyzer.call_hint_result(ty,Flavor::Record));
            if result.is_some(){*value=result;}else{method(analyzer,context,expr,receiver,work,value);}
        },
        Frame::Callback{name,span}=>*value=value.and_then(|ty|analyzer.call_hint_result(ty,Flavor::Callback)).and_then(|ty|analyzer.array_callback_hint_result(name,span,ty)),
        Frame::SetArgument(id)=>*value=value.and_then(|ty|{
            let Type::Set(other)=ty else{return None};
            (analyzer.program.records[id].set_definition.as_ref()?.key==analyzer.program.records[other].set_definition.as_ref()?.key).then_some(Type::Set(id))
        }),
    }
}
fn regular<'a>(analyzer:&Analyzer,context:&Context,expr:&'a Expr,work:&mut Vec<Work<'a>>,value:&mut Option<Type>){
    if let Some(ready)=analyzer.type_hint_ready(context,expr){*value=ready;return;}
    let ExprKind::Call(callee,arguments)=&expr.kind else{unreachable!()};
    if matches!(callee.kind,ExprKind::Conditional{..}|ExprKind::Binary{op:BinaryOp::Coalesce|BinaryOp::PatternDefault,..}|ExprKind::Unary{op:UnaryOp::RestPack|UnaryOp::CoalesceSelected|UnaryOp::OptionalSelected|UnaryOp::NonNull|UnaryOp::OptionalBoundary,..}){
        query(work,Frame::Result(Flavor::Wrapped),callee);return;
    }
    if let ExprKind::Name(name)=&callee.kind{
        if name=="@native:std:result#unwrap" && arguments.len()==1 {query(work,Frame::Unwrap,&arguments[0]);return;}
        if let Some((_,binding))=crate::native_bindings::by_symbol(name){
            *value=if matches!(binding.module,"std:threads"|"std:result"|"std:futures") || matches!(binding.result,crate::native_bindings::NativeType::SharedCallback|crate::native_bindings::NativeType::ObservedValue){None}else{Some(native_type(binding.result,&analyzer.program.records))};return;
        }
        if !context.names.contains_key(name) && !context.pending_names.contains(name) && !analyzer.function_names.contains_key(name)
            && let Some(member)=crate::syntax::global_number_member(name,true){*value=Some(if member.starts_with("parse"){Type::Number}else{Type::Boolean});return;}
        if name=="clone" && arguments.len()==1{query(work,Frame::Own,&arguments[0]);return;}
        *value=match name.as_str(){
            "String"=>Some(Type::String),"Boolean"=>Some(Type::Boolean),"Number"=>Some(Type::Number),
            _=>if let Some(id)=context.names.get(name){analyzer.call_hint_result(context.bindings[*id].ty,Flavor::Named)}else{analyzer.function_names.get(name).map(|id|analyzer.signatures[*id].result)},
        };return;
    }
    if let ExprKind::Field(base,_)=&callee.kind{query(work,Frame::Receiver(expr),base);return;}
    query(work,Frame::Result(Flavor::General),callee);
}
fn method<'a>(analyzer:&Analyzer,context:&Context,expr:&'a Expr,receiver:Option<Type>,work:&mut Vec<Work<'a>>,value:&mut Option<Type>){
    let ExprKind::Call(callee,_)=&expr.kind else{unreachable!()};
    if let ExprKind::Field(_,member)=&callee.kind && let Some(Type::Record(id))=receiver && let Some(operation)=crate::stdlib::sqlite::method(analyzer.program.records[id].native_type,member){
        let binding=crate::native_bindings::lookup("std:sqlite",operation).ok().unwrap();*value=Some(native_type(binding.result,&analyzer.program.records));return;
    }
    if let ExprKind::Field(_,member)=&callee.kind && let Some(result)=http_result(analyzer,receiver,member,false){*value=Some(result);return;}
    let ExprKind::Call(callee,arguments)=&expr.kind else{unreachable!()};let ExprKind::Field(base,name)=&callee.kind else{unreachable!()};
    if let Some(Type::Record(id))=receiver && analyzer.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::HttpContext) && name=="get" {
        *value=(||{let ExprKind::String(key)=&arguments.first()?.kind else{return None;};let Type::Record(variables)=analyzer.program.records[id].application.as_ref()?.resolved_arguments[1]else{return None;};let ty=analyzer.program.records[variables].fields.iter().find(|field|field.name.text==*key)?.ty?;analyzer.unions.intern([ty,Type::Undefined],expr.span).ok().map(|ty|if ty.is_copy(){ty}else{Type::BorrowedUnion(ty.union().unwrap())})})();return;
    }
    if let Some(Type::Record(id))=receiver && analyzer.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::UiSignal) { *value=match name.as_str(){"get"=>analyzer.program.records[id].application.as_ref().map(|a|a.resolved_arguments[0]),"set"=>Some(Type::Void),_=>None};return; }
    if let Some(Type::Record(id))=receiver && crate::stdlib::cache::is_cache(analyzer.program.records[id].native_type){
        *value=match name.as_str() {
            "share"=>Some(Type::Record(id)),
            "getCloned"|"peekCloned"=>analyzer.unions.intern([analyzer.program.records[id].application.as_ref().expect("applied cache").resolved_arguments[1],Type::Undefined],expr.span).ok(),
            "get"|"peek"=>analyzer.unions.intern([analyzer.program.records[id].application.as_ref().expect("applied cache").resolved_arguments[1],Type::Undefined],expr.span).ok().map(|ty|if ty.is_copy(){ty}else{Type::BorrowedUnion(ty.union().unwrap())}),
            "size"=>Some(Type::Int),"clear"=>Some(Type::Void),_=>None,
        };return;
    }
    if let Some(Type::Record(id))=receiver && analyzer.program.records[id].native_element().is_some(){*value=analyzer.native_collection_method_hint(id,name,expr.span);return;}
    if let Some(Type::Map(id))=receiver{
        *value=analyzer.map_method_type(id,name,expr.span).ok().and_then(|result|{
            if name=="get" && Analyzer::stable_at_receiver(base) && !result.is_copy(){result.union().map(Type::BorrowedUnion)}else{Some(result)}
        });return;
    }
    if let Some(Type::Set(id))=receiver{
        if matches!(name.as_str(),"union"|"symmetricDifference"){
            *value=None;if let Some(argument)=arguments.first(){query(work,Frame::SetArgument(id),argument);}
        }else{*value=analyzer.set_method_type(id,name);}return;
    }
    if matches!(name.as_str(),"reduce"|"reduceRight"|"map"|"flatMap"){
        *value=None;let Some(argument)=arguments.first() else{return};
        if let ExprKind::Closure{result:ResultAnnotation::Named(annotation),..}=&argument.kind{
            *value=analyzer.resolve_value_annotation(annotation,true).ok().and_then(|ty|analyzer.array_callback_hint_result(name,argument.span,ty));
        }else{query(work,Frame::Callback{name,span:argument.span},argument);}
        return;
    }
    *value=analyzer.simple_method_hint(context,expr,receiver);
}
impl Analyzer{
    fn call_hint_result(&self,ty:Type,flavor:Flavor)->Option<Type>{
        match ty{
            Type::Closure(id) if flavor!=Flavor::Record && self.deferred_generic_values.contains_key(&id)=>None,
            Type::Closure(id)=>Some(self.closures[id].function.result),
            Type::Callback(id)=>Some(self.callables.callbacks[id].result),
            Type::Callable(id) if matches!(flavor,Flavor::General|Flavor::Callback)=>Some(self.callables.results[id].result),
            Type::StoredCallable(id) if flavor!=Flavor::Callback=>self.callables.storage.declaration(id).map(|id|self.callables.callbacks[id].result),
            Type::BorrowedCallable(id) if matches!(flavor,Flavor::Wrapped|Flavor::Record)=>self.callables.storage.declaration(id).map(|id|self.callables.callbacks[id].result),
            _=>None,
        }
    }
    fn array_callback_hint_result(&self,name:&str,span:Span,result:Type)->Option<Type>{
        if matches!(name,"reduce"|"reduceRight") || (name=="flatMap" && matches!(result,Type::Array(_))){Some(result)}else{Self::transform_array_type(result,span).ok()}
    }
    fn simple_method_hint(&self,context:&Context,expr:&Expr,receiver_hint:Option<Type>)->Option<Type>{
        let ExprKind::Call(callee,arguments)=&expr.kind else{unreachable!()};
        let ExprKind::Field(base,name)=&callee.kind else{unreachable!()};
                    if name == "isArray"
                        && matches!(&base.kind,ExprKind::Name(namespace) if namespace=="Array")
                    {
                        return Some(Type::Boolean);
                    }

                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Object") && name == "keys" {
                        return Some(Type::Array(ArrayElement::String(0)));
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Object") && name == "is" {
                        return Some(Type::Boolean);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "String" && !context.names.contains_key(namespace))
                        && matches!(name.as_str(), "fromCodePoint" | "fromCharCode")
                    {
                        return Some(Type::String);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Number")
                        && matches!(name.as_str(), "parseFloat" | "parseInt")
                    {
                        return Some(Type::Number);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Number")
                        && number_predicate_target(name).is_some()
                    {
                        return Some(Type::Boolean);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Math")
                        && math_target(name).is_some()
                    {
                        return Some(Type::Number);
                    }
                    return match name.as_str() {
                        "trim" | "trimStart" | "trimEnd" | "trimLeft" | "trimRight"
                        | "toLowerCase" | "toUpperCase" | "toLocaleLowerCase" | "toLocaleUpperCase"
                        | "charAt" | "repeat" | "replace"
                        | "replaceAll" | "padStart" | "padEnd" | "substring" | "substr"
                        | "toFixed" | "toExponential" | "toPrecision" => Some(Type::String),
                        "isWellFormed" => Some(Type::Boolean),
                        "toWellFormed" => Some(Type::String),
                        STRING_CODE_POINT_SLOT => Some(Type::Array(ArrayElement::Int(0))),
                        "charCodeAt" => Some(Type::Number),
                        method if string_html_target(method).is_some() => Some(Type::String),
                        "valueOf" => receiver_hint.filter(|ty| {
                            matches!(ty, Type::String | Type::Boolean | Type::Number | Type::Int)
                        }),
                        "toString" => receiver_hint
                            .filter(|ty| {
                                matches!(
                                    ty,
                                    Type::String | Type::Boolean | Type::Int | Type::Array(_)
                                )
                            })
                            .map(|_| Type::String),
                        "split" => Some(Type::Array(ArrayElement::String(0))),
                        "join" => Some(Type::String),
                        "some" | "every" => Some(Type::Boolean),
                        "findIndex" | "findLastIndex" | "unshift" => Some(Type::Int),
                        "concat" => receiver_hint
                            .filter(|ty| matches!(ty, Type::String | Type::Array(_))),
                        "entries" => {
                            let Type::Array(element) = receiver_hint? else { return None; };
                            let label = format!("[int,{}]",self.type_name(element.ty()));
                            self.record_names.get(&label).copied().map(|pair|Type::Array(ArrayElement::Record(pair,0)))
                        }
                        "keys" => receiver_hint
                            .filter(|ty| matches!(ty, Type::Array(_)))
                            .map(|_| Type::Array(ArrayElement::Int(0))),
                        "filter" | "toReversed" | "toSorted" | "with" | "toSpliced" | "splice" | "values" => {
                            receiver_hint
                                .filter(|ty| matches!(ty, Type::Array(_)))
                        }
                        "flat" => {
                            let Type::Array(element) = receiver_hint?
                            else {
                                return None;
                            };
                            let depth = match arguments.first() {
                                None => 1,
                                Some(arg) => Self::flat_literal_depth(arg)
                                    .or_else(|| (element.depth() == 0).then_some(0))?,
                            };
                            Some(Type::Array(element.with_depth(
                                element.depth().saturating_sub(depth.clamp(0, 127) as u8),
                            )))
                        }
                        "slice" => receiver_hint
                            .filter(|ty| matches!(ty, Type::String | Type::Array(_))),
                        "at" | "find" | "findLast" => {
                            match receiver_hint? {
                                Type::Array(element) => {
                                    let result = self
                                        .unions
                                        .intern([element.ty(), Type::Undefined], expr.span)
                                        .ok()?;
                                    Some(if Self::stable_at_receiver(base) && !result.is_copy() {
                                        Type::BorrowedUnion(result.union()?)
                                    } else {
                                        result
                                    })
                                }
                                _ => None,
                            }
                        }
                        "pop" | "shift" => match receiver_hint? {
                            Type::Array(element) => self
                                .unions
                                .intern([element.ty(), Type::Undefined], expr.span)
                                .ok(),
                            _ => None,
                        },
                        "remove" => match receiver_hint? {
                            Type::Array(element) => Some(element.ty()),
                            _ => None,
                        },
                        _ => None,
                    };
    }

}

#[cfg(test)]
mod tests{
    use super::*;
    fn name(value:&str)->Expr{Expr::new(ExprKind::Name(value.into()),Span::default())}
    fn call(callee:Expr,args:Vec<Expr>)->Expr{Expr::new(ExprKind::Call(Box::new(callee),args),Span::default())}
    #[test]
    fn nested_native_receiver_and_clone_hints_share_the_structural_scheduler(){
        let analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();let context=Context::new("main".into());
        let mut value=Expr::new(ExprKind::String("value".into()),Span::default());
        for layer in 0..16_384{
            value=if layer%2==0{call(Expr::new(ExprKind::Field(Box::new(value),"trim".into()),Span::default()),vec![])}else{call(name("clone"),vec![value])};
        }
        assert_eq!(analyzer.expression_type_hint(&context,&value),Some(Type::String));drop(value);
    }
    #[test]
    fn nested_schema_constraints_and_union_hints_keep_identity_before_checked_metadata(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Box={value:int;};function main(){}").unwrap()).unwrap();let mut context=Context::new("main".into());
        let id=analyzer.record_names["Box"];
        let record=&mut analyzer.program.records[id];record.native_type=Some(crate::native_signatures::NativeTypeId::Schema);
        record.application=Some(Box::new(crate::syntax::RecordApplication{resolved_arguments:vec![Type::Int],declaration:record.name.clone(),arguments:vec![]}));
        Analyzer::bind(&mut context,&Token{text:"schema".into(),kind:TokenKind::Name,span:Span::default()},Type::Record(id),false).unwrap();
        let mut value=name("schema");
        for layer in 0..16_384{
            value=if layer%2==0{call(Expr::new(ExprKind::Field(Box::new(value),"default".into()),Span::default()),vec![Expr::new(ExprKind::Integer("1".into()),Span::default())])}
                else{call(name("@schema:union"),vec![Expr::new(ExprKind::Array(vec![value,name("schema")]),Span::default())])};
            value.ty=Some(Type::Boolean);
        }
        assert_eq!(analyzer.expression_type_hint(&context,&value),Some(Type::Record(id)));drop(value);
    }
}

pub(super) fn unwrapped(analyzer:&Analyzer,ty:Type)->Option<Type>{let Type::Union(id)=ty else{return None};let [ok,_]=crate::result_identity::variants(&analyzer.program.records,&analyzer.unions.members(id))?;Some(analyzer.program.records[ok].fields.first().and_then(|field|field.ty).unwrap_or(Type::Void))}
pub(super) fn http_result(analyzer:&Analyzer,receiver:Option<Type>,member:&str,awaited:bool)->Option<Type>{
 let Type::Record(id)=receiver? else{return None};let identity=analyzer.program.records[id].native_type;
 if let Some(operation)=crate::stdlib::sql::method(identity,member){if crate::stdlib::sql::asynchronous(operation)!=awaited{return None;}let binding=crate::native_bindings::lookup("std:sql",operation).ok()?;let ty=native_type(binding.result,&analyzer.program.records);return Some(if awaited{unwrapped(analyzer,ty).unwrap_or(ty)}else{ty});}
 let operation=crate::stdlib::http::method(identity,member)?;
 if crate::stdlib::http::asynchronous(operation)!=awaited{return None;}
 let binding=crate::native_bindings::lookup("std:http",operation).ok()?;if binding.result==crate::native_bindings::NativeType::ObservedValue{return None;}let ty=native_type(binding.result,&analyzer.program.records);Some(unwrapped(analyzer,ty).unwrap_or(ty))
}
