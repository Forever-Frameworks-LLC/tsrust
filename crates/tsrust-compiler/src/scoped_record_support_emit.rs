//! Checked nominal metadata for independent record borrowing support.
use super::*;

#[derive(Default)]
pub(super) struct SupportCache {
    read: std::collections::BTreeMap<usize,Vec<String>>,
    mutate: std::collections::BTreeMap<usize,Vec<String>>,
    bridges: HashSet<usize>,
}
impl Emitter<'_> {
    pub(super) fn record_borrowing_support_cached(&self, record: usize, cache: &mut SupportCache) -> Option<String> {
        self.record_borrowing_support_layout(record,cache)
    }
    fn record_borrowing_support_layout(&self, record: usize, cache: &mut SupportCache) -> Option<String> {
        use crate::{scoped_record_places::{Layout,Leaf},scoped_record_bridge::{Types,OwnedParameters,OwnedAccess}};
        let record=self.program.scoped_records.declaration(record);
        if cache.read.contains_key(&record) && cache.mutate.contains_key(&record) && cache.bridges.contains(&record){return Some(String::new());}
        let layout=Layout::for_record(record,&self.program.records)?;
        if self.program.records[record].fields.is_empty(){return None;}
        let native_field=|node:usize,index:usize,ty:Type| {
            if self.scoped_record_native_parameter(ty) {
                let name=format!("V{node}F{index}");(name.clone(),vec![name])
            }else{(self.native_type(ty),vec![])}
        };
        let mut output=layout.view_declarations_cached(&self.program.records,|id|format!("TsrReadView{id}"),native_field,false,&mut cache.read);
        output.push_str(&layout.view_declarations_cached(&self.program.records,|id|format!("TsrMutateView{id}"),native_field,true,&mut cache.mutate));
        let mut pending=vec![record];
        while let Some(node)=pending.pop(){
            if !cache.bridges.insert(node){continue;}
            let record=&self.program.records[node];
            if record.fields.is_empty(){continue;}
            let layout=Layout::for_record(node,&self.program.records)?;
            for field in record.fields.iter().rev(){if let Some(Type::Record(child))=field.ty{pending.push(child);}}
            let owner_parameters=record.fields.iter().enumerate().filter_map(|(index,field)|self.scoped_record_native_parameter(field.ty.unwrap()).then(||format!("P{index}"))).collect::<Vec<_>>();
            let mut read_bounds=Vec::new();let mut mutate_bounds=Vec::new();
            for (index,field) in record.fields.iter().enumerate(){
                if let Some(Type::Record(child))=field.ty && self.scoped_record_native_parameter(Type::Record(child)){
                    read_bounds.push(format!("P{index}:TsrReadRecord{child}"));
                    mutate_bounds.push(format!("P{index}:TsrMutateRecord{child}"));
                }
            }
            let view_leaf=|leaf:&Leaf|{
                let mut parent=node;
                for &field in &leaf.path[..leaf.path.len()-1]{let Type::Record(child)=self.program.records[parent].fields[field].ty.unwrap() else{unreachable!("validated field path")};parent=child;}
                let index=*leaf.path.last().unwrap();
                let (mut ty,parameters)=native_field(parent,index,leaf.ty);
                if !parameters.is_empty(){for &field in leaf.path[..leaf.path.len()-1].iter().rev(){ty=format!("N{field}{ty}");}}
                (ty,!parameters.is_empty())
            };
            let view_parameters=layout.leaves.iter().filter_map(|leaf|{let(ty,generic)=view_leaf(leaf);generic.then_some(ty)}).collect::<Vec<_>>();
            let owned=format!("Record{node}{}",lifetime_parameters(&owner_parameters));
            let arguments=std::iter::once("'a".into()).chain(view_parameters.iter().cloned()).collect::<Vec<_>>();
            let shared=format!("TsrReadView{node}{}",lifetime_parameters(&arguments));
            let exclusive=format!("TsrMutateView{node}{}",lifetime_parameters(&arguments));
            output.push_str(&crate::scoped_record_bridge::declarations_with_owned(&layout,Types{
                read_trait:&format!("TsrReadRecord{node}"),mutate_trait:&format!("TsrMutateRecord{node}"),owned:&owned,shared:&shared,exclusive:&exclusive,parameters:&view_parameters,
            },|leaf|view_leaf(leaf).0,OwnedParameters{parameters:&owner_parameters,read_bounds:&read_bounds,mutate_bounds:&mutate_bounds},|leaf|{
                let first=leaf.path[0];let field_type=record.fields[first].ty.unwrap();
                if leaf.path.len()>1 && self.scoped_record_native_parameter(field_type){
                    let Type::Record(child)=field_type else{unreachable!("validated nested record")};
                    let associated=leaf.path[1..].iter().map(|index|format!("Field{index}")).collect::<String>();
                    let method=leaf.path[1..].iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_");
                    OwnedAccess{ty:format!("<P{first} as TsrReadRecord{child}>::{associated}"),read:format!("self.field{first}.tsr_{method}()"),mutate:format!("self.field{first}.tsr_{method}_mut()")}
                }else{
                    let ty=if leaf.path.len()==1 && self.scoped_record_native_parameter(field_type){format!("P{first}")}else{self.native_type(leaf.ty)};
                    let place=leaf.path.iter().map(|index|format!(".field{index}")).collect::<String>();
                    OwnedAccess{ty,read:format!("&self{place}"),mutate:format!("&mut self{place}")}
                }
            }));
        }
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_metadata_supports_distinct_nested_callback_environments() {
        let source=crate::format(r#"type Holder={run:ReadOnly<Function<int>>;label:string;};type Pair={left:Holder;right:Holder;};function probe(pair:ReadOnly<Pair>):int{return pair.left.run()+pair.right.run();}function main(){}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let emitter=emitter_for(&checked.program,"");
        let outer=checked.program.records.iter().position(|record|record.name.text=="Pair").unwrap();
        let Type::Record(inner)=checked.program.records[outer].fields[0].ty.unwrap() else{panic!("child record")};
        let mut cache=SupportCache::default();
        let mut support=emitter.record_borrowing_support_cached(outer,&mut cache).expect("nominal borrowing support");
        support.push_str(&emitter.record_borrowing_support_cached(inner,&mut cache).expect("shared child support"));
        support.push_str(&emitter.record_borrowing_support_cached(outer,&mut cache).expect("repeated root support"));
        let mut rust=emit(&checked.program);
        assert!(!support.is_empty());
        assert_eq!(rust.matches(&format!("struct TsrReadView{inner}<")).count(),1,"one shared child declaration");
        assert_eq!(rust.matches(&format!("trait TsrReadRecord{outer} ")).count(),1,"one nominal bridge");
        rust.push_str(&format!(r#"
            struct Left(std::string::String);struct Right(Vec<i64>);
            impl ReadCallback<(),i64> for Left{{fn invoke_read(&self,_:())->i64{{self.0.len() as i64}}}}
            impl ReadCallback<(),i64> for Right{{fn invoke_read(&self,_:())->i64{{self.0[0]}}}}
            fn inspect<V:TsrReadRecord{outer}>(value:&V)->i64 where V::Field0Field0:ReadCallback<(),i64>,V::Field1Field0:ReadCallback<(),i64>{{ReadCallback::invoke_read(value.tsr_field0_field0(),())+ReadCallback::invoke_read(value.tsr_field1_field0(),())}}
            fn verify(){{
                let value=Record{outer}{{field0:Record{inner}{{field0:Left("abc".into()),field1:String::Owned("left".into())}},field1:Record{inner}{{field0:Right(vec![11]),field1:String::Owned("right".into())}}}};
                assert_eq!(inspect(&value),14);
                let shared=TsrReadView{outer}{{field0:TsrReadView{inner}{{field0:&value.field0.field0,field1:&value.field0.field1}},field1:TsrReadView{inner}{{field0:&value.field1.field0,field1:&value.field1.field1}}}};
                assert_eq!(inspect(&shared),14);
            }}
        "#));
        assert!(rust.contains("fn main() -> () {"),"native entry signature");
        rust=rust.replace("fn main() -> () {","fn main() -> () { verify();");
        crate::scoped_record_places::tests::native(&rust);
    }
}
