//! Optional-default effects are joined only after a suspended initializer succeeds.
use super::super::*;
pub(in crate::analysis) struct DefaultFrame {
    pub(in crate::analysis) mode:Mode,
    pub(in crate::analysis) expected:Option<Type>,
    declared:Type,
    prior:Vec<Availability>,
    supplied_flow:crate::flow::Facts,
    argument:Option<super::arguments::ArgumentFrame>,
}
impl Analyzer {
    pub(in crate::analysis) fn prepare_default_work(&self,context:&Context,value:&Expr,expected:Type,data_mode:Mode)->DefaultFrame{
        let prior=context.bindings.iter().map(Availability::of).collect();
        let supplied_flow=context.flow.clone();
        let kind=match self.unions.payload(expected).unwrap_or(expected){Type::Callback(id)=>Some(self.callables.callbacks[id].kind),_=>None};
        let mode=if kind==Some(CallbackKind::Read){Mode::Read}else if kind==Some(CallbackKind::Reusable){Mode::Mutate}else if kind.is_some(){Mode::Consume}else{data_mode};
        let mode=if self.expression_type_hint(context,value).is_some_and(|ty|matches!(ty,Type::Null|Type::Undefined)){Mode::Copy}else{mode};
        let argument=if matches!(mode,Mode::Read|Mode::Mutate) && expected.union().is_some() && matches!(value.kind,ExprKind::Name(_)|ExprKind::Field(..)|ExprKind::Index(..)){
            Some(self.prepare_parameter_argument(context,value,ArgumentContract{rest:false,ty:expected,mode,omission:crate::ParameterOmission::Defaulted}))
        }else{None};
        let (mode,hint)=if let Some(frame)=&argument{(frame.mode,frame.expected)}else{
            (if matches!(mode,Mode::Read|Mode::Mutate) && expected.union().is_some(){Mode::Consume}else{mode},Some(expected))
        };
        DefaultFrame{mode,expected:hint,declared:expected,prior,supplied_flow,argument}
    }
    pub(in crate::analysis) fn finish_default_work(&mut self,context:&mut Context,value:&mut Expr,frame:DefaultFrame,actual:Type)->Result<(),Diagnostic>{
        let actual=if let Some(argument)=frame.argument{self.finish_parameter_argument(context,value,argument,actual)?}else{actual};
        self.require(actual,Some(frame.declared),value.span)?;
        for(binding,supplied)in context.bindings.iter_mut().zip(frame.prior){supplied.join(Availability::of(binding)).restore(binding);}
        context.flow=crate::flow::join(&frame.supplied_flow,&context.flow);
        Ok(())
    }
}
