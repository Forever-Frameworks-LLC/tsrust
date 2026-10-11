//! Borrowed lookup keeps owned string keys reusable and uses tree search.
use super::*;
impl Emitter<'_> {
    pub(super) fn ordered_set_method_expression(&self,function:&CheckedFunction,callee:&Expr,arguments:&[Expr],method:OrderedSetMethod)->String{
        let ExprKind::Field(base,_)=&callee.kind else{unreachable!()};
        let Type::Record(id)=base.ty.unwrap() else{unreachable!()};
        let receiver=self.expr(function,base,if method==OrderedSetMethod::Has{Mode::Read}else{Mode::Mutate});
        let operation=match method{
            OrderedSetMethod::Add=>format!("set_receiver.values.insert(TsrOrdered::new({}));",self.expr(function,&arguments[0],Mode::Consume)),
            OrderedSetMethod::Clear=>"set_receiver.values.clear()".into(),
            OrderedSetMethod::Has|OrderedSetMethod::Delete=>{
                let suffix=if self.program.records[id].native_element()==Some(Type::String){"string"}else{"copy"};
                let name=if method==OrderedSetMethod::Has{"contains"}else{"remove"};
                format!("set_receiver.values.{name}_{suffix}_key({})",self.expr(function,&arguments[0],Mode::Read))
            }
        };format!("{{let set_receiver={receiver};{operation}}}")
    }
}
