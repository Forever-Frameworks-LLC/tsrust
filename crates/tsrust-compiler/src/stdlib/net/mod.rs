//! Address values, system lookup and owned Tokio endpoints.
pub(crate) mod bindings;
mod wrappers;
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
pub(crate) const DNS_RUNTIME:&str=include_str!("dns_runtime.rs");
pub(crate) const TRANSPORT_RUNTIME:&str=include_str!("transport_runtime.rs");
pub(crate) const IO_RUNTIME:&str=include_str!("io_runtime.rs");
pub(crate) fn asynchronous(name:&str)->bool{matches!(name,"lookup"|"listenTcp"|"connectTcp"|"bindUdp"|"accept"|"sendTo"|"receiveFrom"|"shutdownWrite")}
pub(crate) fn method(identity:Option<crate::native_signatures::NativeTypeId>,name:&str)->Option<&'static crate::native_bindings::NativeBinding>{use crate::native_signatures::NativeTypeId::*;use bindings::*;Some(match(identity?,name){(TcpListener,"accept")=>&ACCEPT,(TcpListener,"localAddress")=>&LISTENER_ADDRESS,(TcpListener,"close")=>&LISTENER_CLOSE,(TcpStream,"localAddress")=>&STREAM_LOCAL,(TcpStream,"peerAddress")=>&STREAM_PEER,(TcpStream,"setNoDelay")=>&NO_DELAY,(TcpStream,"shutdownWrite")=>&SHUTDOWN,(TcpStream,"split")=>&SPLIT,(TcpStream,"close")=>&STREAM_CLOSE,(UdpSocket,"sendTo")=>&SEND_TO,(UdpSocket,"receiveFrom")=>&RECEIVE_FROM,(UdpSocket,"localAddress")=>&UDP_ADDRESS,(UdpSocket,"close")=>&UDP_CLOSE,_=>return None})}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
 use crate::syntax::ExprKind;
 let mut values=std::collections::BTreeSet::new();let mut direct=std::collections::HashSet::new();let mut dns=false;let mut transport=program.records.iter().any(|record|matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::TcpListener|crate::native_signatures::NativeTypeId::TcpStream|crate::native_signatures::NativeTypeId::UdpSocket|crate::native_signatures::NativeTypeId::Datagram)));
 for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{
  if let ExprKind::Call(callee,args)=&mut expression.kind && let ExprKind::Name(symbol)=&callee.kind && let Some((module,name))=symbol.strip_prefix("@native:std:dns#").map(|name|("std:dns",name)).or_else(||symbol.strip_prefix("@native:std:net#").map(|name|("std:net",name))){
   if module=="std:dns"{dns=true;}else if asynchronous(name){transport=true;}
   if asynchronous(name){let operation=format!("__{name}");let inputs=std::mem::take(args);crate::stdlib::futures::lazy_result(expression,module,&operation,inputs);}
   else{direct.insert(callee.span);}
  }else if let ExprKind::Name(symbol)=&expression.kind && let Some((module,name))=symbol.strip_prefix("@native:std:dns#").map(|name|("std:dns",name)).or_else(||symbol.strip_prefix("@native:std:net#").map(|name|("std:net",name))) && asynchronous(name) && !name.starts_with('_') && !direct.contains(&expression.span){values.insert((module,name.to_owned()));expression.kind=ExprKind::Name(wrappers::name(name));if module=="std:dns"{dns=true;}else{transport=true;}}Ok(())})?;}
 for(module,name)in values{let binding=crate::native_bindings::lookup(module,&name).expect("resolved network export");let wrapper=wrappers::adapter(program,binding)?;program.functions.push(wrapper);}
 if dns{for binding in bindings::OPERATIONS{crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}}
 if transport{for binding in bindings::TRANSPORT{crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}}
 Ok(())
}
