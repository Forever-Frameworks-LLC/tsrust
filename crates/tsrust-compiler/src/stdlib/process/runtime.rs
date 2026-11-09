mod tsrust_native_process {
 use std::{io,process::Stdio,sync::atomic::{AtomicUsize,Ordering},time::Duration};
 use tokio::{io::{AsyncRead,AsyncReadExt,AsyncWriteExt},process::{Child as BackendChild,ChildStdin,Command}};
 #[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum StdioMode{Inherit,Null,Pipe}
 pub enum Input{Bytes(Vec<u8>),Reader(crate::tsrust_native_io::Reader)}
 pub enum InputRef<'a>{Bytes(&'a[u8]),Reader(&'a crate::tsrust_native_io::Reader)}
 #[derive(Default)]pub struct Options{pub cwd:Option<std::string::String>,pub env:Vec<(std::string::String,std::string::String)>,pub clear_env:bool,pub stdin:Option<StdioMode>,pub stdout:Option<StdioMode>,pub stderr:Option<StdioMode>,pub input:Option<Input>,pub deadline_ms:Option<i64>,pub max_output_bytes:Option<i64>}
 #[derive(Default)]pub struct OptionsRef<'a>{pub cwd:Option<&'a str>,pub env:Vec<(&'a str,&'a str)>,pub clear_env:bool,pub stdin:Option<StdioMode>,pub stdout:Option<StdioMode>,pub stderr:Option<StdioMode>,pub input:Option<InputRef<'a>>,pub deadline_ms:Option<i64>,pub max_output_bytes:Option<i64>}
 impl Options{fn view(&self)->OptionsRef<'_>{OptionsRef{cwd:self.cwd.as_deref(),env:self.env.iter().map(|(name,value)|(name.as_str(),value.as_str())).collect(),clear_env:self.clear_env,stdin:self.stdin,stdout:self.stdout,stderr:self.stderr,input:self.input.as_ref().map(|value|match value{Input::Bytes(bytes)=>InputRef::Bytes(bytes),Input::Reader(reader)=>InputRef::Reader(reader)}),deadline_ms:self.deadline_ms,max_output_bytes:self.max_output_bytes}}}
 #[derive(Debug)]pub struct Error{pub code:&'static str,pub operation:&'static str,pub message:std::string::String,pub os_code:Option<i64>}
 fn error(code:&'static str,operation:&'static str,message:impl ToString)->Error{Error{code,operation,message:message.to_string(),os_code:None}}
 fn io_error(operation:&'static str,value:io::Error)->Error{let code=match value.kind(){io::ErrorKind::NotFound=>"not-found",io::ErrorKind::PermissionDenied=>"permission-denied",io::ErrorKind::InvalidInput=>"invalid-input",io::ErrorKind::BrokenPipe=>"broken-pipe",_=>"io"};Error{code,operation,message:value.to_string(),os_code:value.raw_os_error().map(i64::from)}}
 fn input_error(operation:&'static str,value:crate::tsrust_native_io::Error)->Error{Error{code:value.code,operation,message:value.message,os_code:value.os_code}}
 #[derive(Clone,Debug,PartialEq,Eq)]pub struct ExitStatus{pub success:bool,pub code:Option<i64>,pub signal:Option<i64>}
 fn status(value:std::process::ExitStatus)->ExitStatus{#[cfg(unix)]let signal={use std::os::unix::process::ExitStatusExt;value.signal().map(i64::from)};#[cfg(not(unix))]let signal=None;ExitStatus{success:value.success(),code:value.code().map(i64::from),signal}}
 pub struct Output{pub status:ExitStatus,pub stdout:Vec<u8>,pub stderr:Vec<u8>}
 pub struct Child{inner:Option<BackendChild>,settled:bool}
 impl Child{
  pub fn id(&self)->Option<i64>{self.inner.as_ref().and_then(BackendChild::id).map(i64::from)}
  pub fn take_stdin(&mut self)->Option<crate::tsrust_native_io::Writer>{self.inner.as_mut().unwrap().stdin.take().map(|pipe|crate::tsrust_native_io::Writer::asynchronous(Box::new(pipe)))}
  pub fn take_stdout(&mut self)->Option<crate::tsrust_native_io::Reader>{self.inner.as_mut().unwrap().stdout.take().map(|pipe|crate::tsrust_native_io::Reader::asynchronous(Box::new(pipe)))}
  pub fn take_stderr(&mut self)->Option<crate::tsrust_native_io::Reader>{self.inner.as_mut().unwrap().stderr.take().map(|pipe|crate::tsrust_native_io::Reader::asynchronous(Box::new(pipe)))}
  pub async fn wait(&mut self)->Result<ExitStatus,Error>{let value=self.inner.as_mut().unwrap().wait().await.map_err(|value|io_error("wait",value))?;self.settled=true;Ok(status(value))}
  pub fn try_wait(&mut self)->Result<Option<ExitStatus>,Error>{let value=self.inner.as_mut().unwrap().try_wait().map_err(|value|io_error("tryWait",value))?;if value.is_some(){self.settled=true;}Ok(value.map(status))}
  pub async fn kill(&mut self)->Result<(),Error>{self.inner.as_mut().unwrap().kill().await.map_err(|value|io_error("kill",value))?;self.settled=true;Ok(())}
  async fn terminate(&mut self)->Result<(),Error>{if self.try_wait()?.is_some(){return Ok(());}self.inner.as_mut().unwrap().start_kill().map_err(|value|io_error("cleanup.kill",value))?;self.wait().await.map(|_|())}
 }
 impl Drop for Child{fn drop(&mut self){if self.settled{return;}let Some(mut child)=self.inner.take()else{return;};crate::tsrust_native_futures::cleanup(async move{if child.try_wait().ok().flatten().is_none(){let _=child.start_kill();}let _=child.wait().await;});}}
 fn stdio(mode:StdioMode)->Stdio{match mode{StdioMode::Inherit=>Stdio::inherit(),StdioMode::Null=>Stdio::null(),StdioMode::Pipe=>Stdio::piped()}}
 fn command(program:&str,args:&[&str],options:&OptionsRef<'_>,running:bool)->Command{let mut value=Command::new(program);value.args(args);if let Some(cwd)=options.cwd{value.current_dir(cwd);}if options.clear_env{value.env_clear();}value.envs(options.env.iter().copied());value.stdin(stdio(options.stdin.unwrap_or(if running{if options.input.is_some(){StdioMode::Pipe}else{StdioMode::Null}}else{StdioMode::Inherit}))).stdout(stdio(options.stdout.unwrap_or(if running{StdioMode::Pipe}else{StdioMode::Inherit}))).stderr(stdio(options.stderr.unwrap_or(if running{StdioMode::Pipe}else{StdioMode::Inherit})));
  // Explicit owner backstop. Ordinary Tokio defaults false. Awaited cleanup
  // uses B06 scopes; this does not promise process-tree kill or external reaping.
  value.kill_on_drop(true);value
 }
 fn validate(options:&OptionsRef<'_>,running:bool)->Result<(Option<tokio::time::Instant>,Option<usize>),Error>{
  if !running&&(options.input.is_some()||options.deadline_ms.is_some()||options.max_output_bytes.is_some()){return Err(error("option","spawn","input, deadlineMs and maxOutputBytes require run"));}
  if running&&options.input.is_none()&&options.stdin==Some(StdioMode::Pipe){return Err(error("option","run","stdin pipe is inaccessible without owned input"));}
  if options.input.is_some()&&matches!(options.stdin,Some(StdioMode::Inherit|StdioMode::Null)){return Err(error("option","run","owned input requires piped stdin"));}
  let deadline=options.deadline_ms.map(|value|{let value=u64::try_from(value).map_err(|_|error("range","run","deadlineMs must be nonnegative"))?;tokio::time::Instant::now().checked_add(Duration::from_millis(value)).ok_or_else(||error("range","run","deadlineMs exceeds backend Instant range"))}).transpose()?;
  let bound=options.max_output_bytes.map(|value|usize::try_from(value).map_err(|_|error("range","run","maxOutputBytes must be nonnegative and fit target"))).transpose()?;Ok((deadline,bound))
 }
 fn spawn_inner(program:&str,args:&[&str],options:&OptionsRef<'_>,running:bool)->Result<Child,Error>{tokio::runtime::Handle::try_current().map_err(|_|error("runtime","spawn","Process operations require an active runtime"))?;let inner=command(program,args,options,running).spawn().map_err(|value|io_error("spawn",value))?;Ok(Child{inner:Some(inner),settled:false})}
 pub fn spawn(program:&str,args:&[&str],options:OptionsRef<'_>)->Result<Child,Error>{validate(&options,false)?;spawn_inner(program,args,&options,false)}
 async fn capture<A:AsyncRead+Unpin>(mut pipe:Option<A>,bound:Option<usize>,total:&AtomicUsize,operation:&'static str)->Result<Vec<u8>,Error>{let mut output=Vec::new();if let Some(pipe)=pipe.as_mut(){if let Some(bound)=bound{let mut bytes=[0u8;8192];loop{let count=pipe.read(&mut bytes).await.map_err(|value|io_error(operation,value))?;if count==0{break;}if total.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|previous|previous.checked_add(count).filter(|next|*next<=bound)).is_err(){return Err(error("output-limit",operation,"combined stdout/stderr exceeds maxOutputBytes"));}output.try_reserve(count).map_err(|value|error("allocation",operation,value))?;output.extend_from_slice(&bytes[..count]);}}else{pipe.read_to_end(&mut output).await.map_err(|value|io_error(operation,value))?;}}Ok(output)}
 async fn write_input(mut pipe:Option<ChildStdin>,input:Option<Input>)->Result<(),Error>{if let Some(input)=input{let stdin=pipe.as_mut().expect("validated owned input pipe");match input{
  Input::Bytes(bytes)=>stdin.write_all(&bytes).await.map_err(|value|io_error("stdin.write",value))?,
  Input::Reader(mut reader)=>{let copied=async{let mut bytes=[0u8;crate::tsrust_native_io::BUFFER_SIZE];loop{let count=reader.read_into(&mut bytes).await.map_err(|value|input_error("stdin.read",value))? as usize;if count==0{break;}stdin.write_all(&bytes[..count]).await.map_err(|value|io_error("stdin.write",value))?;}Ok::<_,Error>(())}.await;let closed=reader.close().await.map_err(|value|input_error("stdin.input.close",value));copied?;closed?;},
 }stdin.shutdown().await.map_err(|value|io_error("stdin.close",value))?;}drop(pipe);Ok(())}
 async fn run_inner(program:&str,args:&[&str],mut options:Options)->Result<Output,Error>{let(mut child,deadline,bound)={let view=options.view();let(deadline,bound)=validate(&view,true)?;(spawn_inner(program,args,&view,true)?,deadline,bound)};let stdin=child.inner.as_mut().unwrap().stdin.take();let stdout=child.inner.as_mut().unwrap().stdout.take();let stderr=child.inner.as_mut().unwrap().stderr.take();let input=options.input.take();let total=AtomicUsize::new(0);let operation=async{let(status,stdout,stderr,())=tokio::try_join!(child.wait(),capture(stdout,bound,&total,"stdout.read"),capture(stderr,bound,&total,"stderr.read"),write_input(stdin,input))?;Ok(Output{status,stdout,stderr})};let result=if let Some(deadline)=deadline{tokio::time::timeout_at(deadline,operation).await.unwrap_or_else(|_|Err(error("timeout","run","explicit process deadline elapsed")))}else{operation.await};if let Err(cause)=&result{if let Err(mut cleanup)=child.terminate().await{cleanup.message=format!("{}; original failure: {}",cleanup.message,cause.message);return Err(cleanup);}}result}
 pub async fn run(program:&str,args:&[&str],options:Options)->Result<Output,Error>{let scope=crate::tsrust_native_futures::Scope::group();let result=crate::tsrust_native_futures::in_scope(scope.clone(),run_inner(program,args,options)).await;scope.drain(result.is_err()).await;result}
}
