//! Collection validation moves entries and retains reversible child projections.
use super::*;
use crate::stdlib::schema::pipelines::CollectionKind;

impl Emitter<'_>{
    pub(in crate::emit) fn schema_collection_trial_body(&self,id:usize,input:Type,generated:&mut String,seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String{
        let plan=self.program.schema_pipelines.collection(id).unwrap();
        let output=self.native_type(plan.output);
        let reject=||format!("issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Object,tsrust_schema_issues::Kind::{});Trial::Rejected(input)",self.schema_kind(input));
        let record=match input { Type::Record(id)|Type::Map(id)|Type::Set(id)=>id,_=>return reject() };
        let definition=&self.program.records[record];
        if plan.kind==CollectionKind::Set{
            let Some(set)=&definition.set_definition else{return reject()};
            let child=self.schema_pipeline_trial(plan.element,set.key,generated,seen,errors);
        let child_roots=self.schema_root_arguments(plan.element);
            let trial=format!("tsrust_schema_trial::sequence(input.tsr_owned_items().collect(),|index,value|{{path.push(tsrust_schema_issues::Segment::Index(index));let trial={child}({child_roots}&schema.field0,value,path,issues,failure);path.pop();trial}}).remap_input(|values|<{}>::from_values(values))",self.native_type(input));
            let checked=self.schema_collection_collision_check(&trial,false,generated);
            return format!("({checked}).inject(|values|<{output}>::from_values(values),|value|value.tsr_owned_items().collect())");
        }
        let (key_type,value_type,setup,inverse)=if let Some(map)=&definition.map_definition{
            (map.key,map.value,"let entries=input.tsr_owned_items().map(TsrMapEntry::into_pair).collect::<Vec<_>>();".into(),format!("<{}>::from_values(values)",self.native_type(input)))
        }else if plan.kind==CollectionKind::Record && definition.native_type==Some(crate::native_signatures::NativeTypeId::JsonValue){
            let record=self.native_record_declaration(record);
            (Type::String,input,format!("let entries=match input.value{{tsrust_native_json::Value::Object(values)=>values.into_iter().map(|(key,value)|(String::Owned(key),Record{record}{{value}})).collect::<Vec<_>>(),value=>{{issues.push(path,tsrust_schema_issues::Code::Type,tsrust_schema_issues::Kind::Object,tsrust_schema_observed(&value));return Trial::Rejected(Record{record}{{value}});}}}};"),format!("Record{record}{{value:tsrust_native_json::Value::Object(values.into_iter().map(|(key,value)|(key.into_owned(),value.value)).collect())}}"))
        }else{return reject()};
        let child=self.schema_pipeline_trial(plan.element,value_type,generated,seen,errors);
        let child_roots=self.schema_root_arguments(plan.element);
        let entry=if let Some(key)=plan.key{
            let key_child=self.schema_pipeline_trial(key,key_type,generated,seen,errors);
        let key_child_roots=self.schema_root_arguments(key);
            format!("path.push(tsrust_schema_issues::Segment::Index(index));path.push(tsrust_schema_issues::Segment::Field(\"key\"));let key={key_child}({key_child_roots}&schema.field0,key,path,issues,failure);path.pop();let trial=tsrust_schema_trial::pair(key,value,|value|{{path.push(tsrust_schema_issues::Segment::Field(\"value\"));let trial={child}({child_roots}&schema.field1,value,path,issues,failure);path.pop();trial}});path.pop();trial")
        }else{
            format!("let mut field_path=path.clone();field_path.push(tsrust_schema_issues::Segment::Field(key.as_ref()));let value={child}({child_roots}&schema.field0,value,&mut field_path,issues,failure);drop(field_path);tsrust_schema_trial::pair(value,key,|key|{{let trial:Trial<_,_,Box<dyn FnOnce(String)->String>,SchemaFactoryFailure>=Trial::Prepared{{value:key,restoration:Box::new(|key|key)}};trial}}).inject(|(value,key)|(key,value),|(key,value)|(value,key)).remap_input(|(value,key)|(key,value))")
        };
        // Record keys are preserved, so only native Map key transformations can
        // introduce collisions. JSON duplicate keys were already resolved.
        let trial=format!("tsrust_schema_trial::sequence(entries,|index,(key,value)|{{{entry}}}).remap_input(|values|{{{inverse}}})");
        let checked=if plan.kind==CollectionKind::Map{self.schema_collection_collision_check(&trial,true,generated)}else{trial};
        format!("{setup}({checked}).inject(|values|<{output}>::from_values(values),|value|value.tsr_owned_items().map(TsrMapEntry::into_pair).collect())")
    }

    fn schema_collection_collision_check(&self,trial:&str,map:bool,generated:&mut String)->String{
        let marker="/*schema_collection_duplicates*/";
        if !generated.contains(marker){
            generated.push_str(marker);
            generated.push_str("fn tsrust_schema_duplicate<T,K:TsrSetKey>(values:&[T],key:impl Fn(&T)->&K)->Option<usize>{if values.len()<=8{for index in 1..values.len(){let value=key(&values[index]);if values[..index].iter().any(|previous|value.tsr_same(key(previous))){return Some(index);}}return None;}let mut buckets=std::collections::HashMap::<u64,Vec<usize>>::new();for(index,value)in values.iter().enumerate(){let value=key(value);let bucket=buckets.entry(value.tsr_hash()).or_default();if bucket.iter().any(|previous|value.tsr_same(key(&values[*previous]))){return Some(index);}bucket.push(index);}None}");
        }
        let key=if map{"|value|&value.0"}else{"|value|value"};
        let report="let mut duplicate_path=path.clone();duplicate_path.push(tsrust_schema_issues::Segment::Index(index));issues.push(&duplicate_path,tsrust_schema_issues::Code::Collision,tsrust_schema_issues::Kind::Object,tsrust_schema_issues::Kind::Object);";
        format!("match {trial}{{Trial::Prepared{{value,restoration}}=>{{if let Some(index)=tsrust_schema_duplicate(&value,{key}){{{report}Trial::Rejected(restoration(value))}}else{{Trial::Prepared{{value,restoration}}}}}},Trial::Committed(Some(value))=>{{if let Some(index)=tsrust_schema_duplicate(&value,{key}){{{report}Trial::Committed(None)}}else{{Trial::Committed(Some(value))}}}},other=>other}}")
    }
}
