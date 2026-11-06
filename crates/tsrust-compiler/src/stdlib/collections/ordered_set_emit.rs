//! Borrowed lookup keeps owned string keys reusable and uses tree search.
use super::*;
impl Emitter<'_> {
    pub(super) fn ordered_set_method_expression(&self,function:&CheckedFunction,expression:&Expr,callee:&Expr,arguments:&[Expr],method:OrderedSetMethod)->String{
        let ExprKind::Field(base,_)=&callee.kind else{unreachable!()};
        let Type::Record(id)=base.ty.unwrap() else{unreachable!()};
        let receiver=self.expr(function,base,if matches!(method,OrderedSetMethod::Add|OrderedSetMethod::Delete|OrderedSetMethod::Clear){Mode::Mutate}else{Mode::Read});
        let operation=match method{
            OrderedSetMethod::Add=>format!("set_receiver.values.insert(TsrOrdered::new({}));",self.expr(function,&arguments[0],Mode::Consume)),
            OrderedSetMethod::Values=>self.flat_array_result(expression.ty.unwrap(),"set_receiver.values.key_iter().cloned().collect::<Vec<_>>()"),
            OrderedSetMethod::Entries(pair)=>self.flat_array_result(expression.ty.unwrap(),&format!("set_receiver.values.key_iter().map(|key|Record{pair}{{field0:key.clone(),field1:key.clone()}}).collect::<Vec<_>>()")),
            OrderedSetMethod::First|OrderedSetMethod::Last=>{
                let (Type::Union(union)|Type::BorrowedUnion(union))=expression.ty.unwrap() else{unreachable!()};
                let borrowed=matches!(expression.ty,Some(Type::BorrowedUnion(_)));let tag=if borrowed{"ViewUnion"}else{"Union"};let nil=if borrowed{"&TsrUndefined"}else{"TsrUndefined"};
                let absent=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined).unwrap();
                let element=self.program.records[id].native_element().unwrap();let present=self.optional_element_present(union,element,borrowed,"payload");
                let operation=if method==OrderedSetMethod::First{"next"}else{"next_back"};
                let prefix=if element.is_copy(){""}else{"&"};
                format!("match set_receiver.values.iter().{operation}().map(|value|{prefix}value.0){{Some(payload)=>{present},None=>{tag}{}::V{absent}({nil})}}",union.index())
            },
            OrderedSetMethod::Clear=>"set_receiver.values.clear()".into(),
            OrderedSetMethod::Has|OrderedSetMethod::Delete=>{
                let suffix=if self.program.records[id].native_element()==Some(Type::String){"string"}else{"copy"};
                let name=if method==OrderedSetMethod::Has{"contains"}else{"remove"};
                format!("set_receiver.values.{name}_{suffix}_key({})",self.expr(function,&arguments[0],Mode::Read))
            }
        };format!("{{let set_receiver={receiver};{operation}}}")
    }
}
