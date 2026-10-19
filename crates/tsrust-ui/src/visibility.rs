//! Rectangle visibility reports transitions without owning renderer resources.
use crate::{Kind,Node};

pub(crate) struct Observation {
    previous:Option<bool>,
    changed:Box<dyn FnMut(bool)->Result<(),String>>,
}
impl Node {
    pub fn with_visibility(mut self,changed:impl FnMut(bool)->Result<(),String>+'static)->Result<Self,String> {
        if self.kind!=Kind::View{return Err("Visibility handlers require View".into());}
        self.visibility=Some(Observation{previous:None,changed:Box::new(changed)});Ok(self)
    }
    pub(crate) fn observed_nodes(&self,visible_only:bool,ids:&mut Vec<u64>) {
        if visible_only && self.collapsed{return;}
        if self.visibility.is_some(){ids.push(self.identity.get());}
        for child in &self.children{child.observed_nodes(visible_only,ids);}
    }
    pub(crate) fn visibility_changed(&mut self,target:u64,visible:bool)->Result<bool,String> {
        if self.identity.get()==target {
            if let Some(observer)=&mut self.visibility {
                if observer.previous==Some(visible){return Ok(false);}
                observer.previous=Some(visible);(observer.changed)(visible)?;return Ok(true);
            }
        }
        for child in &mut self.children{if child.visibility_changed(target,visible)?{return Ok(true);}}
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{rc::Rc,cell::RefCell};
    #[test]fn observation_reports_initial_state_transitions_and_releases_captures(){
        let seen=Rc::new(RefCell::new(Vec::new()));let output=seen.clone();let weak=Rc::downgrade(&output);
        let mut node=Node::view(vec![],String::new()).unwrap().with_visibility(move|visible|{output.borrow_mut().push(visible);Ok(())}).unwrap();node.ensure_identities();let id=node.identity.get();
        for value in [false,false,true,true,false]{node.visibility_changed(id,value).unwrap();}
        assert_eq!(*seen.borrow(),[false,true,false]);drop(seen);assert!(weak.upgrade().is_some());drop(node);assert!(weak.upgrade().is_none());
    }
}
