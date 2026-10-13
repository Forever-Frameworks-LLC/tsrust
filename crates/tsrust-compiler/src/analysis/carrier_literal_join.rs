//! Paired owned parents preserve literal mutations while joining child carriers.
use super::*;
struct Pending { index:usize,left:usize,right:usize,lt:Type,rt:Type }
struct Fields {
    declaration:usize,
    names:std::iter::Enumerate<std::vec::IntoIter<String>>,
    types:Vec<Type>,
    changed:bool,
    pending:Option<Pending>,
}
struct Parent { left:Expr,right:Expr,fields:Fields }
enum Start { Done(Option<Type>), Fields(Fields) }
fn take(value:&mut Expr)->Expr{std::mem::replace(value,Expr::new(ExprKind::Null,value.span))}
fn next(left:&mut Expr,right:&mut Expr,fields:&mut Fields)->Option<(Expr,Expr)>{
    let(ExprKind::Record(left),ExprKind::Record(right))=(&mut left.kind,&mut right.kind)else{unreachable!()};
    for(index,name)in fields.names.by_ref(){
        let Some(l)=left.iter().position(|(key,_)|key.text==name)else{continue;};
        let Some(r)=right.iter().position(|(key,_)|key.text==name)else{continue;};
        fields.pending=Some(Pending{index,left:l,right:r,lt:left[l].1.ty.unwrap(),rt:right[r].1.ty.unwrap()});
        return Some((take(&mut left[l].1),take(&mut right[r].1)));
    }
    None
}
impl Analyzer {
    pub(super) fn join_record_values(&mut self,left:&mut Expr,right:&mut Expr)->Result<Option<Type>,Diagnostic>{
        let(mut l,mut r)=(take(left),take(right));
        let mut parents=Vec::<Parent>::new();
        'walk: loop{
            let mut result=match self.start_literal_join(&mut l,&mut r){
                Ok(Start::Done(result))=>Ok(result),
                Ok(Start::Fields(mut fields))=>{
                    if let Some((child_l,child_r))=next(&mut l,&mut r,&mut fields){
                        parents.push(Parent{left:l,right:r,fields});l=child_l;r=child_r;continue;
                    }
                    Ok(self.finish_literal_join(&mut l,&mut r,fields))
                }
                Err(error)=>Err(error),
            };
            while let Some(mut parent)=parents.pop(){
                let pending=parent.fields.pending.take().expect("pending literal field");
                let(ExprKind::Record(left_fields),ExprKind::Record(right_fields))=(&mut parent.left.kind,&mut parent.right.kind)else{unreachable!()};
                left_fields[pending.left].1=l;right_fields[pending.right].1=r;
                if let Ok(joined)=result{
                    result=self.finish_literal_field(&mut left_fields[pending.left].1,&mut right_fields[pending.right].1,&mut parent.fields,&pending,joined).map(|()|None);
                    if result.is_ok(){
                        if let Some((child_l,child_r))=next(&mut parent.left,&mut parent.right,&mut parent.fields){
                            parents.push(parent);l=child_l;r=child_r;continue 'walk;
                        }
                        result=Ok(self.finish_literal_join(&mut parent.left,&mut parent.right,parent.fields));
                    }
                }
                l=parent.left;r=parent.right;
            }
            *left=l;*right=r;return result;
        }
    }
    fn start_literal_join(&mut self,left:&mut Expr,right:&mut Expr)->Result<Start,Diagnostic>{
        let(Some(Type::Record(a)),Some(Type::Record(b)))=(left.ty,right.ty)else{return Ok(Start::Done(None));};
        let declaration=self.scoped_records.declaration(a);
        if declaration!=self.scoped_records.declaration(b){return Ok(Start::Done(None));}
        if !matches!((&left.kind,&right.kind),(ExprKind::Record(_),ExprKind::Record(_))){
            let joined=self.join_record_types(a,b,left.span)?;
            if let Some(ty)=joined{for value in [left,right]{if value.ty!=Some(ty){value.wrap_record_carrier(ty);}}}
            return Ok(Start::Done(joined));
        }
        let fields=&self.program.records[declaration].fields;
        Ok(Start::Fields(Fields{declaration,names:fields.iter().map(|field|field.name.text.clone()).collect::<Vec<_>>().into_iter().enumerate(),types:fields.iter().map(|field|field.ty.unwrap()).collect(),changed:false,pending:None}))
    }
    fn finish_literal_field(&mut self,l:&mut Expr,r:&mut Expr,fields:&mut Fields,pending:&Pending,joined:Option<Type>)->Result<(),Diagnostic>{
        let Pending{index,lt,rt,..}=*pending;
        if let Some(ty)=joined{fields.types[index]=ty;fields.changed=true;}
        else if let Some(ty)=self.common_native_view_storage(lt,rt,l.span)?{
            for value in [l,r]{value.wrap_record_carrier(ty);}
            fields.types[index]=ty;fields.changed=true;
        }
        else if lt!=rt && let Some(ty)=self.join_scoped_map_types(lt,rt,l.span)?{
            for value in [l,r]{if value.ty!=Some(ty){value.wrap_record_carrier(ty);}}
            fields.types[index]=ty;fields.changed=true;
        }
        else if lt!=rt && let Some(family)=self.common_borrowed_conditional_family(lt,rt,l.span)?{
            for value in [l,r]{if !matches!(value.kind,ExprKind::Unary{op:UnaryOp::CallableStore,..}){value.wrap_storage(family);}value.ty=Some(Type::BorrowedCallable(family));}
            fields.types[index]=Type::BorrowedCallable(family);fields.changed=true;
        }else if lt==rt{fields.types[index]=lt;}
        Ok(())
    }
    fn finish_literal_join(&mut self,left:&mut Expr,right:&mut Expr,fields:Fields)->Option<Type>{
        if !fields.changed{return None;}
        let instance=self.scoped_records.materialize(fields.declaration,fields.types,&mut self.program.records);
        if self.fields.len()<=instance{self.fields.push(self.program.records[instance].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
        let ty=Type::Record(instance);left.ty=Some(ty);right.ty=Some(ty);Some(ty)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn analyzer()->Analyzer{Analyzer::new(crate::syntax::parse("type Item={value:int;};function main(){}").unwrap()).unwrap()}
    fn leaf(ty:Type,span:Span)->Expr{let mut value=Expr::new(ExprKind::Name("leaf".into()),span);value.ty=Some(ty);value}
    fn nest(analyzer:&Analyzer,mut value:Expr,count:usize)->Expr{
        let name=analyzer.program.records[0].fields[0].name.clone();
        for _ in 0..count{value=Expr::new(ExprKind::Record(vec![(name.clone(),value)]),Span::default());value.ty=Some(Type::Record(0));}
        value
    }
    fn depth(value:&Expr)->(usize,&Expr){
        let mut count=0;let mut value=value;
        while let ExprKind::Record(fields)=&value.kind{assert_eq!(fields.len(),1);count+=1;value=&fields[0].1;}
        (count,value)
    }
    #[test]
    fn deep_literal_join_restores_both_expression_trees(){
        let mut analyzer=analyzer();
        let mut left=nest(&analyzer,leaf(Type::Int,Span::default()),16384);
        let mut right=nest(&analyzer,leaf(Type::Int,Span::default()),16384);
        assert_eq!(analyzer.join_record_values(&mut left,&mut right).unwrap(),None);
        assert_eq!(depth(&left).0,16384);assert_eq!(depth(&right).0,16384);
        assert_eq!(depth(&left).1.ty,Some(Type::Int));assert_eq!(depth(&right).1.ty,Some(Type::Int));
        drop((left,right));
    }
    #[test]
    fn deep_literal_diagnostic_restores_children_and_preserves_leaf_span(){
        let mut checked=crate::check("function main(){const data=[1];const first=(x:int):int=>data[0]+x;const second=(x:string):int=>data[0]+x.length;console.log(first(1),second(\"a\"));}").unwrap();
        let mut analyzer=analyzer();analyzer.closures=std::mem::take(&mut checked.program.closures);
        assert_eq!(analyzer.closures.len(),2);assert!(analyzer.closures.iter().all(|closure|closure.borrowed));
        let span=Span::new(20,24);
        let mut left=nest(&analyzer,leaf(Type::Closure(0),span),4096);
        let mut right=nest(&analyzer,leaf(Type::Closure(1),Span::new(30,34)),4096);
        let error=analyzer.join_record_values(&mut left,&mut right).unwrap_err();
        assert_eq!(error.code,"type");assert_eq!(error.span,span);
        assert_eq!(depth(&left).0,4096);assert_eq!(depth(&right).0,4096);
        assert_eq!(depth(&left).1.ty,Some(Type::Closure(0)));assert_eq!(depth(&right).1.ty,Some(Type::Closure(1)));
        assert!(matches!(depth(&left).1.kind,ExprKind::Name(_)));assert!(matches!(depth(&right).1.kind,ExprKind::Name(_)));
        drop((left,right));
    }
    #[test]
    fn incompatible_nominal_records_leave_both_values_unchanged(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("type A={value:int;};type B={value:int;};function main(){}").unwrap()).unwrap();
        let mut left=leaf(Type::Record(0),Span::default());let mut right=leaf(Type::Record(1),Span::default());
        assert_eq!(analyzer.join_record_values(&mut left,&mut right).unwrap(),None);
        assert!(matches!(left.kind,ExprKind::Name(_)));assert!(matches!(right.kind,ExprKind::Name(_)));
    }
}
