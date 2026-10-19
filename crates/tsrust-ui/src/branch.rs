//! Private transparent slots own lazy branches without adding layout or accessibility nodes.
use super::{Kind,Node};

// Static document-only builds retain factories without a refresh loop.
#[cfg_attr(not(any(test,feature="native",all(target_arch="wasm32",feature="ui-web"))),allow(dead_code))]
pub(crate) struct Branch {
    selected:bool,
    predicate:Box<dyn FnMut()->Result<bool,String>>,
    present:Box<dyn FnMut()->Result<Option<Node>,String>>,
    absent:Box<dyn FnMut()->Result<Option<Node>,String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Child,Signal,TextPart};
    use std::{cell::RefCell,rc::Rc};
    struct Probe(Rc<RefCell<usize>>);
    impl Drop for Probe{fn drop(&mut self){*self.0.borrow_mut()+=1;}}
    #[test]
    fn branches_are_lazy_keep_live_state_and_retire_identities_and_resources_once() {
        let show=Signal::new(true);let select=show.handle();let control=show.handle();let released=Rc::new(RefCell::new(0));let created=Rc::new(RefCell::new(0));let builds=created.clone();let drops=released.clone();
        let branch=Node::branch(move||select.get(),move||{
            *builds.borrow_mut()+=1;let count=Signal::new(1);let read=count.handle();let press=count.handle();let probe=Probe(drops.clone());
            Ok(Some(Node::button_children(vec![Child::Text(TextPart::Signal(read.text_binding(i64::to_string)))],String::new(),false,Some(Box::new(move||{let _=&probe;press.set(press.get()?+1)})))?.own_state(count)?))
        },||Ok(None)).unwrap();
        let input=Node::input_control(None,Some("stable".into()),"{}",String::new(),None).unwrap();
        let mut root=Node::view(vec![Child::Node(branch),Child::Node(input)],"flex-row gap-4".into()).unwrap().own_state(show).unwrap();root.refresh().unwrap();
        let old=root.children[0].visible_root().unwrap().identity.get();let stable=root.children[1].identity.get();
        assert!(root.activate(old).unwrap());root.refresh().unwrap();assert_eq!(root.children[0].visible_root().unwrap().content_text(),"2");assert_eq!(*created.borrow(),1);
        assert!(root.input(stable,"edited").unwrap());control.set(false).unwrap();root.refresh().unwrap();
        assert_eq!(*released.borrow(),1);assert_eq!(root.visible_children().count(),1);assert_eq!(root.children[1].identity.get(),stable);assert_eq!(root.children[1].text,"edited");assert!(!root.activate(old).unwrap());
        control.set(true).unwrap();root.refresh().unwrap();let fresh=root.children[0].visible_root().unwrap().identity.get();assert!(fresh>stable&&fresh!=old);assert_eq!(*created.borrow(),2);assert_eq!(root.children[0].visible_root().unwrap().content_text(),"1");
        assert!(root.activate(fresh).unwrap());root.refresh().unwrap();assert_eq!(root.children[0].visible_root().unwrap().content_text(),"2");
        drop(root);assert_eq!(*released.borrow(),2);assert!(control.get().is_err());
    }
    #[test]
    fn root_slots_are_transparent_and_failed_replacements_drop_the_previous_branch() {
        let show=Signal::new(true);let select=show.handle();let control=show.handle();let released=Rc::new(RefCell::new(0));let drops=released.clone();
        let mut root=Node::branch(move||select.get(),move||{let probe=Probe(drops.clone());Ok(Some(Node::button_children(vec![],String::new(),false,Some(Box::new(move||{let _=&probe;Ok(())})))?))},||Err("Branch construction failed".into())).unwrap().own_state(show).unwrap();
        root.refresh().unwrap();let html=crate::document(&root,"Branch");assert_eq!(html.matches("data-tsrust-node=\"").count(),1);assert!(!html.contains(" data-tsrust-view"));
        control.set(false).unwrap();assert_eq!(root.refresh().unwrap_err(),"Branch construction failed");assert_eq!(*released.borrow(),1);drop(root);assert_eq!(*released.borrow(),1);assert!(control.get().is_err());
    }
    #[test]
    fn changed_branches_revalidate_button_label_separator_and_hidden_ancestry() {
        let show=Signal::new(false);let select=show.handle();let control=show.handle();
        let branch=Node::branch(move||select.get(),||Ok(Some(Node::button_children(vec![],String::new(),false,None)?)),||Ok(None)).unwrap();
        let mut root=Node::button_children(vec![Child::Node(branch)],String::new(),false,None).unwrap();control.set(true).unwrap();assert!(root.refresh().unwrap_err().contains("noninteractive phrasing"));
        for metadata in [r#"{"tag":"label"}"#,r#"{"role":"separator"}"#,r#"{"hidden":true}"#] {
            let show=Signal::new(false);let select=show.handle();let mut children=vec![];
            if metadata.contains("label"){children.push(Child::Node(Node::input_control(None,None,"{}",String::new(),None).unwrap()));}
            children.push(Child::Node(Node::branch(move||select.get(),||Ok(Some(Node::input_control(None,None,"{}",String::new(),None)?)),||Ok(None)).unwrap()));
            let mut root=Node::view(children,String::new()).unwrap().with_semantics(metadata).unwrap();show.set(true).unwrap();assert!(root.refresh().is_err(),"{metadata}");
        }
    }
}
impl Node {
    pub fn branch(
        mut predicate:impl FnMut()->Result<bool,String>+'static,
        mut present:impl FnMut()->Result<Option<Node>,String>+'static,
        mut absent:impl FnMut()->Result<Option<Node>,String>+'static,
    )->Result<Self,String> {
        let selected=predicate()?;let child=if selected{present()?}else{absent()?};
        let mut node=Self::new(Kind::View,String::new(),String::new(),vec![])?;
        node.children=child.into_iter().collect();node.depth=node.children.first().map_or(0,|child|child.depth);for child in &node.children{child.reset_identities();}
        node.branch=Some(Branch{selected,predicate:Box::new(predicate),present:Box::new(present),absent:Box::new(absent)});Ok(node)
    }
    #[cfg(any(test,feature="native",all(target_arch="wasm32",feature="ui-web")))]
    pub(crate) fn refresh_branch(&mut self)->Result<usize,String> {
        let Some(branch)=&mut self.branch else{return Ok(0)};
        let selected=(branch.predicate)()?;
        if selected==branch.selected{return Ok(0);}
        self.children.clear();branch.selected=selected;
        if let Some(mut child)=if selected{(branch.present)()?}else{(branch.absent)()?} {
            child.reset_identities();if child.theme.is_none(){child.theme=self.theme.clone();child.theme_inherited=child.theme.is_some();child.inherit_branch_theme();}
            self.children.push(child);
        }
        Ok(1)
    }
    pub(crate) fn inherit_branch_theme(&mut self) {
        if self.branch.is_some(){if let Some(child)=self.children.first_mut(){if child.theme.is_none()||child.theme_inherited{child.theme=self.theme.clone();child.theme_inherited=child.theme.is_some();child.inherit_branch_theme();}}}
    }
    pub(crate) fn visible_root(&self)->Option<&Node> {
        if self.collapsed{None}else if self.branch.is_some(){self.children.first().and_then(Self::visible_root)}else if self.anonymous_text&&self.text.is_empty(){None}else{Some(self)}
    }
    #[cfg(any(test,feature="native"))]
    pub(crate) fn window_drag_at(&self,contains:impl Fn(u64)->bool)->bool {
        fn visit(node:&Node,inherited:bool,blocked:bool,contains:&impl Fn(u64)->bool)->Option<bool> {
            if node.collapsed{return None;}
            let blocked=blocked||node.semantics.tab_index.is_some()||matches!(node.kind,Kind::Button|Kind::DisabledButton|Kind::Input|Kind::Textarea)||node.tag()=="label";
            let drag=(inherited||node.semantics.window_drag_region)&&!blocked;
            for child in node.children.iter().rev(){if let Some(hit)=visit(child,drag,blocked,contains){return Some(hit);}}
            if node.branch.is_none()&&contains(node.identity.get()){Some(drag)}else{None}
        }
        visit(self,false,false,&contains).unwrap_or(false)
    }
    pub(crate) fn visible_children(&self)->impl Iterator<Item=&Node> {self.children.iter().filter_map(Self::visible_root)}
    pub(crate) fn reset_identities(&self) {
        self.identity.set(0);self.next_identity.set(0);for child in &self.children{child.reset_identities();}
    }
    pub(crate) fn ensure_identities(&self) {
        fn assign(node:&Node,next:&mut u64,reset:bool) {
            if node.branch.is_none() && (reset||node.identity.get()==0) {
                assert!(*next<(1u64<<63),"UI node identities exhausted");node.identity.set(*next);*next+=1;
            }
            for child in &node.children{assign(child,next,reset);}
        }
        let reset=self.next_identity.get()==0;let mut next=self.next_identity.get().max(1);
        assign(self,&mut next,reset);self.next_identity.set(next);
    }
    pub(crate) fn phrasing(&self)->bool {
        self.visible_root().is_none_or(|node|matches!(node.tag(),"span"|"code"|"kbd"|"strong"|"em"|"del")
            && matches!(node.semantics.role.as_str(),""|"none"|"presentation")
            && node.semantics.tab_index.is_none() && !matches!(node.kind,Kind::Button|Kind::DisabledButton)
            && node.children.iter().all(Self::phrasing))
    }
    pub(crate) fn validate_structure(&self)->Result<(),String> {
        if self.kind==Kind::Text {for child in &self.children {child.validate_inline()?;}}
        if matches!(self.kind,Kind::Button|Kind::DisabledButton) && !self.children.iter().all(Self::phrasing){return Err("Button children must be noninteractive phrasing content: text, span, code or kbd".into());}
        if self.semantics.role=="separator" && (!self.text.is_empty()||self.visible_children().next().is_some()){return Err("A separator cannot contain text or children".into());}
        if self.tag()=="label" {
            fn labels(node:&Node)->bool{node.visible_root().is_some_and(|node|node.tag()=="label"||node.children.iter().any(labels))}
            fn controls(node:&Node)->usize{node.visible_root().map_or(0,|node|usize::from(matches!(node.kind,Kind::Button|Kind::DisabledButton|Kind::Input|Kind::Textarea))+node.children.iter().map(controls).sum::<usize>())}
            if self.children.iter().any(labels)||controls(self)>1{return Err("Labels cannot nest or contain more than one control".into());}
        }
        if self.semantics.hidden && self.has_enabled_controls(){return Err("An aria-hidden subtree cannot contain enabled interactive controls".into());}
        Ok(())
    }
}
