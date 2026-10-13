//! Named owned payloads break recursive generic defaults without extra allocation.
use super::*;

impl Emitter<'_> {
    pub(super) fn recursive_payload_name(&self, union:UnionId, member:Type)->Option<String> {
        let definition=&self.program.unions[union.index()];
        if !definition.recursive || !matches!(member,Type::Array(_)|Type::Union(_)) { return None; }
        let variant=definition.members.iter().position(|candidate|*candidate==member)?;
        Some(format!("RecursivePayload{}_{variant}",union.index()))
    }

    pub(super) fn emit_recursive_payload(&self, output:&mut String, union:UnionId, member:Type, cloneable:bool) {
        let Some(name)=self.recursive_payload_name(union,member) else { return; };
        let native=self.native_type(member);
        let derive=if cloneable {"#[derive(Clone)] "}else{""};
        let _=writeln!(output,"{derive}struct {name}({native});");
        let _=writeln!(output,"impl std::borrow::Borrow<{native}> for {name} {{ fn borrow(&self)->&{native} {{ &self.0 }} }}");
        let _=writeln!(output,"impl std::ops::Deref for {name} {{ type Target={native}; fn deref(&self)->&{native} {{ &self.0 }} }}");
        let _=writeln!(output,"impl std::ops::DerefMut for {name} {{ fn deref_mut(&mut self)->&mut {native} {{ &mut self.0 }} }}");
    }

}
