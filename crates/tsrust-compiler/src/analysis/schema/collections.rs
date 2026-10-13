//! Collection schemas own exact child plans and preserve input/output domains.
use super::*;
use crate::stdlib::schema::pipelines::{CollectionKind,CollectionPlan};

impl Analyzer {
    pub(in crate::analysis) fn schema_collection_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
        let kind=match name.as_str(){
            "@schema:record"=>CollectionKind::Record,
            "@schema:map"=>CollectionKind::Map,
            "@schema:set"=>CollectionKind::Set,
            _=>return Ok(None),
        };
        let count=if kind==CollectionKind::Map{2}else{1};
        if args.len()!=count{return Err(Diagnostic::new("arity",format!("Collection constructor expects {count} owned child schemas."),expr.span));}
        let mut children=Vec::new();let mut contracts=Vec::new();
        for arg in args.iter_mut(){
            let child=self.expression(context,arg,Mode::Consume,loans,None)?;
            let Type::Record(id)=child else{return Err(Diagnostic::new("schema-type","Collection elements and keys require schemas.",arg.span))};
            let types=if let Some(types)=self.schema_pipelines.types(id){types}
                else if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema){
                    let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];(output,output,true)
                }else{return Err(Diagnostic::new("schema-type","Collection elements and keys require schemas.",arg.span))};
            children.push(id);contracts.push(types);
        }
        let element=*children.last().unwrap();let (element_input,element_output,json)=*contracts.last().unwrap();
        let key=(kind==CollectionKind::Map).then(||children[0]);
        let (key_input,key_output)=if kind==CollectionKind::Map{(contracts[0].0,contracts[0].1)}else{(Type::String,Type::String)};
        let input=self.schema_collection_value_type(kind,key_input,element_input,expr.span)?;
        let output=self.schema_collection_value_type(kind,key_output,element_output,expr.span)?;
        let plan=CollectionPlan{kind,key,element,input,output,json_input:kind==CollectionKind::Record&&json};
        let id=if let Some(id)=self.schema_pipelines.collection_record(&plan){id}else{
            let id=self.program.records.len();let name=format!("@schema:pipeline-collection:{id}");
            let fields=children.iter().enumerate().map(|(index,&child)|Ok(RecordField{
                name:Token{text:format!("@child{index}"),span:expr.span,kind:TokenKind::Name},
                annotation:self.checked_value_annotation(Type::Record(child),expr.span)?,
                ty:Some(Type::Record(child)),callable_declaration:None,optional:false,
            })).collect::<Result<Vec<_>,Diagnostic>>()?;
            self.record_names.insert(name.clone(),id);
            self.fields.push(fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());
            self.program.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,application:None,
                name:Token{text:name,span:expr.span,kind:TokenKind::Name},type_parameters:Vec::new(),fields});
            self.schema_pipelines.insert_collection(id,plan);id
        };
        self.require(Type::Record(id),expected,expr.span)?;
        let values=std::mem::take(args).into_iter().enumerate().map(|(index,value)|(Token{text:format!("@child{index}"),span:expr.span,kind:TokenKind::Name},value)).collect();
        expr.kind=ExprKind::Record(values);
        Ok(Some(Type::Record(id)))
    }

    fn schema_collection_value_type(&mut self,kind:CollectionKind,key:Type,value:Type,span:Span)->Result<Type,Diagnostic>{
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind,TypeName};
        let arguments=if kind==CollectionKind::Set{vec![self.checked_value_annotation(value,span)?]}else{
            vec![self.checked_value_annotation(key,span)?,self.checked_value_annotation(value,span)?]
        };
        let mut annotation=ValueAnnotation{span,kind:ValueAnnotationKind::Applied{
            name:TypeName{text:if kind==CollectionKind::Set{"Set"}else{"Map"}.into(),span},arguments,
        }};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        self.resolve_value_annotation(&annotation,false)
    }
}
