//! Ordered Map emission, separate from recursive dispatch.
use super::*;
#[path="../runtime/maps.rs"] mod runtime;
impl Emitter<'_> {
    pub(super) fn ordered_map_entry_view(&self,id:usize,key:&str,value:&str)->String {
        let map=self.program.records[id].map_definition.as_ref().expect("checked ordered map");
        let pair=self.program.scoped_records.declaration(map.entry.expect("checked entry"));
        let payload=if let Type::Record(child)=map.value && !self.program.records[child].fields.is_empty() && !self.record_borrow_leaf(child) {
            let child=self.program.scoped_records.declaration(child);
            let view=self.record_view_from_fields(child,&mut Vec::new(),false,"ordered_fields");
            format!("{{let ordered_fields=({value}).tsr_fields();{view}}}")
        }else{value.into()};
        format!("TsrReadView{pair}{{field0:{key},field1:{payload}}}")
    }
    pub(super) fn map_types(&self, output: &mut String) {
        if !self.program.records.iter().any(|record| record.map_definition.is_some()) { return; }
        output.push_str(runtime::MAP_RUNTIME);
        let mut scoped_entries=std::collections::HashSet::new();
        // Every concrete environment of one Map type shares its entry layout,
        // so a caller's Map matches a callee that iterates the entries.
        let nominal=|map:&crate::syntax::MapDefinition|match map.value{Type::Record(value)=>Some((map.key,self.program.scoped_records.declaration(value))),_=>None};
        let mut shared_entries=std::collections::HashMap::new();
        for record in &self.program.records { if let Some(map)=&record.map_definition && self.scoped_record_native_parameter(map.value) && let (Some(key),Some(pair))=(nominal(map),map.entry) {
            shared_entries.entry(key).or_insert(pair);
        }}
        for (id, record) in self.program.records.iter().enumerate() { if let Some(map) = &record.map_definition {
            let key=self.native_type(map.key);let value=self.native_type(map.value);
            if record.native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap) {
                if self.scoped_record_native_parameter(map.value) {let _=writeln!(output,"type Map{id}<P>=TsrOrderedMap<TsrOrdered<{key}>,P>;");}
                else {let _=writeln!(output,"type Map{id}=TsrOrderedMap<TsrOrdered<{key}>,{value}>;");}
                continue;
            }
            if self.scoped_record_native_parameter(map.value) {
                if let Some(pair)=map.entry.or_else(||nominal(map).and_then(|key|shared_entries.get(&key).copied())) {
                    let pair=self.program.scoped_records.declaration(pair);
                    let _=writeln!(output,"type Map{id}<P>=TsrMap<{key},P,Record{pair}<P>>;");
                    if scoped_entries.insert(pair) {
                        let _=writeln!(output,"impl<P> TsrMapEntry<{key},P> for Record{pair}<P> {{fn from_pair(key:{key},value:P)->Self{{Self{{field0:key,field1:value}}}}fn key(&self)->&{key}{{&self.field0}}fn value(&self)->&P{{&self.field1}}fn value_mut(&mut self)->&mut P{{&mut self.field1}}fn into_pair(self)->({key},P){{(self.field0,self.field1)}}}}");
                    }
                } else {let _=writeln!(output,"type Map{id}<P>=TsrMap<{key},P>;");}
                continue;
            }
            if let Some(pair)=map.entry {
                let _=writeln!(output,"type Map{id}=TsrMap<{key},{value},Record{pair}>;");
                let _=writeln!(output,"impl TsrMapEntry<{key},{value}> for Record{pair} {{fn from_pair(key:{key},value:{value})->Self{{Self{{field0:key,field1:value}}}}fn key(&self)->&{key}{{&self.field0}}fn value(&self)->&{value}{{&self.field1}}fn value_mut(&mut self)->&mut {value}{{&mut self.field1}}fn into_pair(self)->({key},{value}){{(self.field0,self.field1)}}}}");
            }else{let _=writeln!(output,"type Map{id}=TsrMap<{key},{value}>;");}
        }}
    }
    /// Assembles a Map from its already rendered read-mode source.
    pub(super) fn map_constructor_expression(&self, expression: &Expr, mode: Mode, value: Option<String>) -> String {
        let ExprKind::MapConstructor(arguments, _) = &expression.kind else { unreachable!() };
        let Type::Map(id) = expression.ty.unwrap() else { unreachable!() };
        let map = self.program.records[id].map_definition.as_ref().unwrap();
        let native=self.native_type(Type::Map(id));
        let code=if let Some(source)=arguments.first() {
            let value=value.expect("rendered Map source");
            let items=match source.ty.unwrap() {
                Type::Null|Type::Undefined=>None,
                Type::Map(id)=>{
                    let from=self.program.records[id].map_definition.as_ref().unwrap();
                    let key=self.element_alternative(from.key,map.key,"key.clone()");
                    let value=self.element_alternative(from.value,map.value,"value.clone()");
                    let iterator=if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap){"source_iter"}else{"iter"};
                    Some(format!("map_source.{iterator}().map(|(key,value)|({key},{value}))"))
                },
                Type::Array(ArrayElement::Record(pair,0))=>{
                    let fields=&self.program.records[pair].fields;
                    let key=self.element_alternative(fields[0].ty.unwrap(),map.key,"pair.field0.clone()");
                    let value=self.element_alternative(fields[1].ty.unwrap(),map.value,"pair.field1.clone()");
                    Some(format!("map_source.iter().map(|pair|({key},{value}))"))
                }, _=>unreachable!("checked Map source"),
            };
            if let Some(items)=items { format!("{{let map_source={value};{native}::from_values({items})}}") }
            else {format!("{{let _={value};<{native}>::new()}}")}
        } else {format!("<{native}>::new()")};
        match mode {Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code}
    }
    #[inline(never)]
    pub(super) fn map_method_expression(&self, function: &CheckedFunction, expression: &Expr, callee: &Expr, arguments: &[Expr], method: MapMethod) -> String {
        let ExprKind::Field(base,_) = &callee.kind else {unreachable!()};
        let Type::Map(id)=base.ty.unwrap() else {unreachable!()};
        let map=self.program.records[id].map_definition.as_ref().unwrap();
        let value_type=map.value;
        let ordered=self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap);
        let query_suffix=if map.key==Type::String {"string_key"}else{"copy_key"};
        let iterator=if ordered {"source_iter"}else{"iter"};
        let borrowed=matches!(expression.ty,Some(Type::BorrowedUnion(_)));
        let mode=match method {MapMethod::Set|MapMethod::Delete|MapMethod::Clear=>Mode::Mutate,MapMethod::Get if !borrowed && !value_type.is_copy()=>Mode::Consume,_=>Mode::Read};
        let receiver=self.expr(function,base,mode);
        let mut action=String::new();
        if let Some(key)=arguments.first() {let key=self.expr(function,key,if method==MapMethod::Set{Mode::Consume}else{Mode::Read});let _=write!(action,"let map_key={key};");}
        match method {
            MapMethod::Set=>{
                let mut root=base.as_ref();
                while let ExprKind::Field(parent,_) = &root.kind {root=parent;}
                let value=if root.binding.is_some_and(|binding|function.storage_exits.map_entry_epoch_root(binding)) {
                    let mut value=&arguments[1];
                    while let ExprKind::Unary{op:UnaryOp::RecordCarrier,value:inner,..}=&value.kind {value=inner;}
                    let input=self.expr(function,value,Mode::Consume);
                    let source=self.owned_record_source_type(function,value).unwrap_or(value.ty.unwrap());
                    let converted=self.map_entry_payload_projection(source,value_type,"entry_input");
                    format!("{{let entry_input={input};{converted}}}")
                }else{self.expr(function,&arguments[1],Mode::Consume)};
                if ordered {let _=write!(action,"let map_value={value};let _=map_receiver.insert(TsrOrdered::new(map_key),map_value);");}
                else {let _=write!(action,"let map_value={value};map_receiver.set(map_key,map_value)");}
            },
            MapMethod::Has=>action.push_str(&if ordered{format!("map_receiver.contains_{query_suffix}(map_key)")}else{"map_receiver.has(map_key)".into()}),
            MapMethod::Delete=>action.push_str(&if ordered{format!("map_receiver.remove_{query_suffix}(map_key).is_some()")}else{"map_receiver.delete(map_key)".into()}),
            MapMethod::Clear=>action.push_str("map_receiver.clear()"),
            MapMethod::Keys=>action.push_str(&self.flat_array_result(expression.ty.unwrap(),&format!("map_receiver.{iterator}().map(|(key,_)|key.clone()).collect::<Vec<_>>()"))),
            MapMethod::Values=>action.push_str(&self.flat_array_result(expression.ty.unwrap(),&format!("map_receiver.{iterator}().map(|(_,value)|value.clone()).collect::<Vec<_>>()"))),
            MapMethod::Entries(pair)=>{
                let pair_type=self.native_type(Type::Record(pair));
                action.push_str(&self.flat_array_result(expression.ty.unwrap(),&format!("map_receiver.{iterator}().map(|(key,value)|{pair_type}{{field0:key.clone(),field1:value.clone()}}).collect::<Vec<_>>()")));
            },
            MapMethod::Get=>{
                let get=if ordered{format!("map_receiver.get_{query_suffix}(map_key)")}else{"map_receiver.get(map_key)".into()};
                let selected=if value_type.is_copy(){format!("{get}.copied()")}else if borrowed{get}else if ordered{format!("map_receiver.into_value_{query_suffix}(map_key)")}else{"map_receiver.into_value(map_key)".into()};
                match expression.ty.unwrap() {
                    Type::Undefined=>{let _=write!(action,"let _={selected};TsrUndefined");},
                    Type::Union(id)|Type::BorrowedUnion(id)=>{
                        let absent=self.program.unions[id.index()].members.iter().position(|ty|*ty==Type::Undefined).unwrap();
                        let present=self.optional_element_present(id,value_type,borrowed,"payload");
                        let tag=if borrowed{"ViewUnion"}else{"Union"};let nil=if borrowed{"&TsrUndefined"}else{"TsrUndefined"};
                        let _=write!(action,"match {selected}{{Some(payload)=>{present},None=>{tag}{}::V{absent}({nil})}}",id.index());
                    },_=>unreachable!("optional Map result"),
                }
            },
        }
        format!("{{let map_receiver={receiver};{action}}}")
    }
}

