//! Heap keys wrap the checked source order without cloning stored values.
use super::*;
impl Emitter<'_> {
    pub(super) fn heap_method_expression(&self,function:&CheckedFunction,expression:&Expr,callee:&Expr,arguments:&[Expr],method:HeapMethod)->String{
        let ExprKind::Field(base,_)=&callee.kind else{unreachable!()};
        let Type::Record(id)=base.ty.unwrap() else{unreachable!()};let element=self.program.records[id].native_element().unwrap();
        let read=method==HeapMethod::Peek;
        let receiver=self.expr(function,base,if read{Mode::Read}else{Mode::Mutate});
        let selected=match method{
            HeapMethod::Push=>format!("heap_receiver.values.push(TsrOrdered::new({}))",self.expr(function,&arguments[0],Mode::Consume)),
            HeapMethod::Clear=>"heap_receiver.values.clear()".into(),
            HeapMethod::Pop=>"heap_receiver.values.pop().map(|value|value.0)".into(),
            HeapMethod::Peek=>format!("heap_receiver.values.peek().map(|value|{}value.0)",if element.is_copy(){""}else{"&"}),
        };
        let result=if let Some(union)=expression.ty.unwrap().union(){
            let borrowed=matches!(expression.ty,Some(Type::BorrowedUnion(_)));let tag=if borrowed{"ViewUnion"}else{"Union"};let nil=if borrowed{"&TsrUndefined"}else{"TsrUndefined"};
            let absent=self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined).unwrap();
            let present=self.optional_element_present(union,element,borrowed,"payload");
            format!("match {selected}{{Some(payload)=>{present},None=>{tag}{}::V{absent}({nil})}}",union.index())
        }else{selected};
        format!("{{let heap_receiver={receiver};{result}}}")
    }
}
