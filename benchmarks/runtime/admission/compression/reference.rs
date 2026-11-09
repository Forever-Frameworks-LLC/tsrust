// Admission only: backend-owned codecs, no public compiler/runtime implementation.
use std::io::{self,Read,BufRead,Write};
use flate2::Compression;
#[derive(Clone,Copy,Debug)]enum Format{Gzip,Zlib,Raw}
impl Format{fn name(self)->&'static str{match self{Self::Gzip=>"gzip",Self::Zlib=>"zlib",Self::Raw=>"raw"}}fn parse(value:&str)->Self{match value{"gzip"=>Self::Gzip,"zlib"=>Self::Zlib,"raw"=>Self::Raw,_=>panic!("unknown format")}}}
#[derive(Clone,Copy,Debug)]enum Strategy{Writer,Reader}
impl Strategy{fn name(self)->&'static str{match self{Self::Writer=>"writer",Self::Reader=>"reader"}}}
fn failure(message:&'static str)->io::Error{io::Error::new(io::ErrorKind::InvalidData,message)}
fn level(value:i32)->io::Result<Compression>{if value==-1{Ok(Compression::default())}else if(0..=9).contains(&value){Ok(Compression::new(value as u32))}else{Err(failure("compression level must be 0 through 9"))}}
struct Chunks<'a>{rest:&'a[u8],width:usize}
impl<'a> Chunks<'a>{fn new(rest:&'a[u8],width:usize)->Self{assert!(width>0);Self{rest,width}}}
impl Read for Chunks<'_>{fn read(&mut self,destination:&mut[u8])->io::Result<usize>{let count=destination.len().min(self.rest.len()).min(self.width);destination[..count].copy_from_slice(&self.rest[..count]);self.rest=&self.rest[count..];Ok(count)}}
impl BufRead for Chunks<'_>{fn fill_buf(&mut self)->io::Result<&[u8]>{Ok(&self.rest[..self.width.min(self.rest.len())])}fn consume(&mut self,count:usize){self.rest=&self.rest[count..];}}
fn collect<R:Read>(reader:&mut R,limit:Option<usize>)->io::Result<Vec<u8>>{
 let mut output=Vec::new();let mut scratch=[0u8;8192];
 loop{
  // Only an explicit caller policy constrains expansion. One scratch-byte probe
  // distinguishes exact-limit EOF from excess output without retaining excess.
  let count=limit.map_or(scratch.len(),|limit|limit.saturating_sub(output.len()).max(1).min(scratch.len()));
  let count=match reader.read(&mut scratch[..count]){Err(error)if error.kind()==io::ErrorKind::Interrupted=>continue,result=>result?};
  if count==0{return Ok(output);}
  if limit.is_some_and(|limit|count>limit.saturating_sub(output.len())){return Err(failure("explicit decompressed-byte limit exceeded"));}
  output.extend_from_slice(&scratch[..count]);
 }
}
fn compress(format:Format,input:&[u8],selected_level:i32,chunk:usize,strategy:Strategy)->io::Result<Vec<u8>>{
 let compression=level(selected_level)?;
 match strategy{
  Strategy::Writer=>{
   macro_rules! encode{($encoder:expr)=>{{let mut encoder=$encoder;for part in input.chunks(chunk){encoder.write_all(part)?;}encoder.finish()}};}
   match format{
    Format::Gzip=>encode!(flate2::GzBuilder::new().mtime(0).operating_system(255).write(Vec::new(),compression)),
    Format::Zlib=>encode!(flate2::write::ZlibEncoder::new(Vec::new(),compression)),
    Format::Raw=>encode!(flate2::write::DeflateEncoder::new(Vec::new(),compression)),
   }
  },
  Strategy::Reader=>{
   let source=Chunks::new(input,chunk);
   match format{
    Format::Gzip=>collect(&mut flate2::GzBuilder::new().mtime(0).operating_system(255).buf_read(source,compression),None),
    Format::Zlib=>collect(&mut flate2::bufread::ZlibEncoder::new(source,compression),None),
    Format::Raw=>collect(&mut flate2::bufread::DeflateEncoder::new(source,compression),None),
   }
  }
 }
}
fn decompress(format:Format,input:&[u8],chunk:usize,limit:Option<usize>)->io::Result<Vec<u8>>{
 let source=Chunks::new(input,chunk);
 macro_rules! decode{($decoder:expr)=>{{let mut decoder=$decoder;let output=collect(&mut decoder,limit)?;if !decoder.into_inner().rest.is_empty(){return Err(failure("trailing compressed data"));}Ok(output)}};}
 match format{
  Format::Gzip=>decode!(flate2::bufread::MultiGzDecoder::new(source)),
  Format::Zlib=>decode!(flate2::bufread::ZlibDecoder::new(source)),
  Format::Raw=>decode!(flate2::bufread::DeflateDecoder::new(source)),
 }
}
fn input(size:usize,random:bool,seed:u32)->Vec<u8>{
 let mut state=seed; (0..size).map(|index|if random{state=state.wrapping_mul(1664525).wrapping_add(1013904223);(state>>24)as u8}else{b"tsrust-compression-stream\n"[index%26]}).collect()
}
fn checksum(bytes:&[u8])->u64{bytes.iter().fold(7,|sum,&value|(sum*131+u64::from(value))%1_000_000_007)}
fn hex(bytes:&[u8])->String{bytes.iter().map(|byte|format!("{byte:02x}")).collect()}
fn unhex(text:&str)->Vec<u8>{assert_eq!(text.len()%2,0);text.as_bytes().chunks_exact(2).map(|pair|u8::from_str_radix(std::str::from_utf8(pair).unwrap(),16).unwrap()).collect()}
struct FinishFailure;
impl Write for FinishFailure{fn write(&mut self,_:&[u8])->io::Result<usize>{Err(io::Error::other("synthetic finish write failure"))}fn flush(&mut self)->io::Result<()>{Err(io::Error::other("synthetic flush failure"))}}
fn correctness(){
 for format in[Format::Gzip,Format::Zlib,Format::Raw]{for strategy in[Strategy::Writer,Strategy::Reader]{for selected_level in[0,1,-1,9]{for random in[false,true]{for size in[0,1,64,16384]{
  let original=input(size,random,7);let compressed=compress(format,&original,selected_level,64,strategy).unwrap();
  for chunk in[1,64,8192]{assert_eq!(decompress(format,&compressed,chunk,None).unwrap(),original);assert_eq!(decompress(format,&compressed,chunk,Some(size)).unwrap(),original);if size>0{assert!(decompress(format,&compressed,chunk,Some(size-1)).is_err());}}
  assert_eq!(compress(format,&original,selected_level,64,strategy).unwrap(),compressed);
  for removed in[1,2]{if compressed.len()>removed{assert!(decompress(format,&compressed[..compressed.len()-removed],64,None).is_err(),"truncation {format:?}/{strategy:?}/{selected_level}/{size}");}}
  let mut trailing=compressed.clone();trailing.extend_from_slice(b"unexpected tail");assert!(decompress(format,&trailing,64,None).is_err());
  if !matches!(format,Format::Raw){let mut corrupt=compressed.clone();let last=corrupt.len()-1;corrupt[last]^=1;assert!(decompress(format,&corrupt,64,None).is_err());}
  if matches!(format,Format::Gzip){let mut corrupt=compressed.clone();let crc=corrupt.len()-8;corrupt[crc]^=1;assert!(decompress(format,&corrupt,64,None).is_err());}
 }}}}}
 let one=compress(Format::Gzip,b"one",-1,64,Strategy::Writer).unwrap();let two=compress(Format::Gzip,b"two",-1,64,Strategy::Writer).unwrap();let members=[one,two].concat();assert_eq!(decompress(Format::Gzip,&members,1,None).unwrap(),b"onetwo");
 assert!(level(-2).is_err());assert!(level(10).is_err());assert_eq!(level(-1).unwrap().level(),6);
 let encoder=flate2::GzBuilder::new().mtime(0).operating_system(255).write(FinishFailure,Compression::default());assert!(encoder.finish().is_err());
 println!("{{\"kind\":\"contracts\",\"roundTrips\":true,\"incrementalLimit\":true,\"truncation\":true,\"checksums\":true,\"strictTrailingData\":true,\"gzipMembers\":true,\"deterministicGzip\":true,\"finishFailure\":true,\"backendDefaultLevel\":6}}");
}

// Backend Status certifies writer completion; total_in alone cannot prove a
// truncated raw/zlib stream reached its terminator. No DEFLATE state is reimplemented.
struct StrictInflateWriter<W>{backend:flate2::Decompress,sink:W,ended:bool,written:usize,limit:Option<usize>}
impl<W:Write> StrictInflateWriter<W>{
 fn new(sink:W,zlib:bool,limit:Option<usize>)->Self{Self{backend:flate2::Decompress::new(zlib),sink,ended:false,written:0,limit}}
 fn advance(&mut self,input:&[u8],flush:flate2::FlushDecompress)->io::Result<usize>{
  if self.ended{return if input.is_empty(){Ok(0)}else{Err(failure("trailing compressed data"))};}
  let mut consumed=0;let mut scratch=[0u8;8192];
  loop{
   let before_in=self.backend.total_in();let before_out=self.backend.total_out();
   let capacity=self.limit.map_or(scratch.len(),|limit|limit.saturating_sub(self.written).max(1).min(scratch.len()));
   let status=self.backend.decompress(&input[consumed..],&mut scratch[..capacity],flush).map_err(|_|failure("invalid compressed input"))?;
   let used=(self.backend.total_in()-before_in)as usize;let produced=(self.backend.total_out()-before_out)as usize;consumed+=used;
   if self.limit.is_some_and(|limit|produced>limit.saturating_sub(self.written)){return Err(failure("explicit decompressed-byte limit exceeded"));}
   self.sink.write_all(&scratch[..produced])?;self.written+=produced;
   if status==flate2::Status::StreamEnd{self.ended=true;if consumed<input.len(){return Err(failure("trailing compressed data"));}return Ok(consumed);}
   if consumed==input.len() && flush!=flate2::FlushDecompress::Finish{return Ok(consumed);}
   if used==0&&produced==0{return Err(io::Error::new(io::ErrorKind::UnexpectedEof,"incomplete compressed input"));}
  }
 }
 fn finish(mut self)->io::Result<W>{if !self.ended{self.advance(&[],flate2::FlushDecompress::Finish)?;}Ok(self.sink)}
}
impl<W:Write> Write for StrictInflateWriter<W>{fn write(&mut self,input:&[u8])->io::Result<usize>{self.advance(input,flate2::FlushDecompress::None)}fn flush(&mut self)->io::Result<()>{self.sink.flush()}}
struct LimitedSink{bytes:Vec<u8>,limit:Option<usize>}
impl Write for LimitedSink{fn write(&mut self,input:&[u8])->io::Result<usize>{if self.limit.is_some_and(|limit|input.len()>limit.saturating_sub(self.bytes.len())){return Err(failure("explicit decompressed-byte limit exceeded"));}self.bytes.extend_from_slice(input);Ok(input.len())}fn flush(&mut self)->io::Result<()>{Ok(())}}
fn decompress_writer(format:Format,input:&[u8],chunk:usize,limit:Option<usize>)->io::Result<Vec<u8>>{
 if matches!(format,Format::Gzip){let sink=LimitedSink{bytes:Vec::new(),limit};let mut decoder=flate2::write::MultiGzDecoder::new(sink);for part in input.chunks(chunk){decoder.write_all(part)?;}Ok(decoder.finish()?.bytes)}
 else{let mut decoder=StrictInflateWriter::new(Vec::new(),matches!(format,Format::Zlib),limit);for part in input.chunks(chunk){decoder.write_all(part)?;}decoder.finish()}
}
fn writer_contracts(){
 for format in[Format::Gzip,Format::Zlib,Format::Raw]{
  let original=input(16384,true,7);let compressed=compress(format,&original,-1,64,Strategy::Writer).unwrap();
  for chunk in[1,64,8192]{assert_eq!(decompress_writer(format,&compressed,chunk,None).unwrap(),original);assert_eq!(decompress_writer(format,&compressed,chunk,Some(original.len())).unwrap(),original);assert!(decompress_writer(format,&compressed,chunk,Some(original.len()-1)).is_err());assert!(decompress_writer(format,&compressed[..compressed.len()-1],chunk,None).is_err());let mut tail=compressed.clone();tail.extend_from_slice(b"tail");assert!(decompress_writer(format,&tail,chunk,None).is_err());}
 }
 let mut raw=StrictInflateWriter::new(FinishFailure,false,None);assert!(raw.write_all(&compress(Format::Raw,b"output",-1,64,Strategy::Writer).unwrap()).is_err());
 println!("{{\"kind\":\"writer-contracts\",\"explicitStreamEnd\":true,\"incrementalLimit\":true,\"truncation\":true,\"strictTrailingData\":true,\"sinkFailure\":true}}");
}

struct ShortSink{bytes:Vec<u8>,interrupt:bool,fail:std::sync::Arc<std::sync::atomic::AtomicBool>}
impl Write for ShortSink{
 fn write(&mut self,input:&[u8])->io::Result<usize>{
  if self.fail.load(std::sync::atomic::Ordering::SeqCst){return Err(io::Error::other("synthetic final trailer write failure"));}
  if self.interrupt{self.interrupt=false;return Err(io::Error::from(io::ErrorKind::Interrupted));}
  let count=input.len().min(2);self.bytes.extend_from_slice(&input[..count]);Ok(count)
 }
 fn flush(&mut self)->io::Result<()>{if self.fail.load(std::sync::atomic::Ordering::SeqCst){Err(io::Error::other("synthetic flush failure"))}else{Ok(())}}
}
fn short_io_contracts(){
 let fail=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
 let sink=ShortSink{bytes:Vec::new(),interrupt:true,fail:fail.clone()};let mut encoder=flate2::GzBuilder::new().mtime(0).operating_system(255).write(sink,Compression::default());encoder.write_all(b"short writes and interruption").unwrap();let sink=encoder.finish().unwrap();assert_eq!(decompress(Format::Gzip,&sink.bytes,1,None).unwrap(),b"short writes and interruption");
 let sink=ShortSink{bytes:Vec::new(),interrupt:false,fail:fail.clone()};let mut encoder=flate2::GzBuilder::new().mtime(0).operating_system(255).write(sink,Compression::default());encoder.write_all(b"deferred final trailer").unwrap();fail.store(true,std::sync::atomic::Ordering::SeqCst);assert!(encoder.finish().is_err());
 println!("{{\"kind\":\"short-io-contracts\",\"shortWrites\":true,\"interruptedWrite\":true,\"finishTrailerFailure\":true}}");
}
