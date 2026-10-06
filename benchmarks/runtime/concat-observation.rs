// Concat-only complete external observer, actual native acceptance pending. Legacy observer is unchanged.
enum SliceToken { ArrayStart, ArrayEnd, Int(i64), Float(f64), StringStart, Scalar(u32), StringEnd }
trait SliceTree { fn visit(&self, visitor: &mut dyn FnMut(SliceToken)); }
impl SliceTree for i64 { fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::Int(*self));} }
impl SliceTree for bool { fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::Int(i64::from(*self)));} }
impl SliceTree for f64 { fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::Float(*self));} }
impl SliceTree for std::borrow::Cow<'static,str> {
 fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::StringStart);for scalar in self.chars(){v(SliceToken::Scalar(u32::from(scalar)));}v(SliceToken::StringEnd);}
}
impl<T:SliceTree> SliceTree for Vec<T> {
 fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::ArrayStart);for item in self{item.visit(v);}v(SliceToken::ArrayEnd);}
}
impl<T:SliceTree> SliceTree for Array<T> {
 fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::ArrayStart);for item in self.iter(){item.visit(v);}v(SliceToken::ArrayEnd);}
}
impl<T:SliceTree+Copy+Default> SliceTree for NumericArray<T> {
 fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::ArrayStart);for item in self.iter(){item.visit(v);}v(SliceToken::ArrayEnd);}
}
// Profiling uses this visitor only, constructing no observer strings/arrays.
fn slice_checksum<T:SliceTree>(value:&T)->i64 {
 let mut result=0_i64;let mut depth=0_usize;
 let mut add=|code:i64|{result=(result*131+code)%1_000_000_007;};
 value.visit(&mut |token|match token{
  SliceToken::ArrayStart=>{assert!(depth<16);depth+=1;add(256)},SliceToken::ArrayEnd=>{assert!(depth>0);depth-=1;add(257)},SliceToken::Int(value)=>add(value),
  SliceToken::Float(value)=>{let bits=if value.is_nan(){0x7ff8_0000_0000_0000}else{value.to_bits()};for byte in bits.to_be_bytes(){add(i64::from(byte));}},
  SliceToken::StringStart=>add(258),SliceToken::Scalar(code)=>add(i64::from(code)),SliceToken::StringEnd=>add(259)
 });assert_eq!(depth,0);result
}
fn slice_exact<T:SliceTree>(value:&T,numbers:bool)->std::string::String {
 use std::fmt::Write as _;
 let mut text=std::string::String::from(if numbers{"m:"}else{"t:"});
 let mut first=[true;16];let mut depth=0_usize;let mut string_first=true;
 let begin=|text:&mut std::string::String,first:&mut [bool;16],depth:usize|{if depth>0{if !first[depth-1]{text.push(',');}first[depth-1]=false;}};
 value.visit(&mut |token|match token{
  SliceToken::ArrayStart=>{begin(&mut text,&mut first,depth);assert!(depth<first.len());first[depth]=true;depth+=1;text.push('[');},
  SliceToken::ArrayEnd=>{depth-=1;text.push(']');},
  SliceToken::Int(value)=>{begin(&mut text,&mut first,depth);write!(&mut text,"{value}").unwrap();},
  SliceToken::Float(value)=>{begin(&mut text,&mut first,depth);if value.is_nan(){text.push_str("nan");}else{write!(&mut text,"{:016x}",value.to_bits()).unwrap();}},
  SliceToken::StringStart=>{begin(&mut text,&mut first,depth);text.push_str("s(");string_first=true;},
  SliceToken::Scalar(code)=>{if !string_first{text.push(',');}string_first=false;write!(&mut text,"{code}").unwrap();},
  SliceToken::StringEnd=>text.push(')')
 });assert_eq!(depth,0);text
}
fn probe_checksum<T:SliceTree>(value:&T)->i64 { slice_checksum(value) }
