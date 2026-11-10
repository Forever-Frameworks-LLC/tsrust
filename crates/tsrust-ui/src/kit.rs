//! GPUI Kit components: a named component with string props, live props and
//! named events. Values are strings at this boundary; the generated `std:ui`
//! wrappers check their types and the renderer parses them per component.
#![cfg_attr(not(feature = "gpui"), allow(dead_code))]
use super::{Child, Node, TextBinding};
use std::collections::BTreeMap;

pub(crate) type Predicate = std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(&str)->Result<bool,String>>>>;
pub(crate) type NumberStep = std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(f64,bool)->Result<f64,String>>>>;
pub(crate) fn predicate(handler:&Predicate,value:&str)->Result<bool,String> {
    handler.try_borrow_mut().map_err(|_|"Input predicate cannot reenter itself".to_owned())?(value)
}
pub(crate) type EventHandler = Box<dyn FnMut(&str) -> Result<(), String>>;

#[cfg(any(test,feature="gpui"))]
pub(crate) fn source_language(name:&str)->&str {
    if name.eq_ignore_ascii_case("tsr") || name.eq_ignore_ascii_case("tsrust") {"typescript"}else if name.eq_ignore_ascii_case("tsrx"){"tsx"}else{name}
}
/// Highlight Tsrust fences with the corresponding Kit grammar, preserving code bytes.
#[cfg(any(test, feature = "gpui"))]
pub(crate) fn highlighted_markdown(source: &str) -> String {
    let mut output=String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let replacement=[("```tsrx","```tsx"),("```tsr","```typescript"),("~~~tsrx","~~~tsx"),("~~~tsr","~~~typescript")].into_iter().find_map(|(prefix,replacement)| {
            line.strip_prefix(prefix).filter(|tail|tail.is_empty() || tail.starts_with(char::is_whitespace)).map(|tail|(replacement,tail))
        });
        match replacement {Some((prefix,tail))=>{output.push_str(prefix);output.push_str(tail);},None=>output.push_str(line)}
    }
    output
}

#[cfg(test)]
mod markdown_tests {
    #[test]
    fn source_names_select_existing_grammars_without_renaming_other_languages() {
        for name in ["tsr","TSR","tsrust"] {assert_eq!(super::source_language(name),"typescript");}
        assert_eq!(super::source_language("tsrx"),"tsx");assert_eq!(super::source_language("rust"),"rust");
    }
    #[test]
    fn tsrust_fences_use_kit_grammars_without_changing_code_or_other_languages() {
        let source="```tsrx\r\n<Text>hello</Text>\r\n```\r\n```tsr\nconst x = 1;\n```\n```bash\ntsrust dev\n```\ninline ```tsr\n~~~tsrust\n";
        assert_eq!(super::highlighted_markdown(source),source.replacen("```tsrx\r\n","```tsx\r\n",1).replacen("```tsr\nconst","```typescript\nconst",1));
    }
}

pub(crate) struct Kit {
    pub(crate) name: String,
    pub(crate) props: BTreeMap<String, String>,
    bindings: Vec<(String, TextBinding)>,
    pub(crate) events: Vec<(String, EventHandler)>,
    pub(crate) predicates: BTreeMap<String,Predicate>,
    pub(crate) number_step: Option<NumberStep>,
    /// State props the component keeps itself because the caller left them unset.
    local: Vec<&'static str>,
}

/// The prop a component's change event updates when the caller leaves it unset,
/// like an uncontrolled React input; a set or bound prop stays caller-controlled.
fn state_prop(component: &str) -> Option<(&'static str, &'static str)> {
    match component {
        "Checkbox" | "Switch" | "Radio" | "Toggle" => Some(("checked", "change")), "RadioGroup" => Some(("selected", "change")),
        "Rating" | "NumberInput" | "Select" | "Combobox" | "Tabs" | "Accordion" => Some(("value", "change")),
        "Dialog" | "Sheet" | "AlertDialog" | "Collapsible" => Some(("open", "openChange")),
        "Pagination" => Some(("page", "change")), "Stepper" => Some(("selected", "change")),
        _ => None,
    }
}

