//! Move proven array survivors without cloning their borrowed environments.
use super::*;

impl Emitter<'_> {
    pub(super) fn array_storage_epoch_code(&self, fun: &CheckedFunction, epoch: &crate::analysis::MapEntryEpoch) -> String {
        let mut source = epoch.input;
        let mut target = epoch.output;
        let mut indices = Vec::new();
        for name in &epoch.place.fields {
            let (Type::Record(input), Type::Record(output)) = (source, target) else { unreachable!("checked array field path") };
            let index = self.program.records[input].fields.iter().position(|field| field.name.text == *name).expect("checked array field");
            indices.push(index);
            source = self.program.records[input].fields[index].ty.unwrap();
            target = self.program.records[output].fields[index].ty.unwrap();
        }
        let (Type::Array(source), Type::Array(target)) = (source, target) else { unreachable!("checked array epoch") };
        let converted = self.map_entry_payload_projection(source.ty(), target.ty(), "array_epoch_item");
        let native = self.native_type(Type::Array(target));
        let binding = epoch.place.binding;
        let place = if indices.is_empty() { format!("value{binding}") } else { self.owned_record_leaf_place(fun, binding, &indices).expect("owning array leaf") };
        format!("let mut {place}: {native} = ({place}).tsr_owned_items().map(|array_epoch_item|{{{converted}}}).collect::<{native}>();")
    }
}
