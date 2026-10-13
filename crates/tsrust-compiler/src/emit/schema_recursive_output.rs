//! Move inferred schema output into its checked, explicitly named shape.
use super::*;

impl Emitter<'_> {
    pub(super) fn schema_recursive_output(&self,source:Type,target:Type,value:&str)->String {
        if source==target { return value.into(); }
        match (source,target) {
            (Type::Record(source),Type::Record(target))=>{
                let original=&self.program.records[source].fields;
                let destination=&self.program.records[target].fields;
                let positions=destination.iter().map(|field|original.iter().position(|other|other.name.text==field.name.text).expect("checked schema field")).collect::<Vec<_>>();
                let fields=positions.iter().enumerate().map(|(to,&from)|format!("field{to}:{}",self.schema_recursive_output(original[from].ty.unwrap(),destination[to].ty.unwrap(),&format!("recursive_value.field{from}")))).collect::<Vec<_>>().join(",");
                let mapping=(0..original.len()).map(|from|positions.iter().position(|position|*position==from).unwrap().to_string()).collect::<Vec<_>>().join(",");
                let order=if self.record_runtime.ordered.contains(&source) { format!("recursive_value.key_order.map(|index|[{mapping}][index])") } else { format!("[{mapping}]") };
                let result=self.schema_restored_record(target,&fields,&order);
                format!("{{let recursive_value={value};{result}}}")
            }
            (Type::Array(source),Type::Array(target))=>{
                let converted=self.schema_recursive_output(source.ty(),target.ty(),"item");
                let native=self.native_type(Type::Array(target));
                format!("{{let mut output=<{native}>::from([]);for item in ({value}).tsr_owned_items(){{output.push({converted});}}output}}")
            }
            (Type::Map(source),Type::Map(target))=>{
                let original=self.program.records[source].map_definition.as_ref().unwrap();
                let destination=self.program.records[target].map_definition.as_ref().unwrap();
                let key=self.schema_recursive_output(original.key,destination.key,"key");
                let item=self.schema_recursive_output(original.value,destination.value,"item");
                let native=self.native_type(Type::Map(target));
                format!("<{native}>::from_values(({value}).tsr_owned_items().map(TsrMapEntry::into_pair).map(|(key,item)|({key},{item})))")
            }
            (Type::Set(source),Type::Set(target))=>{
                let original=self.program.records[source].set_definition.as_ref().unwrap().key;
                let destination=self.program.records[target].set_definition.as_ref().unwrap().key;
                let item=self.schema_recursive_output(original,destination,"item");
                let native=self.native_type(Type::Set(target));
                format!("<{native}>::from_values(({value}).tsr_owned_items().map(|item|{item}))")
            }
            (Type::Union(source),Type::Union(target))=>{
                let destination=&self.program.unions[target.index()].members;
                let arms=self.program.unions[source.index()].members.iter().enumerate().map(|(from,&member)|{
                    let to=destination.iter().position(|candidate|*candidate==member).or_else(||destination.iter().position(|candidate|crate::stdlib::schema::type_contract::equivalent(member,*candidate,&self.program.records,&|id|self.program.unions[id.index()].members.clone()))).expect("checked schema alternative");
                    let extracted=self.union_move_payload(source,member,"payload");
                    let converted=self.schema_recursive_output(member,destination[to],&extracted);
                    let stored=self.union_store_payload(target,destination[to],&converted);
                    format!("Union{}::V{from}(payload)=>Union{}::V{to}({stored})",source.index(),target.index())
                }).collect::<Vec<_>>().join(",");
                format!("match {value}{{{arms}}}")
            }
            _=>unreachable!("checked recursive output contract"),
        }
    }
}
