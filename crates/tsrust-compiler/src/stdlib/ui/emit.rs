use crate::{
    analysis::CheckedProgram,
    native_bindings::{NativeBackend, NativeBinding, NativeType},
    native_signatures::NativeTypeId,
    syntax::Type,
};
pub(crate) fn bridge(program: &CheckedProgram, binding: &NativeBinding) -> String {
    if matches!(binding.name,"computed"|"computedGet"|"bindSemantics"|"bindClass"|"bindVariant"|"bindPath"|"visibility"|"signal"|"handle"|"get"|"set"|"ownState"|"input"|"inputChange"|"bindText"|"branch"|"route"){return String::new();}
    let node = program
        .records
        .iter()
        .position(|r| r.native_type == Some(NativeTypeId::UiNode))
        .unwrap();
    let NativeBackend::Bundled { symbol } = binding.backend;
    let (parameters,call)=match binding.name {
        "svg"|"view"|"staticButton"|"button"|"link"|"text" => {
            let array=crate::native_binding_types::resolve(&program.records,&super::CHILDREN).unwrap();
            let Type::Array(element)=array else{unreachable!()};let Type::Union(union)=element.ty() else{unreachable!()};
            let members=&program.unions[union.index()].members;
            let literal=members.iter().position(|ty|*ty==Type::String).unwrap();
            let bindings=signal_arms(program,members,union.index(),true);
            let absent=absent_arms(members,union.index());
            let scalars=scalar_arms(program,members,union.index(),true);
            let child=members.iter().position(|ty|*ty==Type::Record(node)).unwrap();
            let mut parameters=format!("mut children:{},class_name:&String",crate::emit::ty(array));
            let factory=match binding.name {
                "text"=>"tsrust_native_ui::Node::text_children(parts,class_name.as_ref().to_owned())",
                "link"=>{parameters.push_str(",href:&String");"tsrust_native_ui::navigation::link(parts,class_name.as_ref().to_owned(),href.as_ref())"},
                "svg"=>{parameters.push_str(",options:&String");"tsrust_native_ui::Node::svg(parts,class_name.as_ref().to_owned(),options.as_ref())"},
                "view"=>"tsrust_native_ui::Node::view(parts,class_name.as_ref().to_owned())",
                "staticButton"=>{parameters.push_str(",disabled:bool");"tsrust_native_ui::Node::button_children(parts,class_name.as_ref().to_owned(),disabled,None)"},
                "button"=>{parameters.push_str(",disabled:bool,handler:impl FnMut()->Result<(),std::string::String>+'static");"tsrust_native_ui::Node::button_children(parts,class_name.as_ref().to_owned(),disabled,Some(Box::new(handler)))"},
                _=>unreachable!(),
            };
            (parameters,format!("{{let mut parts=Vec::with_capacity(children.len());while let Some(child)=children.pop(){{parts.push(match child{{{absent}{scalars}Union{union}::V{literal}(text)=>tsrust_native_ui::Child::Text(tsrust_native_ui::TextPart::Literal(text.as_ref().to_owned())),{bindings}Union{union}::V{child}(node)=>tsrust_native_ui::Child::Node(node)}});}}parts.reverse();{factory}}}",union=union.index()))
        },
        "path"=>("data:&String,class_name:&String,options:&String".into(),"tsrust_native_ui::Node::svg_path(data.as_ref(),class_name.as_ref().to_owned(),options.as_ref())".into()),
        "validateRoot"=>(format!("node:&Record{node}"),"node.validate_root()".into()),
        "copyButton" => ("value:&String,class_name:&String".into(),"tsrust_native_ui::clipboard::button(value.as_ref().to_owned(),class_name.as_ref().to_owned())".into()),
        "location" => (String::new(),"tsrust_native_ui::navigation::location(|value|String::Owned(value),|value:&String|value.as_ref().to_owned())".into()),
        "navigate" => ("path:&String".into(),"tsrust_native_ui::navigation::navigate(path.as_ref())".into()),
        "traverse" => ("forward:bool".into(),"tsrust_native_ui::navigation::traverse(forward)".into()),
        "bindHidden" => (format!("node:Record{node},predicate:impl FnMut()->Result<bool,std::string::String>+'static"),"node.with_hidden(predicate)".into()),
        "bindReadOnly" => (format!("node:Record{node},predicate:impl FnMut()->Result<bool,std::string::String>+'static"),"node.with_read_only(predicate)".into()),
        "bindDisabled" => (format!("node:Record{node},predicate:impl FnMut()->Result<bool,std::string::String>+'static"),"node.with_disabled(predicate)".into()),
        "theme" => (format!("node:Record{node},config:&String,mode:&String"),"node.with_theme(config.as_ref(),mode.as_ref())".into()),
        "themeMode" => {
            let mode=crate::native_binding_types::resolve(&program.records,&super::SIGNAL_STRING).unwrap();
            (format!("node:Record{node},config:&String,mode:&{}",crate::emit::ty(mode)),"node.with_theme_binding(config.as_ref(),mode.text_binding(|value|value.as_ref().to_owned()))".into())
        },
        "semantics" => (format!("node:Record{node},config:&String"),"node.with_semantics(config.as_ref())".into()),
        "document" => return format!("fn {symbol}(node:&Record{node},title:&String)->String{{String::Owned(tsrust_native_ui::document(node,title.as_ref()))}}\n"),
        "run" => (format!("node:Record{node},title:&String,options:&String"),"tsrust_native_ui::run_configured(node,title.as_ref().to_owned(),options.as_ref())".into()),
        _=>unreachable!(),
    };
    let NativeType::Structured(signature) = binding.result else {
        unreachable!()
    };
    let Type::Union(result) =
        crate::native_binding_types::resolve(&program.records, signature).unwrap()
    else {
        unreachable!()
    };
    let members = &program.unions[result.index()].members;
    let [ok, err] = crate::result_identity::variants(&program.records, members).unwrap();
    let tag = |id| {
        members
            .iter()
            .position(|ty| *ty == Type::Record(id))
            .unwrap()
    };
    let error = program
        .records
        .iter()
        .position(|r| r.native_type == Some(NativeTypeId::UiError))
        .unwrap();
    let success = if program.records[ok].fields.is_empty() {
        format!("Record{ok}{{}}")
    } else {
        format!("Record{ok}{{field0:value}}")
    };
    format!(
        "fn {symbol}({parameters})->Union{result}{{match {call}{{Ok(value)=>Union{result}::V{ok_tag}({success}),Err(message)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(\"UiError\"),field1:String::Owned(message)}}}})}}}}\n",
        result = result.index(),
        ok_tag = tag(ok),
        err_tag = tag(err)
    )
}

