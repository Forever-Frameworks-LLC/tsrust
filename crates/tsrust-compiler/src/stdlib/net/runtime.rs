mod tsrust_native_net {
 pub type IpAddress=std::net::IpAddr;
 pub type SocketAddress=std::net::SocketAddr;
 #[derive(Debug)]pub struct Error{pub code:&'static str,pub operation:&'static str,pub message:std::string::String,pub os_code:Option<i64>}
 pub fn error(code:&'static str,operation:&'static str,message:impl ToString)->Error{Error{code,operation,message:message.to_string(),os_code:None}}
 pub fn io_error(operation:&'static str,value:std::io::Error)->Error{use std::io::ErrorKind;let code=match value.kind(){ErrorKind::ConnectionRefused=>"connection-refused",ErrorKind::ConnectionReset=>"connection-reset",ErrorKind::ConnectionAborted=>"connection-aborted",ErrorKind::AddrInUse=>"address-in-use",ErrorKind::AddrNotAvailable=>"address-not-available",ErrorKind::PermissionDenied=>"permission-denied",ErrorKind::InvalidInput=>"invalid-input",ErrorKind::BrokenPipe=>"broken-pipe",ErrorKind::TimedOut=>"timeout",_=>"io"};Error{code,operation,message:value.to_string(),os_code:value.raw_os_error().map(i64::from)}}
 pub fn parse_ip(text:&str)->Result<IpAddress,Error>{text.parse().map_err(|value|error("syntax","parseIp",value))}
 pub fn format_ip(value:&IpAddress)->std::string::String{value.to_string()}
 pub fn parse_address(text:&str)->Result<SocketAddress,Error>{text.parse().map_err(|value|error("syntax","parseAddress",value))}
 pub fn format_address(value:&SocketAddress)->std::string::String{value.to_string()}
 pub fn is_ipv4(value:&IpAddress)->bool{value.is_ipv4()}
 pub fn is_ipv6(value:&IpAddress)->bool{value.is_ipv6()}
}