impl Kit {
    fn validate_prop(&self,name:&str,value:&str)->Result<(),String> {
        if matches!(self.name.as_str(),"DiffAnnotation"|"DiffDecoration"|"DiffGutterMarker") && matches!(name,"line"|"start"|"end") && !value.parse::<usize>().is_ok_and(|value|value>=1) {return Err(format!("{} {name} requires a one-based integer >= 1",self.name));}
        if self.name=="NumberInput" && matches!(name,"value"|"defaultValue"|"step"|"min"|"max") && !value.parse::<f64>().is_ok_and(|value|value.is_finite() && (name!="step" || value>=0.)) {return Err(format!("NumberInput {name} requires a finite {}number",if name=="step"{"nonnegative "}else{""}));}
        if self.name=="Diff" {
            if name=="contextLines" && !value.parse::<i64>().is_ok_and(|value|value>=-1) {return Err("Diff contextLines requires -1 or a nonnegative integer".into());}
            if matches!(name,"expansionLines"|"minCollapsedLines"|"inlineMaxLineLength"|"syntaxMaxLineLength") && !value.parse::<usize>().is_ok_and(|value|value>=1) {return Err(format!("Diff {name} requires an integer >= 1"));}
        }
        if self.name=="CodeEditor" && matches!(name,"tabSize"|"scrollBeyondLastLine"|"cursorSurroundingLines") {
            let minimum=usize::from(name=="tabSize");
            if !value.parse::<usize>().is_ok_and(|value|value>=minimum) {return Err(format!("CodeEditor {name} requires an integer >= {minimum}"));}
        }
        if self.name=="Plot" && name=="labels" && serde_json::from_str::<Vec<String>>(value).is_err() {return Err("Plot labels require an array of strings".into());}
        if matches!(self.name.as_str(),"PlotLine"|"PlotArea"|"PlotBars") && name=="values" {
            if !value.is_empty() && value.split('\n').any(|value|!value.parse::<f64>().is_ok_and(f64::is_finite)) {return Err(format!("{} values must all be finite numbers",self.name));}
        }
        if (self.name=="PlotLine" && name=="width") || (self.name=="PlotTooltip" && matches!(name,"gap"|"progress")) {
            if !value.parse::<f64>().is_ok_and(|value|value.is_finite() && (name=="progress" || value>=0. && value<=f32::MAX as f64)) {return Err(format!("{} {name} requires a finite {}number within native pixel range",self.name,if name=="progress" {""}else{"nonnegative "}));}
        }
        Ok(())
    }
    pub(crate) fn prop(&self, name: &str) -> Option<&str> {self.props.get(name).map(String::as_str)}
    pub(crate) fn text(&self, name: &str) -> String {self.prop(name).unwrap_or_default().to_owned()}
    pub(crate) fn flag(&self, name: &str) -> bool {self.prop(name) == Some("true")}
    pub(crate) fn number(&self, name: &str) -> Option<f64> {self.prop(name).and_then(|value| value.parse().ok())}
    pub(crate) fn index(&self, name: &str) -> Option<usize> {self.number(name).filter(|value| *value >= 0.).map(|value| value as usize)}
    pub(crate) fn handles(&self, event: &str) -> bool {self.events.iter().any(|(name, _)| name == event)}
}

/// The prop a component's text children fill, so `<Label>Name <Text variant="secondary">(optional)</Text></Label>`
/// composes as TSRX. Secondary-toned Text children fill `secondary`. An explicitly set prop wins.
/// Mirrors `textChildren` in stdlib/ui/kit.json; tools/generate-ui-kit.py checks they agree.
fn text_children_prop(component: &str) -> Option<&'static str> {
    match component {
        "Label" | "ShimmerText" | "MenuLabel" | "AlertTitle" | "AlertDescription" | "SelectLabel" => Some("text"),
        "Kbd" => Some("keys"),
        "Alert" => Some("message"),
        "DescriptionItem" => Some("value"),
        "Checkbox" | "Switch" | "Radio" | "Toggle" | "Separator" | "BreadcrumbItem" | "Tab" | "StepperItem" | "SidebarItem" | "CommandItem" | "GroupButton" | "QuestionChoice" | "InputGroupButton" | "SelectItem" | "TreeItem" | "DataTableColumn" => Some("label"),
        _ => None,
    }
}

