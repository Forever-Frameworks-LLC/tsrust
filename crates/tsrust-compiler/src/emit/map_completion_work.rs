//! Ordered Map cleanup requests; phase setup runs after earlier scopes restore.
use super::*;
use std::rc::Rc;
use statement_work::Work;
type Layout=std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>;
pub(super) struct Context<'a> {
    pub(super) fun:&'a CheckedFunction,
    pub(super) region:&'a ExceptionRegion,
    pub(super) declarations:Rc<Vec<&'a Stmt>>,
    pub(super) name:String,
    pub(super) contract:crate::error_effects::FailureSet,
    pub(super) runtime:Option<Type>,
    pub(super) places:Vec<String>,
    pub(super) storage:String,
    pub(super) pattern:String,
    pub(super) span:crate::Span,
}
pub(super) enum Phase<'a> {
    Cleanup{planned:&'a CheckedFunction,body:&'a [Stmt],id:usize,normal:Option<&'a Layout>,code:String,override_drop:String,fields:Option<String>},
    Return(usize),
    Control(crate::completion::CompletionKind),
    Text(String),
}
impl Emitter<'_> {
    pub(super) fn map_cleanup_step<'a,'s:'a>(&'s self,context:Rc<Context<'a>>,phase:Phase<'a>)->Work<'a,'s> {
        Work::Resume(Box::new(move |output,work,_| {
            let c=&*context;let declarations=Declarations::Visible(&c.declarations);
            match phase {
                Phase::Cleanup{planned,body,id,normal,code,override_drop,fields}=>self.schedule_map_finish(c.fun,planned,c.region,body,declarations,id,&c.name,&c.contract,c.runtime,&c.places,&c.storage,&c.pattern,c.span,normal,&code,&override_drop,fields.as_deref(),work),
                Phase::Return(id)=>self.schedule_map_return_cleanup(c.fun,c.region,declarations,id,&c.name,&c.contract,c.runtime,&c.places,&c.storage,&c.pattern,c.span,work),
                Phase::Control(kind)=>self.schedule_map_control_cleanup(c.fun,c.region,declarations,kind,&c.name,&c.contract,c.runtime,&c.places,&c.storage,&c.pattern,c.span,work),
                Phase::Text(code)=>output.push_str(&code),
            }
        }))
    }
    pub(super) fn queue_map_steps<'a,'s:'a>(&'s self,steps:Vec<Work<'a,'s>>,work:&mut Vec<Work<'a,'s>>) {
        for step in steps.into_iter().rev(){work.push(Work::Captured);work.push(step);work.push(Work::Capture);}
    }
}
