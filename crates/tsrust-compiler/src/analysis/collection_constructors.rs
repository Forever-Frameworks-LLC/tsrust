//! Set and Map construction suspend around their optional source collection.
use super::*;
pub(super) struct Work { kind:Kind, target:Option<Type>, expected:Option<Type> }
enum Kind { Set { key:Option<Type> }, Map { target_types:Option<(Type,Type)> } }
pub(super) struct Input { pub value:Box<Expr>, pub expected:Option<Type> }
impl Analyzer {
    fn shadowed_collection(&self,context:&Context,name:&str)->bool{
        context.names.contains_key(name) || context.pending_names.contains(name) || context.lexical_scopes.iter().any(|scope| scope.contains(name)) || self.function_names.contains_key(name)
    }
    /// Checks the constructor's own contract and moves out its source, if any.
    pub(super) fn start_collection_constructor(&mut self,context:&Context,expr:&mut Expr,expected:Option<Type>)->Result<(Work,Option<Input>),Diagnostic>{
        let span=expr.span;
        let (arguments,annotation,set)=match &mut expr.kind{
            ExprKind::SetConstructor(arguments,annotation)=>(arguments,annotation,true),
            ExprKind::MapConstructor(arguments,annotation)=>(arguments,annotation,false),
            _=>unreachable!("collection constructor"),
        };
        if set{
            if arguments.len()>1{return Err(Diagnostic::new("arity","Set accepts zero or one source collection.",span));}
            if self.shadowed_collection(context,"Set"){return Err(Diagnostic::new("unsupported","A shadowed Set name cannot construct the built-in collection.",span));}
        }else{
            if arguments.len()>1{return Err(Diagnostic::new("arity","Map accepts zero or one entry source.",span));}
            if self.shadowed_collection(context,"Map"){return Err(Diagnostic::new("unsupported","A shadowed Map name cannot construct the built-in collection.",span));}
        }
        let declared=annotation.as_ref().map(|annotation|self.resolve_value_annotation(annotation,false)).transpose()?;
        let (kind,target,source_expected)=if set{
            if let Some(declared)=declared{self.require(declared,expected,span)?;}
            let target=declared.or(expected);
            let key=if let Some(Type::Set(id))=target{Some(self.program.records[id].set_definition.as_ref().expect("set definition").key)}else{None};
            let source_expected=if matches!(arguments.first().map(|source|&source.kind),Some(ExprKind::Array(_))){key.and_then(ArrayElement::from_value_type).map(Type::Array)}else{None};
            (Kind::Set{key},target,source_expected)
        }else{
            let empty_scoped=arguments.is_empty()&&declared.zip(expected).is_some_and(|(declared,expected)|self.scoped_map_contract(expected,declared));
            if let Some(declared)=declared && !empty_scoped{self.require(declared,expected,span)?;}
            let target=if empty_scoped{expected}else{declared.or(expected)};
            let target_types=if let Some(Type::Map(id))=target{
                let map=self.program.records[id].map_definition.as_ref().unwrap();Some((map.key,map.value))
            }else if let Some(source)=arguments.first(){self.map_literal_types(context,source)?}else{None};
            let mut source_expected=None;
            if let Some(source)=arguments.first() && matches!(source.kind,ExprKind::Array(_)) && let Some((key,value))=target_types{
                let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
                let mut pair=crate::type_syntax::ValueAnnotation{span:source.span,kind:crate::type_syntax::ValueAnnotationKind::Tuple(vec![types.annotation(key,source.span)?,types.annotation(value,source.span)?])};
                self.generic_records.materialize(&mut pair,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
                append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
                let pair=self.resolve_value_annotation(&pair,false)?;
                source_expected=ArrayElement::from_value_type(pair).map(Type::Array);
            }
            (Kind::Map{target_types},target,source_expected)
        };
        let input=arguments.first_mut().map(|source|{
            let span=source.span;
            Input{value:Box::new(std::mem::replace(source,Expr::new(ExprKind::Undefined,span))),expected:source_expected}
        });
        Ok((Work{kind,target,expected},input))
    }
    /// Restores the checked source before applying its result to the constructor type.
    pub(super) fn finish_collection_constructor(&mut self,expr:&mut Expr,work:Work,source:Option<(Box<Expr>,Result<Type,Diagnostic>)>)->Result<Type,Diagnostic>{
        let span=expr.span;
        let actual=match source{
            Some((value,actual))=>{
                let (ExprKind::SetConstructor(arguments,_)|ExprKind::MapConstructor(arguments,_))=&mut expr.kind else{unreachable!("collection constructor")};
                arguments[0]=*value;
                Some(actual?)
            },
            None=>None,
        };
        let source_span=|expr:&Expr|match &expr.kind{ExprKind::SetConstructor(arguments,_)|ExprKind::MapConstructor(arguments,_)=>arguments[0].span,_=>unreachable!()};
        let Work{kind,target,expected}=work;
        let (name,arguments)=match kind{
            Kind::Set{key}=>{
                let key=match actual{
                    Some(actual)=>{
                        let actual_key=match actual{
                            Type::Array(element) if element.depth()==0=>Some(element.ty()),
                            Type::Set(id)=>Some(self.program.records[id].set_definition.as_ref().expect("set definition").key),
                            Type::Null|Type::Undefined=>None,
                            Type::String=>Some(Type::String),
                            _=>return Err(Diagnostic::new("type","Set construction requires a dense array, string, or another Set.",source_span(expr))),
                        };
                        if let (Some(actual),Some(key))=(actual_key,key){self.require(actual,Some(key),source_span(expr))?;}
                        actual_key.or(key)
                    },
                    None=>key,
                };
                let Some(key)=key else{return Err(Diagnostic::new("set-type","An empty Set needs an explicit key type or Set annotation.",span));};
                if let Some(Type::Set(id))=target{return Ok(Type::Set(id));}
                let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
                ("Set",vec![types.annotation(key,span)?])
            },
            Kind::Map{target_types}=>{
                let source_types=match actual{
                    Some(actual)=>{
                        let types=match actual{
                            Type::Map(id)=>{let map=self.program.records[id].map_definition.as_ref().unwrap();Some((map.key,map.value))},
                            Type::Array(ArrayElement::Record(id,0)) if self.program.records[id].is_tuple()&&self.program.records[id].fields.len()==2=>{
                                let fields=&self.program.records[id].fields;Some((fields[0].ty.unwrap(),fields[1].ty.unwrap()))
                            },
                            Type::Null|Type::Undefined=>None,
                            _=>return Err(Diagnostic::new("type","Map construction requires an array of key/value tuples or another Map.",source_span(expr))),
                        };
                        if let Some((key,value))=types{
                            let source=source_span(expr);
                            if let Some((target_key,target_value))=target_types{self.require(key,Some(target_key),source)?;self.require(value,Some(target_value),source)?;}
                            self.require_cloneable(value,source)?;
                        }
                        types.or(target_types)
                    },
                    None=>target_types,
                };
                let Some((key,value))=source_types else{return Err(Diagnostic::new("map-type","An empty Map needs key/value types or a Map annotation.",span));};
                if let Some(Type::Map(id))=target{return Ok(Type::Map(id));}
                let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
                ("Map",vec![types.annotation(key,span)?,types.annotation(value,span)?])
            },
        };
        let mut annotation=crate::type_syntax::ValueAnnotation{span,kind:crate::type_syntax::ValueAnnotationKind::Applied{name:crate::type_syntax::TypeName{text:name.into(),span},arguments}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        // Set keys need only field slots; Map tuples also publish checked fields.
        if name=="Set"{while self.fields.len()<self.program.records.len(){self.fields.push(HashMap::new());}}
        else{append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;}
        let result=self.resolve_value_annotation(&annotation,false)?;
        self.require(result,expected,span)?;
        Ok(result)
    }
}
