//! Ordered Map emission, separate from recursive dispatch.
use super::*;
#[path="runtime_maps.rs"] mod runtime;
impl Emitter<'_> {
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
    #[inline(never)]
    pub(super) fn map_constructor_expression(&self, function: &CheckedFunction, expression: &Expr, mode: Mode) -> String {
        let ExprKind::MapConstructor(arguments, _) = &expression.kind else { unreachable!() };
        let Type::Map(id) = expression.ty.unwrap() else { unreachable!() };
        let map = self.program.records[id].map_definition.as_ref().unwrap();
        let native=self.native_type(Type::Map(id));
        let code=if let Some(source)=arguments.first() {
            let value=self.expr(function, source, Mode::Read);
            let items=match source.ty.unwrap() {
                Type::Null|Type::Undefined=>None,
                Type::Map(id)=>{
                    let from=self.program.records[id].map_definition.as_ref().unwrap();
                    let key=self.element_alternative(from.key,map.key,"key.clone()");
                    let value=self.element_alternative(from.value,map.value,"value.clone()");
                    Some(format!("map_source.iter().map(|(key,value)|({key},{value}))"))
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
    #[test]
    fn ordered_map_core_emission_uses_tree_queries_and_sorted_snapshots() {
        // Exercise backend selection while OrderedMap remains unexported. Source
        // checking still supplies the ordinary map ownership and method plans.
        let source=r#"function owned():Map<int,string>{const map=new Map<int,string>();map.set(1,'moved');return map;}function main(){if(owned().get(1)!=='moved'){throw 'transfer';}const values=new Map<string,int>();values.set('z',9);values.set('a',1);values.set('m',5);values.set('a',2);const query='a';if(values.get(query)!==2 || !values.has(query) || values.size!==3){throw 'lookup';}const keys=values.keys();if(keys.join(',')!=='a,m,z'){throw 'order';}let visited='';values.forEach((value,key)=>{visited+=key;});if(visited!=='amz'){throw 'callback order';}const snapshot=values.values();if(snapshot.join(',')!=='2,5,9'){throw 'values';}const pairs=values.entries();if(pairs[0][0]!=='a' || pairs[2][1]!==9){throw 'entries';}const copy=clone(values);if(!copy.delete(query) || copy.delete(query) || copy.size!==2 || values.size!==3){throw 'clone';}copy.clear();if(copy.size!==0){throw 'clear';}const numbers=new Map<number,string>();numbers.set(-0.0,'zero');numbers.set(NaN,'nan');if(numbers.get(0.0)!=='zero' || numbers.get(NaN)!=='nan'){throw 'number';}}
"#;
        let formatted=crate::format(source).unwrap();assert_eq!(crate::format(&formatted).unwrap(),formatted);
        let mut checked=crate::check(&formatted).unwrap();
        for record in &mut checked.program.records {if record.map_definition.is_some(){record.native_type=Some(crate::native_signatures::NativeTypeId::OrderedMap);}}
        crate::scoped_record_places::tests::native(&super::emit(&checked.program));
    }
}
