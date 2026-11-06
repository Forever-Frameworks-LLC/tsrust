//! Borrowed typed work items keep recursive record comparison stack safe.
use super::*;
impl Emitter<'_>{
 pub(super) fn testing_deep(&self,fun:&CheckedFunction,args:&[Expr])->String{
  let root=args[0].ty.unwrap().owning();let mut work=vec![root];let mut seen=HashSet::new();let mut records=Vec::new();
  while let Some(ty)=work.pop(){if !seen.insert(ty){continue;}match ty{
   Type::Record(id)=>{records.push(id);work.extend(self.program.records[id].fields.iter().filter_map(|field|field.ty));},
   Type::Array(element)=>work.push(element.ty()),Type::Union(id)|Type::BorrowedUnion(id)=>work.extend(self.program.unions[id.index()].members.iter().copied()),
   Type::Map(id)=>{let map=self.program.records[id].map_definition.as_ref().unwrap();work.extend([map.key,map.value]);},Type::Set(id)=>work.push(self.program.records[id].set_definition.as_ref().unwrap().key),_=>{},
  }}records.sort_unstable();
  let setup=if records.is_empty(){String::new()}else{let variants=records.iter().map(|id|format!("R{id}(&'a {},&'a {})",self.native_type(Type::Record(*id)),self.native_type(Type::Record(*id)))).collect::<Vec<_>>().join(",");format!("enum DeepWork<'a>{{{variants}}}let mut deep_work=Vec::<DeepWork<'_>>::new();")};
  let start=self.testing_deep_step(root,"deep_left","deep_right",false);
  let drain=if records.is_empty(){String::new()}else{let arms=records.iter().map(|id|{let fields=self.program.records[*id].fields.iter().enumerate().map(|(field,metadata)|self.testing_deep_step(metadata.ty.unwrap(),&format!("&left.field{field}"),&format!("&right.field{field}"),true)).collect::<String>();format!("DeepWork::R{id}(left,right)=>{{{fields}}}")}).collect::<String>();format!("while let Some(item)=deep_work.pop(){{match item{{{arms}}}}}")};
  format!("{{let deep_left={};let deep_right={};(||{{{setup}{start}{drain}true}})()}}",self.expr(fun,&args[0],Mode::Read),self.expr(fun,&args[1],Mode::Read))
 }
 fn testing_deep_step(&self,ty:Type,left:&str,right:&str,nested:bool)->String{
  match ty{
   Type::Literal(_,kind)=>self.testing_deep_step(kind.base(),left,right,nested),
   Type::Record(id)=>format!("deep_work.push(DeepWork::R{id}({left},{right}));"),
   Type::Array(element)=>{let child=self.testing_deep_step(element.ty(),"left_item","right_item",true);format!("{{let left={left};let right={right};if left.len()!=right.len(){{return false;}}for(left_item,right_item)in left.iter().zip(right.iter()){{{child}}}}")},
   Type::Map(id)=>{let map=self.program.records[id].map_definition.as_ref().unwrap();let child=self.testing_deep_step(map.value,"left_item","right_item",true);format!("{{let left={left};let right={right};if left.len()!=right.len(){{return false;}}for(key,left_item)in left.iter(){{let Some(right_item)=right.get(key)else{{return false;}};{child}}}}")},
   Type::Set(_)=>format!("{{let left={left};let right={right};if left.len()!=right.len()||!left.iter().all(|item|right.has(item)){{return false;}}}}"),
   Type::Union(id)|Type::BorrowedUnion(id)=>{let left=if nested&&!id.copy{format!("({left}).tsr_data_view()")}else{left.into()};let right=if nested&&!id.copy{format!("({right}).tsr_data_view()")}else{right.into()};let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,member)|{let child=self.testing_deep_step(*member,"left_item","right_item",true);let prefix=if id.copy{"Union"}else{"ViewUnion"};format!("({prefix}{}::V{tag}(left_item),{prefix}{}::V{tag}(right_item))=>{{{child}}},",id.index(),id.index())}).collect::<String>();format!("match ({left},{right}){{{arms}_=>return false,}}")},
   Type::Number=>format!("if !((({left}).to_bits()==({right}).to_bits())||(({left}).is_nan()&&({right}).is_nan())){{return false;}}"),
   Type::Enum(_,crate::syntax::EnumKind::Number)=>format!("{{let left=({left}).tsr_value();let right=({right}).tsr_value();if !(left.to_bits()==right.to_bits()||(left.is_nan()&&right.is_nan())){{return false;}}}}"),
   Type::Enum(_,crate::syntax::EnumKind::String)=>format!("if ({left}).tsr_value()!=({right}).tsr_value(){{return false;}}"),
   Type::Null|Type::Undefined|Type::Void=>String::new(),
   Type::String=>format!("if ({left}).as_ref()!=({right}).as_ref(){{return false;}}"),
   Type::Int|Type::Boolean=>format!("if {left}!={right}{{return false;}}"),
   _=>unreachable!("checked structural assertion"),
  }
 }
}
