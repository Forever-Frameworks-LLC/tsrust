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
    let (parameters,call)=match binding.name {
        "text"|"disabledButton" => {
            let kind=if binding.name=="text"{"Text"}else{"DisabledButton"};
            ("text:&String,class_name:&String".into(),format!("tsrust_native_ui::Node::new(tsrust_native_ui::Kind::{kind},class_name.as_ref().to_owned(),text.as_ref().to_owned(),Vec::new())"))
        }
        "view" => (format!("mut children:RecordArray{node},class_name:&String"),"{let mut nodes=Vec::with_capacity(children.len());while let Some(child)=children.pop(){nodes.push(child);}nodes.reverse();tsrust_native_ui::Node::new(tsrust_native_ui::Kind::View,class_name.as_ref().to_owned(),std::string::String::new(),nodes)}".into()),
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
