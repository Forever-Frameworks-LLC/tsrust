//! Resolve diagnostic paths against the value seen by the refinement callback.
use super::*;

use crate::stdlib::schema::pipelines::DiagnosticSegment as SchemaDiagnosticSegment;

impl Analyzer {
    pub(super) fn schema_diagnostic_path(
        &self, value: Type, expression: &Expr,
    ) -> Result<Vec<SchemaDiagnosticSegment>, Diagnostic> {
        let ExprKind::Array(items) = &expression.kind else {
            return Err(Diagnostic::new("schema-path", "A refinement path requires a statically checked array of field names and nonnegative array indices.", expression.span));
        };
        let mut types = vec![value];
        let mut path = Vec::new();
        for item in items {
            let segment = match &item.kind {
                ExprKind::String(name) => SchemaDiagnosticSegment::Field(name.clone()),
                ExprKind::Integer(text) => SchemaDiagnosticSegment::Index(text.parse::<usize>()
                    .map_err(|_| Diagnostic::new("schema-path", "Array path indices must be nonnegative integers in range.", item.span))?),
                _ => return Err(Diagnostic::new("schema-path", "Use a field name or a nonnegative integer array index in a refinement path.", item.span)),
            };
            let mut next = Vec::new();
            for ty in types {
                self.schema_diagnostic_path_step(ty, &segment, &mut next, item.span)?;
            }
            if next.is_empty() {
                return Err(Diagnostic::new("schema-path", "The path cannot continue through a missing or null value.", item.span));
            }
            let mut seen = HashSet::new();
            next.retain(|ty| seen.insert(*ty));
            types = next;
            path.push(segment);
        }
        Ok(path)
    }

    fn schema_diagnostic_path_step(
        &self, ty: Type, segment: &SchemaDiagnosticSegment,
        output: &mut Vec<Type>, span: Span,
    ) -> Result<(), Diagnostic> {
        if let Type::Union(id) = ty {
            for member in self.unions.members(id) {
                if !matches!(member, Type::Null | Type::Undefined) {
                    self.schema_diagnostic_path_step(member, segment, output, span)?;
                }
            }
            return Ok(());
        }
        let found = match (ty, segment) {
            (Type::Record(id), SchemaDiagnosticSegment::Field(name)) => self.program.records[id].fields.iter()
                .find(|field| field.name.text == *name).and_then(|field| field.ty),
            (Type::Array(element), SchemaDiagnosticSegment::Index(_)) => Some(element.ty()),
            (Type::Map(id), SchemaDiagnosticSegment::Field(_)) => self.program.records[id].map_definition.as_ref()
                .filter(|map| map.key == Type::String).map(|map| map.value),
            _ => None,
        };
        let Some(found) = found else {
            return Err(Diagnostic::new("schema-path", "This path segment is not valid for every possible value at this location. Use declared fields and numeric array indices.", span));
        };
        output.push(found);
        Ok(())
    }
}
