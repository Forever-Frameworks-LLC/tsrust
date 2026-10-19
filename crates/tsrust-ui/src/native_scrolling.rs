//! Scroll offsets stay in logical coordinates until painting or accessibility export.
use super::*;

pub(super) struct Region {
    pub viewport: viewport::Viewport,
    pub inset: (f64, f64),
    pub wheel: bool,
}
impl Painter {
    pub(super) fn retain_owned_scrolls(&mut self, node: &Node) {
        fn collect(node: &Node, live: &mut std::collections::BTreeSet<u64>) {
            live.insert(node.identity.get());
            for child in &node.children { collect(child, live); }
        }
        let mut live = std::collections::BTreeSet::new();
        collect(node, &mut live);
        self.scrolls.retain(|id, _| live.contains(id));
    }
    pub(super) fn configure_scrolls(&mut self, item: &Item, ancestors: &[u64]) -> Result<(), String> {
        let mut chain = ancestors.to_vec();
        if item.overflow != 0 {
            let layout = self.tree.layout(item.id).map_err(|error| error.to_string())?;
            let size = ((layout.size.width-layout.border.left-layout.border.right).max(0.) as f64,
                (layout.size.height-layout.border.top-layout.border.bottom).max(0.) as f64);
            let region = self.scrolls.entry(item.identity).or_insert_with(|| Region {
                viewport: Default::default(), inset: (0., 0.), wheel: false,
            });
            region.inset = (layout.border.left as f64, layout.border.top as f64);
            region.viewport.size = Some(size);
            region.viewport.content = (size.0.max(layout.scrollable_overflow_rect.right as f64),
                size.1.max(layout.scrollable_overflow_rect.bottom as f64));
            region.viewport.scroll_by(0., 0.);
            region.wheel = item.overflow == 2;
            chain.insert(0, item.identity);
            self.scroll_order.push(item.identity);
        }
        self.scroll_chains.insert(item.identity, chain.clone());
        for child in &item.children { self.configure_scrolls(child, &chain)?; }
        fn register(parts:&[Inline],chain:&[u64],map:&mut std::collections::BTreeMap<u64,Vec<u64>>){for part in parts {map.insert(part.identity,chain.to_vec());register(&part.children,chain,map);}}
        register(&item.inline,&chain,&mut self.scroll_chains);
        Ok(())
    }
    pub(super) fn visibility(&self,ids:&[u64],nodes:&[(accesskit::NodeId,accesskit::Node)],scale:f64)->Vec<(u64,bool)> {
        let bounds=nodes.iter().filter_map(|(id,node)|node.bounds().map(|bounds|(id.0,bounds))).collect::<std::collections::BTreeMap<_,_>>();
        let size=self.viewport.size.unwrap_or((0.,0.));
        ids.iter().map(|id|{
            let mut rectangle=bounds.get(id).copied().unwrap_or(accesskit::Rect::ZERO);
            let mut clip=|bounds:accesskit::Rect|{rectangle=accesskit::Rect::new(rectangle.x0.max(bounds.x0),rectangle.y0.max(bounds.y0),rectangle.x1.min(bounds.x1),rectangle.y1.min(bounds.y1));};
            clip(accesskit::Rect::new(0.,0.,size.0*scale,size.1*scale));
            for parent in self.scroll_chains.get(id).into_iter().flatten().filter(|parent|*parent!=id){
                if let (Some(bounds),Some(region))=(bounds.get(parent),self.scrolls.get(parent)) {
                    let size=region.viewport.size.unwrap_or((0.,0.));let x=bounds.x0+region.inset.0*scale;let y=bounds.y0+region.inset.1*scale;
                    clip(accesskit::Rect::new(x,y,x+size.0*scale,y+size.1*scale));
                }
            }
            (*id,rectangle.x1>rectangle.x0 && rectangle.y1>rectangle.y0)
        }).collect()
    }
    pub(super) fn scroll_offset(&self, id: u64) -> (f64, f64) {
        self.scrolls.get(&id).map(|region| region.viewport.offset).unwrap_or((0., 0.))
    }
    pub(super) fn scroll_chain(&mut self, target: Option<u64>, mut dx: f64, mut dy: f64) {
        let chain = target.and_then(|id| self.scroll_chains.get(&id)).cloned().unwrap_or_default();
        for id in chain {
            let Some(region) = self.scrolls.get_mut(&id).filter(|region| region.wheel) else { continue; };
            let before = region.viewport.offset;
            region.viewport.scroll_by(dx, dy);
            dx -= region.viewport.offset.0-before.0;
            dy -= region.viewport.offset.1-before.1;
        }
        self.viewport.scroll_by(dx, dy);
    }
    pub(super) fn reveal_scrolled(&mut self, target: u64, nodes: &[(accesskit::NodeId, accesskit::Node)], scale: f64) {
        let bounds = |id| nodes.iter().find(|(node_id, _)| node_id.0 == id).and_then(|(_, node)| node.bounds());
        let Some(rect) = bounds(target) else { return; };
        let mut rect = accesskit::Rect::new(rect.x0/scale, rect.y0/scale, rect.x1/scale, rect.y1/scale);
        let chain = self.scroll_chains.get(&target).cloned().unwrap_or_default();
        for id in chain.into_iter().filter(|id| *id != target) {
            let Some(parent) = bounds(id) else { continue; };
            let region = self.scrolls.get_mut(&id).expect("visible scroll ancestor");
            let before = region.viewport.offset;
            let origin = (parent.x0/scale+region.inset.0-before.0, parent.y0/scale+region.inset.1-before.1);
            region.viewport.reveal(accesskit::Rect::new(rect.x0-origin.0, rect.y0-origin.1, rect.x1-origin.0, rect.y1-origin.1));
            let delta = (region.viewport.offset.0-before.0, region.viewport.offset.1-before.1);
            rect = accesskit::Rect::new(rect.x0-delta.0, rect.y0-delta.1, rect.x1-delta.0, rect.y1-delta.1);
        }
        self.viewport.reveal(rect);
    }
    fn keyboard_scroll_target(&self, focus: Option<u64>) -> Option<u64> {
        focus.and_then(|id| self.scroll_chains.get(&id)).and_then(|chain| chain.iter()
            .find(|id| self.scrolls.get(id).is_some_and(|region| region.wheel)).copied())
    }
    pub(super) fn scroll_key(&mut self, focus: Option<u64>, key: &Key, shift: bool) {
        let target = self.keyboard_scroll_target(focus);
        let viewport = target.map(|id| &self.scrolls[&id].viewport).unwrap_or(&self.viewport);
        let page = viewport.size.map(|size| size.1*0.9).unwrap_or(0.);
        let destination = match key {
            Key::Named(NamedKey::Home) => Some((0., 0.)),
            Key::Named(NamedKey::End) => Some(viewport.maximum()),
            _ => None,
        };
        if let Some((x, y)) = destination {
            let viewport = target.map(|id| &mut self.scrolls.get_mut(&id).unwrap().viewport).unwrap_or(&mut self.viewport);
            viewport.scroll_to(x, y);
        } else {
            let (x, y) = match key {
                Key::Named(NamedKey::PageDown) => (0., page),
                Key::Named(NamedKey::PageUp) => (0., -page),
                Key::Named(NamedKey::Space) => (0., if shift { -page } else { page }),
                Key::Named(NamedKey::ArrowDown) => (0., 32.),
                Key::Named(NamedKey::ArrowUp) => (0., -32.),
                Key::Named(NamedKey::ArrowRight) => (32., 0.),
                Key::Named(NamedKey::ArrowLeft) => (-32., 0.),
                _ => return,
            };
            // Space activates buttons. It scrolls only an explicitly focused pane.
            if *key == Key::Named(NamedKey::Space) && !focus.is_some_and(|id| self.scroll_order.contains(&id)) { return; }
            self.scroll_chain(target, x, y);
        }
    }
    pub(super) fn scroll_action(&mut self, request: &accesskit::ActionRequest, scale: f64) -> bool {
        let id = request.target_node.0;
        let viewport = if id == 0 { &mut self.viewport } else {
            if !self.scroll_order.contains(&id) { return false; }
            let Some(region) = self.scrolls.get_mut(&id).filter(|region| region.wheel) else { return false; };
            &mut region.viewport
        };
        let size = viewport.size.unwrap_or((0., 0.));
        let step = if matches!(request.data, Some(accesskit::ActionData::ScrollUnit(accesskit::ScrollUnit::Item))) { (32., 32.) } else { size };
        match request.action {
            accesskit::Action::ScrollDown => { viewport.scroll_by(0., step.1); }
            accesskit::Action::ScrollUp => { viewport.scroll_by(0., -step.1); }
            accesskit::Action::ScrollLeft => { viewport.scroll_by(-step.0, 0.); }
            accesskit::Action::ScrollRight => { viewport.scroll_by(step.0, 0.); }
            accesskit::Action::SetScrollOffset => if let Some(accesskit::ActionData::SetScrollOffset(point)) = request.data { viewport.scroll_to(point.x/scale, point.y/scale); },
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn view(class: &str, children: Vec<Node>) -> Node {
        Node::new(Kind::View, class.into(), String::new(), children).unwrap()
    }
    fn bounds(tree: &accesskit::TreeUpdate, id: u64) -> accesskit::Rect {
        tree.nodes.iter().find(|(node, _)| node.0 == id).unwrap().1.bounds().unwrap()
    }
    #[test]
    fn nested_scroll_bubbles_unused_deltas_without_moving_window() {
        let inner = view("h-32 shrink-0 overflow-auto border p-2", vec![view("h-96 shrink-0", vec![])]);
        let outer = view("h-screen overflow-auto", vec![inner, view("h-96 shrink-0", vec![])]);
        let mut painter = Painter::new(); painter.viewport.size = Some((320., 240.));
        let (_, tree) = painter.scene(&outer, 320., 2., None).unwrap();
        let inner = outer.children[0].identity.get(); let outer = outer.identity.get();
        let inner_max = painter.scrolls[&inner].viewport.maximum().1;
        assert!(inner_max > 200.);
        assert!(painter.scrolls[&outer].viewport.maximum().1 > 0.);
        painter.scroll_chain(Some(inner), 0., inner_max+20.);
        assert_eq!(painter.scrolls[&inner].viewport.offset.1, inner_max);
        assert_eq!(painter.scrolls[&outer].viewport.offset.1, 20.);
        assert_eq!(painter.viewport.offset, (0., 0.));
        painter.scroll_chain(Some(inner), 0., -inner_max-100.);
        assert_eq!(painter.scrolls[&inner].viewport.offset, (0., 0.));
        assert_eq!(painter.scrolls[&outer].viewport.offset, (0., 0.));
        let node = &tree.nodes.iter().find(|(id, _)| id.0 == inner).unwrap().1;
        assert_eq!(node.scroll_y_max(), Some(inner_max*2.));
        assert!(node.supports_action(accesskit::Action::ScrollDown));
    }
    #[test]
    fn focus_reveal_moves_paint_hits_accessibility_and_editor_origin_together() {
        let field = Node::input_control(None, Some("draft".into()), "{}", "h-8 shrink-0".into(), None).unwrap();
        let pane = view("h-32 shrink-0 overflow-auto rounded-xl border p-2", vec![view("h-96 shrink-0", vec![]), field])
            .with_semantics(r#"{"role":"region","label":"Article","tabIndex":0}"#).unwrap();
        let root = view("h-screen overflow-hidden", vec![view("h-16 shrink-0", vec![]), pane]);
        let mut painter = Painter::new(); painter.viewport.size = Some((320., 240.));
        painter.scene(&root, 320., 2., None).unwrap();
        let pane = root.children[1].identity.get(); let input = root.children[1].children[1].identity.get();
        painter.viewport.reveal_focus = true;
        let (_, tree) = painter.scene(&root, 320., 2., Some(input)).unwrap();
        let field = bounds(&tree, input); let container = bounds(&tree, pane);
        assert!(field.y0 >= container.y0 && field.y1 <= container.y1, "{field:?} inside {container:?}");
        assert!(painter.scrolls[&pane].viewport.offset.1 > 0.);
        assert_eq!(painter.viewport.offset, (0., 0.));
        assert_eq!(painter.editors[&input].origin.1, field.y0/2.);
        let point = vello::kurbo::Point::new((field.x0+field.x1)/2., (field.y0+field.y1)/2.);
        assert!(painter.hit_regions[&input].iter().all(|path| path.contains(point)));
        assert!(!painter.hit_regions[&input].iter().all(|path| path.contains(vello::kurbo::Point::new(point.x, container.y1+2.))));
        assert_eq!(painter.control_order, vec![pane, input]);
    }
    #[test]
    fn hidden_panes_keep_offsets_but_removed_panes_release_them() {
        let hidden = crate::Signal::new(false); let read = hidden.handle();
        let pane = view("h-32 overflow-auto shrink-0", vec![view("h-96 shrink-0", vec![])])
            .with_hidden(move || read.get()).unwrap();
        let mut root = view("h-screen overflow-hidden", vec![pane]);
        let mut painter = Painter::new(); painter.viewport.size = Some((320., 240.));
        painter.scene(&root, 320., 1., None).unwrap(); let id = root.children[0].identity.get();
        painter.scroll_chain(Some(id), 0., 80.);
        hidden.set(true).unwrap(); root.refresh().unwrap(); painter.scene(&root, 320., 1., None).unwrap();
        assert_eq!(painter.scrolls[&id].viewport.offset.1, 80.);
        assert!(!painter.scroll_order.contains(&id));
        hidden.set(false).unwrap(); root.refresh().unwrap(); painter.scene(&root, 320., 1., None).unwrap();
        assert_eq!(painter.scrolls[&id].viewport.offset.1, 80.);
        root.children.clear(); painter.scene(&root, 320., 1., None).unwrap(); assert!(!painter.scrolls.contains_key(&id));assert_eq!(painter.scrolls.len(),1);
    }
    #[test]
    fn pane_keyboard_scrolling_respects_tab_order_and_hidden_wheel_policy() {
        let pane = view("h-32 overflow-auto shrink-0", vec![view("h-96 shrink-0", vec![])])
            .with_semantics(r#"{"role":"region","label":"Article","tabIndex":0}"#).unwrap();
        let hidden = view("h-32 overflow-hidden shrink-0", vec![view("h-96 shrink-0", vec![])])
            .with_semantics(r#"{"tabIndex":-1}"#).unwrap();
        let root = view("h-screen overflow-hidden", vec![pane, hidden]);
        let mut painter = Painter::new(); painter.viewport.size = Some((320., 320.));
        painter.scene(&root, 320., 1., None).unwrap(); let pane = root.children[0].identity.get(); let hidden = root.children[1].identity.get();
        assert_eq!(painter.control_order, vec![pane, hidden]); assert!(painter.tab_excluded.contains(&hidden));
        painter.scroll_key(Some(pane), &Key::Named(NamedKey::PageDown), false);
        assert!(painter.scrolls[&pane].viewport.offset.1 > 0.);
        painter.scroll_key(Some(pane), &Key::Named(NamedKey::End), false);
        assert_eq!(painter.scrolls[&pane].viewport.offset, painter.scrolls[&pane].viewport.maximum());
        painter.scroll_key(Some(pane), &Key::Named(NamedKey::Home), false);
        assert_eq!(painter.scrolls[&pane].viewport.offset, (0., 0.));
        painter.scroll_chain(Some(hidden), 0., 64.); assert_eq!(painter.scrolls[&hidden].viewport.offset, (0., 0.));
        painter.scrolls.get_mut(&hidden).unwrap().viewport.scroll_to(0., 30.);
        painter.viewport.size = Some((640., 1000.)); painter.scene(&root, 640., 1., None).unwrap();
        assert_eq!(painter.viewport.maximum(), (0., 0.));
        for metadata in [r#"{"tabIndex":1}"#, r#"{"tabIndex":-2}"#, r#"{"tabIndex":0,"hidden":true}"#] {
            assert!(view("", vec![]).with_semantics(metadata).is_err(), "{metadata}");
        }
    }
    #[test]
    fn visibility_follows_nested_clips_fractional_scale_hidden_and_resize() {
        let target=view("h-16 shrink-0",vec![]);
        let pane=view("h-24 overflow-auto shrink-0",vec![view("h-32 shrink-0",vec![]),target,view("h-32 shrink-0",vec![])]);
        let mut root=view("h-screen overflow-hidden",vec![pane]);let mut painter=Painter::new();painter.viewport.size=Some((320.,240.));
        let (_,snapshot)=painter.scene(&root,320.,1.5,None).unwrap();let pane=root.children[0].identity.get();let target=root.children[0].children[1].identity.get();
        assert_eq!(painter.visibility(&[target],&snapshot.nodes,1.5),[(target,false)]);
        painter.scroll_chain(Some(pane),0.,64.);let (_,snapshot)=painter.scene(&root,320.,1.5,None).unwrap();
        assert_eq!(painter.visibility(&[target],&snapshot.nodes,1.5),[(target,true)]);
        painter.scroll_chain(Some(pane),0.,-32.);let (_,snapshot)=painter.scene(&root,320.,1.5,None).unwrap();
        assert_eq!(painter.visibility(&[target],&snapshot.nodes,1.5),[(target,false)],"Edge contact is not positive area");
        painter.viewport.size=Some((320.,80.));painter.scroll_chain(Some(pane),0.,128.);let (_,snapshot)=painter.scene(&root,320.,2.,None).unwrap();
        assert_eq!(painter.visibility(&[target],&snapshot.nodes,2.),[(target,true)]);
        root.children[0].collapsed=true;let (_,snapshot)=painter.scene(&root,320.,2.,None).unwrap();assert_eq!(painter.visibility(&[target],&snapshot.nodes,2.),[(target,false)]);
        root.children.clear();let (_,snapshot)=painter.scene(&root,320.,2.,None).unwrap();assert_eq!(painter.visibility(&[target],&snapshot.nodes,2.),[(target,false)]);
    }

}
