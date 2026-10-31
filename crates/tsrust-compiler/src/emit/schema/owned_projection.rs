//! Reversible owned schema projections used before a consuming callback commits.
use super::*;

impl Emitter<'_> {
    /// Preparation has already checked the input and evaluated defaults once.
    /// The inverse owns only removed storage and conversion metadata. It never
    /// revalidates input, reruns a factory, or clones a user payload.
    pub(in crate::emit) fn schema_owned_projection(
        &self, id: usize, input: Type, generated: &mut String,
        seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet,
    ) -> String {
        let name = self.schema_owned_prepared_validator(id, input, generated, seen, errors);
        let marker = format!("/*schema_owned_projection_{name}*/");
        if generated.contains(&marker) { return name; }
        generated.push_str(&marker);
        let output = self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let mut arms = Vec::new();
        if let Type::Union(union) = input {
            for (tag, member) in self.program.unions[union.index()].members.iter().enumerate() {
                let child = self.schema_owned_projection(id, *member, generated, seen, errors);
                arms.push(format!("Prepared{name}::I{tag}(state)=>{{let Union{}::V{tag}(input)=input else{{unreachable!(\"prepared input alternative\")}};let(value,restore)=project_{child}(input,*state);(value,Box::new(move|value|Union{}::V{tag}(restore(value))))}}", union.index(), union.index()));
            }
        } else {
            for (plan, choice) in self.program.schema_choices.for_output(output) {
                if choice.presence == Some(input) {
                    arms.push(format!("Prepared{name}::P{plan}Absent=>({},Box::new(|value|{}))", self.schema_output_injection(input, output, "input"), self.schema_selected_output(input, output, "value")));
                }
                for (branch, child_output) in choice.branches.iter().enumerate() {
                    let child = self.schema_owned_projection(self.schema_child(*child_output), input, generated, seen, errors);
                    arms.push(format!("Prepared{name}::P{plan}B{branch}(state)=>{{let(value,restore)=project_{child}(input,*state);({},Box::new(move|value|restore({})))}}", self.schema_output_injection(*child_output, output, "value"), self.schema_selected_output(*child_output, output, "value")));
                }
            }
            let body = self.schema_owned_base_projection(id, input, output, &name, generated, seen, errors);
            arms.push(format!("Prepared{name}::Base(state)=>{{{body}}}"));
        }
        let raw = self.native_type(input);
        let value = self.native_type(output);
        generated.push_str(&format!("fn project_{name}<'a>(input:{raw},prepared:Prepared{name})->({value},Box<dyn FnOnce({value})->{raw}+'a>)where {raw}:'a,{value}:'a{{match prepared{{{}}}}}", arms.join(",")));
        name
    }

    fn schema_owned_base_projection(
        &self, id: usize, input: Type, output: Type, name: &str,
        generated: &mut String, seen: &mut HashSet<(usize, Type)>,
        errors: &crate::error_effects::FailureSet,
    ) -> String {
        if let Some((state, _, commit)) = self.schema_owned_leaf_preparation(output, input) {
            if state == "std::convert::Infallible" { return "match state{}".into(); }
            // Copy inputs retain their exact source representation, including an
            // integer converted to a checked floating-point output.
            // A leaf whose output type is its input type commits `*input`
            // unchanged, so it restores itself without a captured copy.
            if input == output && matches!(input, Type::Int | Type::Number | Type::Boolean) {
                return format!("let value={{{commit}}};(value,Box::new(|value|value))");
            }
            if input.is_copy() {
                return format!("let original=input;let value={{{commit}}};(value,Box::new(move|_|original))");
            }
            if matches!(output, Type::Enum(_, _)) {
                return "(state,Box::new(move|_|input))".into();
            }
            return format!("let value={{{commit}}};(value,Box::new(|value|value))");
        }
        match output {
            Type::Union(union) if self.schema_enum_members(output).is_some() => {
                if input.scalar_base() != Type::String { return "match state{}".into(); }
                let inverse = self.program.unions[union.index()].members.iter().enumerate().map(|(tag, _)|format!("Union{}::V{tag}(value)=>value", union.index())).collect::<Vec<_>>().join(",");
                format!("let value=commit_{name}(input,Prepared{name}::Base(state));(value,Box::new(|value|match value{{{inverse}}}))")
            }
            Type::Union(union) => {
                if matches!(input, Type::Null | Type::Undefined) {
                    if self.program.unions[union.index()].members.contains(&input) {
                        return format!("({},Box::new(|value|{}))", self.schema_output_injection(input, output, "input"), self.schema_selected_output(input, output, "value"));
                    }
                    return "match state{}".into();
                }
                let Some(base) = self.schema_presence_element(output) else { return "match state{}".into(); };
                let child = self.schema_owned_projection(self.schema_child(base), input, generated, seen, errors);
                format!("let(value,restore)=project_{child}(input,state);({},Box::new(move|value|restore({})))", self.schema_output_injection(base, output, "value"), self.schema_selected_output(base, output, "value"))
            }
            Type::Array(element) => {
                let Type::Array(raw_element) = input else { return "match state{}".into(); };
                let child = self.schema_owned_projection(self.schema_child(element.ty()), raw_element.ty(), generated, seen, errors);
                format!("let mut output=<{}>::from([]);let mut restores=Vec::with_capacity(state.len());for(value,state)in input.tsr_owned_items().zip(state){{let(value,restore)=project_{child}(value,state);output.push(value);restores.push(restore);}}(output,Box::new(move|value|{{let mut input=<{}>::from([]);for(value,restore)in value.tsr_owned_items().zip(restores){{input.push(restore(value));}}input}}))", self.native_type(output), self.native_type(input))
            }
            Type::Record(object) if self.program.records[object].name.text.starts_with("@schema:retained:") => {
                let fields = &self.program.records[object].fields;
                let inner = fields[0].ty.unwrap();
                let raw = if input == output { inner } else { input };
                let child = self.schema_owned_projection(self.schema_child(inner), raw, generated, seen, errors);
                let saved_order = if self.record_runtime.ordered.contains(&object) { "let original_order=input.key_order;" } else { "" };
                let value = self.schema_output_record(object,"field0:value,field1:input.field1");
                let restored = self.schema_restored_record(object,"field0:restore(value.field0),field1:value.field1","original_order");
                if input == output {
                    format!("{saved_order}let(value,restore)=project_{child}(input.field0,state);({value},Box::new(move|value|{restored}))")
                } else {
                    format!("let(value,restore)=project_{child}(input,state);({},Box::new(move|value|restore(value.field0)))", self.schema_output_record(object,&format!("field0:value,field1:<{}>::new()",self.native_type(fields[1].ty.unwrap()))))
                }
            }
            Type::Record(object) => {
                let Type::Record(raw) = input else { return "match state{}".into(); };
                if self.program.records[raw].native_type.is_some() { return "match state{}".into(); }
                self.schema_owned_object_projection(object, raw, generated, seen, errors)
            }
            _ => { let _ = id; "match state{}".into() }
        }
    }

    fn schema_owned_object_projection(
        &self, object: usize, raw: usize, generated: &mut String,
        seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet,
    ) -> String {
        let input_fields = &self.program.records[raw].fields;
        let output_fields = &self.program.records[object].fields;
        let mut project = if self.record_runtime.ordered.contains(&raw) { String::from("let original_order=input.key_order;") } else { String::new() };
        let mut inverse = Vec::new();
        for (position, field) in input_fields.iter().enumerate() {
            if !output_fields.iter().any(|candidate| candidate.name.text == field.name.text) {
                project.push_str(&format!("let original{position}=input.field{position};"));
                inverse.push(format!("field{position}:original{position}"));
            }
        }
        for (index, field) in output_fields.iter().enumerate() {
            let provided = input_fields.iter().enumerate().find(|(_, candidate)| candidate.name.text == field.name.text);
            if let Some((position, provided)) = provided {
                let child = self.schema_owned_projection(self.schema_child(field.ty.unwrap()), provided.ty.unwrap(), generated, seen, errors);
                // A default for a present field does not currently occur, but
                // retaining its raw input makes that state reversible as well.
                project.push_str(&format!("let(field{index},restore{index}):({},Box<dyn FnOnce({})->{}+'a>)=match state.{index}{{PreparedField::Present(state)=>project_{child}(input.field{position},state),PreparedField::Default(value)=>{{let original=input.field{position};(value,Box::new(move|_|original))}}}};", self.native_type(field.ty.unwrap()), self.native_type(field.ty.unwrap()), self.native_type(provided.ty.unwrap())));
                inverse.push(format!("field{position}:restore{index}(value.field{index})"));
            } else {
                project.push_str(&format!("let field{index}=match state.{index}{{PreparedField::Default(value)=>value,PreparedField::Present(state)=>match state{{}}}};"));
            }
        }
        let fields = (0..output_fields.len()).map(|index|format!("field{index}")).collect::<Vec<_>>().join(",");
        project.push_str(&format!("({},Box::new(move|value|{}))", self.schema_output_record(object,&fields), self.schema_restored_record(raw,&inverse.join(","),"original_order")));
        project
    }
}
