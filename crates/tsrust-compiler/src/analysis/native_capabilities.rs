//! Native capability checks use declaration identity, including nested owners.
use super::*;
use crate::native_types::Capability;

impl Analyzer {
    pub(super) fn require_array_text(&self, root: Type, span: Span) -> Result<(), Diagnostic> {
        if array_text_supported(root, &self.program.records, |id| self.unions.members(id)) { Ok(()) }
        else { Err(Diagnostic::new("void-array-text", "Unit result arrays do not support string conversion; inspect their length or iterate them.", span)) }
    }
    #[inline(never)]
    pub(super) fn require_native_capability(&self, ty: Type, capability: Capability, span: Span, nested: bool) -> Result<(), Diagnostic> {
        if capability == Capability::StringConversion { self.require_array_text(ty, span)?; }
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

// Checker diagnostics and emitted registrations must agree on unit-array text.
pub(crate) fn array_text_supported(root: Type, records: &[Record], members: impl Fn(UnionId) -> Vec<Type>) -> bool {
    let mut pending = vec![root];
    let mut seen = HashSet::new();
    while let Some(ty) = pending.pop() {
        if !seen.insert(ty) { continue; }
        match ty {
            Type::Array(ArrayElement::Void(_)) => return false,
            Type::Array(element) => pending.push(element.ty()),
            Type::Record(id) if records[id].is_tuple() => pending.extend(records[id].fields.iter().filter_map(|field| field.ty)),
            Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(members(id)),
            _ => (),
        }
    }
    true
}

/// Receiver-dependent restrictions for array methods, independent of arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArrayCapabilities {
    optional_element: bool,
    text: bool,
    pub default_sort: bool,
}
impl ArrayCapabilities {
    pub(crate) fn intersection(self, other: Self) -> Self {
        Self { optional_element: self.optional_element && other.optional_element, text: self.text && other.text, default_sort: self.default_sort && other.default_sort }
    }
    pub fn supports(self, method: &str) -> bool {
        match method {
            "at" | "pop" | "shift" | "find" | "findLast" | "entries" => self.optional_element,
            "join" | "toString" | "toLocaleString" => self.text,
            _ => true,
        }
    }
}
pub(crate) fn checked_array_capabilities(root: Type, records: &[Record], members: impl Fn(UnionId) -> Vec<Type>) -> Option<ArrayCapabilities> {
    let Type::Array(element) = root else { return None; };
    Some(ArrayCapabilities {
        optional_element: element.ty() != Type::Void,
        text: array_text_supported(root, records, members),
        default_sort: matches!(element, ArrayElement::Int(0) | ArrayElement::Number(0) | ArrayElement::Boolean(0) | ArrayElement::String(0) | ArrayElement::Null(0) | ArrayElement::Undefined(0) | ArrayElement::Void(0)),
    })
}
