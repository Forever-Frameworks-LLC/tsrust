//! Presence and default wrappers preserve the source pipeline's input contract.
use super::*;
use crate::stdlib::schema::pipelines::{Wrapper, WrapperKind};

impl Analyzer {
    pub(super) fn schema_pipeline_wrapper(
        &mut self, expr: &mut Expr, source: usize, receiver: Expr,
        kind: WrapperKind, factory: Option<Expr>, expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let (input, output, json_input)=self.schema_pipelines.types(source).unwrap_or_else(|| { let output=self.program.records[source].application.as_ref().unwrap().resolved_arguments[0]; (output,output,true) });
        let (input,output)=match kind {
            WrapperKind::Message=>(input,output),
            WrapperKind::Presence(added)=>(self.unions.intern(vec![input,added],expr.span)?,self.unions.intern(vec![output,added],expr.span)?),
            // Omission is tracked on the input side. A default factory returns
            // source input; normal source validation determines final output.
            WrapperKind::Default(_)=>(self.unions.intern(vec![input,Type::Undefined],expr.span)?,output),
        };
        let plan=Wrapper {source,kind,input,output,json_input};
        let id=if let Some(id)=self.schema_pipelines.wrapper_record(&plan){id}else{
            let id=self.program.records.len();
            let name=format!("@schema:pipeline-wrapper:{id}");
            let mut members=vec![("@schema",Type::Record(source))];
            if let WrapperKind::Default(family)=kind{members.push(("@factory",Type::StoredCallable(family)));}
            if kind==WrapperKind::Message{members.push(("@message",Type::String));}
            let fields=members.into_iter().map(|(name,ty)|Ok(RecordField {
                name:Token{text:name.into(),kind:TokenKind::Name,span:expr.span},
                annotation:self.checked_value_annotation(ty,expr.span)?,ty:Some(ty),callable_declaration:None,optional:false,
            })).collect::<Result<Vec<_>,Diagnostic>>()?;
            self.record_names.insert(name.clone(),id);
            self.fields.push(fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());
            self.program.records.push(Record {native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,application:None,
                name:Token{text:name,kind:TokenKind::Name,span:expr.span},type_parameters:vec![],fields});
            self.schema_pipelines.insert_wrapper(id,plan);id
        };
        self.require(Type::Record(id),expected,expr.span)?;
        let mut members=vec![(Token{text:"@schema".into(),kind:TokenKind::Name,span:expr.span},receiver)];
        if let Some(factory)=factory{members.push((Token{text:if kind==WrapperKind::Message{"@message"}else{"@factory"}.into(),kind:TokenKind::Name,span:expr.span},factory));}
        expr.kind=ExprKind::Record(members);
        Ok(Type::Record(id))
    }
}
