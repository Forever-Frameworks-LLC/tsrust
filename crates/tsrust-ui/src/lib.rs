//! Owned semantic nodes shared by the web and native renderers.
//! This static slice has no event callbacks or executor authority.

#[cfg(feature = "native")]
mod native;
mod style;
mod web;
#[cfg(feature = "native")]
pub use native::run;
#[cfg(feature = "acceptance")]
#[doc(hidden)]
pub use native::verify_window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    View,
    Text,
    DisabledButton,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    depth: usize,
    pub(crate) kind: Kind,
    pub(crate) class_name: String,
    pub(crate) text: String,
    pub(crate) children: Vec<Node>,
}

impl Node {
    pub fn new(
        kind: Kind,
        class_name: String,
        text: String,
        children: Vec<Node>,
    ) -> Result<Self, String> {
        let depth = 1 + children.iter().map(|child| child.depth).max().unwrap_or(0);
        if depth > 128 {
            return Err("UI nesting exceeds 128 nodes".into());
        }
        let mut categories = std::collections::BTreeSet::new();
        for token in class_name.split_whitespace() {
            let utility = style::Utility::parse(token)
                .ok_or_else(|| format!("Unsupported UI utility: {token}"))?;
            if !categories.insert(utility.category()) {
                return Err(format!("Conflicting UI utility: {token}"));
            }
        }
        match kind {
            Kind::View if !text.is_empty() => return Err("View text must be inside Text".into()),
            Kind::Text | Kind::DisabledButton if !children.is_empty() => {
                return Err("Text and Button accept text children only".into());
            }
            _ => {}
        }
        let class_name = class_name.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(Self {
            depth,
            kind,
            class_name,
            text,
            children,
        })
    }
}

pub fn document(root: &Node, title: &str) -> String {
    web::document(root, title)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_shape_conflicting_utilities_and_depth_reject() {
        assert!(
            Node::new(
                Kind::View,
                "flex-row flex-col".into(),
                String::new(),
                vec![]
            )
            .is_err()
        );
        assert!(Node::new(Kind::View, String::new(), "unwrapped text".into(), vec![]).is_err());
        let leaf = Node::new(Kind::Text, String::new(), "text".into(), vec![]).unwrap();
        assert!(
            Node::new(
                Kind::DisabledButton,
                String::new(),
                String::new(),
                vec![leaf.clone()]
            )
            .is_err()
        );
        let mut node = leaf;
        for _ in 1..128 {
            node = Node::new(Kind::View, String::new(), String::new(), vec![node]).unwrap();
        }
        assert!(Node::new(Kind::View, String::new(), String::new(), vec![node]).is_err());
    }
}
