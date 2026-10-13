//! Infer output alternatives while preserving each owned branch plan and its order.
use super::*;
use crate::stdlib::schema::plans::{Choice, Selector, Tag};

impl Analyzer {
    pub(in crate::analysis) fn schema_union_constructor(&mut self, context: &mut Context, expr: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(None); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        if !matches!(name.as_str(), "@schema:union" | "@schema:discriminatedUnion") { return Ok(None); }
        let discriminated = name == "@schema:discriminatedUnion";
        let count = if discriminated { 2 } else { 1 };
        if args.len() != count { return Err(Diagnostic::new("arity", if discriminated { "s.discriminatedUnion expects a literal field name and a nonempty array of owned schemas." } else { "s.union expects one nonempty array of owned schemas." }, expr.span)); }
        let field = if discriminated {
            let ExprKind::String(field) = &args[0].kind else { return Err(Diagnostic::new("schema-discriminator", "The discriminator field name must be a string literal.", args[0].span)); };
            Some(field.clone())
        } else { None };
        let argument = &mut args[count - 1];
        let ExprKind::Array(branches) = &mut argument.kind else { return Err(Diagnostic::new("schema-union", "Union branches require an array literal so each schema output is known.", argument.span)); };
        if branches.is_empty() { return Err(Diagnostic::new("schema-union", "A union schema requires at least one branch.", argument.span)); }
        let mut outputs = Vec::new();
        let mut tags = Vec::new();
        let mut selected = HashSet::new();
        for branch in branches.iter_mut() {
            let actual = self.expression(context, branch, Mode::Consume, loans, None)?;
            let Type::Record(id) = actual else { return Err(Diagnostic::new("schema-union", "Every union branch must be an owned schema.", branch.span)); };
            let record = &self.program.records[id];
            if record.native_type != Some(crate::native_signatures::NativeTypeId::Schema) && self.schema_pipelines.types(id).is_none() { return Err(Diagnostic::new("schema-union", "Every union branch must be an owned schema.", branch.span)); }
            let output = self.schema_pipelines.types(id).map(|(_, output, _)| output)
                .unwrap_or_else(|| record.application.as_ref().unwrap().resolved_arguments[0]);
            if let Some(field) = &field {
                // Selection inspects raw input before any consuming conversion.
                // A transform may return a scalar without losing the original
                // object's discriminator. A coercion must expose closed tags on
                // its own raw input; converted tags cannot choose a branch.
                let discriminator_input = self.schema_pipelines.types(id).map(|(input, _, _)| input).unwrap_or(output);
                let Type::Record(object) = discriminator_input else { return Err(Diagnostic::new("schema-discriminator", "Discriminated union branches require object inputs with closed discriminator values.", branch.span)); };
                let record = &self.program.records[object];
                if record.native_type.is_some() || record.name.text.starts_with("@schema:retained:") { return Err(Diagnostic::new("schema-discriminator", "The discriminator must belong to the direct typed object input.", branch.span)); }
                let Some(member) = record.fields.iter().find(|member| &member.name.text == field) else { return Err(Diagnostic::new("schema-discriminator", "Every branch must declare the discriminator field.", branch.span)); };
                if member.optional { return Err(Diagnostic::new("schema-discriminator", "A discriminator field must be required.", member.name.span)); }
                let values = self.schema_discriminator_tags(member.ty.unwrap(), member.name.span)?;
                let mut branch_tags = Vec::new();
                for value in values {
                    if branch_tags.contains(&value) { continue; }
                    if !selected.insert(value.clone()) { return Err(Diagnostic::new("schema-discriminator", "Discriminator values must not overlap between branches.", branch.span)); }
                    branch_tags.push(value);
                }
                if branch_tags.is_empty() { return Err(Diagnostic::new("schema-discriminator", "A discriminator must have at least one known value.", branch.span)); }
                tags.push(branch_tags);
            }
            outputs.push(output);
        }
        if branches.iter().any(|branch| matches!(branch.ty,Some(Type::Record(id)) if self.schema_pipelines.types(id).is_some())) {
            let branches = std::mem::take(branches);
            let selector = field.map(|field| Selector { field, tags });
            return self.schema_pipeline_union(expr, branches, selector, expected).map(Some);
        }
        let output = self.unions.intern(outputs.clone(), expr.span)?;
        let plan = self.schema_choices.intern(Choice { output, branches: outputs, selector: field.map(|field| Selector { field, tags }), presence: None });
        let types = crate::generic_inference::Types { callable_annotations: None, callable_shapes: None, inline_shapes: None, records: &self.program.records, names: &self.record_names, unions: &self.unions };
        use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind, TypeName};
        let mut annotation = ValueAnnotation { span: expr.span, kind: Kind::Applied { name: TypeName { text: crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::Schema), span: expr.span }, arguments: vec![types.annotation(output, expr.span)?] } };
        self.generic_records.materialize(&mut annotation, &mut self.program.records, &mut self.record_names, &self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records, &self.record_names, &self.unions, &mut self.fields, &mut self.callables)?;
        let result = self.resolve_value_annotation(&annotation, false)?;
        self.require(result, expected, expr.span)?;
        let arguments = std::mem::take(branches);
        expr.kind = ExprKind::NativeConstructor(arguments, Some(Box::new(annotation)));
        expr.target = Some(CallTarget::SchemaChoice(plan));
        Ok(Some(result))
    }

    pub(in crate::analysis) fn schema_discriminator_tags(&self, ty: Type, span: Span) -> Result<Vec<Tag>, Diagnostic> {
        use crate::literal_types::LiteralValue;
        Ok(match ty {
            Type::Literal(id, _) => vec![match self.program.records[id].literal_definition.as_ref().unwrap() {
                LiteralValue::String(value) => Tag::String(value.clone()),
                LiteralValue::Int(value) => Tag::Integer(*value),
                LiteralValue::Number(bits) => Tag::number(f64::from_bits(*bits)),
                LiteralValue::Boolean(value) => Tag::Boolean(*value),
            }],
            Type::Null => vec![Tag::Null],
            Type::Enum(id, _) => self.program.records[id].enum_definition.as_ref().unwrap().members.iter().map(|member| match &member.value {
                crate::syntax::EnumValue::String(value) => Ok(Tag::String(value.clone())),
                crate::syntax::EnumValue::Number(value) if value.is_finite() => Ok(Tag::number(*value)),
                _ => Err(Diagnostic::new("schema-discriminator", "Numeric discriminator values must be finite.", span)),
            }).collect::<Result<Vec<_>, _>>()?,
            Type::Union(id) => self.unions.members(id).into_iter().map(|member| self.schema_discriminator_tags(member, span)).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect(),
            _ => return Err(Diagnostic::new("schema-discriminator", "Discriminator fields require literal or closed enum values, without undefined.", span)),
        })
    }
}