impl Kit {
    /// Whether the component keeps this prop itself (uncontrolled state or text children).
    pub(crate) fn is_local(&self, prop: &str) -> bool {self.local.iter().any(|name| *name == prop)}
}

impl Node {
    /// Fill a component's text prop from its text children after they refresh.
    #[cfg(any(test, feature = "gpui"))]
    pub(crate) fn refresh_text_children(&mut self) -> usize {
        let Some(kit) = self.kit.as_deref() else {return 0};
        let Some(prop) = text_children_prop(&kit.name) else {return 0};
        if self.children.is_empty() || (kit.props.contains_key(prop) && !kit.local.contains(&prop)) {return 0;}
        let (mut main, mut secondary) = (Vec::new(), Vec::new());
        // Item components (AlertTitle, EmptyMedia, ...) are slots, not text.
        for child in self.children.iter().filter(|child| child.kit.as_deref().is_none_or(|kit| kit.name.is_empty())) {
            let text = child.content_text();
            let text = text.trim();
            if text.is_empty() {continue;}
            if child.option("variant") == Some("secondary") {secondary.push(text.to_owned());} else {main.push(text.to_owned());}
        }
        let kit = self.kit.as_deref_mut().expect("kit");
        let mut changed = 0;
        for (name, parts) in [(prop, main), ("secondary", secondary)] {
            if parts.is_empty() && name == "secondary" {continue;}
            let value = parts.join(" ");
            if kit.props.get(name) != Some(&value) {kit.props.insert(name.to_owned(), value); changed += 1;}
            if !kit.local.contains(&name) {kit.local.push(name);}
        }
        changed
    }
    /// A GPUI Kit component. Children are ordinary nodes or the component's own
    /// item/slot components.
    pub fn kit(name: &str, children: Vec<Child>, class_name: String) -> Result<Self, String> {
        let mut node = Self::view(children, class_name)?;
        node.kit = Some(Box::new(Kit {name: name.to_owned(), props: BTreeMap::new(), bindings: Vec::new(), events: Vec::new(), predicates:BTreeMap::new(), number_step:None, local: Vec::new()}));
        Ok(node)
    }
    /// Kit props on a core node (such as Button options) live in an unnamed Kit record.
    fn kit_mut(&mut self) -> Result<&mut Kit, String> {
        Ok(self.kit.get_or_insert_with(|| Box::new(Kit {name: String::new(), props: BTreeMap::new(), bindings: Vec::new(), events: Vec::new(), predicates:BTreeMap::new(), number_step:None, local: Vec::new()})))
    }
    /// A Kit-style option set on this node, if any.
    pub(crate) fn option(&self, name: &str) -> Option<&str> {self.kit.as_deref().and_then(|kit| kit.prop(name))}
    pub fn with_kit_prop(mut self, name: &str, value: &str) -> Result<Self, String> {
        let kit=self.kit_mut()?;
        kit.validate_prop(name,value)?;
        kit.props.insert(name.to_owned(), value.to_owned());
        Ok(self)
    }
    /// A live prop: its value is read now and after every handler.
    pub fn with_kit_binding(mut self, name: &str, binding: TextBinding) -> Result<Self, String> {
        let value = binding.value()?;
        let kit = self.kit_mut()?;
        kit.validate_prop(name,&value)?;
        kit.props.insert(name.to_owned(), value);
        kit.bindings.push((name.to_owned(), binding));
        Ok(self)
    }
    /// A text, number or live value (signal, reactive prop or translation); None leaves the prop unset.
    pub fn with_kit_value(self, name: &str, value: Option<Box<dyn FnMut() -> Result<String, String>>>) -> Result<Self, String> {
        match value {Some(value) => self.with_kit_binding(name, TextBinding::computed(value)), None => Ok(self)}
    }
    pub fn with_kit_event(mut self, event: &str, handler: impl FnMut(&str) -> Result<(), String> + 'static) -> Result<Self, String> {
        self.kit_mut()?.events.push((event.to_owned(), Box::new(handler)));
        Ok(self)
    }
    /// Synchronous decision callbacks retain their own weak navigation context.
    pub fn with_kit_predicate(mut self,event:&str,mut handler:impl FnMut(&str)->Result<bool,String>+'static)->Result<Self,String> {
        let navigation=self.navigation.clone();
        self.kit_mut()?.predicates.insert(event.into(),std::rc::Rc::new(std::cell::RefCell::new(Box::new(move|value| {let _scope=navigation.enter();handler(value)}))));
        Ok(self)
    }
    pub fn with_kit_step(mut self,mut handler:impl FnMut(f64,bool)->Result<f64,String>+'static)->Result<Self,String> {
        let navigation=self.navigation.clone();
        self.kit_mut()?.number_step=Some(std::rc::Rc::new(std::cell::RefCell::new(Box::new(move|value,increment| {
            let _scope=navigation.enter();let step=handler(value,increment)?;
            if !step.is_finite() || step<0. {return Err("NumberInput stepBy must return a finite nonnegative step".into());}Ok(step)
        }))));
        Ok(self)
    }
    pub(crate) fn refresh_kit(&mut self) -> Result<usize, String> {
        let Some(kit) = self.kit.as_deref_mut() else {return Ok(0)};
        let mut changed = 0;
        for (name, binding) in &kit.bindings {
            let value = binding.value()?;
            kit.validate_prop(name,&value)?;
            if kit.props.get(name) != Some(&value) {kit.props.insert(name.clone(), value); changed += 1;}
        }
        Ok(changed)
    }
    /// Deliver a component event to the visible node with this identity.
        pub(crate) fn kit_event(&mut self, target: u64, event: &str, value: &str) -> Result<bool, String> {
        let _navigation=self.navigation.enter();
        self.ensure_identities();
        fn visit(node: &mut Node, target: u64, event: &str, value: &str) -> Result<bool, String> {
            if node.collapsed {return Ok(false);}
            if node.identity.get() == target {
                let Some(kit) = node.kit.as_deref_mut() else {return Ok(false)};
                let mut handled = false;
                if let Some((prop, _)) = state_prop(&kit.name).filter(|(_, name)| *name == event) {
                    let controlled = kit.bindings.iter().any(|(name, _)| name == prop) || (kit.props.contains_key(prop) && !kit.local.contains(&prop));
                    if !controlled {
                        kit.props.insert(prop.to_owned(), value.to_owned());
                        if !kit.local.contains(&prop) {kit.local.push(prop);}
                        handled = true;
                    }
                }
                for (name, handler) in &mut kit.events {if name == event {handler(value)?; return Ok(true);}}
                return Ok(handled);
            }
            for child in &mut node.children {if visit(child, target, event, value)? {return Ok(true);}}
            Ok(false)
        }
        visit(self, target, event, value)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Child, Node, Signal};
    #[test]
    fn kit_props_refresh_from_bindings_and_events_reach_their_node() {
        let checked = Signal::new(false);
        let read = checked.handle();
        let write = checked.handle();
        let node = Node::kit("Checkbox", vec![], String::new()).unwrap()
            .with_kit_prop("label", "Accept").unwrap()
            .with_kit_binding("checked", read.text_binding(bool::to_string)).unwrap()
            .with_kit_event("change", move |value| write.set(value == "true")).unwrap();
        let mut root = Node::view(vec![Child::Node(node)], String::new()).unwrap().own_state(checked).unwrap();
        root.refresh().unwrap();
        let id = root.children[0].identity.get();
        assert_eq!(root.children[0].kit.as_ref().unwrap().prop("checked"), Some("false"));
        assert!(root.kit_event(id, "change", "true").unwrap());
        assert!(!root.kit_event(id, "missing", "true").unwrap());
        root.refresh().unwrap();
        let kit = root.children[0].kit.as_ref().unwrap();
        assert!(kit.flag("checked"));
        assert_eq!(kit.text("label"), "Accept");
    }

