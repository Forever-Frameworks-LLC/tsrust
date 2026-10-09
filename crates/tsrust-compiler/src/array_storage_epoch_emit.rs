//! Move proven array survivors without cloning their borrowed environments.
use super::*;

impl Emitter<'_> {
    pub(super) fn array_storage_epoch_code(&self, epoch: &crate::analysis::MapEntryEpoch) -> String {
        let (Type::Array(source), Type::Array(target)) = (epoch.input, epoch.output) else { unreachable!("checked array epoch") };
        let converted = self.map_entry_payload_projection(source.ty(), target.ty(), "array_epoch_item");
        let native = self.native_type(epoch.output);
        let binding = epoch.place.binding;
        format!("let mut value{binding}: {native} = (value{binding}).tsr_owned_items().map(|array_epoch_item|{{{converted}}}).collect::<{native}>();")
    }
}
