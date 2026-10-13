//! Record and scoped-map joins share one continuation stack.
use super::*;

enum Resume { Record, Array(u8), Map }
struct Fields {
    declaration:usize,
    pairs:std::vec::IntoIter<(Type,Type)>,
    fields:Vec<Type>,
    changed:bool,
    pending:Option<(Type,Type,Resume)>,
}
struct Sticky {
    original_left:usize,original_right:usize,left:usize,right:usize,
    paths:std::vec::IntoIter<Vec<String>>,pending:Option<Vec<String>>,
}
enum Task {
    Nullable(Type,Type),
    NullablePayload(Type),
    Sticky(Sticky),
    StickyResult(usize,usize),
    Record(usize,usize),
    Map(usize,usize),
    MapValue(usize),
    Fields(Fields),
}

impl Analyzer {
    pub(super) fn join_nullable_record_unions(&mut self,left:Type,right:Type,span:Span)->Result<Option<Type>,Diagnostic>{
        self.join_carrier_work(Task::Nullable(left,right),span)
    }
    pub(super) fn join_sticky_record_types(&mut self,a:usize,b:usize,span:Span)->Result<Option<Type>,Diagnostic>{
        self.join_carrier_work(self.sticky_join_task(a,b),span)
    }
    fn sticky_join_task(&self,a:usize,b:usize)->Task{
        Task::Sticky(Sticky{original_left:a,original_right:b,left:a,right:b,paths:self.nullable_record_field_paths(Type::Record(a)).into_iter(),pending:None})
    }
    pub(super) fn join_record_types(&mut self,a:usize,b:usize,span:Span)->Result<Option<Type>,Diagnostic>{
        self.join_carrier_work(Task::Record(a,b),span)
    }
    pub(super) fn join_scoped_map_types(&mut self,left:Type,right:Type,span:Span)->Result<Option<Type>,Diagnostic>{
        let(Type::Map(a),Type::Map(b))=(left,right)else{return Ok(None);};
        self.join_carrier_work(Task::Map(a,b),span)
    }
    fn join_carrier_callback_field(&mut self,left:Type,right:Type,span:Span)->Result<Option<Type>,Diagnostic>{
        if let Some(family)=self.common_borrowed_conditional_family(left,right,span)?{return Ok(Some(Type::BorrowedCallable(family)));}
        Ok(self.record_field_callback_family(left,right,span)?.map(Type::BorrowedCallable))
    }
    fn join_carrier_work(&mut self,root:Task,span:Span)->Result<Option<Type>,Diagnostic>{
        let mut work=vec![root];
        let mut result=None;
        while let Some(task)=work.pop(){
            match task {
                Task::Nullable(left,right)=>{
                    result=None;
                    if left==right{continue;}
                    if left.union().is_none() || right.union().is_none(){continue;}
                    let(Some(Type::Record(a)),Some(Type::Record(b)))=(self.unions.payload(left),self.unions.payload(right))else{continue;};
                    let declaration=self.scoped_records.declaration(a);
                    if declaration!=self.scoped_records.declaration(b){continue;}
                    let empty=self.empty_scoped_record_environment(declaration)?;
                    work.push(Task::NullablePayload(left));
                    if a==b || empty==Type::Record(b) || b==declaration{result=Some(Type::Record(a));}
                    else if empty==Type::Record(a) || a==declaration{result=Some(Type::Record(b));}
                    else{work.push(self.sticky_join_task(a,b));}
                }
                Task::NullablePayload(left)=>{
                    if let Some(payload)=result{
                        let members=self.unions.members(left.union().unwrap()).into_iter().map(|member|if matches!(member,Type::Record(_)){payload}else{member}).collect::<Vec<_>>();
                        result=Some(self.unions.intern(members,span)?);
                    }
                }
                Task::Sticky(mut frame)=>{
                    if let Some(path)=frame.pending.take() && let Some(joined)=result{
                        frame.left=self.record_type_with_field(frame.left,&path,joined).expect("checked nullable field path");
                        frame.right=self.record_type_with_field(frame.right,&path,joined).expect("checked nullable field path");
                    }
                    let mut child=None;
                    for path in frame.paths.by_ref(){
                        let(Some(first),Some(second))=(self.record_field_type(Type::Record(frame.left),&path),self.record_field_type(Type::Record(frame.right),&path))else{continue;};
                        frame.pending=Some(path);child=Some(Task::Nullable(first,second));break;
                    }
                    if let Some(child)=child{work.push(Task::Sticky(frame));work.push(child);}
                    else if frame.left==frame.right{result=(frame.left!=frame.original_left || frame.right!=frame.original_right).then_some(Type::Record(frame.left));}
                    else{work.push(Task::StickyResult(frame.original_left,frame.left));work.push(Task::Record(frame.left,frame.right));}
                }
                Task::StickyResult(original,left)=>{result=result.or((left!=original).then_some(Type::Record(left)));}
                Task::Record(a,b)=>{
                    let declaration=self.scoped_records.declaration(a);
                    if declaration!=self.scoped_records.declaration(b){result=None;continue;}
                    let pairs=self.program.records[a].fields.iter().zip(&self.program.records[b].fields).map(|(a,b)|(a.ty.unwrap(),b.ty.unwrap())).collect::<Vec<_>>().into_iter();
                    work.push(Task::Fields(Fields{declaration,pairs,fields:Vec::new(),changed:false,pending:None}));
                }
                Task::Map(left,right)=>{
                    result=None;
                    if self.program.records[left].native_type!=self.program.records[right].native_type{continue;}
                    let a=self.program.records[left].map_definition.as_ref().unwrap().clone();
                    let b=self.program.records[right].map_definition.as_ref().unwrap().clone();
                    if a.key!=b.key || !self.aggregate_callable_facts(a.value).scoped || !self.aggregate_callable_facts(b.value).scoped{continue;}
                    let(Type::Record(a),Type::Record(b))=(a.value,b.value)else{continue;};
                    if a==b{result=Some(Type::Map(self.scoped_map_value_instance(left,Type::Record(a))));}
                    else{work.push(Task::MapValue(left));work.push(Task::Record(a,b));}
                }
                Task::MapValue(left)=>{
                    if let Some(value)=result{result=Some(Type::Map(self.scoped_map_value_instance(left,value)));}
                }
                Task::Fields(mut frame)=>{
                    if let Some((left,right,resume))=frame.pending.take(){
                        let joined=match resume {
                            Resume::Record=>Some(result.unwrap_or(left)),
                            Resume::Array(depth)=>match result{Some(Type::Record(record))=>Some(Type::Array(ArrayElement::Record(record,depth))),_=>None},
                            Resume::Map=>match result{Some(value)=>Some(value),None=>self.join_carrier_callback_field(left,right,span)?},
                        };
                        let Some(joined)=joined else{result=None;continue;};
                        frame.changed|=joined!=left || joined!=right;frame.fields.push(joined);
                    }
                    let mut suspended=None;
                    let mut failed=false;
                    while let Some((left,right))=frame.pairs.next(){
                        let joined=if left==right{Some(left)}
                        else if self.absent_callable_type(left) && matches!(right,Type::Closure(_)|Type::BorrowedCallable(_)){Some(right)}
                        else if self.absent_callable_type(right) && matches!(left,Type::Closure(_)|Type::BorrowedCallable(_)){Some(left)}
                        else if let Some(joined)=self.common_native_view_storage(left,right,span)?{Some(joined)}
                        else{
                            let child=match(left,right){
                                (Type::Record(a),Type::Record(b))=>Some((Task::Record(a,b),Resume::Record)),
                                (Type::Array(ArrayElement::Record(a,depth)),Type::Array(ArrayElement::Record(b,other_depth)))=>{
                                    if depth!=other_depth{failed=true;break;}
                                    Some((Task::Record(a,b),Resume::Array(depth)))
                                }
                                (Type::Map(a),Type::Map(b))=>Some((Task::Map(a,b),Resume::Map)),
                                _=>None,
                            };
                            if let Some((child,resume))=child{
                                frame.pending=Some((left,right,resume));
                                suspended=Some(child);break;
                            }
                            self.join_carrier_callback_field(left,right,span)?
                        };
                        let Some(joined)=joined else{failed=true;break;};
                        frame.changed|=joined!=left || joined!=right;frame.fields.push(joined);
                    }
                    if let Some(child)=suspended{
                        work.push(Task::Fields(frame));work.push(child);
                    }else if failed || !frame.changed{result=None;}
                    else{
                        let instance=self.scoped_records.materialize(frame.declaration,frame.fields,&mut self.program.records);
                        if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
                        result=Some(Type::Record(instance));
                    }
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn analyzer()->Analyzer{
        Analyzer::new(crate::syntax::parse("type Item={value:int;};type Other={value:boolean;};function main(){}").unwrap()).unwrap()
    }
    fn instance(analyzer:&mut Analyzer,field:Type)->usize{
        let id=analyzer.scoped_records.materialize(0,vec![field],&mut analyzer.program.records);
        while analyzer.fields.len()<analyzer.program.records.len(){
            let index=analyzer.fields.len();
            analyzer.fields.push(analyzer.program.records[index].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());
        }
        id
    }
    #[test]
    fn deep_record_joins_preserve_left_fallback_without_recursive_descent(){
        let mut analyzer=analyzer();let(mut left,mut right)=(0,1);
        for _ in 0..16384{
            left=instance(&mut analyzer,Type::Record(left));
            right=instance(&mut analyzer,Type::Record(right));
        }
        assert_eq!(analyzer.join_record_types(left,right,Span::default()).unwrap(),Some(Type::Record(left)));
    }
    #[test]
    fn array_joins_require_a_child_join_and_matching_rank(){
        let mut analyzer=analyzer();
        let a=instance(&mut analyzer,Type::Array(ArrayElement::Record(0,0)));
        let b=instance(&mut analyzer,Type::Array(ArrayElement::Record(1,0)));
        assert_eq!(analyzer.join_record_types(a,b,Span::default()).unwrap(),None);
        let c=instance(&mut analyzer,Type::Array(ArrayElement::Record(0,1)));
        assert_eq!(analyzer.join_record_types(a,c,Span::default()).unwrap(),None);
        assert_eq!(analyzer.join_record_types(a,a,Span::default()).unwrap(),None);
    }
    #[test]
    fn incompatible_scalar_fields_and_non_map_inputs_do_not_join(){
        let mut analyzer=analyzer();
        let a=instance(&mut analyzer,Type::Int);
        let b=instance(&mut analyzer,Type::Boolean);
        assert_eq!(analyzer.join_record_types(a,b,Span::default()).unwrap(),None);
        assert_eq!(analyzer.join_scoped_map_types(Type::Record(a),Type::Record(b),Span::default()).unwrap(),None);
    }
}

#[cfg(test)]
mod nullable_tests {
    use super::*;
    #[test]
    fn nullable_join_keeps_left_nullish_members_and_absent_environment_rules(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Holder={run:ReadOnly<Function<int>>;};function main(){}").unwrap()).unwrap();
        let empty=analyzer.empty_scoped_record_environment(0).unwrap();
        let left=analyzer.unions.intern(vec![Type::Record(0),Type::Null],Span::default()).unwrap();
        let right=analyzer.unions.intern(vec![empty,Type::Undefined],Span::default()).unwrap();
        let joined=analyzer.join_nullable_record_unions(left,right,Span::default()).unwrap().unwrap();
        assert!(analyzer.unions.members(joined.union().unwrap()).contains(&Type::Null));
        assert!(!analyzer.unions.members(joined.union().unwrap()).contains(&Type::Undefined));
        assert_eq!(analyzer.unions.payload(joined),Some(Type::Record(0)));
        assert_eq!(analyzer.join_nullable_record_unions(left,left,Span::default()).unwrap(),None);
        assert_eq!(analyzer.join_sticky_record_types(0,0,Span::default()).unwrap(),None);
    }
}