    #[test]
    fn text_children_fill_the_component_text() {
        let main = Node::text(vec![super::super::TextPart::Literal("Company Address".into())], String::new()).unwrap();
        let note = Node::text(vec![super::super::TextPart::Literal("(optional)".into())], String::new()).unwrap().with_kit_prop("variant", "secondary").unwrap();
        let mut label = Node::kit("Label", vec![Child::Node(main), Child::Node(note)], String::new()).unwrap();
        label.refresh().unwrap();
        let kit = label.kit.as_ref().unwrap();
        assert_eq!(kit.text("text"), "Company Address");
        assert_eq!(kit.text("secondary"), "(optional)");
    }

    #[test]
    fn unset_state_props_are_kept_by_the_component() {
        let free = Node::kit("Checkbox", vec![], String::new()).unwrap();
        let fixed = Node::kit("Checkbox", vec![], String::new()).unwrap().with_kit_prop("checked", "false").unwrap();
        let mut root = Node::view(vec![Child::Node(free), Child::Node(fixed)], String::new()).unwrap();
        root.ensure_identities();
        let (free, fixed) = (root.children[0].identity.get(), root.children[1].identity.get());
        assert!(root.kit_event(free, "change", "true").unwrap());
        assert!(root.kit_event(free, "change", "false").unwrap());
        assert!(!root.kit_event(fixed, "change", "true").unwrap());
        assert_eq!(root.children[0].kit.as_ref().unwrap().prop("checked"), Some("false"));
        assert_eq!(root.children[1].kit.as_ref().unwrap().prop("checked"), Some("false"));
    }
}

