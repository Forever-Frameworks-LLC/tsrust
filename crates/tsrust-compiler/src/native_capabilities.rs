//! Native capability checks use declaration identity, including nested owners.
use super::*;
use crate::native_types::Capability;

impl Analyzer {
    #[inline(never)]
    pub(super) fn require_native_capability(&self, ty: Type, capability: Capability, span: Span, nested: bool) -> Result<(), Diagnostic> {
        let mut pending = vec![ty];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            match ty {
                Type::Record(id) => {
                    let record = &self.program.records[id];
                    if let Some(id) = record.native_type {
                        let declaration = crate::native_types::declaration(id);
                        if !declaration.capabilities.contains(&capability) {
                            let operation = match capability {
                                Capability::Inspection => "console inspection",
                                Capability::StringConversion => "string conversion",
                                Capability::PropertyReflection => "property reflection",
                                Capability::ObjectSpread => "object spread",
                                Capability::Equality => "value comparison",
                            };
                            return Err(Diagnostic::new("native-capability", format!("Native type {:?} does not support {operation}.", declaration.name), span));
                        }
                    }
                    if nested { pending.extend(record.fields.iter().filter_map(|field| field.ty)); }
                }
                Type::Array(element) if nested => pending.push(element.ty()),
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(self.unions.members(id)),
                Type::Map(id) if nested => {
                    let map = self.program.records[id].map_definition.as_ref().expect("checked map");
                    pending.extend([map.key, map.value]);
                }
                Type::Set(id) if nested => pending.push(self.program.records[id].set_definition.as_ref().expect("checked set").key),
                _ => {},
            }
        }
        Ok(())
    }
}
