//! GPUI Kit components: a named component with string props, live props and
//! named events. Values are strings at this boundary; the generated `std:ui`
//! wrappers check their types and the renderer parses them per component.
#![cfg_attr(not(feature = "gpui"), allow(dead_code))]
use super::{Child, Node, TextBinding};
use std::collections::BTreeMap;

pub(crate) type EventHandler = Box<dyn FnMut(&str) -> Result<(), String>>;

pub(crate) struct Kit {
    pub(crate) name: String,
    pub(crate) props: BTreeMap<String, String>,
    bindings: Vec<(String, TextBinding)>,
    pub(crate) events: Vec<(String, EventHandler)>,
    /// State props the component keeps itself because the caller left them unset.
    local: Vec<&'static str>,
}

/// The prop a component's change event updates when the caller leaves it unset,
/// like an uncontrolled React input; a set or bound prop stays caller-controlled.
fn state_prop(component: &str) -> Option<&'static str> {
    match component {"Checkbox" | "Switch" | "Radio" | "Toggle" => Some("checked"), "RadioGroup" => Some("selected"), "Rating" => Some("value"), _ => None}
}

impl Kit {
    pub(crate) fn prop(&self, name: &str) -> Option<&str> {self.props.get(name).map(String::as_str)}
    pub(crate) fn text(&self, name: &str) -> String {self.prop(name).unwrap_or_default().to_owned()}
    pub(crate) fn flag(&self, name: &str) -> bool {self.prop(name) == Some("true")}
    pub(crate) fn number(&self, name: &str) -> Option<f64> {self.prop(name).and_then(|value| value.parse().ok())}
    pub(crate) fn index(&self, name: &str) -> Option<usize> {self.number(name).filter(|value| *value >= 0.).map(|value| value as usize)}
    pub(crate) fn handles(&self, event: &str) -> bool {self.events.iter().any(|(name, _)| name == event)}
}

/// The prop a component's text children fill, so `<Label>Name <Text variant="secondary">(optional)</Text></Label>`
/// composes like JSX. Secondary-toned Text children fill `secondary`. An explicitly set prop wins.
/// Mirrors `textChildren` in stdlib/ui/kit.json; tools/generate-ui-kit.py checks they agree.
fn text_children_prop(component: &str) -> Option<&'static str> {
    match component {
        "Label" | "ShimmerText" | "MenuLabel" | "AlertTitle" | "AlertDescription" => Some("text"),
        "Kbd" => Some("keys"),
        "Alert" => Some("message"),
        "DescriptionItem" => Some("value"),
        "Checkbox" | "Switch" | "Radio" | "Toggle" | "Separator" | "BreadcrumbItem" | "Tab" | "StepperItem" | "SidebarItem" | "CommandItem" | "GroupButton" | "QuestionChoice" => Some("label"),
        _ => None,
    }
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
        node.kit = Some(Box::new(Kit {name: name.to_owned(), props: BTreeMap::new(), bindings: Vec::new(), events: Vec::new(), local: Vec::new()}));
        Ok(node)
    }
    /// Kit props on a core node (such as Button options) live in an unnamed Kit record.
    fn kit_mut(&mut self) -> Result<&mut Kit, String> {
        Ok(self.kit.get_or_insert_with(|| Box::new(Kit {name: String::new(), props: BTreeMap::new(), bindings: Vec::new(), events: Vec::new(), local: Vec::new()})))
    }
    /// A Kit-style option set on this node, if any.
    pub(crate) fn option(&self, name: &str) -> Option<&str> {self.kit.as_deref().and_then(|kit| kit.prop(name))}
    pub fn with_kit_prop(mut self, name: &str, value: &str) -> Result<Self, String> {
        self.kit_mut()?.props.insert(name.to_owned(), value.to_owned());
        Ok(self)
    }
    /// A live prop: its value is read now and after every handler.
    pub fn with_kit_binding(mut self, name: &str, binding: TextBinding) -> Result<Self, String> {
        let value = binding.value()?;
        let kit = self.kit_mut()?;
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
    pub(crate) fn refresh_kit(&mut self) -> Result<usize, String> {
        let Some(kit) = self.kit.as_deref_mut() else {return Ok(0)};
        let mut changed = 0;
        for (name, binding) in &kit.bindings {
            let value = binding.value()?;
            if kit.props.get(name) != Some(&value) {kit.props.insert(name.clone(), value); changed += 1;}
        }
        Ok(changed)
    }
    /// Deliver a component event to the visible node with this identity.
        pub(crate) fn kit_event(&mut self, target: u64, event: &str, value: &str) -> Result<bool, String> {
        self.ensure_identities();
        fn visit(node: &mut Node, target: u64, event: &str, value: &str) -> Result<bool, String> {
            if node.collapsed {return Ok(false);}
            if node.identity.get() == target {
                let Some(kit) = node.kit.as_deref_mut() else {return Ok(false)};
                let mut handled = false;
                if let Some(prop) = state_prop(&kit.name).filter(|_| event == "change") {
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
