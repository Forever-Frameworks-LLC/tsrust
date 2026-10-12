//! Checked nominal metadata for independent record borrowing support.
use super::*;

#[derive(Default)]
pub(super) struct SupportCache {
    read: std::collections::BTreeMap<usize,Vec<String>>,
    mutate: std::collections::BTreeMap<usize,Vec<String>>,
    bridges: HashSet<usize>,
}
impl Emitter<'_> {
    pub(super) fn record_spread_slot_type(&self,id:usize)->String {
        let node=self.program.scoped_records.declaration(id);
        let native=self.native_type(Type::Record(id));
        let layout=self.record_borrow_layout(node).expect("borrowed record layout");
        let fields=layout.leaves.iter().map(|leaf|format!("<{native} as TsrReadRecord{node}>::{}",leaf.path.iter().map(|i|format!("Field{i}")).collect::<String>())).collect::<Vec<_>>().join(",");
        format!("TsrSpreadSlot<'_,{native},TsrReadRecord{node}Fields<'_,{fields}>>")
    }
    fn record_spread_support(&self,node:usize,layout:&crate::scoped_record_places::Layout)->String {
        let fields=layout.leaves.iter().map(|leaf|(leaf.path.iter().map(|i|format!("Field{i}")).collect::<String>(),leaf.path.iter().map(|i|format!("field{i}")).collect::<Vec<_>>().join("_"))).collect::<Vec<_>>();
        let types=fields.iter().map(|(associated,_)|format!("T::{associated}")).collect::<Vec<_>>().join(",");
        let transport=format!("TsrReadRecord{node}Fields<'a,{types}>");
        let mut output=format!("impl<'a,T:TsrReadRecord{node}> TsrReadRecord{node} for TsrSpreadSlot<'a,T,{transport}>{{");
        for (associated,method) in &fields {
            let _=write!(output,"type {associated}=T::{associated};fn tsr_{method}(&self)->&Self::{associated}{{match self{{Self::View(fields)=>fields.{method},_=>self.tsr_read().tsr_{method}()}}}}");
        }
        let mut members=fields.iter().map(|(_,method)|format!("{method}:self.tsr_{method}()")).collect::<Vec<_>>();
        for (path,width) in self.record_order_paths(node) {
            let member=Self::record_order_member(&path);
            let _=write!(output,"fn tsr_{member}(&self)->&[usize;{width}]{{match self{{Self::View(fields)=>fields.{member},_=>self.tsr_read().tsr_{member}()}}}}");
            members.push(format!("{member}:self.tsr_{member}()"));
        }
        let _=write!(output,"fn tsr_fields(&self)->TsrReadRecord{node}Fields<'_,{types}>{{TsrReadRecord{node}Fields{{{}}}}}}}",members.join(","));
        output
    }

    pub(super) fn record_borrow_leaf(&self,record:usize)->bool {
        self.native_views.recursive(record) || self.record_runtime.recursive(record)
    }
    pub(super) fn record_borrow_layout(&self,record:usize)->Option<crate::scoped_record_places::Layout> {
        crate::scoped_record_places::Layout::for_record_with_leaf(record,&self.program.records,|child|self.record_borrow_leaf(child))
    }
    /// Compute nominal borrowing families once, including ordinary record children.
    pub(super) fn record_borrowing_family(&self)->HashSet<usize> {
        let mut family=HashSet::new();
        let ordered_entries=self.program.records.iter().filter(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap)).filter_map(|record|record.map_definition.as_ref()?.entry).map(|id|self.program.scoped_records.declaration(id)).collect::<HashSet<_>>();
        for id in 0..self.program.records.len() {
            let root=self.program.scoped_records.declaration(id);
            if family.contains(&root) || (!ordered_entries.contains(&root) && !self.callback_record_native_parameter(Type::Record(root))) {continue;}
            if self.record_borrow_layout(root).is_none(){continue;}
            let mut pending=vec![root];
            while let Some(record)=pending.pop() {
                let record=self.program.scoped_records.declaration(record);
                if self.record_borrow_leaf(record) || !family.insert(record){continue;}
                pending.extend(self.program.records[record].fields.iter().filter_map(|field|if let Some(Type::Record(child))=field.ty{Some(child)}else{None}));
            }
        }
        family
    }
    pub(super) fn record_order_paths(&self,root:usize)->Vec<(Vec<usize>,usize)> {
        let mut result=Vec::new();let mut pending=vec![(root,Vec::new())];
        while let Some((record,path))=pending.pop(){
            if self.record_runtime.ordered.contains(&record) && !self.program.records[record].fields.is_empty(){result.push((path.clone(),self.program.records[record].fields.len()));}
            for (field,item) in self.program.records[record].fields.iter().enumerate().rev(){if let Some(Type::Record(child))=item.ty && !self.program.records[child].fields.is_empty() && !self.record_borrow_leaf(child){let mut next=path.clone();next.push(field);pending.push((child,next));}}
        }
        result
    }
    pub(super) fn record_order_member(path:&[usize])->String {
        if path.is_empty(){"key_order".into()}else{format!("{}_key_order",path.iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_"))}
    }
    pub(super) fn record_borrowing_support_cached(&self, record: usize, cache: &mut SupportCache) -> Option<String> {
        self.record_borrowing_support_layout(record,cache)
    }
    fn record_borrowing_support_layout(&self, record: usize, cache: &mut SupportCache) -> Option<String> {
        use crate::{scoped_record_places::Leaf,scoped_record_bridge::{Types,OwnedParameters,OwnedAccess}};
        let record=self.program.scoped_records.declaration(record);
        if cache.read.contains_key(&record) && cache.mutate.contains_key(&record) && cache.bridges.contains(&record){return Some(String::new());}
        let layout=self.record_borrow_layout(record)?;
        if self.program.records[record].fields.is_empty(){return None;}
        let native_field=|node:usize,index:usize,ty:Type| {
            if self.scoped_record_native_parameter(ty) {
                let name=format!("V{node}F{index}");(name.clone(),vec![name])
            }else{(self.native_type(ty),vec![])}
        };
        let metadata=|id,mutable|if self.record_runtime.ordered.contains(&id){format!(",key_order:&'a {}[usize;{}]",if mutable{"mut "}else{""},self.program.records[id].fields.len())}else{String::new()};
        let mut output=layout.view_declarations_with_metadata(&self.program.records,|id|format!("TsrReadView{id}"),native_field,false,&mut cache.read,|id|metadata(id,false));
        output.push_str(&layout.view_declarations_with_metadata(&self.program.records,|id|format!("TsrMutateView{id}"),native_field,true,&mut cache.mutate,|id|metadata(id,true)));
        let mut pending=vec![record];
        while let Some(node)=pending.pop(){
            if self.record_borrow_leaf(node) || !cache.bridges.insert(node){continue;}
            let record=&self.program.records[node];
            if record.fields.is_empty(){continue;}
            let layout=self.record_borrow_layout(node)?;
            for field in record.fields.iter().rev(){if let Some(Type::Record(child))=field.ty{pending.push(child);}}
            let owner_parameters=record.fields.iter().enumerate().filter_map(|(index,field)|self.scoped_record_native_parameter(field.ty.unwrap()).then(||format!("P{index}"))).collect::<Vec<_>>();
            let mut read_bounds=Vec::new();let mut mutate_bounds=Vec::new();
            for (index,field) in record.fields.iter().enumerate(){
                if let Some(Type::Record(child))=field.ty && !self.program.records[child].fields.is_empty() && !self.record_borrow_leaf(child) && self.scoped_record_native_parameter(Type::Record(child)){
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
            let orders=self.record_order_paths(node).into_iter().map(|(path,width)|{
                let member=Self::record_order_member(&path);
                let place=format!("{}.key_order",path.iter().map(|index|format!(".field{index}")).collect::<String>());
                let owned=if let Some(&first)=path.first() && self.scoped_record_native_parameter(record.fields[first].ty.unwrap()){
                    format!("self.field{first}.tsr_{}()",Self::record_order_member(&path[1..]))
                }else{format!("&self{place}")};
                (path,crate::scoped_record_bridge::OrderAccess{member,place,width,owned})
            }).collect::<Vec<_>>();
            let transport_body=|mutable:bool| {
                let mut prefix=String::new();
                for (index,field) in record.fields.iter().enumerate() {
                    if let Some(Type::Record(child))=field.ty && !self.program.records[child].fields.is_empty() && !self.record_borrow_leaf(child) && self.scoped_record_native_parameter(Type::Record(child)) {
                        let method=if mutable{"tsr_fields_mut"}else{"tsr_fields"};
                        prefix.push_str(&format!("let child{index}=self.field{index}.{method}();"));
                    }
                }
                let mut members=layout.leaves.iter().map(|leaf|{
                    let name=leaf.path.iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_");
                    let first=leaf.path[0];
                    let place=if leaf.path.len()>1 && self.scoped_record_native_parameter(record.fields[first].ty.unwrap()) {
                        let child_name=leaf.path[1..].iter().map(|index|format!("field{index}")).collect::<Vec<_>>().join("_");
                        format!("child{first}.{child_name}")
                    }else{
                        let modifier=if mutable{"mut "}else{""};
                        let path=leaf.path.iter().map(|index|format!(".field{index}")).collect::<String>();
                        format!("&{modifier}self{path}")
                    };
                    format!("{name}:{place}")
                }).collect::<Vec<_>>().join(",");
                for (path,order) in &orders {
                    let value=if let Some(&first)=path.first() && self.scoped_record_native_parameter(record.fields[first].ty.unwrap()) {
                        format!("child{first}.{}",Self::record_order_member(&path[1..]))
                    }else{format!("&{}self{}",if mutable{"mut "}else{""},order.place)};
                    let _=write!(members,",{}:{value}",order.member);
                }
                let mode=if mutable{"Mutate"}else{"Read"};
                format!("{prefix}Tsr{mode}Record{node}Fields{{{members}}}")
            };
            output.push_str(&self.record_spread_support(node,&layout));
            output.push_str(&crate::scoped_record_bridge::declarations_with_transport(&layout,Types{
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
            },Some(crate::scoped_record_bridge::OwnedTransport{read:transport_body(false),mutate:transport_body(true),orders:orders.iter().map(|(_,order)|crate::scoped_record_bridge::OrderAccess{member:order.member.clone(),place:order.place.clone(),width:order.width,owned:order.owned.clone()}).collect()})));
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
    #[test]
    fn separate_mutable_getters_cannot_hold_sibling_fields_together() {
        let source=crate::format(r#"type Holder={run:ReadOnly<Function<int>>;label:string;};type Pair={left:Holder;right:Holder;};function touch(left:Mutable<Holder>,right:Mutable<Holder>):void{left.label="left!";right.label="right!";}function probe(holder:ReadOnly<Holder>):int{return holder.run();}function main(){}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let outer=checked.program.records.iter().position(|record|record.name.text=="Pair").unwrap();
        let Type::Record(inner)=checked.program.records[outer].fields[0].ty.unwrap() else{panic!("child record")};
        let baseline=emit(&checked.program);
        crate::scoped_record_places::tests::native(&baseline);
        let mut rust=baseline;
        rust.push_str(&format!(r#"
            struct Action;
            impl ReadCallback<(),i64> for Action{{fn invoke_read(&self,_:())->i64{{3}}}}
            fn update<V:TsrMutateRecord{outer}<Field0Field1=String,Field1Field1=String>>(value:&mut V){{
                let left=value.tsr_field0_field1_mut();
                let right=value.tsr_field1_field1_mut();
                *left=String::Owned("left!".into());
                *right=String::Owned("right!".into());
            }}
            fn verify(){{
                let mut value=Record{outer}{{field0:Record{inner}{{field0:Action,field1:String::Owned("left".into())}},field1:Record{inner}{{field0:Action,field1:String::Owned("right".into())}}}};
                update(&mut value);
                assert_eq!(value.field0.field1.as_ref(),"left!");assert_eq!(value.field1.field1.as_ref(),"right!");
            }}
        "#));
        assert!(rust.contains("fn main() -> () {"));
        rust=rust.replace("fn main() -> () {","fn main() -> () { verify();");
        // Each getter borrows the whole view. Sibling fields are reached together
        // through tsr_fields_mut, which the next test exercises.
        crate::scoped_record_places::tests::native_rejection(&rust,"E0499");
    }

    #[test]
    fn nominal_whole_view_transport_preserves_disjoint_sibling_fields() {
        let source=crate::format(r#"type Holder={run:ReadOnly<Function<int>>;label:string;};type Pair={left:Holder;right:Holder;};function touch(left:Mutable<Holder>,right:Mutable<Holder>):void{left.label="left!";right.label="right!";}function probe(holder:ReadOnly<Holder>):int{return holder.run();}function main(){}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap();
        let outer=checked.program.records.iter().position(|record|record.name.text=="Pair").unwrap();
        let Type::Record(inner)=checked.program.records[outer].fields[0].ty.unwrap() else{panic!("child record")};
        let baseline=emit(&checked.program);
        crate::scoped_record_places::tests::native(&baseline);
        let mut rust=baseline;
        rust.push_str(&format!(r#"
            struct Left(std::string::String);struct Right(Vec<i64>);
            impl ReadCallback<(),i64> for Left{{fn invoke_read(&self,_:())->i64{{self.0.len() as i64}}}}
            impl ReadCallback<(),i64> for Right{{fn invoke_read(&self,_:())->i64{{self.0[0]}}}}
            fn inspect<V:TsrReadRecord{outer}<Field0Field1=String,Field1Field1=String>>(value:&V)->i64 where V::Field0Field0:ReadCallback<(),i64>,V::Field1Field0:ReadCallback<(),i64>{{
                let fields=value.tsr_fields();
                ReadCallback::invoke_read(fields.field0_field0,())+ReadCallback::invoke_read(fields.field1_field0,())
            }}
            fn update<V:TsrMutateRecord{outer}<Field0Field1=String,Field1Field1=String>>(value:&mut V){{
                let fields=value.tsr_fields_mut();
                let left=fields.field0_field1;
                let right=fields.field1_field1;
                *left=String::Owned("left!".into());
                *right=String::Owned("right!".into());
            }}
            fn verify(){{
                let mut value=Record{outer}{{field0:Record{inner}{{field0:Left("abc".into()),field1:String::Owned("left".into())}},field1:Record{inner}{{field0:Right(vec![11]),field1:String::Owned("right".into())}}}};
                assert_eq!(inspect(&value),14);
                let left_pointer=value.field0.field1.as_ref().as_ptr();
                {{let shared=value.tsr_fields();assert_eq!(shared.field0_field1.as_ref().as_ptr(),left_pointer);}}
                {{let shared=TsrReadView{outer}{{field0:TsrReadView{inner}{{field0:&value.field0.field0,field1:&value.field0.field1}},field1:TsrReadView{inner}{{field0:&value.field1.field0,field1:&value.field1.field1}}}};assert_eq!(inspect(&shared),14);}}
                {{let mut exclusive=TsrMutateView{outer}{{field0:TsrMutateView{inner}{{field0:&mut value.field0.field0,field1:&mut value.field0.field1}},field1:TsrMutateView{inner}{{field0:&mut value.field1.field0,field1:&mut value.field1.field1}}}};update(&mut exclusive);assert_eq!(inspect(&exclusive),14);}}
                update(&mut value);
                assert_eq!(value.field0.field1.as_ref(),"left!");assert_eq!(value.field1.field1.as_ref(),"right!");
            }}
        "#));
        assert!(rust.contains("fn main() -> () {"));
        rust=rust.replace("fn main() -> () {","fn main() -> () { verify();");
        crate::scoped_record_places::tests::native(&rust);
        let invalid=rust.replace("let shared=TsrReadView", "let mut shared=TsrReadView").replace("assert_eq!(inspect(&shared),14);", "update(&mut shared);");
        crate::scoped_record_places::tests::native_rejection(&invalid,&format!("TsrMutateRecord{outer}"));
    }

    #[test]
    fn nested_mutable_arguments_borrow_disjoint_sibling_records() {
        let source=crate::format(r#"type Holder={run:ReadOnly<Function<int>>;label:string;};type Pair={left:Holder;right:Holder;};function touch(left:Mutable<Holder>,right:Mutable<Holder>):void{left.label="left!";right.label="right!";}function relay(pair:Mutable<Pair>):void{touch(pair.left,pair.right);}function main(){}"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).expect("disjoint record fields must not overlap");
        crate::scoped_record_places::tests::native(&emit(&checked.program));
    }

}
