//! Typed object-field assembly consumes input storage without dynamic output records.

use super::{scalar::{Validate,reject},issues::{Code,Issues,Kind,Segment}};
use crate::stdlib::json::Value;

pub(super) trait ObjectFields {
    type Output;
    fn take(&self,input:&mut serde_json::Map<String,Value>,path:&[Segment<'_>],issues:&mut Issues)->Option<Self::Output>;
}
pub(super) struct End;
impl ObjectFields for End {
    type Output=();
    fn take(&self,_input:&mut serde_json::Map<String,Value>,_path:&[Segment<'_>],_issues:&mut Issues)->Option<()>{Some(())}
}
pub(super) struct Fields<P,T> {name:String,plan:P,tail:T}
impl<P,T> Fields<P,T> {
    pub(super) fn new(name:&str,plan:P,tail:T)->Self{Self{name:name.into(),plan,tail}}
}
impl<P:Validate,T:ObjectFields> ObjectFields for Fields<P,T> {
    type Output=(P::Output,T::Output);
    fn take(&self,input:&mut serde_json::Map<String,Value>,path:&[Segment<'_>],issues:&mut Issues)->Option<Self::Output>{
        let mut field_path=path.to_vec();field_path.push(Segment::Field(&self.name));
        let head=self.plan.validate(input.remove(&self.name),&field_path,issues);
        // Continue collecting bounded sibling issues after a field fails.
        let tail=self.tail.take(input,path,issues);
        match (head,tail) {(Some(head),Some(tail))=>Some((head,tail)),_=>None}
    }
}
pub(super) struct ObjectPlan<F>(pub F);
impl<F:ObjectFields> Validate for ObjectPlan<F> {
    type Output=F::Output;
    fn validate(&self,input:Option<Value>,path:&[Segment<'_>],issues:&mut Issues)->Option<Self::Output>{
        let mut fields=match input {Some(Value::Object(fields))=>fields,other=>{reject(other,Kind::Object,path,issues);return None;}};
        let output=self.0.take(&mut fields,path,issues);
        let unknown=!fields.is_empty();
        for (name,value) in fields {
            let observed=match value {Value::Null=>Kind::Null,Value::Bool(_)=>Kind::Boolean,Value::Number(_)=>Kind::Number,Value::String(_)=>Kind::String,Value::Array(_)=>Kind::Array,Value::Object(_)=>Kind::Object};
            let mut field_path=path.to_vec();field_path.push(Segment::Field(&name));
            issues.push(&field_path,Code::UnknownField,Kind::Missing,observed);
        }
        if unknown {None}else{output}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{scalar::{Validate,StringPlan},presence::OptionalPlan,issues::{Code,Issues,OwnedSegment}};
    use crate::stdlib::json::Value;
    #[test]
    fn typed_fields_move_storage_and_keep_absence_distinct() {
        let plan=ObjectPlan(Fields::new("name",StringPlan,Fields::new("note",OptionalPlan(StringPlan),End)));
        let text="owned name".to_owned();let pointer=text.as_ptr();
        let input=Value::Object(serde_json::Map::from_iter([("name".into(),Value::String(text))]));
        let mut issues=Issues::new(4,8,128);
        let (name,(note,()))=plan.validate(Some(input),&[],&mut issues).unwrap();
        assert_eq!(name.as_ptr(),pointer);assert_eq!(note,None);assert!(issues.is_valid());
    }
    #[test]
    fn unknown_and_invalid_fields_report_original_paths() {
        let plan=ObjectPlan(Fields::new("name",StringPlan,End));
        let input=serde_json::from_str(r#"{"name":false,"secret": "private"}"#).unwrap();
        let mut issues=Issues::new(4,8,128);
        assert!(plan.validate(Some(input),&[super::super::issues::Segment::Field("user")],&mut issues).is_none());
        assert_eq!(issues.items.len(),2);
        assert_eq!(issues.items[0].code,Code::Type);
        assert_eq!(issues.items[1].code,Code::UnknownField);
        assert_eq!(issues.items[1].path,[OwnedSegment::Field("user".into()),OwnedSegment::Field("secret".into())]);
        assert!(!format!("{issues:?}").contains("private"));
        let mut hidden=Issues::new(0,0,0);
        assert!(ObjectPlan(End).validate(Some(serde_json::from_str(r#"{"extra":1}"#).unwrap()),&[],&mut hidden).is_none());
        assert!(!hidden.is_valid()&&hidden.truncated);
    }
    #[test]
    fn nested_objects_keep_typed_output_and_reject_missing_required_fields() {
        let plan=ObjectPlan(Fields::new("user",ObjectPlan(Fields::new("name",StringPlan,End)),End));
        let mut issues=Issues::new(4,8,128);
        let input=serde_json::from_str(r#"{"user":{"name":"Ada"}}"#).unwrap();
        let ((name,()),())=plan.validate(Some(input),&[],&mut issues).unwrap();
        assert_eq!(name,"Ada");
        let input=serde_json::from_str(r#"{"user":{}}"#).unwrap();
        assert!(plan.validate(Some(input),&[],&mut issues).is_none());
        assert_eq!(issues.items[0].code,Code::Missing);
        assert_eq!(issues.items[0].path,[OwnedSegment::Field("user".into()),OwnedSegment::Field("name".into())]);
    }
}
