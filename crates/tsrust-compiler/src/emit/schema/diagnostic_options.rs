//! Shared emission of checked relative paths and schema-owned messages.
use super::*;
use crate::stdlib::schema::pipelines::{DiagnosticSegment, Stage};

impl Emitter<'_> {
    pub(in crate::emit) fn schema_constraint_options_value(&self,fun:&CheckedFunction,expr:&Expr,args:&[Expr],index:usize)->String {
        use crate::stdlib::schema::pipelines::WrapperKind;
        let Type::Record(id)=expr.ty.unwrap() else{unreachable!()};
        let plan=self.program.schema_pipelines.wrapper(id).unwrap();
        let WrapperKind::Constraint(options)=plan.kind else{unreachable!()};
        let ExprKind::Record(fields)=&args[1].kind else{unreachable!()};
        let bound=&fields.iter().find(|(name,_)|name.text=="bound").unwrap().1;
        let message=&fields.iter().find(|(name,_)|name.text=="message").unwrap().1;
        let ExprKind::Call(_,bound_args)=&bound.kind else{unreachable!()};
        let receiver=self.expr(fun,&args[0],Mode::Consume);
        let scalar=self.expr(fun,&bound_args[0],Mode::Consume);
        let limit=self.expr(fun,&bound_args[1],Mode::Copy);
        let message=self.expr(fun,message,Mode::Consume);
        let mut source=plan.source;
        let mut access="constraint_source".to_owned();
        let inherited=loop {
            if let Some(wrapper)=self.program.schema_pipelines.wrapper(source) {
                match wrapper.kind {
                    WrapperKind::Constraint(_)=>break Some(format!("{access}.field1.field0.bounds")),
                    WrapperKind::Message|WrapperKind::Default(_)=>{source=wrapper.source;access.push_str(".field0");},
                    _=>break None,
                }
            }else if let Some(stage)=self.program.schema_pipelines.get(source) {
                if stage.operation==crate::stdlib::schema::pipelines::Operation::Refine {
                    source=stage.source;access.push_str(".field0");
                }else{break None;}
            }else if self.program.records[source].native_type==Some(crate::native_signatures::NativeTypeId::Schema) {
                break Some(format!("{access}.bounds"));
            }else{break None;}
        };
        // Bounds contain only numeric endpoints. Copying these scalars does not
        // copy schema plans, callback environments or message strings.
        let inherit=inherited.map(|bounds|format!("constraint_scalar.bounds={bounds};")).unwrap_or_default();
        let name=self.native_call_name(index,bound_args);
        let call=format!("{name}(constraint_scalar,constraint_limit)");
        let call=if let Some((_,source_name))=self.result_functions.borrow().get(&index) {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
            self.exception_regions.function_result_call(&call,index,source_name,&self.program.error_effects,&unions,expr.span).expect("checked constraint construction")
        }else{call};
        let options_native=self.native_record_declaration(options);
        let fields=self.order_values(options,fields,"field0:constraint_bound,field1:constraint_message".into());
        let native=self.native_record_declaration(id);
        format!("{{let constraint_source={receiver};let mut constraint_scalar={scalar};let constraint_limit={limit};let constraint_message={message};{inherit}let constraint_bound={call};Record{native}{{field0:constraint_source,field1:Record{options_native}{{{fields}}}}}}}")
    }

    pub(in crate::emit) fn schema_refinement_path_setup(&self, stage: &Stage) -> String {
        if stage.diagnostic_path.is_empty() { return "let refinement_path=&*path;".into(); }
        let mut setup = String::from("let mut refinement_path=path.clone();");
        for segment in &stage.diagnostic_path {
            setup.push_str(&match segment {
                DiagnosticSegment::Field(name) => format!("refinement_path.push(tsrust_schema_issues::Segment::Field({name:?}));"),
                DiagnosticSegment::Index(index) => format!("refinement_path.push(tsrust_schema_issues::Segment::Index({index}));"),
            });
        }
        setup
    }
    pub(in crate::emit) fn schema_refinement_message(&self, stage: &Stage) -> &'static str {
        if stage.custom_message { "issues.message_since(issue_start,schema.field2.as_ref());" } else { "" }
    }
}
