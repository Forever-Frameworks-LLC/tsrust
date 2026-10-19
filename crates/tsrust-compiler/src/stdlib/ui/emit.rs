use crate::{
    analysis::CheckedProgram,
    native_bindings::{NativeBackend, NativeBinding, NativeType},
    native_signatures::NativeTypeId,
    syntax::Type,
};
pub(crate) fn bridge(program: &CheckedProgram, binding: &NativeBinding) -> String {
    let node = program
        .records
        .iter()
        .position(|r| r.native_type == Some(NativeTypeId::UiNode))
        .unwrap();
    let NativeBackend::Bundled { symbol } = binding.backend;
    let record=|kind|program.records.iter().position(|r|r.native_type==Some(kind)).unwrap();
    match binding.name {
        "signal"=>{let state=record(NativeTypeId::UiSignal);return format!("fn {symbol}(initial:i64)->Record{state}{{tsrust_native_ui::Signal::new(initial)}}\n");},
        "handle"=>{let state=record(NativeTypeId::UiSignal);let handle=record(NativeTypeId::UiSignal);return format!("fn {symbol}(state:&Record{state})->Record{handle}{{state.handle()}}\n");},
        _=>{}
    }
    let (parameters,call)=match binding.name {
        "get" => (format!("state:&Record{}",record(NativeTypeId::UiSignal)),"state.get()".into()),
        "set" => (format!("state:&Record{},value:i64",record(NativeTypeId::UiSignal)),"state.set(value)".into()),
        "view"|"staticButton"|"button" => {
            let array=crate::native_binding_types::resolve(&program.records,&super::CHILDREN).unwrap();
            let Type::Array(element)=array else{unreachable!()};let Type::Union(union)=element.ty() else{unreachable!()};
            let members=&program.unions[union.index()].members;
            let literal=members.iter().position(|ty|*ty==Type::String).unwrap();
            let handle=members.iter().position(|ty|*ty==Type::Record(record(NativeTypeId::UiSignal))).unwrap();
            let child=members.iter().position(|ty|*ty==Type::Record(node)).unwrap();
            let mut parameters=format!("mut children:{},class_name:&String",crate::emit::ty(array));
            let factory=match binding.name {
                "view"=>"tsrust_native_ui::Node::view(parts,class_name.as_ref().to_owned())",
                "staticButton"=>{parameters.push_str(",disabled:bool");"tsrust_native_ui::Node::button_children(parts,class_name.as_ref().to_owned(),disabled,None)"},
                "button"=>{parameters.push_str(",disabled:bool,handler:impl FnMut()->Result<(),std::string::String>+'static");"tsrust_native_ui::Node::button_children(parts,class_name.as_ref().to_owned(),disabled,Some(Box::new(handler)))"},
                _=>unreachable!(),
            };
            (parameters,format!("{{let mut parts=Vec::with_capacity(children.len());while let Some(child)=children.pop(){{parts.push(match child{{Union{union}::V{literal}(text)=>tsrust_native_ui::Child::Text(tsrust_native_ui::TextPart::Literal(text.as_ref().to_owned())),Union{union}::V{handle}(handle)=>tsrust_native_ui::Child::Text(tsrust_native_ui::TextPart::Signal(handle)),Union{union}::V{child}(node)=>tsrust_native_ui::Child::Node(node)}});}}parts.reverse();{factory}}}",union=union.index()))
        },
        "text" => {
            let array=crate::native_binding_types::resolve(&program.records,&super::TEXT_CHILDREN).unwrap();
            let Type::Array(element)=array else{unreachable!()};let Type::Union(union)=element.ty() else{unreachable!()};
            let members=&program.unions[union.index()].members;
            let literal=members.iter().position(|ty|*ty==Type::String).unwrap();let handle=members.iter().position(|ty|*ty==Type::Record(record(NativeTypeId::UiSignal))).unwrap();
            (format!("mut children:{},class_name:&String",crate::emit::ty(array)),format!("{{let mut parts=Vec::with_capacity(children.len());while let Some(child)=children.pop(){{parts.push(match child{{Union{}::V{literal}(text)=>tsrust_native_ui::TextPart::Literal(text.as_ref().to_owned()),Union{}::V{handle}(handle)=>tsrust_native_ui::TextPart::Signal(handle)}});}}parts.reverse();tsrust_native_ui::Node::text(parts,class_name.as_ref().to_owned())}}",union.index(),union.index()))
        },
        "bindDisabled" => (format!("node:Record{node},predicate:impl FnMut()->Result<bool,std::string::String>+'static"),"node.with_disabled(predicate)".into()),
        "theme" => (format!("node:Record{node},config:&String,mode:&String"),"node.with_theme(config.as_ref(),mode.as_ref())".into()),
        "semantics" => (format!("node:Record{node},config:&String"),"node.with_semantics(config.as_ref())".into()),
        "ownState" => (format!("node:Record{node},state:Record{}",record(NativeTypeId::UiSignal)),"node.own_state(state)".into()),
        "document" => return format!("fn {symbol}(node:&Record{node},title:&String)->String{{String::Owned(tsrust_native_ui::document(node,title.as_ref()))}}\n"),
        "run" => (format!("node:Record{node},title:&String"),"tsrust_native_ui::run(node,title.as_ref().to_owned())".into()),
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
