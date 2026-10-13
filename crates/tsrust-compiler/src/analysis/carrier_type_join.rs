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
enum Task {
    Record(usize,usize),
    Map(usize,usize),
    MapValue(usize),
    Fields(Fields),
}

impl Analyzer {
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
