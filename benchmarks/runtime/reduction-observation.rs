// Benchmark-only typed visitor, appended after an actual-emission ABI/schema guard.
// The checksum visitor allocates nothing; exact formatting occurs after timing.
#[derive(Clone,Copy)]
enum ReductionToken { ArrayStart,ArrayEnd,RecordStart,RecordEnd,Int(i64),Bool(bool),StringStart,Scalar(u32),StringEnd }
trait ReductionTree { fn visit(&self,v:&mut dyn FnMut(ReductionToken)); }
impl ReductionTree for i64 { fn visit(&self,v:&mut dyn FnMut(ReductionToken)){v(ReductionToken::Int(*self));} }
impl ReductionTree for bool { fn visit(&self,v:&mut dyn FnMut(ReductionToken)){v(ReductionToken::Bool(*self));} }
impl ReductionTree for std::borrow::Cow<'static,str> {fn visit(&self,v:&mut dyn FnMut(ReductionToken)){v(ReductionToken::StringStart);for scalar in self.chars(){v(ReductionToken::Scalar(u32::from(scalar)));}v(ReductionToken::StringEnd);}}
impl<T:ReductionTree> ReductionTree for Vec<T> {fn visit(&self,v:&mut dyn FnMut(ReductionToken)){v(ReductionToken::ArrayStart);for item in self{item.visit(v);}v(ReductionToken::ArrayEnd);}}
impl<T:ReductionTree> ReductionTree for Array<T> {fn visit(&self,v:&mut dyn FnMut(ReductionToken)){v(ReductionToken::ArrayStart);for item in self.iter(){item.visit(v);}v(ReductionToken::ArrayEnd);}}
impl<T:ReductionTree+Copy+Default> ReductionTree for NumericArray<T> {fn visit(&self,v:&mut dyn FnMut(ReductionToken)){v(ReductionToken::ArrayStart);for item in self.iter(){item.visit(v);}v(ReductionToken::ArrayEnd);}}
fn reduction_checksum<T:ReductionTree>(value:&T)->i64 {
 let mut checksum=0_i64;let mut add=|code:i64|{checksum=(checksum*131+code)%1_000_000_007;};
 value.visit(&mut |token|match token{
  ReductionToken::ArrayStart=>add(256),ReductionToken::ArrayEnd=>add(257),ReductionToken::RecordStart=>add(260),ReductionToken::RecordEnd=>add(261),
  ReductionToken::Int(value)=>add(value),ReductionToken::Bool(value)=>{add(262);add(i64::from(value));},
  ReductionToken::StringStart=>add(258),ReductionToken::Scalar(code)=>add(i64::from(code)),ReductionToken::StringEnd=>add(259)
 });checksum
}
fn reduction_exact<T:ReductionTree>(value:&T)->std::string::String {
 use std::fmt::Write as _;
 let mut text=std::string::String::from("r:");let mut depth=0_usize;let mut first=[true;8];let mut string_first=true;
 let begin=|text:&mut std::string::String,first:&mut [bool;8],depth:usize|{if depth>0{if !first[depth-1]{text.push(',');}first[depth-1]=false;}};
 value.visit(&mut |token|match token{
  ReductionToken::ArrayStart|ReductionToken::RecordStart=>{begin(&mut text,&mut first,depth);assert!(depth<first.len());first[depth]=true;depth+=1;text.push_str(if matches!(token,ReductionToken::ArrayStart){"a("}else{"o("});},
  ReductionToken::ArrayEnd|ReductionToken::RecordEnd=>{assert!(depth>0);depth-=1;text.push(')');},
  ReductionToken::Int(value)=>{begin(&mut text,&mut first,depth);write!(&mut text,"i({value})").unwrap();},
  ReductionToken::Bool(value)=>{begin(&mut text,&mut first,depth);write!(&mut text,"b({})",u8::from(value)).unwrap();},
  ReductionToken::StringStart=>{begin(&mut text,&mut first,depth);text.push_str("s(");string_first=true;},
  ReductionToken::Scalar(code)=>{if !string_first{text.push(',');}string_first=false;write!(&mut text,"{code}").unwrap();},ReductionToken::StringEnd=>text.push(')')
 });assert_eq!(depth,0);text
}
fn probe_checksum<T:ReductionTree>(value:&T)->i64{reduction_checksum(value)}
