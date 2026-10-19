//! Clipboard operations belong to a control, never a global UI owner.
use crate::Node;
pub(crate) struct Copy {pub value:String,pub pending:bool,pub serial:u64,pub error:String}
pub fn button(value:String,class_name:String)->Result<Node,String>{
    let mut node=Node::button_children(vec![],class_name,false,None)?;node.text="Copy".into();
    node.copy=Some(Copy{value,pending:false,serial:0,error:String::new()});Ok(node)
}
impl Node {
    pub(crate) fn begin_copy(&mut self)->Option<(u64,String)>{
        if self.is_disabled(){return None;}
        let copy=self.copy.as_mut()?;copy.serial=copy.serial.checked_add(1).expect("Clipboard request serial exhausted");copy.pending=true;copy.error.clear();self.text="Copying…".into();Some((copy.serial,copy.value.clone()))
    }
    pub(crate) fn finish_copy(&mut self,serial:u64,result:Result<(),String>){
        let Some(copy)=&mut self.copy else{return;};if !copy.pending||copy.serial!=serial{return;}
        copy.pending=false;copy.error=result.err().unwrap_or_default();self.text=if copy.error.is_empty(){"Copied"}else{"Copy failed. Try again."}.into();
    }
    #[cfg(any(feature="native",all(target_arch="wasm32",feature="ui-web")))]
    pub(crate) fn copy_node_mut(&mut self,id:u64)->Option<&mut Node>{
        if self.collapsed{return None;}if self.identity.get()==id{return self.copy.is_some().then_some(self);}
        self.children.iter_mut().find_map(|child|child.copy_node_mut(id))
    }
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn clipboard_state_preserves_exact_source_and_rejects_stale_completion(){
  let mut node=button("café 文\r\n  ".into(),String::new()).unwrap();let (first,value)=node.begin_copy().unwrap();assert_eq!(value,"café 文\r\n  ");assert!(node.begin_copy().is_none());node.finish_copy(first,Err("permission denied".into()));assert!(node.text.contains("failed"));assert_eq!(node.copy.as_ref().unwrap().error,"permission denied");let (second,_)=node.begin_copy().unwrap();node.finish_copy(first,Ok(()));assert!(node.copy.as_ref().unwrap().pending);node.finish_copy(second,Ok(()));assert_eq!(node.text,"Copied");
 }
}