#[cfg(test)]
mod plot_value_tests {
 use super::*;
 #[test]
 fn plot_values_and_tooltip_geometry_reject_nonfinite_or_negative_inputs() {
  for component in ["PlotLine","PlotArea","PlotBars"] {
   for invalid in ["NaN","inf","1\n-inf","1\nno"] {assert!(Node::kit(component,vec![],String::new()).unwrap().with_kit_prop("values",invalid).is_err());}
   assert!(Node::kit(component,vec![],String::new()).unwrap().with_kit_prop("values","-2\n0\n3.5").is_ok());
  }
  for (component,prop) in [("PlotLine","width"),("PlotTooltip","gap")] {
   for invalid in ["-1","NaN","inf","1e100"] {assert!(Node::kit(component,vec![],String::new()).unwrap().with_kit_prop(prop,invalid).is_err());}
  }
  assert!(Node::kit("PlotTooltip",vec![],String::new()).unwrap().with_kit_prop("progress","-1").is_ok());
 }
}

#[cfg(test)]
mod diff_value_tests {
 use super::*;
 #[test]
 fn diff_counts_reject_invalid_geometry_before_mounting() {
  for prop in ["expansionLines","minCollapsedLines","inlineMaxLineLength","syntaxMaxLineLength"] {
   for invalid in ["-1","0","1.5","NaN","18446744073709551616"] {assert!(Node::kit("Diff",vec![],String::new()).unwrap().with_kit_prop(prop,invalid).is_err(),"{prop} {invalid}");}
   assert!(Node::kit("Diff",vec![],String::new()).unwrap().with_kit_prop(prop,"1").is_ok());
  }
  for (component,prop) in [("DiffAnnotation","line"),("DiffDecoration","start"),("DiffDecoration","end"),("DiffGutterMarker","line")] {
   assert!(Node::kit(component,vec![],String::new()).unwrap().with_kit_prop(prop,"0").is_err());
   assert!(Node::kit(component,vec![],String::new()).unwrap().with_kit_prop(prop,"1").is_ok());
  }
  for valid in ["-1","0","3"] {assert!(Node::kit("Diff",vec![],String::new()).unwrap().with_kit_prop("contextLines",valid).is_ok());}
  for invalid in ["-2","NaN","1.5"] {assert!(Node::kit("Diff",vec![],String::new()).unwrap().with_kit_prop("contextLines",invalid).is_err());}
 }
}

