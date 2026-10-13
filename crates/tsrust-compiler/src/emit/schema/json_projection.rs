//! Move validated JSON into typed storage with an exact, uniquely owned inverse.
use super::*;

impl Emitter<'_> {
    pub(in crate::emit) fn schema_json_trial(&self, id: usize, generated: &mut String, owned_seen: &mut HashSet<(usize, Type)>, errors: &crate::error_effects::FailureSet) {
        let marker = format!("/*schema_json_trial{id}*/");
        if generated.contains(&marker) { return; }
        generated.push_str(&marker);
        self.schema_json_prepared_validator(id, generated, &mut HashSet::new(), owned_seen, errors);
        self.schema_json_projection(id, generated, &mut HashSet::new());
        let output = self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        generated.push_str(&format!("fn trial_json{id}(schema:&Record{},input:tsrust_native_json::Value,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->tsrust_schema_trial::Trial<tsrust_native_json::Value,{},RestoreJson{id},SchemaFactoryFailure>{{use tsrust_schema_trial::Trial;match prepare_json{id}(schema,&input,path,issues,failure,None){{Some(state)=>{{let(value,restoration)=project_json{id}(input,state);Trial::Prepared{{value,restoration}}}},None=>match failure.take(){{Some(error)=>Trial::Aborted(error),None=>Trial::Rejected(input)}}}}}}", self.native_record_declaration(id), self.native_type(output)));
    }

    pub(in crate::emit) fn schema_selected_output(&self, source: Type, target: Type, value: &str) -> String {
        if source == target { return value.into(); }
        let Type::Union(target) = target else { return value.into(); };
        let members = &self.program.unions[target.index()].members;
        if let Some(tag)=members.iter().position(|member|*member==source) {
            return format!("match {value}{{Union{}::V{tag}(value)=>{},_=>unreachable!(\"readonly projection changed alternative\")}}",target.index(),self.union_move_payload(target,source,"value"));
        }
        let arms = if let Type::Union(source) = source {
            self.program.unions[source.index()].members.iter().enumerate().map(|(tag, ty)| {
                let from = members.iter().position(|member| member == ty).expect("selected schema output");
                format!("Union{}::V{from}(value)=>Union{}::V{tag}(value)", target.index(), source.index())
            }).collect::<Vec<_>>()
        } else {
            let tag = members.iter().position(|member| *member == source).expect("selected schema output");
            vec![format!("Union{}::V{tag}(value)=>value", target.index())]
        };
        format!("match {value}{{{},_=>unreachable!(\"readonly projection changed alternative\")}}", arms.join(","))
    }

    /// PreparedJson declarations must already exist. This pass never validates
    /// again or reruns factories: it consumes the saved preparation state.
    pub(in crate::emit) fn schema_json_projection(&self, id: usize, generated: &mut String, seen: &mut HashSet<usize>) {
        if !seen.insert(id) { return; }
        let marker = format!("/*schema_json_projection{id}*/");
        if generated.contains(&marker) { return; }
        generated.push_str(&marker);
        let output = self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let native = self.native_type(output);
        let mut variants = Vec::new();
        let mut projects = Vec::new();
        let mut restores = Vec::new();
        let mut extras = Vec::new();
        for (plan, choice) in self.program.schema_choices.for_output(output) {
            if choice.presence == Some(Type::Null) {
                variants.push(format!("P{plan}Absent"));
                projects.push(format!("PreparedJson{id}::P{plan}Absent=>({},RestoreJson{id}::P{plan}Absent)", self.schema_output_injection(Type::Null, output, "TsrNull")));
                restores.push(format!("RestoreJson{id}::P{plan}Absent=>tsrust_native_json::Value::Null"));
                extras.push(format!("RestoreJson{id}::P{plan}Absent=>None"));
            }
            for (branch, child_output) in choice.branches.iter().enumerate() {
                let child = self.schema_child(*child_output);
                self.schema_json_projection(child, generated, seen);
                let variant = format!("P{plan}B{branch}");
                variants.push(format!("{variant}(Box<RestoreJson{child}>)"));
                projects.push(format!("PreparedJson{id}::{variant}(state)=>{{let(value,restore)=project_json{child}(input,*state);({},RestoreJson{id}::{variant}(Box::new(restore)))}}", self.schema_output_injection(*child_output, output, "value")));
                restores.push(format!("RestoreJson{id}::{variant}(state)=>restore_json{child}({},*state,extra)", self.schema_selected_output(*child_output, output, "value")));
                extras.push(format!("RestoreJson{id}::{variant}(state)=>take_json_extra{child}(state)"));
            }
        }
        let (state, project, restore, extra) = self.schema_json_base_projection(id, output, generated, seen);
        variants.insert(0, format!("Base({state})"));
        projects.insert(0, format!("PreparedJson{id}::Base(state)=>{{let(value,restore)={{{project}}};(value,RestoreJson{id}::Base(restore))}}"));
        restores.insert(0, format!("RestoreJson{id}::Base(state)=>{{{restore}}}"));
        extras.insert(0, format!("RestoreJson{id}::Base(state)=>{{{extra}}}"));
        generated.push_str(&format!("enum RestoreJson{id}{{{}}}\nfn project_json{id}(input:tsrust_native_json::Value,prepared:PreparedJson{id})->({native},RestoreJson{id}){{match prepared{{{}}}}}\nfn restore_json{id}(value:{native},restore:RestoreJson{id},extra:Option<tsrust_native_json::Map<std::string::String,tsrust_native_json::Value>>)->tsrust_native_json::Value{{match restore{{{}}}}}\nfn take_json_extra{id}(restore:&mut RestoreJson{id})->Option<tsrust_native_json::Map<std::string::String,tsrust_native_json::Value>>{{match restore{{{}}}}}\nimpl tsrust_schema_trial::Restore<tsrust_native_json::Value,{native}> for RestoreJson{id}{{fn restore(self,value:{native})->tsrust_native_json::Value{{restore_json{id}(value,self,None)}}}}\n", variants.join(","), projects.join(","), restores.join(","), extras.join(",")));
    }

    fn schema_json_base_projection(&self, id: usize, output: Type, generated: &mut String, seen: &mut HashSet<usize>) -> (String, String, String, String) {
        let none = String::from("None");
        match output {
            Type::String | Type::Literal(_, crate::literal_types::LiteralKind::String) => (
                "()".into(), "let tsrust_native_json::Value::String(value)=input else{unreachable!(\"prepared string\")};(String::Owned(value),())".into(),
                "tsrust_native_json::Value::String(value.into_owned())".into(), none),
            Type::Int | Type::Number | Type::Enum(_, _) | Type::Literal(_, crate::literal_types::LiteralKind::Int | crate::literal_types::LiteralKind::Number) => (
                "tsrust_native_json::Value".into(), "(state,input)".into(), "drop(value);state".into(), none),
            Type::Boolean | Type::Literal(_, crate::literal_types::LiteralKind::Boolean) => (
                "()".into(), "(state,())".into(), "tsrust_native_json::Value::Bool(value)".into(), none),
            Type::Null => ("()".into(), "(TsrNull,())".into(), "tsrust_native_json::Value::Null".into(), none),
            Type::Undefined => ("std::convert::Infallible".into(), "match state{}".into(), "match state{}".into(), none),
            Type::Array(element) => {
                let child = self.schema_child(element.ty());
                self.schema_json_projection(child, generated, seen);
                let native = self.native_type(output);
                (format!("(Vec<tsrust_native_json::Value>,Vec<RestoreJson{child}>)"),
                    format!("let tsrust_native_json::Value::Array(mut input)=input else{{unreachable!(\"prepared array\")}};let mut values=<{native}>::from([]);let mut restores=Vec::with_capacity(state.len());for(value,state)in input.drain(..).zip(state){{let(value,restore)=project_json{child}(value,state);values.push(value);restores.push(restore);}}(values,(input,restores))"),
                    format!("let(mut input,states)=state;for(value,state)in value.tsr_owned_items().zip(states){{input.push(restore_json{child}(value,state,None));}}tsrust_native_json::Value::Array(input)"), none)
            }
            Type::Union(union) if self.schema_enum_members(output).is_some() => {
                let arms = self.program.unions[union.index()].members.iter().enumerate().map(|(tag, _)| format!("{tag}=>Union{}::V{tag}(String::Owned(value))", union.index())).collect::<Vec<_>>().join(",");
                let inverse = self.program.unions[union.index()].members.iter().enumerate().map(|(tag, _)| format!("Union{}::V{tag}(value)=>tsrust_native_json::Value::String(value.into_owned())", union.index())).collect::<Vec<_>>().join(",");
                ("()".into(), format!("let tsrust_native_json::Value::String(value)=input else{{unreachable!(\"prepared enum\")}};(match state{{{arms},_=>unreachable!(\"prepared enum tag\")}},())"), format!("match value{{{inverse}}}"), none)
            }
            Type::Union(_) => {
                let Some(base) = self.schema_presence_element(output) else {
                    return ("std::convert::Infallible".into(), "match state{}".into(), "match state{}".into(), none);
                };
                let child = self.schema_child(base);
                self.schema_json_projection(child, generated, seen);
                let injection = self.schema_output_injection(base, output, "value");
                let inverse = self.schema_selected_output(base, output, "value");
                (format!("Option<Box<RestoreJson{child}>>"),
                    format!("match state{{PreparedPresence::Absent(value)=>(value,None),PreparedPresence::Present(state)=>{{let(value,restore)=project_json{child}(input,state);({injection},Some(Box::new(restore)))}}}}"),
                    format!("match state{{Some(state)=>restore_json{child}({inverse},*state,extra),None=>tsrust_native_json::Value::Null}}"),
                    format!("state.as_mut().and_then(|state|take_json_extra{child}(state))"))
            }
            Type::Record(object) if self.program.records[object].name.text.starts_with("@schema:retained:") => {
                let fields = &self.program.records[object].fields;
                let child = self.schema_child(fields[0].ty.unwrap());
                self.schema_json_projection(child, generated, seen);
                let json = self.program.records.iter().position(|record| record.native_type == Some(crate::native_signatures::NativeTypeId::JsonValue)).unwrap();
                let extra = self.native_type(fields[1].ty.unwrap());
                (format!("RestoreJson{child}"),
                    format!("let(value,mut restore)=project_json{child}(input,state);let extra=take_json_extra{child}(&mut restore).unwrap_or_default();(Record{}{{field0:value,field1:<{extra}>::from_values(extra.into_iter().map(|(key,value)|(String::Owned(key),Record{}{{value}}))){order}}},restore)", self.native_record_declaration(object), self.native_record_declaration(json),order=self.schema_record_order(object)),
                    format!("let mut extra=tsrust_native_json::Map::new();for entry in value.field1.tsr_owned_items(){{let(key,value)=entry.into_pair();extra.insert(key.into_owned(),value.value);}}restore_json{child}(value.field0,state,Some(extra))"), none)
            }
            Type::Record(object) => self.schema_json_object_projection(object, generated, seen),
            _ => unreachable!("checked schema projection output {output:?} for schema {id}"),
        }
    }

    fn schema_json_object_projection(&self, object: usize, generated: &mut String, seen: &mut HashSet<usize>) -> (String, String, String, String) {
        let mut types = vec!["tsrust_native_json::Map<std::string::String,tsrust_native_json::Value>".to_owned()];
        let mut project = String::from("let tsrust_native_json::Value::Object(mut input)=input else{unreachable!(\"prepared object\")};");
        let mut restore = String::from("let mut input=state.0;if let Some(extra)=extra{input.extend(extra);}");
        let mut values = Vec::new();
        let mut states = vec!["input".to_owned()];
        for (field, member) in self.program.records[object].fields.iter().enumerate() {
            let child = self.schema_child(member.ty.unwrap());
            self.schema_json_projection(child, generated, seen);
            types.push(format!("Option<(std::string::String,RestoreJson{child})>"));
            project.push_str(&format!("let(field{field},restore{field})=match state.{field}{{PreparedField::Present(state)=>{{let(key,value)=input.remove_entry({:?}).expect(\"prepared field\");let(value,restore)=project_json{child}(value,state);(value,Some((key,restore)))}},PreparedField::Default(value)=>(value,None)}};", member.name.text));
            restore.push_str(&format!("if let Some((key,restore))=state.{}{{input.insert(key,restore_json{child}(value.field{field},restore,None));}}", field + 1));
            values.push(format!("field{field}")); states.push(format!("restore{field}"));
        }
        project.push_str(&format!("({},({},))", self.schema_output_record(object,&values.join(",")), states.join(",")));
        restore.push_str("tsrust_native_json::Value::Object(input)");
        (format!("({},)", types.join(",")), project, restore, "Some(std::mem::take(&mut state.0))".into())
    }
}
