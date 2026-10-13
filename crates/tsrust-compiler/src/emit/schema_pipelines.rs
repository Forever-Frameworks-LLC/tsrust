//! Execute owned pipeline stages while borrowing their reusable callback plans.
use super::*;

impl Emitter<'_> {
    pub(super) fn schema_root_parameters(&self,id:usize)->String {
        self.program.schema_pipelines.borrowed_roots(id).into_iter().map(|root|
            format!("schema_root{root}:&Record{},",self.native_record_declaration(root))).collect()
    }

    pub(super) fn schema_root_arguments(&self,id:usize)->String {
        self.program.schema_pipelines.borrowed_roots(id).into_iter().map(|root|
            format!("schema_root{root},")).collect()
    }

    pub(super) fn schema_pipeline_callback_result(&self, family: usize, receiver: &str, errors: &crate::error_effects::FailureSet) -> String {
        let declaration = self.program.stored_contracts.declaration(family).expect("checked pipeline callback");
        let argument = match self.program.callback_contracts[declaration].params.first() {
            None => "",
            Some(parameter) => match parameter.mode {
                Mode::Read if matches!(parameter.ty,Type::Union(union) if !union.copy) => "value.tsr_view()",
                Mode::Read => "&value", Mode::Mutate => "&mut value",
                Mode::Copy | Mode::Consume => "value",
            },
        };
        let context = self.program.callback_contracts[declaration].params.get(1).map(|parameter| {
            if parameter.mode == Mode::Mutate { ",&mut issue_context" } else { ",&issue_context" }
        }).unwrap_or("");
        let call = format!("{receiver}.call({argument}{context})");
        if let Some((source_name, source)) = self.result_storage.borrow().get(&family).cloned() {
            let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let runtime = self.program.records.iter().position(|record| record.name.text == "#RuntimeError"
                && record.name.kind == crate::syntax::TokenKind::Symbol).map(Type::Record);
            let mapping = source.native_injection(errors, &unions, runtime, crate::Span::default()).expect("schema callback error injection");
            let conversion = super::exception_emit::error_conversion("error", &source_name, "SchemaFactoryError", &mapping);
            format!("{call}.map_err(|error|{conversion})")
        } else { format!("Ok({call})") }
    }

    pub(super) fn schema_issue_context_setup(&self, family: usize) -> String {
        let declaration = self.program.stored_contracts.declaration(family).expect("checked refinement callback");
        let Type::Record(context) = self.program.callback_contracts[declaration].params[1].ty else { unreachable!() };
        format!("let mut issue_context=Record{}{{issues:tsrust_schema_issues::Issues::new(64,32,1024)}};", self.native_record_declaration(context))
    }

    pub(super) fn schema_pipeline_validator(
        &self, id: usize, input: Type, generated: &mut String,
        seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet,
    ) -> String {
        let name = format!("pipeline{id}_{input:?}").chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' }).collect::<String>();
        let marker = format!("/*{name}*/");
        if generated.contains(&marker) { return name; }
        generated.push_str(&marker);
        let trial = self.schema_pipeline_trial(id, input, generated, seen, errors);
        let roots=self.schema_root_parameters(id);
        let root_args=self.schema_root_arguments(id);
        generated.push_str(&format!("fn {name}({roots}schema:&Record{},input:{},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->Option<{}>{{if failure.is_some(){{return None;}}match {trial}({root_args}schema,input,path,issues,failure).finish(){{Ok(value)=>value,Err(error)=>{{*failure=Some(error);None}}}}}}",self.native_record_declaration(id),self.native_type(input),self.native_type(self.schema_pipeline_output(id))));
        name
    }
}
