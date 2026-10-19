//! The event loop owns the surface, renderer and window. No global window registry.
use crate::{Kind, Node, style::{Utility,State}};
use parley::{Alignment, AlignmentOptions, FontContext, FontWeight, Layout, LayoutContext, PositionedLayoutItem, StyleProperty};
use std::sync::{Arc, Mutex};
use taffy::prelude::*;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, RenderParams, Renderer, RendererOptions, Scene};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{WindowEvent, ElementState, MouseButton},
    keyboard::{Key, NamedKey},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[derive(Clone, Copy)]
struct TextStyle {
    size: f32,
    bold: bool,
    align: u8,
    color: super::theme::ResolvedColor,
}
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.,
            bold: false,
            align: 0,
            color: super::theme::ResolvedColor::rgb8(23,32,51),
        }
    }
}
fn paint_color(color:super::theme::ResolvedColor)->Color {Color::new(color.components())}
fn layout_text(text: &mut Layout<()>, width: f32, button: bool) {
    text.break_all_lines(Some(width.max(0.)));
    if button { text.align(Alignment::Center, AlignmentOptions::default()); }
}

struct Item {
    role: Kind,
    semantics: super::Semantics,
    label: String,
    id: NodeId,
    text: Option<Layout<()>>,
    color: super::theme::ResolvedColor,
    background: Option<super::theme::ResolvedColor>,
    radius: f64,
    border: f64,
    border_color: super::theme::ResolvedColor,
    ring: super::theme::ResolvedColor,
    align: u8,
    shadow_width:f64,
    shadow_color:super::theme::ResolvedColor,
    outline:Option<super::theme::ResolvedColor>,
    disabled: bool,
    children: Vec<Item>,
}
struct Painter {
    hover: std::collections::BTreeSet<u64>,
    pressed_nodes: std::collections::BTreeSet<u64>,
    focus: Option<u64>,
    fonts: FontContext,
    layouts: LayoutContext<()>,
    tree: TaffyTree<Layout<()>>,
}
impl Painter {
    fn new() -> Self {
        Self {
            hover: Default::default(), pressed_nodes:Default::default(), focus: None,
            fonts: FontContext::new(),
            layouts: LayoutContext::new(),
            tree: TaffyTree::new(),
        }
    }
    fn node(&mut self,node:&Node,inherited:TextStyle)->Result<Item,String> {
        self.themed_node(node,inherited,&super::theme::Theme::default(),"light",&mut 1,&mut Vec::new())
    }
    fn themed_node<'a>(&mut self, node: &'a Node, mut inherited: TextStyle, theme:&super::theme::Theme, mode:&str, next:&mut u64,ancestors:&mut Vec<&'a Node>) -> Result<Item, String> {
        let id=*next;*next+=1;
        let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));
        if node.theme.is_some() {inherited.size=theme.typography.base as f32;inherited.color=theme.color(mode,"foreground")?;}

        let mut style = Style {
            flex_direction: FlexDirection::Column,
            ..Style::default()
        };
        let (mut background, mut radius) = (node.theme.as_ref().map(|_|theme.color(mode,"background").unwrap()), 0.);
        if matches!(node.kind,Kind::Button|Kind::DisabledButton){inherited.align=1;style.flex_direction=FlexDirection::Row;style.align_items=AlignItems::CENTER;style.justify_content=JustifyContent::CENTER;}
        let (mut border,mut border_color,mut ring)=(0.,theme.color(mode,"border")?,theme.color(mode,"ring")?);
        let (mut shadow_width,mut shadow_color,mut outline)=(0.,theme.color(mode,"ring")?,None);
        if matches!(node.kind,Kind::DisabledButton|Kind::Button) {
            style.padding = Rect {
                left: length(16.),
                right: length(16.),
                top: length(8.),
                bottom: length(8.),
            };
        }
        for (state,utility) in super::style::resolved_node(node,ancestors,theme,mode) {
            if !match state {State::Base=>true,State::Hover=>self.hover.contains(&id) && node.kind!=Kind::DisabledButton,State::Active=>self.pressed_nodes.contains(&id) && node.kind!=Kind::DisabledButton,State::Focus=>self.focus==Some(id),State::Disabled=>node.kind==Kind::DisabledButton} {continue;}
            match utility {
                Utility::Row=>style.flex_direction=FlexDirection::Row,
                Utility::Column=>style.flex_direction=FlexDirection::Column,
                Utility::GapAxis(axis,v)=>{if axis==0{style.gap.height=length(v);}else{style.gap.width=length(v);}},
                Utility::Padding(side,v)=>match side {0=>style.padding.top=length(v),1=>style.padding.right=length(v),2=>style.padding.bottom=length(v),_=>style.padding.left=length(v)},
                Utility::Display(grid)=>style.display=if grid{Display::Grid}else{Display::Flex},
                Utility::Dimension(property,value)=>{
                    let value:LengthPercentageAuto=match value {super::style::Length::Pixels(v)=>length(v),super::style::Length::Percent(v)=>percent(v),super::style::Length::Auto|super::style::Length::None=>auto()};
                    match property {0=>style.size.width=value.into(),1=>style.size.height=value.into(),2=>style.min_size.width=value,3=>style.min_size.height=value,4=>style.max_size.width=value,_=>style.max_size.height=value};
                },
                Utility::SelfAlign(axis,value)=>{let value=match value {0=>Some(AlignSelf::START),1=>Some(AlignSelf::CENTER),2=>Some(AlignSelf::END),3=>Some(AlignSelf::STRETCH),_=>None};if axis==0{style.align_self=value;}else{style.justify_self=value;}},
                Utility::Shrink(v)=>style.flex_shrink=v,Utility::Grow(v)=>style.flex_grow=v,Utility::Wrap(v)=>style.flex_wrap=if v{FlexWrap::Wrap}else{FlexWrap::NoWrap},
                Utility::Tracks(axis,count)=>{
                    let tracks=match count {255=>vec![fr(1.),auto()],254=>vec![auto(),auto()],_=>vec![flex(1.);count as usize]};
                    if axis==0{style.grid_template_columns=tracks;}else{style.grid_template_rows=tracks;}
                },
                Utility::Placement(property,count)=>match property {0=>style.grid_column.start=line(count as i16),1=>style.grid_column.end=span(count),2=>style.grid_row.start=line(count as i16),_=>style.grid_row.end=span(count)},
                Utility::AutoRowsMin=>style.grid_auto_rows=vec![min_content()],
                Utility::FontSize(v)=>inherited.size=v,Utility::Bold(v)=>inherited.bold=v,
                Utility::Foreground(v)=>inherited.color=v,Utility::Background(v)=>background=Some(v),
                Utility::Rounded(v)=>radius=v as f64,Utility::BorderWidth(v)=>{border=v as f64;style.border=Rect{left:length(v),right:length(v),top:length(v),bottom:length(v)};},
                Utility::BorderColor(v)=>border_color=v,Utility::Ring(v)=>{ring=v;outline=Some(v);},Utility::Align(v)=>inherited.align=v,
                Utility::RingWidth(v)=>shadow_width=v as f64,Utility::RingShadowColor(v)=>shadow_color=v,
                Utility::Items(v)=>style.align_items=match v {0=>AlignItems::FLEX_START,1=>AlignItems::CENTER,_=>AlignItems::STRETCH},
                Utility::Justify(v)=>style.justify_content=match v {0=>JustifyContent::FLEX_START,1=>JustifyContent::CENTER,_=>JustifyContent::SPACE_BETWEEN},
            }
        }
        ancestors.push(node);
        let children = node
            .children
            .iter()
            .map(|child| self.themed_node(child, inherited,theme,mode,next,ancestors))
            .collect::<Result<Vec<_>, _>>()?;
        ancestors.pop();
        let (text, id) = if node.kind == Kind::View || !children.is_empty() {
            (
                None,
                self.tree
                    .new_with_children(
                        style,
                        &children.iter().map(|item| item.id).collect::<Vec<_>>(),
                    )
                    .map_err(|e| e.to_string())?,
            )
        } else {
            let mut builder = self
                .layouts
                .ranged_builder(&mut self.fonts, &node.text, 1., true);
            builder.push_default(StyleProperty::FontSize(inherited.size));
            builder.push_default(StyleProperty::FontWeight(FontWeight::new(
                if inherited.bold { 700. } else { 400. },
            )));
            let layout: Layout<()> = builder.build(&node.text);
            let id = self
                .tree
                .new_leaf_with_context(style, layout.clone())
                .map_err(|e| e.to_string())?;
            (Some(layout), id)
        };
        Ok(Item {
            role: node.kind,
            semantics: node.semantics.clone(),
            label: if matches!(node.kind,Kind::Button|Kind::DisabledButton){node.content_text()}else{node.text.clone()},
            id,
            text,
            color: inherited.color,
            background,
            radius,
            border,border_color,ring,align:inherited.align,shadow_width,shadow_color,outline,
            disabled: node.kind == Kind::DisabledButton,
            children,
        })
    }
    fn scene(
        &mut self,
        node: &Node,
        width: f32,
        scale: f64,
        focus: Option<u64>,
    ) -> Result<(Scene, accesskit::TreeUpdate), String> {
        self.tree.clear();
        self.focus=focus;
        let mut root = self.node(node, TextStyle::default())?;
        let mut root_style = self.tree.style(root.id).map_err(|e| e.to_string())?.clone();
        if root_style.size.width.is_auto() {root_style.size.width = length(width);}
        self.tree.set_style(root.id, root_style).map_err(|e| e.to_string())?;
        self.tree
            .compute_layout_with_measure(
                root.id,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::MaxContent,
                },
                |inputs, _, context, style| {
                    taffy::compute_leaf_layout(
                        inputs,
                        style,
                        |_, _| 0.,
                        |known, available| {
                            let Some(text) = context else {
                                return Size::ZERO;
                            };
                            let width = known.width.or(match available.width {
                                AvailableSpace::Definite(v) => Some(v),
                                AvailableSpace::MinContent => Some(0.),
                                _ => None,
                            });
                            text.break_all_lines(width);
                            Size {
                                width: known.width.unwrap_or(text.width()),
                                height: known.height.unwrap_or(text.height()),
                            }
                        },
                    )
                },
            )
            .map_err(|e| e.to_string())?;
        let mut scene = Scene::new();
        self.paint(&mut root, &mut scene, 0., 0., scale, focus, &mut 1)?;
        let mut nodes = Vec::new();
        let mut next = 1;
        self.accessible(&root, &mut nodes, &mut next, 0., 0., scale)?;
        let mut window = accesskit::Node::new(accesskit::Role::Window);
        window.set_children(vec![accesskit::NodeId(1)]);
        nodes.push((accesskit::NodeId(0), window));
        Ok((
            scene,
            accesskit::TreeUpdate {
                nodes,
                tree: Some(accesskit::TreeInfo::new(accesskit::NodeId(0))),
                tree_id: accesskit::TreeId::ROOT,
                focus: accesskit::NodeId(0),
            },
        ))
    }
    fn accessible(
        &self,
        item: &Item,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        next: &mut u64,
        parent_x: f64,
        parent_y: f64,
        scale: f64,
    ) -> Result<accesskit::NodeId, String> {
        let id = accesskit::NodeId(*next);
        *next += 1;
        let layout = self.tree.layout(item.id).map_err(|e| e.to_string())?;
        let (x, y) = (
            parent_x + layout.location.x as f64,
            parent_y + layout.location.y as f64,
        );
        let default_role=match item.role {
            Kind::View => accesskit::Role::GenericContainer,
            Kind::Text => accesskit::Role::Label,
            Kind::DisabledButton | Kind::Button => accesskit::Role::Button,
        };
        let role=match item.semantics.role.as_str(){
            "alert"=>accesskit::Role::Alert,"status"=>accesskit::Role::Status,"separator"=>accesskit::Role::Splitter,
            "group"=>accesskit::Role::Group,"note"=>accesskit::Role::Note,"region"=>accesskit::Role::Region,
            "none"|"presentation"=>accesskit::Role::GenericContainer,
            _ if matches!(item.semantics.tag.as_str(),"h1"|"h2"|"h3"|"h4"|"h5"|"h6")=>accesskit::Role::Heading,
            _ if item.semantics.tag=="p"=>accesskit::Role::Paragraph,
            _ if item.semantics.tag=="code"=>accesskit::Role::Code,
            _ if item.semantics.tag=="section" && !item.semantics.label.trim().is_empty()=>accesskit::Role::Region,
            _=>default_role,
        };
        let mut node = accesskit::Node::new(role);
        if role==accesskit::Role::Heading{node.set_level((item.semantics.tag.as_bytes()[1]-b'1') as usize);}
        if role==accesskit::Role::Splitter{node.set_orientation(if item.semantics.orientation=="vertical"{accesskit::Orientation::Vertical}else{accesskit::Orientation::Horizontal});}
        if item.semantics.hidden{node.set_hidden();}
        if item.semantics.role=="alert"{node.set_live(accesskit::Live::Assertive);}else if item.semantics.role=="status"{node.set_live(accesskit::Live::Polite);}
        if !item.semantics.label.is_empty(){node.set_label(item.semantics.label.clone());}
        else if !item.label.is_empty() && matches!(role,accesskit::Role::Label|accesskit::Role::Button|accesskit::Role::Heading) {
            node.set_label(item.label.clone());
        }
        if item.disabled { node.set_disabled(); }
        else if item.role==Kind::Button {node.add_action(accesskit::Action::Click);node.add_action(accesskit::Action::Focus);}
        node.set_bounds(accesskit::Rect::new(
            x * scale,
            y * scale,
            (x + layout.size.width as f64) * scale,
            (y + layout.size.height as f64) * scale,
        ));
        let mut children = item
            .children
            .iter()
            .map(|child| self.accessible(child, nodes, next, x, y, scale))
            .collect::<Result<Vec<_>, _>>()?;
        if item.text.is_some() && !item.label.is_empty() && !matches!(role,accesskit::Role::Label|accesskit::Role::Button) {
            // Semantic text containers own content, not an invented accessible name.
            let content_id=accesskit::NodeId(id.0 | (1u64<<63));
            let mut content=accesskit::Node::new(accesskit::Role::Label);
            content.set_label(item.label.clone());content.set_bounds(node.bounds().expect("text bounds"));
            children.push(content_id);nodes.push((content_id,content));
        }
        node.set_children(children);
        nodes.push((id, node));
        Ok(id)
    }
    fn visual_bounds(&self,item:&Item,parent_x:f64,parent_y:f64)->Result<vello::kurbo::Rect,String> {
        let layout=self.tree.layout(item.id).map_err(|e|e.to_string())?;
        let x=parent_x+layout.location.x as f64;let y=parent_y+layout.location.y as f64;
        let outset=item.shadow_width.max(if item.outline.is_some(){4.}else{0.});
        let mut bounds=vello::kurbo::Rect::new(x-outset,y-outset,x+layout.size.width as f64+outset,y+layout.size.height as f64+outset);
        for child in &item.children {bounds=bounds.union(self.visual_bounds(child,x,y)?);}
        Ok(bounds)
    }
    fn paint(
        &self,
        item: &mut Item,
        scene: &mut Scene,
        parent_x: f64,
        parent_y: f64,
        scale: f64,
        focus: Option<u64>,
        next: &mut u64,
    ) -> Result<(), String> {
        let layout = self.tree.layout(item.id).map_err(|e| e.to_string())?;
        let (x, y) = (
            parent_x + layout.location.x as f64,
            parent_y + layout.location.y as f64,
        );
        let id = *next;
        *next += 1;
        let transform = Affine::scale(scale);
        if item.disabled {
            scene.push_layer(
                Fill::NonZero,
                vello::peniko::BlendMode::default(),
                0.6,
                transform,
                &self.visual_bounds(item,parent_x,parent_y)?,
            );
        }
        if item.shadow_width>0. {
            let half=item.shadow_width/2.;
            scene.stroke(&vello::kurbo::Stroke::new(item.shadow_width).with_join(vello::kurbo::Join::Miter),transform,paint_color(item.shadow_color),None,
                &RoundedRect::new(x-half,y-half,x+layout.size.width as f64+half,y+layout.size.height as f64+half,if item.radius==0.{0.}else{item.radius+half}));
        }
        if let Some(color) = item.background {
            scene.fill(
                Fill::NonZero,
                transform,
                paint_color(color),
                None,
                &RoundedRect::new(
                    x,
                    y,
                    x + layout.size.width as f64,
                    y + layout.size.height as f64,
                    item.radius,
                ),
            );
        }
        if item.border>0. {
            scene.stroke(&vello::kurbo::Stroke::new(item.border).with_join(vello::kurbo::Join::Miter),transform,paint_color(item.border_color),None,
                &RoundedRect::new(x+item.border/2.,y+item.border/2.,x+layout.size.width as f64-item.border/2.,y+layout.size.height as f64-item.border/2.,item.radius));
        }
        if let Some(text) = &mut item.text {
            layout_text(text, layout.size.width - layout.padding.left - layout.padding.right - layout.border.left - layout.border.right,
                matches!(item.role, Kind::Button | Kind::DisabledButton));
            text.align(match item.align {1=>Alignment::Center,2=>Alignment::End,_=>Alignment::Start},AlignmentOptions::default());
            let color = paint_color(item.color);
            for line in text.lines() {
                for run in line.items() {
                    if let PositionedLayoutItem::GlyphRun(glyph_run) = run {
                        let font_run = glyph_run.run();
                        let coordinates: Vec<_> = font_run
                            .normalized_coords()
                            .iter()
                            .map(|coord| coord.to_bits())
                            .collect();
                        scene
                            .draw_glyphs(font_run.font())
                            .font_size(font_run.font_size())
                            .normalized_coords(&coordinates)
                            .transform(
                                transform
                                    * Affine::translate((
                                        x + (layout.padding.left + layout.border.left) as f64,
                                        y + (layout.padding.top + layout.border.top) as f64,
                                    )),
                            )
                            .brush(color)
                            .draw(
                                Fill::NonZero,
                                glyph_run.positioned_glyphs().map(|glyph| vello::Glyph {
                                    id: glyph.id,
                                    x: glyph.x,
                                    y: glyph.y,
                                }),
                            );
                    }
                }
            }
        }
        for child in &mut item.children {
            self.paint(child, scene, x, y, scale, focus, next)?;
        }
        if let Some(color)=item.outline.or_else(||(focus==Some(id) && item.role==Kind::Button).then_some(item.ring)) {
            scene.stroke(&vello::kurbo::Stroke::new(2.).with_join(vello::kurbo::Join::Miter), transform, paint_color(color), None,
                &RoundedRect::new(x-3., y-3., x + layout.size.width as f64+3., y + layout.size.height as f64+3., if item.radius==0.{0.}else{item.radius+3.}));
        }
        if item.disabled {
            scene.pop_layer();
        }
        Ok(())
    }
}

