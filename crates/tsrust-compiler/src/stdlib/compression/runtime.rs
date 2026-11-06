// Compression is CPU work on an explicit bounded caller-owned pool.
mod tsrust_native_compression {
 use std::convert::Infallible;
 use crate::tsrust_native_threads as pool;
 const WORKING:usize=8192;
 #[derive(Clone,Copy)]pub enum Format{Gzip,Zlib,Raw}
 #[derive(Clone,Copy)]pub enum Direction{Encode,Decode}
 pub struct Error{pub code:&'static str,pub operation:&'static str,pub message:std::string::String}
 impl std::fmt::Display for Error{fn fmt(&self,formatter:&mut std::fmt::Formatter<'_>)->std::fmt::Result{formatter.write_str(&self.message)}}
 impl std::fmt::Debug for Error{fn fmt(&self,formatter:&mut std::fmt::Formatter<'_>)->std::fmt::Result{formatter.debug_struct("CompressionError").field("code",&self.code).field("operation",&self.operation).field("message",&self.message).finish()}}
 impl std::error::Error for Error{}
 pub enum Failure{Codec(Error),Admission(pool::SpawnError),Completion(pool::JoinError<Infallible>)}
 pub(crate) fn error(code:&'static str,operation:&'static str,message:impl std::fmt::Display)->Error{Error{code,operation,message:message.to_string()}}
 pub(crate) fn level(value:Option<i64>,operation:&'static str)->Result<flate2::Compression,Error>{match value{None=>Ok(flate2::Compression::default()),Some(value)if(0..=9).contains(&value)=>Ok(flate2::Compression::new(value as u32)),_=>Err(error("range",operation,"level must be an integer from 0 through 9"))}}
 pub(crate) fn limit(value:Option<i64>,operation:&'static str)->Result<Option<usize>,Error>{value.map(|value|usize::try_from(value).map_err(|_|error("range",operation,"maxOutputBytes must be nonnegative and fit the target"))).transpose()}
 pub(crate) fn compressor(format:Format,level:flate2::Compression)->flate2::Compress{match format{Format::Gzip=>flate2::Compress::new_gzip(level,15),Format::Zlib=>flate2::Compress::new(level,true),Format::Raw=>flate2::Compress::new(level,false)}}
 pub(crate) fn decompressor(format:Format)->flate2::Decompress{match format{Format::Gzip=>flate2::Decompress::new_gzip(15),Format::Zlib=>flate2::Decompress::new(true),Format::Raw=>flate2::Decompress::new(false)}}
 fn append(output:&mut Vec<u8>,bytes:&[u8],operation:&'static str)->Result<(),Error>{output.try_reserve(bytes.len()).map_err(|failure|error("allocation",operation,failure))?;output.extend_from_slice(bytes);Ok(())}
 // Backend owns framing, checksums and compression. No implicit ordinary expansion cap.
 pub fn buffer(input:Vec<u8>,format:Format,direction:Direction,option:Option<i64>,operation:&'static str)->Result<Vec<u8>,Error>{
  let mut output=Vec::new();let mut chunk=[0;WORKING];let mut offset=0;
  match direction{
   Direction::Encode=>{let mut codec=compressor(format,level(option,operation)?);loop{let before=(codec.total_in(),codec.total_out());let status=codec.compress(&input[offset..],&mut chunk,flate2::FlushCompress::Finish).map_err(|failure|error("codec",operation,failure))?;let used=(codec.total_in()-before.0)as usize;let made=(codec.total_out()-before.1)as usize;offset+=used;append(&mut output,&chunk[..made],operation)?;if status==flate2::Status::StreamEnd{assert_eq!(offset,input.len());return Ok(output);}if used==0&&made==0{return Err(error("codec",operation,"encoder made no progress"));}}},
   Direction::Decode=>{let limit=limit(option,operation)?;let mut codec=decompressor(format);loop{let before=(codec.total_in(),codec.total_out());let capacity=limit.map_or(WORKING,|bound|bound.saturating_sub(output.len()).max(1).min(WORKING));let status=codec.decompress(&input[offset..],&mut chunk[..capacity],flate2::FlushDecompress::None).map_err(|failure|error("codec",operation,failure))?;let used=(codec.total_in()-before.0)as usize;let made=(codec.total_out()-before.1)as usize;offset+=used;if limit.is_some_and(|bound|made>bound.saturating_sub(output.len())){return Err(error("output-limit",operation,"decoded output exceeds explicit maxOutputBytes"));}append(&mut output,&chunk[..made],operation)?;
    if status==flate2::Status::StreamEnd{if offset==input.len(){return Ok(output);}if matches!(format,Format::Gzip){codec=decompressor(format);}else{return Err(error("trailing-data",operation,"trailing compressed data"));}}
    else if used==0&&made==0{return Err(error(if offset==input.len(){"truncated"}else{"codec"},operation,"decoder did not reach a complete stream"));}
   }},
  }
 }
 pub async fn run(capability:pool::PoolSubmission,input:Vec<u8>,format:Format,direction:Direction,option:Option<i64>,operation:&'static str)->Result<Vec<u8>,Failure>{
  // Validate before bounded admission, while retaining lazy first-poll semantics.
  match direction{Direction::Encode=>{level(option,operation).map_err(Failure::Codec)?;},Direction::Decode=>{limit(option,operation).map_err(Failure::Codec)?;}}
  match pool::task_async(capability,move||Ok::<_,Infallible>(buffer(input,format,direction,option,operation))).await{Ok(result)=>result.map_err(Failure::Codec),Err(pool::AsyncTaskError::Admission(error))=>Err(Failure::Admission(error)),Err(pool::AsyncTaskError::Completion(error))=>Err(Failure::Completion(error))}
 }
}
