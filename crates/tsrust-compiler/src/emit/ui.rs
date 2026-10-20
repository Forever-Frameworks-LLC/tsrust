//! Scalar state is emitted per checked payload; public component code sees no handles.
use super::*;
const VALUE_BINDING:&str="Box<dyn FnMut()->Result<std::string::String,std::string::String>>";
const CHANGE_HANDLER:&str="Box<dyn FnMut(&str,&str,&str)->Result<(),std::string::String>>";
impl Emitter<'_> {
 pub(super) fn ui_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];
  if binding.module!="std:ui-runtime" || !matches!(binding.name,"visibility"|"signal"|"handle"|"get"|"set"|"ownState"|"input"|"inputChange"|"bindText"|"branch"|"route"){return None;}
  self.native_bindings.borrow_mut().insert(id);
  let read=|arg:&Expr|self.expr(fun,arg,Mode::Read);
  let scalar=|arg:&Expr|{let text=read(arg);if arg.ty==Some(Type::String){format!("String::clone({text})")}else{text}};
  let call=match binding.name {
   "visibility"=>{
    let callback=args[1].ty.unwrap();let invoke=self.ui_callback_arguments(callback,expr.span,"()","(visible,)");
    format!("{{let node={};let mut registered_callback={};node.with_visibility(move |visible:bool|{{{invoke}}})}}",self.expr(fun,&args[0],Mode::Consume),self.expr(fun,&args[1],Mode::Consume))
   },
   "route"=>{
    let callback=args[2].ty.unwrap();let result=self.ui_getter_result(callback);let invoke=self.ui_callback(callback,expr.span,&self.native_type(result));let child=self.ui_branch_child(result,"value");
    format!("{{let path={};let exact={};let mut registered_callback={};tsrust_native_ui::Node::route(path.as_ref(),exact,move||({invoke}).and_then(|value|{child}))}}",read(&args[0]),read(&args[1]),self.expr(fun,&args[2],Mode::Consume))
   },
   "branch"=>{
    let adapters=args.iter().enumerate().map(|(index,arg)|{
     let callback=arg.ty.unwrap();let result=self.ui_getter_result(callback);let invoke=self.ui_callback(callback,expr.span,&self.native_type(result));
     let output=if index==0{invoke}else{format!("({invoke}).and_then(|value|{})",self.ui_branch_child(result,"value"))};
     format!("{{let mut registered_callback={};move||{{{output}}}}}",self.expr(fun,arg,Mode::Consume))
    }).collect::<Vec<_>>();
    format!("{{let predicate={};let present={};let absent={};tsrust_native_ui::Node::branch(predicate,present,absent)}}",adapters[0],adapters[1],adapters[2])
   },
   "bindText"=>{
    let callback=args[0].ty.unwrap();let result=self.ui_getter_result(callback);
    let invoke=self.ui_callback(callback,expr.span,&self.native_type(result));let format=self.ui_text_scalar(result,"value");
    return Some(format!("{{let mut registered_callback={};tsrust_native_ui::TextBinding::computed(move||{{let result={invoke};result.map(|value|{format})}})}}",self.expr(fun,&args[0],Mode::Consume)));
   },
   "input"|"inputChange"=>self.ui_input_constructor(fun,args,binding.name=="inputChange",expr.span),
   "signal"=>return Some(format!("tsrust_native_ui::Signal::new({})",scalar(&args[0]))),
   "handle"=>return Some(format!("({}).handle()",read(&args[0]))),
   "get"=>format!("({}).get()",read(&args[0])),
   "set"=>format!("{{let state={};let next={};state.set(next)}}",read(&args[0]),scalar(&args[1])),
   "ownState"=>format!("{{let node={};let state={};{}}}",self.expr(fun,&args[0],Mode::Consume),self.expr(fun,&args[1],Mode::Consume),self.ui_retained_result(args[0].ty.unwrap(),"node")),
   _=>unreachable!(),
  };
  let union=expr.ty.unwrap().union().expect("UI result");let members=&self.program.unions[union.index()].members;let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
  let wrap=|record:usize,payload:&str|{let tag=members.iter().position(|ty|*ty==Type::Record(record)).unwrap();let fields=if self.program.records[record].fields.is_empty(){String::new()}else{format!("field0:{payload},")};format!("Union{}::V{tag}(Record{}{{{fields}{}}})",union.index(),self.native_record_declaration(record),self.schema_record_order(record).trim_start_matches(','))};
  let Type::Record(error)=self.program.records[err].fields[0].ty.unwrap() else{unreachable!()};
  let failure=format!("Record{}{{field0:String::Borrowed(\"UiError\"),field1:String::Owned(message){}}}",self.native_record_declaration(error),self.schema_record_order(error));
  Some(format!("match {call}{{Ok(value)=>{},Err(message)=>{}}}",wrap(ok,"value"),wrap(err,&failure)))
 }
 fn ui_getter_result(&self,callback:Type)->Type {match callback{Type::Callback(id)=>self.program.callback_contracts[id].result,Type::StoredCallable(family)=>self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked UI getter")].result,Type::Closure(id)=>self.program.closures[id].function.result,_=>unreachable!("checked UI getter")}}
 fn ui_branch_child(&self,ty:Type,value:&str)->String {match ty {
  Type::Union(id)=>{let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,&ty)|format!("Union{}::V{tag}(value)=>{},",id.index(),self.ui_branch_child(ty,"value"))).collect::<String>();format!("match {value}{{{arms}}}")},
  Type::Null|Type::Undefined=>format!("{{let _={value};Ok(None)}}"),
  Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::UiNode)=>format!("Ok(Some({value}))"),
  _=>{
   let part=if let Type::Record(id)=ty {
    let record=&self.program.records[id];let binding=if record.native_type==Some(crate::native_signatures::NativeTypeId::UiTextBinding){value.to_owned()}else{
     let payload=record.application.as_ref().expect("checked signal child").resolved_arguments[0];let format=match payload{Type::String=>"|value|value.as_ref().to_owned()".to_owned(),Type::Number=>"|value|tsr_number_to_string(*value,10,\"UI text\").into_owned()".into(),Type::Int|Type::Boolean=>"|value|value.to_string()".into(),_=>unreachable!()};format!("({value}).text_binding({format})")
    };format!("tsrust_native_ui::TextPart::Signal({binding})")
   }else{format!("tsrust_native_ui::TextPart::Literal({})",self.ui_input_scalar(ty,value))};
   format!("tsrust_native_ui::Node::text_run(vec![{part}]).map(Some)")
  },
 }}
 fn ui_text_scalar(&self,ty:Type,expression:&str)->String {match ty {
  Type::Union(id)=>{let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,&ty)|format!("Union{}::V{tag}(value)=>{},",id.index(),self.ui_text_scalar(ty,"value"))).collect::<String>();format!("match {expression}{{{arms}}}")},
  Type::Boolean=>format!("({expression}).to_string()"),
  Type::Null|Type::Undefined=>format!("{{let _={expression};std::string::String::new()}}"),
  _=>self.ui_input_scalar(ty,expression),
 }}
 fn ui_retained_result(&self,ty:Type,node:&str)->String {match ty {
  Type::Record(_)=>format!("{node}.own_state(state)"),
  Type::Null|Type::Undefined=>format!("{{drop(state);Ok::<_,std::string::String>({node})}}"),
  Type::Union(id)=>{let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,&ty)|format!("Union{}::V{tag}(node)=>({}).map(Union{}::V{tag}),",id.index(),self.ui_retained_result(ty,"node"),id.index())).collect::<String>();format!("match {node}{{{arms}}}")},
  _=>unreachable!("checked nullable component result"),
 }}
 fn ui_input_constructor(&self,fun:&CheckedFunction,args:&[Expr],changed:bool,span:crate::Span)->String {
  let value=self.ui_input_binding(args[0].ty.unwrap(),&self.expr(fun,&args[0],Mode::Consume),span);
  let default=self.ui_input_default(args[1].ty.unwrap(),&self.expr(fun,&args[1],Mode::Consume));
  let handler=if changed {
   let callback_type=args[4].ty.unwrap();
   let declaration=match callback_type{Type::Callback(id)=>id,Type::StoredCallable(family)=>self.program.stored_contracts.declaration(family).expect("checked input callback"),_=>unreachable!()};
   let Type::Record(event)=self.program.callback_contracts[declaration].params[0].ty else{unreachable!()};
   let Some(Type::Record(target))=self.program.records[event].fields[0].ty else{unreachable!()};
   let target_value=format!("Record{}{{field0:String::Owned(value.to_owned()),field1:String::Owned(name.to_owned()),field2:String::Owned(id.to_owned()){} }}",self.native_record_declaration(target),self.schema_record_order(target));
   let event_value=format!("Record{}{{field0:{target_value},field1:{target_value}{} }}",self.native_record_declaration(event),self.schema_record_order(event));
   let callback=self.expr(fun,&args[4],Mode::Consume);let invoke=self.ui_callback_arguments(callback_type,span,"()","(&change_event,)");
   format!("{{let mut registered_callback={callback};Some(Box::new(move |value:&str,name:&str,id:&str|{{let change_event={event_value};{invoke}}}) as {CHANGE_HANDLER})}}")
  }else{"None".into()};
  format!("{{let value={value};let default_value={default};let options={};let class_name={};let changed={handler};tsrust_native_ui::Node::input_control(value,default_value,options.as_ref(),class_name.as_ref().to_owned(),changed)}}",self.expr(fun,&args[2],Mode::Read),self.expr(fun,&args[3],Mode::Read))
 }
 fn ui_input_default(&self,ty:Type,expression:&str)->String {
  if let Type::Union(id)=ty {
   let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,&ty)|format!("Union{}::V{tag}(value)=>{},",id.index(),self.ui_input_default(ty,"value"))).collect::<String>();
   format!("match {expression}{{{arms}}}")
  }else if ty==Type::Undefined {format!("{{let _={expression};None}}")}
  else {format!("Some({})",self.ui_input_scalar(ty,expression))}
 }
 fn ui_input_binding(&self,ty:Type,expression:&str,span:crate::Span)->String {
  match ty {
   Type::Union(id)=>{
    let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,&ty)|format!("Union{}::V{tag}(value)=>{},",id.index(),self.ui_input_binding(ty,"value",span))).collect::<String>();
    format!("match {expression}{{{arms}}}")
   },
   Type::Undefined=>format!("{{let _={expression};None}}"),
   Type::String|Type::Int|Type::Number=>format!("{{let value={};Some(Box::new(move||Ok(value.clone())) as {VALUE_BINDING})}}",self.ui_input_scalar(ty,expression)),
   Type::Record(id)=>{
    let record=&self.program.records[id];
    let payload=if record.native_type==Some(crate::native_signatures::NativeTypeId::UiSignal){record.application.as_ref().expect("checked state binding").resolved_arguments[0]}else{let id=match record.fields[0].ty.unwrap(){Type::Callback(id)=>id,Type::StoredCallable(family)=>self.program.stored_contracts.declaration(family).expect("computed input callback"),_=>unreachable!()};self.program.callback_contracts[id].result};
    let converted=self.ui_input_scalar(payload,"value");
    if record.native_type==Some(crate::native_signatures::NativeTypeId::UiSignal) {
     format!("{{let state={expression};Some(Box::new(move||state.get().map(|value|{converted})) as {VALUE_BINDING})}}")
    }else{
     let invoke=self.ui_callback(record.fields[0].ty.expect("computed input value"),span,&self.native_type(payload));
     format!("{{let mut registered_callback=({expression}).field0;Some(Box::new(move||{{let result={invoke};result.map(|value|{converted})}}) as {VALUE_BINDING})}}")
    }
   },
   _=>unreachable!("checked input value"),
  }
 }
 fn ui_input_scalar(&self,ty:Type,expression:&str)->String {match ty {
  Type::Union(id)=>{let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,&ty)|format!("Union{}::V{tag}(value)=>{},",id.index(),self.ui_input_scalar(ty,"value"))).collect::<String>();format!("match {expression}{{{arms}}}")},
  Type::String=>format!("({expression}).into_owned()"),
  Type::Int=>format!("({expression}).to_string()"),
  Type::Number=>format!("tsr_number_to_string({expression},10,\"Input value\").into_owned()"),
  _=>unreachable!("checked scalar input"),
 }}

}
