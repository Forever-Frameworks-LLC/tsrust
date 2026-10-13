//! Generate owned preparation states and move only the selected branch's input.
use super::*;
use std::collections::HashSet;

impl Emitter<'_> {
    pub(in crate::emit) fn schema_owned_prepared_validator(&self,id:usize,input:Type,generated:&mut String,seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->String {
        let name=format!("owned{id}_{input:?}").chars().map(|c|if c.is_ascii_alphanumeric(){c}else{'_'}).collect::<String>();
        let marker=format!("/*schema_owned_prepared_{name}*/");
        if generated.contains(&marker){return name;}
        generated.push_str(&marker);
        seen.insert((id,input));
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let record=self.native_record_declaration(id);
        let mut variants=Vec::new();let mut prepare=String::new();let mut commits=Vec::new();
        if let Type::Union(union)=input {
            let mut arms=Vec::new();
            for (tag,member) in self.program.unions[union.index()].members.iter().enumerate(){
                let child=self.schema_owned_prepared_validator(id,*member,generated,seen,errors);
                variants.push(format!("I{tag}(Box<Prepared{child}>)"));
                arms.push(format!("Union{}::V{tag}(value)=>prepare_{child}(schema,value,path,issues,failure,policy).map(|state|Prepared{name}::I{tag}(Box::new(state)))",union.index()));
                commits.push(format!("Prepared{name}::I{tag}(state)=>{{let Union{}::V{tag}(value)=input else{{unreachable!(\"prepared input alternative\")}};commit_{child}(value,*state)}}",union.index()));
            }
            prepare.push_str(&format!("match input{{{}}}",arms.join(",")));
        }else{
            if matches!(output,Type::String|Type::Int|Type::Number){
                let value=match (output,input.scalar_base()){
                    (Type::String,Type::String)=>Some(String::from("AsRef::<str>::as_ref(input)")),
                    (Type::Int,Type::Int)|(Type::Number,Type::Number)=>Some(String::from("*input")),
                    (Type::Number,Type::Int)=>Some(String::from("*input as f64")),
                    _=>None,
                };
                if let Some(value)=value{prepare.push_str(&format!("if let Some(code)=schema.bounds.check({value}){{issues.push(path,tsrust_schema_numeric_code(code),Kind::{},Kind::{});return None;}}",self.schema_kind(output),self.schema_kind(input)));}
            }
            let mut choices=String::new();
            for (plan,choice) in self.program.schema_choices.for_output(output){
                let bindings=(0..choice.branches.len()).map(|branch|format!("field{branch}")).collect::<Vec<_>>().join(",");
                let mut body=String::from("let checkpoint=issues.checkpoint();");
                if choice.presence==Some(input){
                    variants.push(format!("P{plan}Absent"));
                    commits.push(format!("Prepared{name}::P{plan}Absent=>{}",self.schema_output_injection(input,output,"input")));
                    body.push_str(&format!("return Some(Prepared{name}::P{plan}Absent);"));
                }
                for (branch,child_output) in choice.branches.iter().enumerate(){
                    let child=self.schema_owned_prepared_validator(self.schema_child(*child_output),input,generated,seen,errors);
                    variants.push(format!("P{plan}B{branch}(Box<Prepared{child}>)"));
                    let commit=self.schema_output_injection(*child_output,output,&format!("commit_{child}(input,*state)"));
                    commits.push(format!("Prepared{name}::P{plan}B{branch}(state)=>{commit}"));
                    let attempt=format!("if let Some(state)=prepare_{child}(field{branch},input,path,issues,failure,policy){{issues.restore(checkpoint);return Some(Prepared{name}::P{plan}B{branch}(Box::new(state)));}}if failure.is_some(){{issues.restore(checkpoint);return None;}}");
                    if let Some(selector)=&choice.selector{
                        let condition=self.schema_owned_discriminator_test(input,"input",selector,branch);
                        body.push_str(&format!("if {condition}{{{attempt}issues.reject();return None;}}"));
                    }else{body.push_str(&attempt);}
                }
                if let Some(selector)=&choice.selector{body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,Code::Discriminator,Kind::Union,Kind::Union);path.pop();",selector.field));}
                body.push_str("issues.reject();return None;");
                choices.push_str(&format!("SchemaChoice{record}::P{plan}{{{bindings}}}=>{{{body}}},"));
            }
            if !choices.is_empty(){
                if matches!(output,Type::Record(object) if !self.program.records[object].name.text.starts_with("@schema:retained:")){prepare.push_str("let policy=policy.or(schema.choice_policy);");}
                prepare.push_str(&format!("if let Some(choice)=&schema.choice{{match choice.as_ref(){{{choices}}}}}"));
            }
            let (state,body,commit)=self.schema_owned_base_preparation(id,input,generated,seen,errors);
            variants.push(format!("Base({state})"));
            prepare.push_str(&format!("(||{{{body}}})().map(Prepared{name}::Base)"));
            commits.push(format!("Prepared{name}::Base(state)=>{{{commit}}}"));
        }
        generated.push_str(&format!("enum Prepared{name}{{{variants}}}fn prepare_{name}(schema:&Record{record},input:&{input_native},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>,policy:Option<bool>)->Option<Prepared{name}>{{if failure.is_some(){{return None;}}use tsrust_schema_issues::{{Code,Kind}};{prepare}}}fn commit_{name}(input:{input_native},prepared:Prepared{name})->{native}{{match prepared{{{commits}}}}}fn {name}(schema:&Record{record},input:{input_native},path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues,failure:&mut Option<SchemaFactoryFailure>)->Option<{native}>{{let prepared=prepare_{name}(schema,&input,path,issues,failure,None)?;Some(commit_{name}(input,prepared))}}",variants=variants.join(","),input_native=self.native_type(input),native=self.native_type(output),commits=commits.join(",")));
        name
    }

    fn schema_owned_base_preparation(&self,id:usize,input:Type,generated:&mut String,seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->(String,String,String){
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        if let Some(leaf)=self.schema_owned_leaf_preparation(output,input){return leaf;}
        let incompatible=||("std::convert::Infallible".into(),format!("issues.push(path,Code::Type,Kind::{},Kind::{});None",self.schema_kind(output),self.schema_kind(input)),"match state{}".into());
        match output{
            Type::Union(union) if self.schema_enum_members(output).is_some()=>{
                if input.scalar_base()!=Type::String{return incompatible();}
                let mut prepare=String::new();let mut commits=Vec::new();
                for (tag,_) in self.program.unions[union.index()].members.iter().enumerate(){
                    prepare.push_str(&format!("if input==&schema.field{tag}.expected{{return Some({tag}usize);}}"));
                    commits.push(format!("{tag}=>Union{}::V{tag}(input)",union.index()));
                }
                prepare.push_str("issues.push(path,Code::Literal,Kind::String,Kind::String);None");
                ("usize".into(),prepare,format!("match state{{{},_=>unreachable!(\"prepared literal alternative\")}}",commits.join(",")))
            },
            Type::Union(union)=>{
                if matches!(input,Type::Null|Type::Undefined){
                    if let Some(tag)=self.program.unions[union.index()].members.iter().position(|member|*member==input){return ("()".into(),"Some(())".into(),format!("Union{}::V{tag}(input)",union.index()));}
                    return incompatible();
                }
                let Some(base)=self.schema_presence_element(output) else{return incompatible();};
                let child=self.schema_owned_prepared_validator(self.schema_child(base),input,generated,seen,errors);
                (format!("Prepared{child}"),format!("prepare_{child}(&schema.inner,input,path,issues,failure,policy)"),self.schema_output_injection(base,output,&format!("commit_{child}(input,state)")))
            },
            Type::Array(element)=>{
                let Type::Array(input_element)=input else{return incompatible();};
                let child=self.schema_owned_prepared_validator(self.schema_child(element.ty()),input_element.ty(),generated,seen,errors);
                let prepare=format!("let mut states=Some(Vec::new());for(index,value)in input.iter().enumerate(){{path.push(tsrust_schema_issues::Segment::Index(index));let state=prepare_{child}(&schema.element,value,path,issues,failure,None);path.pop();match(state,&mut states){{(Some(state),Some(states))=>states.push(state),(None,_)=>states=None,_=>{{}}}}if failure.is_some(){{return None;}}}}states");
                let commit=format!("let mut output=<{}>::from([]);for(value,state)in input.tsr_owned_items().zip(state){{output.push(commit_{child}(value,state));}}output",self.native_type(output));
                (format!("Vec<Prepared{child}>"),prepare,commit)
            },
            Type::Record(object) if self.program.records[object].name.text.starts_with("@schema:retained:")=>{
                let fields=&self.program.records[object].fields;let value=fields[0].ty.unwrap();
                let child_input=if input==output{value}else{input};
                let child=self.schema_owned_prepared_validator(self.schema_child(value),child_input,generated,seen,errors);
                let borrow=if input==output{"&input.field0"}else{"input"};let take=if input==output{"input.field0"}else{"input"};
                let extra=if input==output{String::from("input.field1")}else{format!("<{}>::new()",self.native_type(fields[1].ty.unwrap()))};
                (format!("Prepared{child}"),format!("prepare_{child}(&schema.inner,{borrow},path,issues,failure,Some(false))"),self.schema_output_record(object,&format!("field0:commit_{child}({take},state),field1:{extra}")))
            },
            Type::Record(object)=>{
                let Type::Record(input_object)=input else{return incompatible();};
                if self.program.records[input_object].native_type.is_some(){return incompatible();}
                self.schema_owned_object_preparation(id,object,input_object,generated,seen,errors)
            },
            _=>incompatible(),
        }
    }

    fn schema_owned_object_preparation(&self,id:usize,object:usize,input:usize,generated:&mut String,seen:&mut HashSet<(usize,Type)>,errors:&crate::error_effects::FailureSet)->(String,String,String){
        let mut types=Vec::new();let mut prepare=String::from("let mut valid=true;");let mut commit=String::new();let mut fields=Vec::new();
        let input_fields=&self.program.records[input].fields;
        for (index,field) in self.program.records[object].fields.iter().enumerate(){
            let output=field.ty.unwrap();let provided=input_fields.iter().enumerate().find(|(_,candidate)|candidate.name.text==field.name.text);
            let (state,validation,take)=if let Some((position,provided))=provided{
                let child=self.schema_owned_prepared_validator(self.schema_child(output),provided.ty.unwrap(),generated,seen,errors);
                (format!("Prepared{child}"),format!("prepare_{child}(&schema.field{index},&input.field{position},path,issues,failure,None).map(PreparedField::Present)"),format!("commit_{child}(input.field{position},state)"))
            }else{
                let fallback=self.schema_default_value(output,&format!("schema.field{index}"),generated,seen,errors);
                let missing=if let Type::Union(union)=output && let Some(tag)=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined){format!("Some(PreparedField::Default(Union{}::V{tag}(TsrUndefined)))",union.index())}else{format!("issues.push(path,Code::Missing,Kind::{},Kind::Missing);None",self.schema_kind(output))};
                (String::from("std::convert::Infallible"),format!("if failure.is_some(){{None}}else{{match {fallback}{{Some(value)=>value.map(PreparedField::Default),None=>{{{missing}}}}}}}"),String::from("match state{}"))
            };
            types.push(format!("PreparedField<{state},{}>",self.native_type(output)));
            prepare.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({:?}));let mut field{index}={validation};path.pop();if field{index}.is_none(){{valid=false;}}if !valid{{{}}}",field.name.text,(0..=index).map(|index|format!("field{index}.take();")).collect::<String>()));
            commit.push_str(&format!("let field{index}=match state.{index}{{PreparedField::Present(state)=>{take},PreparedField::Default(value)=>value}};"));
            fields.push(format!("field{index}"));
        }
        for field in input_fields.iter().filter(|field|!self.program.records[object].fields.iter().any(|expected|expected.name.text==field.name.text)){
            prepare.push_str(&format!("if policy.unwrap_or(schema.reject_unknown){{valid=false;path.push(tsrust_schema_issues::Segment::Field({:?}));issues.push(path,Code::UnknownField,Kind::Missing,Kind::{});path.pop();}}",field.name.text,self.schema_kind(field.ty.unwrap())));
        }
        prepare.push_str(&format!("if valid{{Some(({}))}}else{{None}}",fields.iter().map(|field|format!("{field}.unwrap(),")).collect::<String>()));
        commit.push_str(&self.schema_output_record(object,&fields.join(",")));
        let _=id;
        (format!("({})",types.iter().map(|ty|format!("{ty},")).collect::<String>()),prepare,commit)
    }
}