fn signal_arms(program:&CheckedProgram,members:&[Type],union:usize,child:bool)->String {
 members.iter().enumerate().filter_map(|(tag,ty)|{
  let Type::Record(record)=*ty else{return None;};let record=&program.records[record];if record.native_type!=Some(NativeTypeId::UiSignal){return None;}
  let format=match record.application.as_ref().unwrap().resolved_arguments[0]{Type::Int=>"|value|value.to_string()",Type::Boolean=>"|value|value.to_string()",Type::Number=>"|value|tsr_number_to_string(*value,10,\"UI text\").as_ref().to_owned()",Type::String=>"|value|value.as_ref().to_owned()",_=>unreachable!("scalar UI state")};
  let part=format!("tsrust_native_ui::TextPart::Signal(handle.text_binding({format}))");
  let part=if child{format!("tsrust_native_ui::Child::Text({part})")}else{part};
  Some(format!("Union{union}::V{tag}(handle)=>{part},"))
 }).collect()
}

fn absent_arms(members:&[Type],union:usize)->String {
 members.iter().enumerate().filter(|(_,ty)|matches!(ty,Type::Null|Type::Undefined)).map(|(tag,_)|format!("Union{union}::V{tag}(_)=>continue,")).collect()
}

fn scalar_arms(program:&CheckedProgram,members:&[Type],union:usize,child:bool)->String {
 members.iter().enumerate().filter_map(|(tag,ty)|{
  let part=match *ty {
   Type::Int=>"tsrust_native_ui::TextPart::Literal(value.to_string())".to_owned(),
   Type::Number=>"tsrust_native_ui::TextPart::Literal(tsr_number_to_string(value,10,\"UI text\").into_owned())".to_owned(),
   Type::Record(id) if program.records[id].native_type==Some(NativeTypeId::UiTextBinding)=>"tsrust_native_ui::TextPart::Signal(value)".to_owned(),
   _=>return None,
  };let part=if child{format!("tsrust_native_ui::Child::Text({part})")}else{part};Some(format!("Union{union}::V{tag}(value)=>{part},"))
 }).collect()
}
