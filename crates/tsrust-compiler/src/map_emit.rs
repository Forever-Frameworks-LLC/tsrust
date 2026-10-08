//! Ordered Map emission, separate from recursive dispatch.
use super::*;
#[path="runtime_maps.rs"] mod runtime;
impl Emitter<'_> {
    pub(super) fn map_types(&self, output: &mut String) {
        if !self.program.records.iter().any(|record| record.map_definition.is_some()) { return; }
        output.push_str(runtime::MAP_RUNTIME);
        for (id, record) in self.program.records.iter().enumerate() { if let Some(map) = &record.map_definition {
            let key=self.native_type(map.key);let value=self.native_type(map.value);
            if self.scoped_record_native_parameter(map.value) && map.entry.is_none() {
                let _=writeln!(output,"type Map{id}<P>=TsrMap<{key},P>;");
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
        let value_type=self.program.records[id].map_definition.as_ref().unwrap().value;
        let borrowed=matches!(expression.ty,Some(Type::BorrowedUnion(_)));
        let mode=match method {MapMethod::Set|MapMethod::Delete|MapMethod::Clear=>Mode::Mutate,MapMethod::Get if !borrowed && !value_type.is_copy()=>Mode::Consume,_=>Mode::Read};
        let receiver=self.expr(function,base,mode);
        let mut action=String::new();
        if let Some(key)=arguments.first() {let key=self.expr(function,key,if method==MapMethod::Set{Mode::Consume}else{Mode::Read});let _=write!(action,"let map_key={key};");}
        match method {
            MapMethod::Set=>{let value=self.expr(function,&arguments[1],Mode::Consume);let _=write!(action,"let map_value={value};map_receiver.set(map_key,map_value)");},
            MapMethod::Has=>action.push_str("map_receiver.has(map_key)"),
            MapMethod::Delete=>action.push_str("map_receiver.delete(map_key)"),
            MapMethod::Clear=>action.push_str("map_receiver.clear()"),
            MapMethod::Keys=>action.push_str(&self.flat_array_result(expression.ty.unwrap(),"map_receiver.iter().map(|(key,_)|key.clone()).collect::<Vec<_>>()")),
            MapMethod::Values=>action.push_str(&self.flat_array_result(expression.ty.unwrap(),"map_receiver.iter().map(|(_,value)|value.clone()).collect::<Vec<_>>()")),
            MapMethod::Entries(pair)=>{
                let pair_type=self.native_type(Type::Record(pair));
                action.push_str(&self.flat_array_result(expression.ty.unwrap(),&format!("map_receiver.iter().map(|(key,value)|{pair_type}{{field0:key.clone(),field1:value.clone()}}).collect::<Vec<_>>()")));
            },
            MapMethod::Get=>{
                let selected=if value_type.is_copy(){"map_receiver.get(map_key).copied()"}else if borrowed{"map_receiver.get(map_key)"}else{"map_receiver.into_value(map_key)"};
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

impl Emitter<'_> {
    /// End a straight-line local storage lifetime before a later refill.
    /// Declarations outside this block must retain their surrounding storage.
    pub(super) fn local_map_clear_epoch(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<String> {
        let StmtKind::Expr(call) = &statement.kind else { return None; };
        if call.target != Some(CallTarget::MapMethod(MapMethod::Clear)) { return None; }
        let ExprKind::Call(callee, _) = &call.kind else { return None; };
        let ExprKind::Field(base, _) = &callee.kind else { return None; };
        let mut root = base.as_ref();
        let mut names = Vec::new();
        while let ExprKind::Field(parent, name) = &root.kind { names.push(name); root = parent; }
        if !matches!(root.kind, ExprKind::Name(_)) { return None; }
        let binding = root.binding?;
        if !body.iter().any(|item| matches!(item.kind, StmtKind::Let { binding: Some(id), .. } if id == binding)) { return None; }
        let mut ty = fun.bindings[binding].ty;
        let mut path = Vec::new();
        for name in names.into_iter().rev() {
            let Type::Record(record) = ty else { return None; };
            let index = self.fields[record].get(name.as_str()).copied()?;
            path.push(index);
            ty = self.program.records[record].fields[index].ty?;
        }
        let place = if path.is_empty() { format!("value{binding}") }
            else { self.owned_record_leaf_place(fun, binding, &path)? };
        let Type::Map(id) = ty else { return None; };
        let value = self.program.records[id].map_definition.as_ref()?.value;
        if !self.scoped_record_native_parameter(value) { return None; }
        let native = self.native_type(Type::Map(id));
        Some(format!("drop({place}); let mut {place}: {native} = <{native}>::new();"))
    }
}
