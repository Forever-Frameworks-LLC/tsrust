//! Physical payloads of nullable recursive unions; source types remain records.
use super::*;
impl Emitter<'_> {
    pub(super) fn boxed_union_member(&self, union:UnionId, member:Type)->bool {
        matches!(member,Type::Record(record) if self.recursive_links.boxed(union,record))
    }
    pub(super) fn union_stored_type(&self, union:UnionId, member:Type, native:String)->String {
        if let Type::Record(record)=member && self.recursive_links.boxed(union,record) {
            native.replacen(&format!("Record{record}"),&format!("RecordLink{record}"),1)
        }else{native}
    }
    pub(super) fn union_store_payload(&self, union:UnionId, member:Type, value:&str)->String {
        if let Type::Record(record)=member && self.recursive_links.boxed(union,record) {
            format!("RecordLink{record}::new({value})")
        }else{value.to_owned()}
    }
    pub(super) fn union_move_payload(&self, union:UnionId, member:Type, value:&str)->String {
        if self.boxed_union_member(union,member) {format!("({value}).into_inner()")}
        else{value.to_owned()}
    }
    pub(super) fn union_transfer_payload(&self, from:UnionId, to:UnionId, member:Type, value:&str)->String {
        if self.boxed_union_member(from,member)==self.boxed_union_member(to,member) {return value.to_owned();}
        self.union_store_payload(to,member,&self.union_move_payload(from,member,value))
    }
}