#[cfg(test)]
mod callback_tests {
 use super::*;
 use crate::Signal;
 #[test]
 fn synchronous_predicates_keep_errors_and_release_owned_state() {
  let policy=Signal::new(3_i64);let read=policy.handle();let write=policy.handle();
  let node=Node::kit("Input",vec![],String::new()).unwrap().with_kit_predicate("validate",move|value|Ok(value.len() as i64<=read.get()?)).unwrap().own_state(policy).unwrap();
  let callback=node.kit.as_ref().unwrap().predicates["validate"].clone();
  assert!(predicate(&callback,"abc").unwrap());assert!(!predicate(&callback,"abcd").unwrap());
  write.set(4).unwrap();assert!(predicate(&callback,"abcd").unwrap());
  {let _borrow=callback.borrow_mut();assert!(predicate(&callback,"x").unwrap_err().contains("reenter"));}
  drop(node);assert!(predicate(&callback,"x").is_err());
  let node=Node::kit("Input",vec![],String::new()).unwrap().with_kit_predicate("paste",|_|Err("paste failed".into())).unwrap();
  assert_eq!(predicate(&node.kit.as_ref().unwrap().predicates["paste"],"text").unwrap_err(),"paste failed");
 }
 #[test]
 fn numeric_steps_keep_both_directions_and_reject_invalid_results() {
  let node=Node::kit("NumberInput",vec![],String::new()).unwrap().with_kit_step(|value,increment|Ok(if increment && value<1. {0.1}else{0.5})).unwrap();
  let callback=node.kit.as_ref().unwrap().number_step.as_ref().unwrap();
  assert_eq!(callback.borrow_mut()(0.5,true).unwrap(),0.1);assert_eq!(callback.borrow_mut()(1.,false).unwrap(),0.5);
  for step in [f64::NAN,f64::INFINITY,-1.] {
   let node=Node::kit("NumberInput",vec![],String::new()).unwrap().with_kit_step(move|_,_|Ok(step)).unwrap();
   assert!(node.kit.as_ref().unwrap().number_step.as_ref().unwrap().borrow_mut()(0.,true).is_err());
  }
  let node=Node::kit("NumberInput",vec![],String::new()).unwrap().with_kit_step(|_,_|Ok(0.)).unwrap();
  assert_eq!(node.kit.as_ref().unwrap().number_step.as_ref().unwrap().borrow_mut()(1.,true).unwrap(),0.);
 }
}

// Shared normalization keeps painting and hover dots on the same finite scale.
#[cfg(any(test,feature="gpui"))]
pub(crate) fn plot_y(value:f64,low:f64,high:f64,height:f32)->f32 {
 let scale=low.abs().max(high.abs()).max(1.);
 let span=high/scale-low/scale;
 if span>0. {(height as f64*(1.-(value/scale-low/scale)/span)) as f32}else{height}
}
#[cfg(any(test,feature="gpui"))]
pub(crate) fn plot_index(position:f32,width:f32,count:usize)->Option<usize> {
 if count==0 || width<=0. || !width.is_finite() || !position.is_finite() {return None;}
 Some(((position.max(0.)/width*count as f32) as usize).min(count-1))
}
#[cfg(any(test,feature="gpui"))]
pub(crate) fn plot_bar_x(center:f32,step:f32,index:usize,count:usize)->f32 {
 center-step*0.35+step*0.7/count.max(1) as f32*(index as f32+0.5)
}
#[cfg(test)]
mod plot_geometry_tests {
 use super::*;
 #[test]
 fn hover_edges_missing_categories_and_extreme_values_stay_bounded() {
  assert_eq!(plot_index(0.,100.,4),Some(0));assert_eq!(plot_index(50.,100.,4),Some(2));
  assert_eq!(plot_index(100.,100.,4),Some(3));assert_eq!(plot_index(-1.,100.,4),Some(0));
  for (width,count) in [(0.,4),(100.,0)] {assert_eq!(plot_index(10.,width,count),None);}
  assert_eq!(plot_bar_x(50.,100.,0,2),32.5);assert_eq!(plot_bar_x(50.,100.,1,2),67.5);
  assert_eq!(plot_y(-1e308,-1e308,1e308,100.),100.);
  assert_eq!(plot_y(0.,-1e308,1e308,100.),50.);
  assert_eq!(plot_y(1e308,-1e308,1e308,100.),0.);
  assert_eq!(plot_y(0.,0.,0.,100.),100.);
 }
}
