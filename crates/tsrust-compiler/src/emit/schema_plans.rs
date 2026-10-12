//! Typed union storage keeps ordered runtime plans independently of output types.
use super::*;

impl Emitter<'_> {
    pub(super) fn schema_has_choices(&self, output: Type) -> bool {
        self.program.schema_choices.for_output(output).next().is_some()
    }
    pub(super) fn schema_choice_storage(&self) -> &'static str {
        if self.program.schema_choices.is_empty() { return ""; }
        // Only schema outputs participating in choices use optional base slots.
        // The choice owns its branches behind one unique allocation, breaking
        // recursive same-output layouts without cloning any child plan.
        "struct SchemaSlot<T>(Option<T>);impl<T> SchemaSlot<T>{fn value(value:T)->Self{Self(Some(value))}fn empty()->Self{Self(None)}fn into_value(self)->T{self.0.expect(\"checked base schema slot\")}}impl<T> std::ops::Deref for SchemaSlot<T>{type Target=T;fn deref(&self)->&T{self.0.as_ref().expect(\"checked base schema slot\")}}\n"
    }
    fn schema_child_type(&self, output: Type, child: Type) -> String {
        let id=self.native_record_declaration(self.schema_child(child));
        if self.schema_has_choices(output) { format!("SchemaSlot<Record{id}>") } else { format!("Record{id}") }
    }
    pub(super) fn schema_child_value(&self, id: usize, value: &str) -> String {
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        if self.schema_has_choices(output) { format!("SchemaSlot::value({value})") } else { value.into() }
    }
    pub(super) fn schema_take_child(&self, id: usize, value: &str) -> String {
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        if self.schema_has_choices(output) { format!("({value}).into_value()") } else { value.into() }
    }
    pub(super) fn schema_plan_declaration(&self, id: usize) -> Option<String> {
        let record=&self.program.records[id];
        if record.native_type!=Some(crate::native_signatures::NativeTypeId::Schema) { return None; }
        let output=record.application.as_ref().unwrap().resolved_arguments[0];
        let fields=if let Type::Literal(_, kind)=output {
            format!("expected:{}", self.native_type(kind.base()))
        } else if matches!(output,Type::String|Type::Int|Type::Number) {
            let plan=match output {Type::String=>"StringBounds",Type::Int=>"IntBounds",_=>"NumberBounds"};
            format!("bounds:tsrust_schema_plan::{plan}")
        } else if let Some(members)=self.schema_enum_members(output) {
            members.iter().enumerate().map(|(index,child)|format!("field{index}:{}",self.schema_child_type(output,*child))).collect::<Vec<_>>().join(",")
        } else if let Some(child)=self.schema_presence_element(output) {
            format!("inner:{}",self.schema_child_type(output,child))
        } else if let Type::Array(element)=output {
            format!("element:{}",self.schema_child_type(output,element.ty()))
        } else if let Type::Record(object)=output {
            let record=&self.program.records[object];
            if record.name.text.starts_with("@schema:retained:") {
                format!("inner:{}",self.schema_child_type(output,record.fields[0].ty.unwrap()))
            } else {
                let fields=record.fields.iter().enumerate().map(|(field,value)|format!("field{field}:{}",self.schema_child_type(output,value.ty.unwrap()))).collect::<Vec<_>>().join(",");
                format!("reject_unknown:bool,{fields}")
            }
        } else { String::new() };
        let factory=self.schema_factory_declaration(id);
        let mut result=format!("struct Record{id}{{{factory}{fields}}}\n");
        if self.schema_has_choices(output) {
            let variants=self.program.schema_choices.for_output(output).map(|(plan,choice)|{
                let children=choice.branches.iter().enumerate().map(|(index,child)|format!("field{index}:Record{}",self.native_record_declaration(self.schema_child(*child)))).collect::<Vec<_>>().join(",");
                format!("P{plan}{{{children}}}")
            }).collect::<Vec<_>>().join(",");
            result.push_str(&format!("enum SchemaChoice{id}{{{variants}}}\n"));
        }
        Some(result)
    }
    pub(super) fn schema_choice_constructor(&self, fun: &CheckedFunction, expr: &Expr, plan: usize, args: &[Expr], mode: Mode) -> String {
        let Type::Record(id)=expr.ty.unwrap() else { unreachable!() };
        let id=self.native_record_declaration(id);
        let output=self.program.schema_choices.get(plan).output;
        let setup=args.iter().enumerate().map(|(index,arg)|format!("let branch{index}={};",self.expr(fun,arg,Mode::Consume))).collect::<String>();
        let branches=(0..args.len()).map(|index|format!("field{index}:branch{index}")).collect::<Vec<_>>().join(",");
        let factory=self.schema_factory_initializer(id).replace("choice:None,",&format!("choice:Some(Box::new(SchemaChoice{id}::P{plan}{{{branches}}})),"));
        let base=if let Type::Literal(literal,kind)=output {
            let value=match self.program.records[literal].literal_definition.as_ref().unwrap() {
                crate::literal_types::LiteralValue::String(value)=>format!("String::Borrowed({value:?})"),
                crate::literal_types::LiteralValue::Int(value)=>format!("{value}i64"),
                crate::literal_types::LiteralValue::Number(bits)=>float_literal(f64::from_bits(*bits)),
                crate::literal_types::LiteralValue::Boolean(value)=>value.to_string(),
            };
            let _=kind;format!("expected:{value}")
        } else if matches!(output,Type::String|Type::Int|Type::Number) {
            "bounds:Default::default()".into()
        } else if let Some(members)=self.schema_enum_members(output) {
            (0..members.len()).map(|field|format!("field{field}:SchemaSlot::empty()")).collect::<Vec<_>>().join(",")
        } else if self.schema_presence_element(output).is_some() {
            "inner:SchemaSlot::empty()".into()
        } else if matches!(output,Type::Array(_)) {
            "element:SchemaSlot::empty()".into()
        } else if let Type::Record(object)=output {
            if self.program.records[object].name.text.starts_with("@schema:retained:") { "inner:SchemaSlot::empty()".into() }
            else {format!("reject_unknown:true,{}",(0..self.program.records[object].fields.len()).map(|field|format!("field{field}:SchemaSlot::empty()")).collect::<Vec<_>>().join(","))}
        } else {String::new()};
        let code=format!("{{{setup}Record{id}{{{factory}{base}}}}}");
        match mode {Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code}
    }
}
