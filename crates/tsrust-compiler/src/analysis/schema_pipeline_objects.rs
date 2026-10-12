//! Object plans retain each field's actual schema, input and transformed output.
use super::*;

impl Analyzer {
    pub(super) fn schema_pipeline_object_policy(&mut self, expr: &mut Expr, source: usize, receiver: Expr, retaining: bool, expected: Option<Type>) -> Result<Type, Diagnostic> {
        use crate::stdlib::schema::pipelines::ObjectPolicy;
        let mut plan = self.schema_pipelines.object(source).unwrap().clone();
        if plan.policy == ObjectPolicy::Retain {
            return Err(Diagnostic::new("schema-member", "Object policy is already retained.", expr.span));
        }
        plan.policy = if retaining { ObjectPolicy::Retain } else { ObjectPolicy::Strip };
        if retaining { plan.output = self.schema_retained_value_type(plan.value, expr.span)?; }
        let id = if let Some(id) = self.schema_pipelines.object_record(&plan) { id } else {
            let id = self.program.records.len();
            let mut record = self.program.records[source].clone();
            record.name.text = format!("@schema:pipeline-object:{id}");
            record.name.span = expr.span;
            self.record_names.insert(record.name.text.clone(), id);
            self.fields.push(record.fields.iter().map(|field| (field.name.text.clone(), field.ty.unwrap())).collect());
            self.program.records.push(record);
            self.schema_pipelines.insert_object(id, plan);
            id
        };
        self.require(Type::Record(id), expected, expr.span)?;
        let shape = self.schema_pipelines.object(id).unwrap().shape;
        let mut field = Expr::new(ExprKind::Field(Box::new(receiver), "@shape".into()), expr.span);
        field.ty = Some(Type::Record(shape));
        expr.kind = ExprKind::Record(vec![(Token { text: "@shape".into(), span: expr.span, kind: TokenKind::Name }, field)]);
        Ok(Type::Record(id))
    }

    pub(super) fn schema_pipeline_shape(&mut self, fields: Vec<RecordField>, flavor: &str, span: Span) -> usize {
        let prefix = format!("@schema:{flavor}:");
        if let Some(id) = self.program.records.iter().position(|record|
            record.name.text.starts_with(&prefix) && record.fields.len() == fields.len()
                && record.fields.iter().zip(&fields).all(|(left, right)|
                    left.name.text == right.name.text && left.ty == right.ty && left.optional == right.optional)) {
            return id;
        }
        let id = self.program.records.len();
        let name = format!("{prefix}{id}");
        self.record_names.insert(name.clone(), id);
        self.fields.push(fields.iter().map(|field| (field.name.text.clone(), field.ty.unwrap())).collect());
        self.program.records.push(Record {
            native_type: None, map_definition: None, set_definition: None, enum_definition: None,
            literal_definition: None, application: None,
            name: Token { text: name, span, kind: TokenKind::Name },
            type_parameters: Vec::new(), fields,
        });
        id
    }

    pub(super) fn schema_pipeline_object(&mut self, expr: &mut Expr, shape: usize, value: Expr, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let mut inputs = self.program.records[shape].fields.clone();
        let mut outputs = inputs.clone();
        let mut children = Vec::new();
        let mut json_input = true;
        for (input, output) in inputs.iter_mut().zip(&mut outputs) {
            let Some(Type::Record(child)) = input.ty else {
                return Err(Diagnostic::new("schema-type", "Every object field must contain a schema.", input.name.span));
            };
            let (accepted, produced, json) = if let Some(types) = self.schema_pipelines.types(child) { types }
                else if self.program.records[child].native_type == Some(crate::native_signatures::NativeTypeId::Schema) {
                    let output = self.program.records[child].application.as_ref().unwrap().resolved_arguments[0];
                    (output, output, true)
                } else { return Err(Diagnostic::new("schema-type", "Every object field must contain a schema.", input.name.span)); };
            children.push(child);
            json_input &= json;
            input.ty = Some(accepted);
            input.annotation = self.checked_value_annotation(accepted, input.name.span)?;
            input.optional = matches!(accepted,Type::Union(id) if self.unions.members(id).contains(&Type::Undefined));
            output.ty = Some(produced);
            output.annotation = self.checked_value_annotation(produced, output.name.span)?;
            output.optional = matches!(produced,Type::Union(id) if self.unions.members(id).contains(&Type::Undefined));
        }
        let input = self.schema_pipeline_shape(inputs, "input", expr.span);
        let output = self.schema_pipeline_shape(outputs, "object", expr.span);
        let plan = crate::stdlib::schema::pipelines::ObjectPlan {
            shape, fields: children, input: Type::Record(input), output: Type::Record(output), json_input,
            policy: crate::stdlib::schema::pipelines::ObjectPolicy::Reject, value: Type::Record(output),
        };
        let id = if let Some(id) = self.schema_pipelines.object_record(&plan) { id } else {
            let id = self.program.records.len();
            let name = format!("@schema:pipeline-object:{id}");
            let field = RecordField {
                name: Token { text: "@shape".into(), span: expr.span, kind: TokenKind::Name },
                annotation: self.checked_value_annotation(Type::Record(shape), expr.span)?,
                ty: Some(Type::Record(shape)), callable_declaration: None, optional: false,
            };
            self.record_names.insert(name.clone(), id);
            self.fields.push([(field.name.text.clone(), Type::Record(shape))].into_iter().collect());
            self.program.records.push(Record {
                native_type: None, map_definition: None, set_definition: None, enum_definition: None,
                literal_definition: None, application: None,
                name: Token { text: name, span: expr.span, kind: TokenKind::Name },
                type_parameters: Vec::new(), fields: vec![field],
            });
            self.schema_pipelines.insert_object(id, plan);
            id
        };
        self.require(Type::Record(id), expected, expr.span)?;
        expr.kind = ExprKind::Record(vec![(Token { text: "@shape".into(), span: expr.span, kind: TokenKind::Name }, value)]);
        Ok(Type::Record(id))
    }
}
