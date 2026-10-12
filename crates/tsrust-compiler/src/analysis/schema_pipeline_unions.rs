//! Union records own ordered branch plans independently of normalized outputs.
use super::*;

impl Analyzer {
    pub(super) fn schema_pipeline_union(&mut self, expr: &mut Expr, branches: Vec<Expr>, selector: Option<crate::stdlib::schema::plans::Selector>, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut records = Vec::new();
        let mut json_input = false;
        for branch in &branches {
            let Some(Type::Record(id)) = branch.ty else { unreachable!("checked schema branch") };
            let (input, output, json) = self.schema_pipelines.types(id).unwrap_or_else(|| {
                let output = self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
                (output, output, true)
            });
            records.push(id); inputs.push(input); outputs.push(output); json_input |= json;
        }
        let input = self.unions.intern(inputs, expr.span)?;
        let output = self.unions.intern(outputs, expr.span)?;
        let plan = crate::stdlib::schema::pipelines::UnionPlan { branches: records, selector, input, output, json_input };
        let id = if let Some(id) = self.schema_pipelines.union_record(&plan) { id } else {
            let id = self.program.records.len();
            let name = format!("@schema:pipeline-union:{id}");
            let fields = plan.branches.iter().enumerate().map(|(position, child)| Ok(RecordField {
                name: Token { text: format!("@branch{position}"), span: expr.span, kind: TokenKind::Name },
                annotation: self.checked_value_annotation(Type::Record(*child), expr.span)?,
                ty: Some(Type::Record(*child)), callable_declaration: None, optional: false,
            })).collect::<Result<Vec<_>, Diagnostic>>()?;
            self.record_names.insert(name.clone(), id);
            self.fields.push(fields.iter().map(|field| (field.name.text.clone(), field.ty.unwrap())).collect());
            self.program.records.push(Record {
                native_type: None, map_definition: None, set_definition: None, enum_definition: None,
                literal_definition: None, application: None,
                name: Token { text: name, span: expr.span, kind: TokenKind::Name },
                type_parameters: Vec::new(), fields,
            });
            self.schema_pipelines.insert_union(id, plan);
            id
        };
        self.require(Type::Record(id), expected, expr.span)?;
        expr.kind = ExprKind::Record(branches.into_iter().enumerate().map(|(position, branch)|
            (Token { text: format!("@branch{position}"), span: expr.span, kind: TokenKind::Name }, branch)).collect());
        Ok(Type::Record(id))
    }
}
