//! CPU-heavy codecs use the existing explicit bounded pool and lazy Result tasks.
pub(crate) mod bindings;
mod wrappers;
pub(crate) const STREAM_RUNTIME:&str=include_str!("runtime_streams.rs");
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
pub(super) fn lower(expression:&mut crate::syntax::Expr,name:&str,mut arguments:Vec<crate::syntax::Expr>){use crate::syntax::{Expr,ExprKind};let span=expression.span;if arguments.is_empty(){return;}let pool=arguments.remove(0);let capability=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@native:std:threads#__poolSubmission".into()),span)),vec![pool]),span);arguments.insert(0,capability);if constructor(name){expression.kind=ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:std:compression#__{name}")),span)),arguments);}else{crate::stdlib::futures::lazy_result(expression,"std:compression",&format!("__{name}"),arguments);}}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{use crate::syntax::ExprKind;let mut values=std::collections::BTreeSet::new();let mut direct=std::collections::HashSet::new();let mut used=false;let mut stream=program.records.iter().any(|record|matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::CompressionReader|crate::native_signatures::NativeTypeId::CompressionWriter)));
 for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Call(callee,args)=&mut expression.kind&&let ExprKind::Name(symbol)=&callee.kind&&let Some(name)=symbol.strip_prefix("@native:std:compression#"){used=true;stream|=constructor(name.trim_start_matches('_'));if !name.starts_with('_'){let name=name.to_owned();let args=std::mem::take(args);lower(expression,&name,args);}else{direct.insert(callee.span);}}else if let ExprKind::Name(symbol)=&expression.kind&&let Some(name)=symbol.strip_prefix("@native:std:compression#")&&!name.starts_with('_')&&!direct.contains(&expression.span){used=true;stream|=constructor(name);values.insert(name.to_owned());expression.kind=ExprKind::Name(wrappers::name(name));}Ok(())})?;}
 for name in values{let binding=crate::native_bindings::lookup("std:compression",&name).expect("resolved compression export");let adapter=wrappers::adapter(program,binding)?;program.functions.push(adapter);}
 if used{for binding in bindings::BUFFER_OPERATIONS{crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}}if stream{for binding in bindings::STREAM_OPERATIONS{crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}}Ok(())
}

pub(crate) fn constructor(name:&str)->bool{matches!(name,"compressReader"|"decompressReader"|"compressWriter"|"decompressWriter")}
pub(crate) fn method(identity:Option<crate::native_signatures::NativeTypeId>,name:&str)->Option<&'static crate::native_bindings::NativeBinding>{use crate::native_signatures::NativeTypeId;Some(match(identity,name){
(Some(NativeTypeId::CompressionReader),"read")=>&bindings::READER_READ,
(Some(NativeTypeId::CompressionReader),"readInto")=>&bindings::READER_READINTO,
(Some(NativeTypeId::CompressionReader),"readExact")=>&bindings::READER_READEXACT,
(Some(NativeTypeId::CompressionReader),"readAll")=>&bindings::READER_READALL,
(Some(NativeTypeId::CompressionReader),"finish")=>&bindings::READER_FINISH,
(Some(NativeTypeId::CompressionReader),"close")=>&bindings::READER_CLOSE,
(Some(NativeTypeId::CompressionWriter),"write")=>&bindings::WRITER_WRITE,
(Some(NativeTypeId::CompressionWriter),"writeAll")=>&bindings::WRITER_WRITEALL,
(Some(NativeTypeId::CompressionWriter),"flush")=>&bindings::WRITER_FLUSH,
(Some(NativeTypeId::CompressionWriter),"finish")=>&bindings::WRITER_FINISH,
(Some(NativeTypeId::CompressionWriter),"close")=>&bindings::WRITER_CLOSE,
_=>return None})}
pub(crate) fn io_operation(name:&str,writer:bool)->Option<&'static crate::native_bindings::NativeBinding>{Some(match(name,writer){
("read",false)=>&bindings::READER_READ,
("readInto",false)=>&bindings::READER_READINTO,
("readExact",false)=>&bindings::READER_READEXACT,
("readAll",false)=>&bindings::READER_READALL,
("close",false)=>&bindings::READER_CLOSE,
("write",true)=>&bindings::WRITER_WRITE,
("writeAll",true)=>&bindings::WRITER_WRITEALL,
("flush",true)=>&bindings::WRITER_FLUSH,
("close",true)=>&bindings::WRITER_CLOSE,
("copy",_)=>&bindings::COPY,_=>return None})}
