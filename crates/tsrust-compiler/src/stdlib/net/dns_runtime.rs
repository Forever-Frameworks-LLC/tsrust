mod tsrust_native_dns {
 use crate::tsrust_native_net::{IpAddress,Error,error,io_error};
 #[derive(Clone,Copy,Default)]pub enum Family{#[default]Any,Ipv4,Ipv6}
 pub async fn lookup(host:&str,family:Family)->Result<Vec<IpAddress>,Error>{
  tokio::runtime::Handle::try_current().map_err(|_|error("runtime","lookup","An active runtime is required"))?;
  let addresses=tokio::net::lookup_host((host,0)).await.map_err(|value|io_error("lookup",value))?;
  Ok(addresses.map(|address|address.ip()).filter(|address|match family{Family::Any=>true,Family::Ipv4=>address.is_ipv4(),Family::Ipv6=>address.is_ipv6()}).collect())
 }
}
