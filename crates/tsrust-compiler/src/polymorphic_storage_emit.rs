//! Native family storage stays distinct from each concrete environment layout.
use super::*;

impl Emitter<'_> {
    pub(super) fn native_environment_type(&self, id: usize) -> String {
        let closure = &self.program.closures[id];
        let bounds = self.captured_callback_bounds(closure);
        if !bounds.is_empty() {
            let mut parameters = vec!["'_".to_owned(); closure.lifetime_count];
            parameters.extend(bounds.iter().map(|bound| bound.split(':').next().expect("capture generic").trim().to_owned()));
            return format!("Closure{id}<{}>", parameters.join(", "));
        }
        format!("Closure{id}")
    }

    pub(super) fn generic_storage_type(&self, origin: usize) -> String {
        let family = self.program.polymorphic_storage.checked(origin, &self.program.closures, crate::Span::default()).expect("checked native family");
        let parameters = family.origins.iter().map(|&origin| self.native_environment_type(origin)).collect::<Vec<_>>().join(", ");
        format!("TsrGenericFamily{}<{parameters}>", family.id)
    }

    pub(super) fn emit_generic_storage(&self, output: &mut String) {
        let families = self.program.polymorphic_storage.families(&self.program.closures, crate::Span::default())
            .expect("checked generic storage families");
        for family in families {
            // Each environment supplies its own complete native type, including
            // borrowed lifetimes and captured callable parameters. Storage adds
            // no invocation bound or implicit Clone requirement.
            let parameters = (0..family.origins.len()).map(|index| format!("E{index}")).collect::<Vec<_>>();
            output.push_str(&family.declaration(&parameters, |origin| {
                let variant = family.origins.binary_search(&origin).expect("family environment");
                parameters[variant].clone()
            }));
        }
    }

}
