//! Checked, relative issue paths belong to the context's validated output type.
use super::*;
use crate::stdlib::schema::pipelines::IssueCall;

impl Analyzer {
    pub(in crate::analysis) fn schema_context_type(&mut self, output: Type, span: Span) -> Result<Type, Diagnostic> {
        use crate::native_signatures::NativeTypeId;
        if let Some(id) = self.program.records.iter().position(|record|
            record.native_type == Some(NativeTypeId::ValidationContext)
                && record.application.as_ref().is_some_and(|application| application.resolved_arguments == [output])) {
            return Ok(Type::Record(id));
        }
        let argument = self.checked_value_annotation(output, span)?;
        let id = self.program.records.len();
        let declaration = Token { text: crate::native_type_resolution::linked_name(NativeTypeId::ValidationContext), span, kind: TokenKind::Symbol };
        let name = format!("@schema:context:{id}");
        self.record_names.insert(name.clone(), id);
        self.fields.push(HashMap::new());
        self.program.records.push(Record {
            native_type: Some(NativeTypeId::ValidationContext), map_definition: None,
            set_definition: None, enum_definition: None, literal_definition: None,
            application: Some(Box::new(RecordApplication { declaration, arguments: vec![argument], resolved_arguments: vec![output] })),
            name: Token { text: name, span, kind: TokenKind::Symbol },
            type_parameters: Vec::new(), fields: Vec::new(),
        });
        Ok(Type::Record(id))
    }

    pub(in crate::analysis) fn schema_context_issue(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&expr.kind else{return Ok(None)};
        let ExprKind::Field(base,method)=&callee.kind else{return Ok(None)};
        if method!="addIssue" {return Ok(None);}
        let Some(Type::Record(id))=self.expression_type_hint(context,base) else{return Ok(None)};
        if self.program.records[id].native_type!=Some(crate::native_signatures::NativeTypeId::ValidationContext){return Ok(None);}
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let [options]=args.as_slice() else{return Err(Diagnostic::new("arity","addIssue expects one object containing message and an optional path.",expr.span));};
        let ExprKind::Record(fields)=&options.kind else{return Err(Diagnostic::new("schema-options","addIssue expects an object containing message and an optional path.",options.span));};
        let mut path=Vec::new();let mut message=None;let mut seen=HashSet::new();
        for (name,value) in fields {
            if !seen.insert(&name.text){return Err(Diagnostic::new("schema-options","Duplicate issue option.",name.span));}
            match name.text.as_str(){
                "path"=>path=self.schema_diagnostic_path(output,value)?,
                "message"=>message=Some(value.clone()),
                _=>return Err(Diagnostic::new("schema-options","Unknown issue option; use path or message.",name.span)),
            }
        }
        let mut message=message.ok_or_else(||Diagnostic::new("schema-options","addIssue requires a message.",options.span))?;
        let mut receiver=(**base).clone();
        self.expression(context,&mut receiver,Mode::Mutate,loans,None)?;
        self.expression(context,&mut message,Mode::Consume,loans,Some(Type::String))?;
        self.require(Type::Void,expected,expr.span)?;
        let id=self.schema_pipelines.intern_issue_call(IssueCall{context:id,output,path});
        expr.kind=ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@schema:addIssue".into()),expr.span)),vec![receiver,message]);
        expr.target=Some(CallTarget::SchemaAddIssue(id));
        Ok(Some(Type::Void))
    }
}