struct InitialTree(Arc<Mutex<accesskit::TreeUpdate>>);
impl accesskit::ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        Some(self.0.lock().unwrap().clone())
    }
}
struct NoActions;
struct Actions(winit::event_loop::EventLoopProxy<accesskit::ActionRequest>);
impl accesskit::ActionHandler for Actions {
    fn do_action(&mut self, action: accesskit::ActionRequest) { let _=self.0.send_event(action); }
}
impl accesskit::DeactivationHandler for NoActions {
    fn deactivate_accessibility(&mut self) {}
}
struct Active {
    accessibility: accesskit_winit::Adapter,
    snapshot: Arc<Mutex<accesskit::TreeUpdate>>,
    surface: RenderSurface<'static>,
    renderer: Renderer,
    window: Arc<Window>,
}
fn keyboard(focus:&mut Option<u64>,pressed:&mut Option<u64>,ids:&[u64],shift:bool,key:&Key,state:ElementState,repeat:bool)->Option<u64> {
    if repeat {return None;}
    if state==ElementState::Pressed && *key==Key::Named(NamedKey::Tab) {
        *pressed=None;
        if !ids.is_empty(){let current=ids.iter().position(|id|Some(*id)==*focus);let next=match current {Some(i) if shift=>(i+ids.len()-1)%ids.len(),Some(i)=>(i+1)%ids.len(),None if shift=>ids.len()-1,None=>0};*focus=Some(ids[next]);}
    } else if *key==Key::Named(NamedKey::Enter) && state==ElementState::Pressed {
        return focus.filter(|id|ids.contains(id));
    } else if *key==Key::Named(NamedKey::Space) {
        if state==ElementState::Pressed {*pressed=*focus;}
        else if let Some(id)=pressed.take(){if *focus==Some(id) && ids.contains(&id){return Some(id);}}
    }
    None
}
struct App {
    proxy: winit::event_loop::EventLoopProxy<accesskit::ActionRequest>,
    pointer: (f64,f64),
    pressed: Option<u64>,
    focus: Option<u64>,
    shift: bool,
    key_pressed: Option<u64>,
    close_after_frame: bool,
    presented: bool,
    #[cfg(feature = "acceptance")]
    resize_step: usize,
    deadline: Option<std::time::Instant>,
    node: Node,
    title: String,
    context: RenderContext,
    painter: Painter,
    active: Option<Active>,
    error: Option<String>,
}
impl App {
    fn buttons(&self)->Vec<(u64,accesskit::Rect)> {
        self.active.as_ref().map(|active|active.snapshot.lock().unwrap().nodes.iter()
            .filter(|(_,n)|n.role()==accesskit::Role::Button && !n.is_disabled())
            .filter_map(|(id,n)|n.bounds().map(|bounds|(id.0,bounds))).collect()).unwrap_or_default()
    }
    fn pointer_nodes(&self)->std::collections::BTreeSet<u64> {
        self.active.as_ref().map(|active|active.snapshot.lock().unwrap().nodes.iter().filter_map(|(id,node)|node.bounds().filter(|r|self.pointer.0>=r.x0 && self.pointer.0<r.x1 && self.pointer.1>=r.y0 && self.pointer.1<r.y1).map(|_|id.0)).collect()).unwrap_or_default()
    }
    fn pressed_nodes(&self)->std::collections::BTreeSet<u64> {
        let mut ids=std::collections::BTreeSet::new();let Some(mut current)=self.pressed.or(self.key_pressed) else{return ids;};
        let Some(active)=&self.active else{return ids;};let snapshot=active.snapshot.lock().unwrap();
        let parents=snapshot.nodes.iter().flat_map(|(id,node)|node.children().iter().map(move|child|(child.0,id.0))).collect::<std::collections::BTreeMap<_,_>>();
        while ids.insert(current) {let Some(parent)=parents.get(&current) else{break;};current=*parent;}ids
    }
    fn hit(&self)->Option<u64> {
        self.buttons().into_iter().find(|(_,r)|self.pointer.0>=r.x0 && self.pointer.0<r.x1 && self.pointer.1>=r.y0 && self.pointer.1<r.y1).map(|(id,_)|id)
    }
    fn activate(&mut self,id:u64,event_loop:&ActiveEventLoop) {
        match self.node.activate(id).and_then(|_|self.node.refresh()) {
            Ok(_)=>if let Some(active)=&self.active {active.window.request_redraw();},
            Err(error)=>self.fail(event_loop,error),
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl ToString) {
        self.error = Some(error.to_string());
        self.active = None;
        event_loop.exit();
    }
    fn open(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(&self.title)
                        .with_visible(false)
                        .with_inner_size(LogicalSize::new(640., 480.)),
                )
                .map_err(|e| e.to_string())?,
        );
        let size = window.inner_size();
        let surface = pollster::block_on(self.context.create_surface(
            window.clone(),
            size.width.max(1),
            size.height.max(1),
            vello::wgpu::PresentMode::AutoVsync,
        ))
        .map_err(|e| e.to_string())?;
        let renderer = Renderer::new(
            &self.context.devices[surface.dev_id].device,
            RendererOptions {
                use_cpu: false,
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())?;
        let (_, mut snapshot) = self.painter.scene(
            &self.node,
            (size.width as f64 / window.scale_factor()) as f32,
            window.scale_factor(),
            None,
        )?;
        if let Some((_, root)) = snapshot.nodes.iter_mut().find(|(id, _)| id.0 == 0) {
            root.set_label(self.title.clone());
        }
        let snapshot = Arc::new(Mutex::new(snapshot));
        let accessibility = accesskit_winit::Adapter::with_direct_handlers(
            event_loop,
            &window,
            InitialTree(snapshot.clone()),
            Actions(self.proxy.clone()),
            NoActions,
        );
        window.set_visible(true);
        window.request_redraw();
        self.active = Some(Active {
            accessibility,
            snapshot,
            surface,
            renderer,
            window,
        });
        Ok(())
    }
    fn draw(&mut self) -> Result<bool, String> {
        self.painter.hover=self.pointer_nodes();self.painter.pressed_nodes=self.pressed_nodes();
        let Some(active) = &mut self.active else {
            return Ok(false);
        };
        let size = active.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(false);
        }
        if active.surface.config.width != size.width || active.surface.config.height != size.height
        {
            self.context
                .resize_surface(&mut active.surface, size.width, size.height);
        }
        let scale = active.window.scale_factor();
        let (scene, mut snapshot) =
            self.painter
                .scene(&self.node, (size.width as f64 / scale) as f32, scale, self.focus)?;
        if let Some((_, root)) = snapshot.nodes.iter_mut().find(|(id, _)| id.0 == 0) {
            root.set_label(self.title.clone());
        }
        if self.focus.is_some_and(|id|!snapshot.nodes.iter().any(|(node_id,node)|node_id.0==id && node.role()==accesskit::Role::Button && !node.is_disabled())) {self.focus=None;self.key_pressed=None;}
        snapshot.focus=accesskit::NodeId(self.focus.unwrap_or(0));
        *active.snapshot.lock().unwrap() = snapshot.clone();
        active.accessibility.update_if_active(|| snapshot);
        let device = &self.context.devices[active.surface.dev_id];
        active
            .renderer
            .render_to_texture(
                &device.device,
                &device.queue,
                &scene,
                &active.surface.target_view,
                &RenderParams {
                    base_color: {let theme=self.node.theme.as_ref();let default=super::theme::Theme::default();let (theme,mode)=theme.map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));paint_color(theme.color(mode,"background")?)},
                    width: size.width,
                    height: size.height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .map_err(|e| e.to_string())?;
        let texture = match active.surface.surface.get_current_texture() {
            vello::wgpu::CurrentSurfaceTexture::Success(texture)
            | vello::wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            vello::wgpu::CurrentSurfaceTexture::Timeout
            | vello::wgpu::CurrentSurfaceTexture::Occluded => return Ok(false),
            vello::wgpu::CurrentSurfaceTexture::Outdated => {
                self.context
                    .resize_surface(&mut active.surface, size.width, size.height);
                active.window.request_redraw();
                return Ok(false);
            }
            vello::wgpu::CurrentSurfaceTexture::Lost => {
                return Err("The native GPU surface was lost".into());
            }
            vello::wgpu::CurrentSurfaceTexture::Validation => {
                return Err("Native GPU surface validation failed".into());
            }
        };
        let mut encoder = device.device.create_command_encoder(&Default::default());
        active.surface.blitter.copy(
            &device.device,
            &mut encoder,
            &active.surface.target_view,
            &texture.texture.create_view(&Default::default()),
        );
        device.queue.submit([encoder.finish()]);
        active.window.pre_present_notify();
        device.queue.present(texture);
        Ok(true)
    }
}
impl ApplicationHandler<accesskit::ActionRequest> for App {
    fn user_event(&mut self,event_loop:&ActiveEventLoop,request:accesskit::ActionRequest) {
        if request.target_tree!=accesskit::TreeId::ROOT {return;}
        if !self.buttons().iter().any(|(id,_)|*id==request.target_node.0) {return;}
        match request.action {
            accesskit::Action::Click=>self.activate(request.target_node.0,event_loop),
            accesskit::Action::Focus=>{self.focus=Some(request.target_node.0);if let Some(active)=&self.active {active.window.request_redraw();}},
            _=>{}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(deadline) = self.deadline {
            if std::time::Instant::now() >= deadline {
                self.fail(
                    event_loop,
                    "GPU window acceptance timed out before presentation",
                );
            } else {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(deadline));
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.active.is_none()
            && let Err(error) = self.open(event_loop)
        {
            self.fail(event_loop, error);
        }
    }
    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.active = None;
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self
            .active
            .as_ref()
            .is_none_or(|active| active.window.id() != id)
        {
            return;
        }
        if let Some(active) = &mut self.active {
            active.accessibility.process_event(&active.window, &event);
        }
        match event {
            WindowEvent::CursorMoved{position,..}=>{self.pointer=(position.x,position.y);if let Some(active)=&self.active {active.window.request_redraw();}},
            WindowEvent::CursorLeft{..}=>{self.pointer=(-1.,-1.);if let Some(active)=&self.active {active.window.request_redraw();}},
            WindowEvent::ModifiersChanged(modifiers)=>self.shift=modifiers.state().shift_key(),
            WindowEvent::Focused(false)=>{self.pressed=None;self.key_pressed=None;self.focus=None;},
            WindowEvent::MouseInput{state,button:MouseButton::Left,..}=>{
                if state==ElementState::Pressed {self.pressed=self.hit();self.focus=self.pressed;}
                else if let Some(id)=self.pressed.take() {if self.hit()==Some(id){self.activate(id,event_loop);}}
                if let Some(active)=&self.active {active.window.request_redraw();}
            },
            WindowEvent::KeyboardInput{event,..}=>{
                let ids=self.buttons().into_iter().map(|(id,_)|id).collect::<Vec<_>>();
                if let Some(id)=keyboard(&mut self.focus,&mut self.key_pressed,&ids,self.shift,&event.logical_key,event.state,event.repeat){self.activate(id,event_loop);}
                if let Some(active)=&self.active {active.window.request_redraw();}
            },
            WindowEvent::CloseRequested => {
                self.active = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => match self.draw() {
                Ok(true) if self.close_after_frame => {
                    #[cfg(feature = "acceptance")]
                    if self.resize_step < 3 {
                        let widths = [800., 480., 640.];
                        if let Some(active) = &self.active {
                            let _ = active.window.request_inner_size(LogicalSize::new(widths[self.resize_step], 480.));
                            active.window.request_redraw();
                        }
                        self.resize_step += 1;
                        return;
                    }
                    self.presented = true;
                    self.active = None;
                    event_loop.exit();
                }
                Err(error) => self.fail(event_loop, error),
                _ => {}
            },
            WindowEvent::Resized(_) => {
                match self.draw() {
                    Err(error) => self.fail(event_loop, error),
                    Ok(false) => if let Some(active) = &self.active { active.window.request_redraw(); },
                    Ok(true) => {},
                }
            }
            WindowEvent::ScaleFactorChanged { .. }
            | WindowEvent::Occluded(false) => {
                if let Some(active) = &self.active {
                    active.window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
/// Open the owned static tree on the platform event-loop thread; return after close.
pub fn run(node: Node, title: String) -> Result<(), String> {
    run_window(node, title, false)
}
#[cfg(feature = "acceptance")]
pub fn verify_window(node: Node, title: String) -> Result<(), String> {
    run_window(node, title, true)
}
fn run_window(node: Node, title: String, close_after_frame: bool) -> Result<(), String> {
    let event_loop = EventLoop::<accesskit::ActionRequest>::with_user_event().build().map_err(|e| e.to_string())?;
    let mut app = App {
        proxy: event_loop.create_proxy(), pointer:(-1.,-1.),pressed:None,focus:None,shift:false,key_pressed:None,
        close_after_frame,
        presented: false,
        #[cfg(feature = "acceptance")]
        resize_step: 0,
        deadline: close_after_frame
            .then(|| std::time::Instant::now() + std::time::Duration::from_secs(20)),
        node,
        title,
        context: RenderContext::new(),
        painter: Painter::new(),
        active: None,
        error: None,
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    if close_after_frame && !app.presented && app.error.is_none() {
        return Err("Window closed before GPU presentation".into());
    }
    app.error.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_and_accessibility_preserve_static_semantics() {
        let button = Node::new(
            Kind::DisabledButton,
            "p-2".into(),
            "Disabled".into(),
            vec![],
        )
        .unwrap();
        let text = Node::new(
            Kind::Text,
            "text-xl".into(),
            "café 🦀\nsecond line".into(),
            vec![],
        )
        .unwrap();
        let root = Node::new(
            Kind::View,
            "p-4 gap-4".into(),
            String::new(),
            vec![text, button],
        )
        .unwrap();
        let mut painter = Painter::new();
        let (_, one) = painter.scene(&root, 400., 1., None).unwrap();
        let (_, two) = painter.scene(&root, 400., 2., None).unwrap();
        assert_eq!(one.nodes.len(), 4);
        assert_eq!(two.nodes.len(), 4);
        let text = one
            .nodes
            .iter()
            .find(|(_, node)| node.role() == accesskit::Role::Label)
            .unwrap();
        assert_eq!(text.1.label(), Some("café 🦀\nsecond line"));
        assert!(text.1.bounds().unwrap().height() > 24.);
        let button = one
            .nodes
            .iter()
            .find(|(_, node)| node.role() == accesskit::Role::Button)
            .unwrap();
        assert!(button.1.is_disabled());
        assert!(button.1.bounds().unwrap().y0 >= text.1.bounds().unwrap().y1 + 16.);
        for ((id, a), (other, b)) in one.nodes.iter().zip(&two.nodes) {
            assert_eq!(id, other);
            if let (Some(a), Some(b)) = (a.bounds(), b.bounds()) {
                assert_eq!(b.x0, a.x0 * 2.);
                assert_eq!(b.y0, a.y0 * 2.);
            }
        }
    }
}

#[cfg(test)]
mod interactive_accessibility_tests {
    use super::*;
    #[test]
    fn enabled_buttons_offer_focus_and_click_with_updated_bound_labels() {
        let state=crate::Signal::new(0);let action=state.handle();
        let label=Node::bound_text(state.handle(),String::new()).unwrap();
        let button=Node::button("Add".into(),String::new(),false,move||{action.set(action.get()? + 1)?;Ok(())}).unwrap();
        let mut root=Node::new(Kind::View,String::new(),String::new(),vec![label,button]).unwrap().own_state(state).unwrap();
        let mut painter=Painter::new();
        let (_,before)=painter.scene(&root,400.,1.,None).unwrap();
        let button=&before.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Button).unwrap().1;
        assert!(button.supports_action(accesskit::Action::Focus));assert!(button.supports_action(accesskit::Action::Click));
        assert!(root.activate(3).unwrap());assert_eq!(root.refresh().unwrap(),1);
        let (_,after)=painter.scene(&root,400.,1.,None).unwrap();
        assert_eq!(after.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Label).unwrap().1.label(),Some("1"));
    }
}

#[cfg(test)]
mod keyboard_tests {
    use super::*;
    #[test]
    fn focus_order_disabled_exclusion_space_release_and_repeat() {
        let mut focus=None;let mut pressed=None;let ids=[4,8];
        let tab=Key::Named(NamedKey::Tab);let enter=Key::Named(NamedKey::Enter);let space=Key::Named(NamedKey::Space);
        assert_eq!(keyboard(&mut focus,&mut pressed,&ids,false,&tab,ElementState::Pressed,false),None);assert_eq!(focus,Some(4));
        assert_eq!(keyboard(&mut focus,&mut pressed,&ids,false,&enter,ElementState::Pressed,false),Some(4));
        assert_eq!(keyboard(&mut focus,&mut pressed,&ids,false,&enter,ElementState::Pressed,true),None);
        keyboard(&mut focus,&mut pressed,&ids,false,&tab,ElementState::Pressed,false);assert_eq!(focus,Some(8));
        assert_eq!(keyboard(&mut focus,&mut pressed,&ids,false,&space,ElementState::Pressed,false),None);
        assert_eq!(keyboard(&mut focus,&mut pressed,&ids,false,&space,ElementState::Released,false),Some(8));
        keyboard(&mut focus,&mut pressed,&ids,true,&tab,ElementState::Pressed,false);assert_eq!(focus,Some(4));
        focus=Some(5);assert_eq!(keyboard(&mut focus,&mut pressed,&ids,false,&enter,ElementState::Pressed,false),None);
    }
}

#[cfg(test)]
mod presentation_tests {
    use super::*;
    #[test]
    fn button_labels_center_each_line_and_resize_without_changing_font_metrics() {
        let node=Node::button("Increment\nAgain".into(),"rounded".into(),false,||Ok(())).unwrap();
        let mut painter=Painter::new();
        let mut item=painter.node(&node,TextStyle::default()).unwrap();
        let text=item.text.as_mut().unwrap();
        let mut height=None;
        for width in [400.,720.,240.,400.] {
            layout_text(text,width,true);
            if let Some(previous)=height { assert_eq!(text.height(),previous); }
            height=Some(text.height());
            for line in text.lines() {
                let metrics=line.metrics();
                assert!((metrics.offset * 2. + metrics.advance - width).abs()<0.1, "{metrics:?}");
            }
        }
        layout_text(text,400.,false);
        assert_eq!(text.lines().next().unwrap().metrics().offset,0.);
    }
    #[test]
    fn repeated_window_widths_preserve_text_height_padding_and_scaled_bounds() {
        let root=Node::new(Kind::View,"p-4 gap-4".into(),String::new(),vec![
            Node::new(Kind::Text,"text-xl".into(),"Count: 0".into(),vec![]).unwrap(),
            Node::button("Increment".into(),"rounded".into(),false,||Ok(())).unwrap(),
        ]).unwrap();
        let mut painter=Painter::new();let mut previous=None;
        for width in [640.,800.,480.,640.] {
            let (_,snapshot)=painter.scene(&root,width,2.,Some(3)).unwrap();
            let bounds=snapshot.nodes.iter().find(|(id,_)|id.0==3).unwrap().1.bounds().unwrap();
            assert_eq!(bounds.x0,32.);
            assert_eq!(bounds.width(),(width as f64-32.)*2.);
            if let Some((y,height))=previous {assert_eq!((bounds.y0,bounds.height()),(y,height));}
            previous=Some((bounds.y0,bounds.height()));
        }
    }
}

#[cfg(test)]mod project_theme_tests {
 use super::*;
 #[test]fn project_tokens_drive_layout_paint_and_interaction_states() {
  let mut value:serde_json::Value=serde_json::from_str(super::super::theme::DEFAULT_JSON).unwrap();
  value["dark"]["primary"]="#123456".into();value["dark"]["accent"]="#654321".into();value["radius"]["md"]=13.into();value["spacing"]["md"]=21.into();value["typography"]["base"]=19.into();
  let button=Node::button("Centered".into(),"p-4 px-2 bg-primary hover:bg-accent focus:bg-secondary rounded border border-border".into(),false,||Ok(())).unwrap();
  let root=Node::new(Kind::View,String::new(),String::new(),vec![button]).unwrap().with_theme(&value.to_string(),"dark").unwrap();
  let mut painter=Painter::new();let item=painter.node(&root,TextStyle::default()).unwrap();let button=&item.children[0];
  assert_eq!(button.background,Some(super::super::theme::ResolvedColor::rgb8(0x12,0x34,0x56)));assert_eq!(button.radius,13.);assert_eq!(button.border,1.);assert_eq!(button.align,1);
  let style=painter.tree.style(button.id).unwrap();assert_eq!(style.padding.top,length(21.));assert_eq!(style.padding.left,length(8.));
  painter.hover.insert(2);let hovered=painter.node(&root,TextStyle::default()).unwrap();assert_eq!(hovered.children[0].background,Some(super::super::theme::ResolvedColor::rgb8(0x65,0x43,0x21)));
  painter.focus=Some(2);let focused=painter.node(&root,TextStyle::default()).unwrap();assert_eq!(focused.children[0].background,Some(super::super::theme::ResolvedColor::rgb8(0x33,0x41,0x55)));
 }
}

#[cfg(test)]
mod semantic_accessibility_tests {
    use super::*;
    #[test]
    fn headings_live_roles_names_hidden_state_and_separator_orientation() {
        let text=|value:&str,config:&str|Node::new(Kind::Text,String::new(),value.into(),vec![]).unwrap().with_semantics(config).unwrap();
        let root=Node::new(Kind::View,String::new(),String::new(),vec![
            text("First",r#"{"tag":"h1"}"#), text("Last",r#"{"tag":"h6"}"#),
            text("Problem",r#"{"role":"alert","label":"Error summary"}"#),
            text("Saved",r#"{"role":"status"}"#), text("Decoration",r#"{"hidden":true}"#),
            text("",r#"{"role":"separator"}"#), text("",r#"{"role":"separator","orientation":"vertical"}"#),
            Node::new(Kind::View,String::new(),String::new(),vec![]).unwrap().with_semantics(r#"{"tag":"section","label":"Implicit region"}"#).unwrap(),
            text("Paragraph body",r#"{"tag":"p"}"#),text("code body",r#"{"tag":"code"}"#),
        ]).unwrap().with_semantics(r#"{"tag":"section","role":"region","label":"Account"}"#).unwrap();
        let mut painter=Painter::new(); let (_,tree)=painter.scene(&root,400.,1.,None).unwrap();
        let node=|id:u64|&tree.nodes.iter().find(|(key,_)|key.0==id).unwrap().1;
        assert_eq!(node(1).role(),accesskit::Role::Region); assert_eq!(node(1).label(),Some("Account"));
        assert_eq!(node(2).role(),accesskit::Role::Heading); assert_eq!(node(2).level(),Some(0));
        assert_eq!(node(3).level(),Some(5));
        assert_eq!(node(4).role(),accesskit::Role::Alert); assert_eq!(node(4).label(),Some("Error summary")); assert_eq!(node(4).live(),Some(accesskit::Live::Assertive));
        assert_eq!(node(5).live(),Some(accesskit::Live::Polite)); assert_eq!(node(5).label(),None);assert_eq!(node((1u64<<63)|5).label(),Some("Saved"));assert!(node(6).is_hidden());
        assert_eq!(node(7).role(),accesskit::Role::Splitter); assert_eq!(node(7).orientation(),Some(accesskit::Orientation::Horizontal));
        assert_eq!(node(8).orientation(),Some(accesskit::Orientation::Vertical));
        assert!(!node(7).supports_action(accesskit::Action::Focus));
        assert_eq!(node(9).role(),accesskit::Role::Region);
        for (id,role,body) in [(10,accesskit::Role::Paragraph,"Paragraph body"),(11,accesskit::Role::Code,"code body")] {
            assert_eq!(node(id).role(),role);assert_eq!(node(id).label(),None);assert_eq!(node(id|(1u64<<63)).label(),Some(body));
        }
    }
}

#[cfg(test)]
mod composed_button_tests {
    use super::*;
    #[test]
    fn composed_labels_center_and_keep_one_action_target() {
        let text=Node::new(Kind::Text,String::new(),"Two\nlines".into(),vec![]).unwrap();
        let root=Node::button_children(vec![crate::Child::Node(text)],"rounded".into(),false,None).unwrap();
        let mut painter=Painter::new();let (_,snapshot)=painter.scene(&root,320.,1.,None).unwrap();
        assert_eq!(snapshot.nodes.iter().filter(|(_,node)|node.supports_action(accesskit::Action::Click)).count(),1);
        let button=&snapshot.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Button).unwrap().1;
        let label=&snapshot.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Label).unwrap().1;
        assert_eq!(button.label(),Some("Two\nlines"));assert_eq!(label.label(),Some("Two\nlines"));
        let parent=button.bounds().unwrap();let child=label.bounds().unwrap();
        assert!(((child.x0+child.x1)-(parent.x0+parent.x1)).abs()<0.1);
    }
}

#[cfg(test)] mod layout_tests {
 use super::*;
 fn view(classes:&str,children:Vec<Node>)->Node{Node::new(Kind::View,classes.into(),String::new(),children).unwrap()}
 fn bounds(tree:&accesskit::TreeUpdate,id:u64)->accesskit::Rect{tree.nodes.iter().find(|(node,_)|node.0==id).unwrap().1.bounds().unwrap()}
 #[test] fn equal_grid_tracks_spans_and_custom_theme_match_content_geometry() {
  let mut theme=serde_json::from_str::<serde_json::Value>(super::super::theme::DEFAULT_JSON).unwrap();theme["spacing"]["md"]=20.into();
  let root=view("grid grid-cols-2 gap-4 p-4",vec![view("min-w-0 h-8",vec![]),view("col-start-2 row-start-1 row-span-2",vec![]),view("col-start-1 row-start-2 h-8",vec![])]).with_theme(&theme.to_string(),"light").unwrap();
  let mut painter=Painter::new();for width in [360.,640.,320.] {
   let (_,tree)=painter.scene(&root,width,1.,None).unwrap();let a=bounds(&tree,2);let b=bounds(&tree,3);let c=bounds(&tree,4);
   assert!((a.width()-(width as f64-60.)/2.).abs()<0.1);assert_eq!((a.x0,a.y0,a.height()),(20.,20.,40.));
   assert_eq!(b.height(),100.);assert_eq!(b.y0,20.);assert_eq!(c.y0,80.);assert_eq!(c.x0,20.);
   assert!(b.x0>a.x1);assert_eq!(tree.nodes.iter().filter(|(_,node)|node.supports_action(accesskit::Action::Focus)).count(),0);
  }
 }
 #[test] fn preferred_root_sizes_bounds_percentages_and_padding_remain_distinct() {
  let mut painter=Painter::new();let root=view("w-64 max-w-full h-8",vec![]);
  for (width,expected) in [(400.,256.),(200.,200.)] {let (_,tree)=painter.scene(&root,width,1.,None).unwrap();assert_eq!(bounds(&tree,1).width(),expected);}
  let root=view("w-1/2 size-8 w-1/2 min-h-16",vec![]);let (_,tree)=painter.scene(&root,400.,1.,None).unwrap();assert_eq!((bounds(&tree,1).width(),bounds(&tree,1).height()),(200.,64.));
  let child=Node::new(Kind::Text,String::new(),"child".into(),vec![]).unwrap();let root=view("pl-6 p-4 pr-2",vec![child]);let (_,tree)=painter.scene(&root,400.,1.,None).unwrap();let child=bounds(&tree,2);assert_eq!((child.x0,child.y0,child.width()),(16.,16.,376.));
 }
}

#[cfg(test)] mod color_tests {
 use super::*;
 #[test] fn encoded_native_paint_declares_its_premultiplied_eight_bit_boundary() {
  let mut theme:serde_json::Value=serde_json::from_str(super::super::theme::DEFAULT_JSON).unwrap();theme["light"]["primary"]="#3366cc".into();
  let node=Node::new(Kind::View,"size-8 bg-primary/50".into(),String::new(),vec![]).unwrap().with_theme(&theme.to_string(),"light").unwrap();
  let mut painter=Painter::new();let (scene,_)=painter.scene(&node,100.,1.,None).unwrap();
  assert_eq!(scene.encoding().draw_tags.len(),1);assert_eq!(scene.encoding().draw_data,vec![u32::from_le_bytes([26,51,102,128])]);
 }
 #[test] fn surface_rings_and_explicit_outlines_are_independent_of_focus() {
  let node=Node::new(Kind::View,"size-8 bg-primary/50 ring-1 ring-foreground/10 ring".into(),String::new(),vec![]).unwrap();
  let mut painter=Painter::new();let item=painter.node(&node,TextStyle::default()).unwrap();assert_eq!(item.shadow_width,1.);assert!((item.shadow_color.alpha()-0.1).abs()<0.000001);assert_eq!(item.background.unwrap().alpha(),0.5);assert!(item.outline.is_some());
  let (scene,_)=painter.scene(&node,100.,1.,None).unwrap();assert_eq!(scene.encoding().draw_tags.len(),3);
 }
}

#[cfg(test)] mod opacity_bounds_tests {
 use super::*;
 #[test] fn disabled_group_bounds_include_child_overflow_and_surface_rings() {
  let child=Node::new(Kind::View,"w-64 h-8 shrink-0 ring-3".into(),String::new(),vec![]).unwrap().with_semantics(r#"{"tag":"span"}"#).unwrap();
  let node=Node::button_children(vec![crate::Child::Node(child)],"w-16 p-0 ring-1".into(),true,None).unwrap();let mut painter=Painter::new();let root=painter.node(&node,TextStyle::default()).unwrap();
  painter.tree.compute_layout(root.id,Size{width:AvailableSpace::Definite(400.),height:AvailableSpace::MaxContent}).unwrap();let bounds=painter.visual_bounds(&root,0.,0.).unwrap();
  assert_eq!((bounds.x0,bounds.x1),(-99.,163.));assert_eq!((bounds.y0,bounds.y1),(-3.,35.));
 }
}

#[cfg(test)] mod hover_color_tests {
 use super::*;
 #[test] fn hover_opacity_multiplies_color_and_child_rings_reset_to_theme() {
  let button=Node::button_children(vec![crate::Child::Text(crate::TextPart::Literal("Continue".into()))],"bg-foreground hover:bg-foreground/90".into(),false,None).unwrap();
  let child=Node::new(Kind::View,"size-8 ring-1".into(),String::new(),vec![]).unwrap();
  let root=Node::new(Kind::View,"ring-1 ring-foreground/10".into(),String::new(),vec![button,child]).unwrap();let mut painter=Painter::new();painter.hover.insert(2);let item=painter.node(&root,TextStyle::default()).unwrap();
  assert!((item.children[0].background.unwrap().alpha()-0.9).abs()<0.000001);
  assert_eq!(item.children[1].shadow_color,super::super::theme::Theme::default().color("light","ring").unwrap());assert!((item.shadow_color.alpha()-0.1).abs()<0.000001);
 }
}

#[cfg(test)] mod structural_tests {
 use super::*;
 #[test] fn structural_conditions_drive_native_layout_and_keep_source_order() {
  let leaf=Node::new(Kind::View,"p-4 group-data-[size=sm]/panel:px-3".into(),String::new(),vec![]).unwrap().with_semantics(r#"{"slot":"body"}"#).unwrap();
  let root=Node::new(Kind::View,"group/panel p-4 has-data-[slot=body]:pb-0 data-[size=sm]:pt-3".into(),String::new(),vec![leaf]).unwrap().with_semantics(r#"{"size":"sm"}"#).unwrap();
  let mut painter=Painter::new();let item=painter.node(&root,TextStyle::default()).unwrap();
  let style=painter.tree.style(item.id).unwrap();assert_eq!(style.padding.top,length(12.));assert_eq!(style.padding.bottom,length(0.));
  let style=painter.tree.style(item.children[0].id).unwrap();assert_eq!(style.padding.left,length(12.));assert_eq!(style.padding.top,length(16.));
  assert_eq!(item.children[0].semantics.slot,"body");
 }
}