#[cfg(test)]
mod ordered_tests {
    fn ordered_project(source:&str)->String {
        struct Source(String);
        impl crate::project::SourceProvider for Source {
            fn resolve(&self,_:Option<&crate::project::SourceId>,path:&str)->Result<crate::project::SourceId,String>{Ok(crate::project::SourceId(path.into()))}
            fn read(&self,id:&crate::project::SourceId)->Result<String,String>{if id.0=="ordered-map.tsr" {Ok(self.0.clone())} else {Err(format!("Missing source: {}",id.0))}}
        }
        let source=format!("import {{OrderedMap}} from 'std:collections';\n{}",source.replace("Map<","OrderedMap<"));
        let source=crate::format(&source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
        let graph=crate::project::load_project(&Source(source),"ordered-map.tsr").unwrap();
        let checked=crate::project::check_graph(graph).unwrap();
        checked.emit_rust()
    }
    #[test]
    fn ordered_map_entries_borrow_recursive_view_records() {
        let helpers=include_str!("../../tests/fixtures/collections/projected-views.tsr").split("function fill(").next().unwrap();
        let source=format!("{helpers}\n{}",r#"
type Chain={data:ByteView;next?:Chain;};
type Tree={data:ByteView;children:Tree[];};
type OwnedChain={count:int;next?:OwnedChain;};
function treeSize(entry:ReadOnly<[int,Tree]>):int{return length(entry[1].data)+length(entry[1].children[0].data);}
function inspect(entry:ReadOnly<[int,Chain]>):int{return length(entry[1].data)+length(entry[1].next!.data);}
function main(){
  const owner=bytes('tree');const tail:Chain={data:window(owner),next:undefined};
  const values=new Map<int,Chain>();values.set(1,{data:window(owner),next:tail});
  for(const entry of values){strictEqual(inspect(entry),8);strictEqual(text(entry[1].next!.data),'tree');}
  values.clear();strictEqual(length(owner),4);
  const trees=new Map<int,Tree>();trees.set(1,{data:window(owner),children:[{data:window(owner),children:[]}]});
  for(const entry of trees){strictEqual(treeSize(entry),8);}
  const owned=new Map<int,OwnedChain>();owned.set(1,{count:1,next:{count:2,next:undefined}});
  for(const entry of owned){const copy=clone(entry);copy[1].count=7;const child=clone(entry[1]);child.count=9;strictEqual(entry[1].count,1);strictEqual(copy[1].next!.count,2);strictEqual(child.next!.count,2);}
}
"#);
        crate::scoped_record_places::tests::native(&ordered_project(&source));
    }
    #[test]
    fn ordered_map_callback_entries_keep_captured_view_owners() {
        let helpers=include_str!("../../tests/fixtures/collections/projected-views.tsr").split("function fill(").next().unwrap();
        let source=format!("{helpers}\n{}",r#"
type Reader={read:ReadOnly<Function<int>>;};
function main(){
  const owner=bytes('payload');const part=window(owner);const values=new Map<int,Reader>();
  values.set(2,{read:()=>length(part)});values.set(1,{read:()=>length(part)+1});
  let sum=0;let keys='';for(const entry of values){sum+=entry[1].read();keys+=String(entry[0]);}
  strictEqual(sum,15);strictEqual(keys,'12');strictEqual(values.get(1)!.read(),8);
  strictEqual(values.delete(2),true);for(const entry of values){strictEqual(entry[1].read(),8);}
}
"#);
        crate::scoped_record_places::tests::native(&ordered_project(&source));
    }
    #[test]
    fn ordered_map_entries_preserve_byte_view_payloads() {
        let helpers=include_str!("../../tests/fixtures/collections/projected-views.tsr").split("function fill(").next().unwrap();
        let source=format!("{helpers}\n{}",r#"
function inspect(entry:ReadOnly<[int,ByteView]>):int{return length(entry[1]);}
function selected(entry:ReadOnly<[int,ByteView]>):ByteView{match(view(entry[1],0,length(entry[1]))){Ok(result)=>{return result;},Err(error)=>{throw error.message;}}}
function main(){
  const owner=bytes('payload');const values=new Map<int,ByteView>();
  values.set(2,window(owner));values.set(1,window(owner));
  const read=(entry:ReadOnly<[int,ByteView]>):int=>length(entry[1]);
  let seen='';for(const entry of values){strictEqual(inspect(entry),7);strictEqual(read(entry),7);strictEqual(text(selected(entry)),'payload');seen+=String(entry[0])+text(entry[1]);}
  strictEqual(seen,'1payload2payload');strictEqual(text(values.get(1)!),'payload');
  values.clear();strictEqual(length(owner),7);
}
"#);
        crate::scoped_record_places::tests::native(&ordered_project(&source));
    }
    #[test]
    fn ordered_map_entries_borrow_keys_and_cross_readonly_parameters() {
        let source=r#"
function score(entry:ReadOnly<[string,int]>):int{return entry[0].length+entry[1];}
function first(values:ReadOnly<Map<string,int>>):int{for(const entry of values){try{return score(entry);}finally{if(values.size!==2){throw 'finally';}}}return -1;}
type Detail={label:string;count:int;};
function nestedScore(entry:ReadOnly<[string,Detail]>):int{return entry[0].length+entry[1].label.length+entry[1].count;}
function main(){
  const values=new Map<string,int>();values.set('z',9);values.set('ab',1);
  let sum=0;let keys='';
  for(const entry of values){sum+=score(entry);sum+=entry[1];keys+=entry[0];}
  if(sum!==23 || keys!=='abz' || values.size!==2){throw 'borrowed entries';}
  if(first(values)!==3){throw 'completion';}
  const nested=new Map<string,Detail>();nested.set('a',{label:'test',count:4});
  for(const entry of nested){if(nestedScore(entry)!==9 || entry[1].label!=='test'){throw 'nested entry';}const read=()=>nestedScore(entry);if(read()!==9){throw 'capture';}const copy=clone(entry);copy[1].label='changed';if(entry[1].label!=='test' || copy[1].label!=='changed'){throw 'entry clone';}}
}
"#;
        crate::scoped_record_places::tests::native(&ordered_project(source));
    }
    #[test]
    fn ordered_map_core_emission_uses_tree_queries_and_sorted_snapshots() {
        let source=r#"function owned():Map<int,string>{const map=new Map<int,string>();map.set(1,'moved');return map;}function main(){if(owned().get(1)!=='moved'){throw 'transfer';}const values=new Map<string,int>();values.set('z',9);values.set('a',1);values.set('m',5);values.set('a',2);const query='a';if(values.get(query)!==2 || !values.has(query) || values.size!==3){throw 'lookup';}const keys=values.keys();if(keys.join(',')!=='a,m,z'){throw 'order';}let visited='';values.forEach((value,key)=>{visited+=key;});if(visited!=='amz'){throw 'callback order';}const snapshot=values.values();if(snapshot.join(',')!=='2,5,9'){throw 'values';}const pairs=values.entries();if(pairs[0][0]!=='a' || pairs[2][1]!==9){throw 'entries';}const copy=clone(values);if(!copy.delete(query) || copy.delete(query) || copy.size!==2 || values.size!==3){throw 'clone';}copy.clear();if(copy.size!==0){throw 'clear';}const numbers=new Map<number,string>();numbers.set(-0.0,'zero');numbers.set(NaN,'nan');if(numbers.get(0.0)!=='zero' || numbers.get(NaN)!=='nan'){throw 'number';}}
"#;
        crate::scoped_record_places::tests::native(&ordered_project(source));
    }
}
