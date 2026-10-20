//! The event loop owns the surface, renderer and window. No global window registry.
use crate::{Kind, Node, style::{Utility,State}};
use parley::{Alignment, AlignmentOptions, FontContext, FontWeight, Layout, LayoutContext, PositionedLayoutItem, StyleProperty};
use std::sync::{Arc, Mutex};
use taffy::prelude::*;
use vello::kurbo::{Affine,Shape};
#[path = "native_geometry.rs"]
mod geometry;
#[path = "native_editing.rs"]
mod editing;
#[path = "native_viewport.rs"]
mod viewport;
#[path = "native_scrolling.rs"]
mod scrolling;
#[path = "native_inline.rs"]
mod inline;
use inline::{Brush,Inline};
use vello::peniko::{Color, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, RenderParams, Renderer, RendererOptions, Scene};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{WindowEvent, ElementState, MouseButton},
    keyboard::{Key, NamedKey, ModifiersState},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[derive(Clone, Copy)]
struct TextStyle {
    size: f32,
    weight:f32,
    line_height:parley::style::LineHeight,
    leading:Option<f32>,
    family:u8,
    italic:bool,
    strike:bool,
    underline:bool,
    align: u8,
    color: super::theme::ResolvedColor,
    cursor:u8,
    pointer_events:bool,
    selectable:bool,
}
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.,
            weight:400.,line_height:parley::style::LineHeight::NORMAL,leading:None,family:0,italic:false,strike:false,underline:false,
            align: 0,cursor:0,pointer_events:true,selectable:true,
            color: super::theme::ResolvedColor::rgb8(23,32,51),
        }
    }
}
fn paint_color(color:super::theme::ResolvedColor)->Color {Color::new(color.components())}
fn layout_text(text: &mut Layout<Brush>, width: f32, button: bool) {
    text.break_all_lines(Some(width.max(0.)));
    if button { text.align(Alignment::Center, AlignmentOptions::default()); }
}

#[derive(Clone)]
struct FieldInfo {value:String,placeholder:String,read_only:bool,required:bool}
struct Measure {text:Layout<Brush>,nowrap:bool}
struct Item {
    opacity:f32,
    suppress_outline:bool,
    placeholder_color:Option<super::theme::ResolvedColor>,
    field: Option<FieldInfo>,
    copy_error:Option<String>,
    text_style: TextStyle,
    role: Kind,
    semantics: super::Semantics,
    label: String,
    raw_text:String,
    identity:u64,
    id: NodeId,
    text: Option<Layout<Brush>>,
    color: super::theme::ResolvedColor,
    background: Option<super::theme::ResolvedColor>,
    radius: [f64;4],
    border: [f64;4],
    overflow:u8,
    border_color: super::theme::ResolvedColor,
    ring: super::theme::ResolvedColor,
    align: u8,
    shadow_width:f64,
    shadow_color:super::theme::ResolvedColor,
    outline:Option<super::theme::ResolvedColor>,
    disabled: bool,
    children: Vec<Item>,
    inline:Vec<Inline>,
}
struct Painter {
    viewport:viewport::Viewport,
    viewport_width:f32,
    scrolls:std::collections::BTreeMap<u64,scrolling::Region>,
    scroll_chains:std::collections::BTreeMap<u64,Vec<u64>>,
    scroll_order:Vec<u64>,
    control_order:Vec<u64>,
    tab_excluded:std::collections::BTreeSet<u64>,
    reveal_target:Option<u64>,
    cursors:std::collections::BTreeMap<u64,u8>,
    hit_regions:std::collections::BTreeMap<u64,Vec<vello::kurbo::BezPath>>,
    clip_stack:Vec<vello::kurbo::BezPath>,
    hover: std::collections::BTreeSet<u64>,
    pressed_nodes: std::collections::BTreeSet<u64>,
    focus: Option<u64>,
    fonts: FontContext,
    layouts: LayoutContext<Brush>,
    tree: TaffyTree<Measure>,
    editors: std::collections::BTreeMap<u64,editing::Editor>,
    live_editors: std::collections::BTreeSet<u64>,
    readonly:std::collections::BTreeMap<u64,inline::Selectable>,
    live_readonly:std::collections::BTreeSet<u64>,
    selected_text:Option<u64>,
    label_targets:std::collections::BTreeMap<u64,u64>,
}
impl Painter {
    fn new() -> Self {
        // Vello paints fractional logical coordinates. Rounding a measured text
        // width down before painting can add a line without increasing its height.
        let mut tree=TaffyTree::new();tree.disable_rounding();
        Self {
            viewport:Default::default(),
            viewport_width:0.,scrolls:Default::default(),scroll_chains:Default::default(),scroll_order:Vec::new(),control_order:Vec::new(),tab_excluded:Default::default(),reveal_target:None,cursors:Default::default(),hit_regions:Default::default(),clip_stack:Vec::new(),hover: Default::default(), pressed_nodes:Default::default(), focus: None,
            fonts: FontContext::new(),
            layouts: LayoutContext::new(),
            tree,editors:Default::default(),live_editors:Default::default(),readonly:Default::default(),live_readonly:Default::default(),selected_text:None,label_targets:Default::default(),
        }
    }
    fn node(&mut self,node:&Node,inherited:TextStyle)->Result<Item,String> {
        node.ensure_identities();let node=node.visible_root().ok_or("UI root is absent")?;
        self.themed_node(node,inherited,&super::theme::Theme::default(),"light",&mut Vec::new(),&std::collections::BTreeMap::new())
    }
    fn themed_node<'a>(&mut self, node: &'a Node, mut inherited: TextStyle, theme:&super::theme::Theme, mode:&str, ancestors:&mut Vec<&'a Node>,variables:&std::collections::BTreeMap<String,f32>) -> Result<Item, String> {
        let id=node.identity.get();
        if node.enabled_control() {self.control_order.push(id);}
        if node.semantics.tab_index==Some(-1){self.tab_excluded.insert(id);}
        let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));
        if node.theme.is_some() {inherited.size=theme.typography.base as f32;inherited.color=theme.color(mode,"foreground")?;}

        match node.tag(){"strong"=>inherited.weight=700.,"em"=>inherited.italic=true,"del"=>inherited.strike=true,"a" if ancestors.iter().any(|node|node.kind==Kind::Text)=>inherited.underline=true,_=>{}}
        let mut style = Style {
            flex_direction: FlexDirection::Column,
            ..Style::default()
        };
        if let Some(field)=&node.editable {
            style.size.width=length(240.);style.size.height=length(if field.multiline {64.}else{32.});
            let editor=self.editors.entry(id).or_insert_with(||editing::Editor::new(&field.value,field.multiline));
            editor.sync(&field.value);editor.read_only=field.read_only;editor.disabled=field.disabled;
            if field.disabled || field.read_only{editor.cancel_composition(&mut self.fonts,&mut self.layouts);}
            self.live_editors.insert(id);
        }
        let (mut background, mut radius) = (node.theme.as_ref().map(|_|theme.color(mode,"background").unwrap()), [0.;4]);
        if matches!(node.kind,Kind::Button|Kind::DisabledButton)&&node.semantics.href.is_empty(){inherited.align=1;style.flex_direction=FlexDirection::Row;style.align_items=AlignItems::CENTER;style.justify_content=JustifyContent::CENTER;}
        let (mut border,mut border_color,mut ring)=([0.;4],theme.color(mode,"border")?,theme.color(mode,"ring")?);
        let mut overflow=0;let mut opacity=if node.is_disabled(){0.6}else{1.};let mut suppress_outline=false;let mut placeholder_color=None;
        let (mut shadow_width,mut shadow_color,mut outline)=(0.,theme.color(mode,"ring")?,None);
        if matches!(node.kind,Kind::DisabledButton|Kind::Button)&&node.semantics.href.is_empty() {
            style.padding = Rect {
                left: length(16.),
                right: length(16.),
                top: length(8.),
                bottom: length(8.),
            };
        }
        let (rules,variables)=super::style::resolved_scoped_node_at_width(node,ancestors,theme,mode,variables,self.viewport_width);
        for (state,utility) in rules {
            if !match state {State::Base=>true,State::Hover=>self.hover.contains(&id) && !node.is_disabled(),State::Active=>self.pressed_nodes.contains(&id) && !node.is_disabled(),State::Focus=>self.focus==Some(id),State::Disabled=>node.is_disabled()} {continue;}
            match utility {
                Utility::Placeholder(color)=>placeholder_color=Some(color),Utility::Opacity(value)=>opacity=value,Utility::OutlineNone=>{outline=None;suppress_outline=true;},Utility::Cursor(value)=>inherited.cursor=value,Utility::SelectNone=>inherited.selectable=false,Utility::PointerEvents(value)=>inherited.pointer_events=value,Utility::FieldSizingContent=>{if node.kind==Kind::Textarea{style.size.height=auto();}},
                Utility::AspectRatio(value)=>style.aspect_ratio=value,Utility::Relative=>style.position=taffy::Position::Relative,Utility::Absolute=>style.position=taffy::Position::Absolute,
                Utility::Inset(side,value)=>{let value=match value{Length::Pixels(value)=>length(value),Length::Percent(value)=>percent(value),Length::Auto=>auto(),_=>unreachable!("validated inset")};match side{0=>style.inset.top=value,1=>style.inset.right=value,2=>style.inset.bottom=value,_=>style.inset.left=value}},
                Utility::Row=>style.flex_direction=FlexDirection::Row,
                Utility::Column=>style.flex_direction=FlexDirection::Column,
                Utility::GapAxis(axis,v)=>{if axis==0{style.gap.height=length(v);}else{style.gap.width=length(v);}},
                Utility::Padding(side,v)=>match side {0=>style.padding.top=length(v),1=>style.padding.right=length(v),2=>style.padding.bottom=length(v),_=>style.padding.left=length(v)},
                Utility::Display(grid)=>style.display=if grid{Display::Grid}else{Display::Flex},
                Utility::InlineFlex=>style.display=Display::Flex,
                Utility::Dimension(property,value)=>{
                    if value==super::style::Length::Fit {match property{0=>style.size.width=Dimension::fit_content(),1=>style.size.height=Dimension::fit_content(),_=>return Err("Fit-content is supported for preferred dimensions only".into())};continue;}
                    let value:LengthPercentageAuto=match value {super::style::Length::Pixels(v)=>length(v),super::style::Length::Percent(v)=>percent(v),super::style::Length::Viewport(axis)=>length(if axis==0{self.viewport_width}else{self.viewport.size.ok_or("Viewport height is unavailable")?.1 as f32}),super::style::Length::Auto|super::style::Length::None=>auto(),super::style::Length::Fit=>unreachable!()};
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
                Utility::FontSize(v)=>inherited.size=v,Utility::Weight(v)=>inherited.weight=v,Utility::DefaultLeading(v)=>inherited.line_height=parley::style::LineHeight::Absolute(v),Utility::Leading(v)=>inherited.leading=Some(v),Utility::FontFamily(v)=>inherited.family=v,
                Utility::Foreground(v)=>inherited.color=v,Utility::Background(v)=>background=Some(v),
                Utility::Corner(c,v)=>radius[c as usize]=v as f64,Utility::EdgeBorder(side,v)=>{border[side as usize]=v as f64;match side {0=>style.border.top=length(v),1=>style.border.right=length(v),2=>style.border.bottom=length(v),_=>style.border.left=length(v)}},Utility::Overflow(value)=>{overflow=value;let value=match value{1=>taffy::Overflow::Hidden,2=>taffy::Overflow::Scroll,_=>taffy::Overflow::Visible};style.overflow.x=value;style.overflow.y=value;},
                Utility::BorderColor(v)=>border_color=v,Utility::Ring(v)=>{ring=v;outline=Some(v);suppress_outline=false;},Utility::Align(v)=>inherited.align=v,
                Utility::RingWidth(v)=>shadow_width=v as f64,Utility::RingShadowColor(v)=>shadow_color=v,
                Utility::Items(v)=>style.align_items=match v {0=>AlignItems::FLEX_START,1=>AlignItems::CENTER,_=>AlignItems::STRETCH},
                Utility::Justify(v)=>style.justify_content=match v {0=>JustifyContent::FLEX_START,1=>JustifyContent::CENTER,_=>JustifyContent::SPACE_BETWEEN},
            }
        }
        // Taffy transfers max-height into an absent max-width. A separately specified width keeps its own bound.
        if style.aspect_ratio.is_some() && style.size.width.into_raw().is_length_or_percentage() && style.max_size.width.is_auto() && !style.max_size.height.is_auto() {style.max_size.width=length(f32::MAX);}
        ancestors.push(node);
        let mut children = node
            .visible_children()
            .map(|child| self.themed_node(child, inherited,theme,mode,ancestors,&variables))
            .collect::<Result<Vec<_>, _>>()?;
        ancestors.pop();
        let mut inline=Vec::new();let mut spans=Vec::new();let mut paragraph=String::new();
        if node.kind==Kind::Text && !children.is_empty(){for child in std::mem::take(&mut children){inline.push(inline::collect(child,&mut paragraph,&mut spans));}}
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
            let raw=self.editors.get(&id).map(|editor|editor.text.raw_text()).unwrap_or(if inline.is_empty(){&node.text}else{&paragraph});
            let display=node.editable.as_ref().filter(|_|raw.is_empty()).map(|field|field.placeholder.as_str()).unwrap_or(raw);
            let mut builder = self
                .layouts
                .ranged_builder(&mut self.fonts, display, 1., true);
            builder.push_default(StyleProperty::FontSize(inherited.size));
            builder.push_default(StyleProperty::FontWeight(FontWeight::new(
                inherited.weight,
            )));
            builder.push_default(StyleProperty::LineHeight(inherited.leading.map(parley::style::LineHeight::FontSizeRelative).unwrap_or(inherited.line_height)));
            builder.push_default(StyleProperty::from(match inherited.family {1=>parley::style::GenericFamily::Serif,2=>parley::style::GenericFamily::Monospace,_=>parley::style::GenericFamily::SystemUi}));
            inline::default_style(&mut builder,inherited);
            for (range,style) in spans {inline::range_style(&mut builder,range,style);}
            let layout: Layout<Brush> = builder.build(display);
            let id = self
                .tree
                .new_leaf_with_context(style, Measure{text:layout.clone(),nowrap:node.kind==Kind::Input})
                .map_err(|e| e.to_string())?;
            (Some(layout), id)
        };
        Ok(Item {
            identity:node.identity.get(),
            opacity,suppress_outline,placeholder_color,copy_error:node.copy.as_ref().map(|copy|copy.error.clone()),
            field:node.editable.as_ref().map(|field|FieldInfo{value:field.value.clone(),placeholder:field.placeholder.clone(),read_only:field.read_only,required:field.required}),
            raw_text:if inline.is_empty(){node.text.clone()}else{paragraph},
            text_style:inherited,
            role: node.kind,
            semantics: node.semantics.clone(),
            label: if matches!(node.kind,Kind::Button|Kind::DisabledButton) || node.kind==Kind::Text || node.tag()=="label" {node.content_text()}else{node.text.clone()},
            id,
            text,
            color: inherited.color,
            background,
            radius,
            border,overflow,border_color,ring,align:inherited.align,shadow_width,shadow_color,outline,
            disabled: node.is_disabled(),
            children,inline,
        })
    }
    fn scene(
        &mut self,
        node: &Node,
        width: f32,
        scale: f64,
        focus: Option<u64>,
    ) -> Result<(Scene, accesskit::TreeUpdate), String> {
        self.scene_focused(node,width,scale,focus,focus)
    }
    fn scene_focused(&mut self,node:&Node,width:f32,scale:f64,focus:Option<u64>,visible_focus:Option<u64>)->Result<(Scene,accesskit::TreeUpdate),String> {
        self.control_order.clear();self.tab_excluded.clear();self.scroll_chains.clear();self.scroll_order.clear();self.retain_owned_scrolls(node);
        self.tree.clear();self.hit_regions.clear();self.clip_stack.clear();self.live_editors.clear();self.live_readonly.clear();self.cursors.clear();self.viewport_width=width;
        self.focus=visible_focus;
        node.ensure_identities();
        if node.visible_root().is_none(){
            self.viewport.content=(0.,0.);self.viewport.scroll_to(0.,0.);
            for editor in self.editors.values_mut(){editor.cancel_composition(&mut self.fonts,&mut self.layouts);}self.editors.clear();self.label_targets.clear();
            return Ok((Scene::new(),accesskit::TreeUpdate{nodes:vec![(accesskit::NodeId(0),accesskit::Node::new(accesskit::Role::Window))],tree:Some(accesskit::TreeInfo::new(accesskit::NodeId(0))),tree_id:accesskit::TreeId::ROOT,focus:accesskit::NodeId(0)}));
        }
        let mut root = self.node(node, TextStyle::default())?;
        let (editors,fonts,layouts,live)=(&mut self.editors,&mut self.fonts,&mut self.layouts,&self.live_editors);
        editors.retain(|id,editor|{if live.contains(id){true}else{editor.cancel_composition(fonts,layouts);false}});
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
                    use taffy::MaybeResolve;
                    let mut leaf=style.clone();
                    if let Some(ratio)=style.aspect_ratio {
                        let dimensions=inputs.known_dimensions.or(style.size.maybe_resolve(inputs.parent_size,|_,_|0.)).maybe_apply_aspect_ratio(Some(ratio));
                        if dimensions.width.is_some() || dimensions.height.is_some() {
                            // Taffy's leaf ratio floor runs after max-height. Resolve the preference first, then clamp normally.
                            leaf.size=Size{width:dimensions.width.map_or(style.size.width,length),height:dimensions.height.map_or(style.size.height,length)};
                            leaf.aspect_ratio=None;
                        }
                    }
                    taffy::compute_leaf_layout(
                        inputs,
                        &leaf,
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
                            text.text.break_all_lines(if text.nowrap{None}else{width});
                            Size {
                                width: known.width.unwrap_or(text.text.width()),
                                height: known.height.unwrap_or(text.text.height()),
                            }
                        },
                    )
                },
            )
            .map_err(|e| e.to_string())?;
        let layout=self.tree.layout(root.id).map_err(|e|e.to_string())?;
        self.viewport.content=if root.overflow!=0{(layout.size.width as f64,layout.size.height as f64)}else{(layout.size.width.max(layout.scrollable_overflow_rect.right+layout.border.left) as f64,layout.size.height.max(layout.scrollable_overflow_rect.bottom+layout.border.top) as f64)};
        self.configure_scrolls(&root,&[])?;
        self.viewport.scroll_by(0.,0.);
        let mut nodes = Vec::new();
        let root_identity=self.accessible(&root, &mut nodes, 0., 0., scale)?;
        let reveal=self.reveal_target.take().or_else(||self.viewport.reveal_focus.then_some(focus).flatten());
        if let Some(target)=reveal {
            self.reveal_scrolled(target,&nodes,scale);
            nodes.clear();self.accessible(&root,&mut nodes,0.,0.,scale)?;
        }
        self.viewport.reveal_focus=false;
        let (dx,dy)=self.viewport.offset;
        for (_,node) in &mut nodes {if let Some(bounds)=node.bounds(){node.set_bounds(accesskit::Rect::new(bounds.x0-dx*scale,bounds.y0-dy*scale,bounds.x1-dx*scale,bounds.y1-dy*scale));}}
        let mut scene = Scene::new();
        if let Some((width,height))=self.viewport.size {
            let clip=vello::kurbo::Rect::new(0.,0.,width,height);scene.push_layer(Fill::NonZero,vello::peniko::BlendMode::default(),1.,Affine::scale(scale),&clip);
            let mut clip=clip.to_path(0.1);clip.apply_affine(Affine::scale(scale));self.clip_stack.push(clip);
        }
        self.paint(&mut root, &mut scene, -dx, -dy, scale, focus)?;
        let mut owned=std::collections::BTreeSet::new();fn collect(node:&Node,ids:&mut std::collections::BTreeSet<u64>){ids.insert(node.identity.get());for child in &node.children{collect(child,ids);}}collect(node,&mut owned);self.readonly.retain(|id,_|owned.contains(id));if self.selected_text.is_some_and(|id|!self.live_readonly.contains(&id)){self.selected_text=None;}
        if self.viewport.size.is_some(){self.clip_stack.pop();scene.pop_layer();}
        self.label_targets=associate_controls(node,&mut nodes);
        let mut window = accesskit::Node::new(accesskit::Role::Window);
        self.viewport.accessibility(&mut window,scale);
        window.set_children(vec![root_identity]);
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
        parent_x: f64,
        parent_y: f64,
        scale: f64,
    ) -> Result<accesskit::NodeId, String> {
        let id = accesskit::NodeId(item.identity);
        let layout = *self.tree.layout(item.id).map_err(|e| e.to_string())?;
        let (x, y) = (
            parent_x + layout.location.x as f64,
            parent_y + layout.location.y as f64,
        );
        let default_role=match item.role {
            Kind::View => accesskit::Role::GenericContainer,
            Kind::Text => accesskit::Role::Label,
            Kind::DisabledButton | Kind::Button => accesskit::Role::Button,
            Kind::Input => accesskit::Role::TextInput,
            Kind::Textarea => accesskit::Role::MultilineTextInput,
        };
        let role=match item.semantics.role.as_str(){
            _ if !item.semantics.href.is_empty()=>accesskit::Role::Link,
            "alert"=>accesskit::Role::Alert,"status"=>accesskit::Role::Status,"separator"=>accesskit::Role::Splitter,
            "group"=>accesskit::Role::Group,"note"=>accesskit::Role::Note,"region"=>accesskit::Role::Region,
            "none"|"presentation"=>accesskit::Role::GenericContainer,
            _ if matches!(item.semantics.tag.as_str(),"h1"|"h2"|"h3"|"h4"|"h5"|"h6")=>accesskit::Role::Heading,
            _ if item.semantics.tag=="nav"=>accesskit::Role::Navigation,
            _ if item.semantics.tag=="main"=>accesskit::Role::Main,
            _ if item.semantics.tag=="header"=>accesskit::Role::Header,
            _ if item.semantics.tag=="footer"=>accesskit::Role::Footer,
            _ if item.semantics.tag=="aside"=>accesskit::Role::Complementary,
            _ if item.semantics.tag=="article"=>accesskit::Role::Article,
            _ if item.semantics.tag=="blockquote"=>accesskit::Role::Blockquote,
            _ if item.semantics.tag=="ul"=>accesskit::Role::List,
            _ if item.semantics.tag=="ol"=>accesskit::Role::List,
            _ if item.semantics.tag=="li"=>accesskit::Role::ListItem,
            _ if item.semantics.tag=="table"=>accesskit::Role::Table,
            _ if item.semantics.tag=="thead"=>accesskit::Role::RowGroup,
            _ if item.semantics.tag=="tbody"=>accesskit::Role::RowGroup,
            _ if item.semantics.tag=="tr"=>accesskit::Role::Row,
            _ if item.semantics.tag=="th"=>accesskit::Role::ColumnHeader,
            _ if item.semantics.tag=="td"=>accesskit::Role::Cell,
            _ if item.semantics.tag=="label"=>accesskit::Role::Label,
            _ if item.semantics.tag=="p"=>accesskit::Role::Paragraph,
            _ if item.semantics.tag=="code"=>accesskit::Role::Code,
            _ if item.semantics.tag=="strong"=>accesskit::Role::Strong,
            _ if item.semantics.tag=="em"=>accesskit::Role::Emphasis,
            _ if item.semantics.tag=="del"=>accesskit::Role::ContentDeletion,
            _ if item.semantics.tag=="section" && !item.semantics.label.trim().is_empty()=>accesskit::Role::Region,
            _=>default_role,
        };
        let mut node = accesskit::Node::new(role);
        if let Some(current)=super::navigation::current_kind(&item.semantics.href){node.set_aria_current(if current=="page"{accesskit::AriaCurrent::Page}else{accesskit::AriaCurrent::Location});}
        if role==accesskit::Role::Heading{node.set_level((item.semantics.tag.as_bytes()[1]-b'1') as usize);}
        if role==accesskit::Role::Splitter{node.set_orientation(if item.semantics.orientation=="vertical"{accesskit::Orientation::Vertical}else{accesskit::Orientation::Horizontal});}
        if item.semantics.hidden{node.set_hidden();}
        if item.semantics.role=="alert"{node.set_live(accesskit::Live::Assertive);}else if item.semantics.role=="status"{node.set_live(accesskit::Live::Polite);}
        if !item.semantics.label.is_empty(){node.set_label(item.semantics.label.clone());}
        else if !item.label.is_empty() && item.inline.is_empty() && matches!(role,accesskit::Role::Label|accesskit::Role::Button|accesskit::Role::Link|accesskit::Role::Heading) {
            node.set_label(item.label.clone());
        }
        if let Some(field)=&item.field {
            node.set_value(field.value.clone());node.set_placeholder(field.placeholder.clone());
            if field.read_only{node.set_read_only();}if field.required{node.set_required();}
            if !item.disabled{node.add_action(accesskit::Action::Focus);if !field.read_only{node.add_action(accesskit::Action::SetValue);}}
        }
        if let Some(error)=&item.copy_error {node.set_live(accesskit::Live::Polite);if !error.is_empty(){node.set_description(error.clone());}}
        if item.semantics.invalid=="true" {node.set_invalid(accesskit::Invalid::True);}
        if item.disabled { node.set_disabled(); }
        else if item.role==Kind::Button {node.add_action(accesskit::Action::Click);node.add_action(accesskit::Action::Focus);}
        if item.role==Kind::View && item.semantics.tab_index.is_some(){node.add_action(accesskit::Action::Focus);}
        if item.overflow!=0{node.set_clips_children();}
        if item.overflow==2{if let Some(region)=self.scrolls.get(&item.identity){region.viewport.accessibility(&mut node,scale);}}
        node.set_bounds(accesskit::Rect::new(
            x * scale,
            y * scale,
            (x + layout.size.width as f64) * scale,
            (y + layout.size.height as f64) * scale,
        ));
        let (dx,dy)=self.scroll_offset(item.identity);
        let mut children = item
            .children
            .iter()
            .map(|child| self.accessible(child, nodes, x-dx, y-dy, scale))
            .collect::<Result<Vec<_>, _>>()?;
        if !item.inline.is_empty(){
            let text=self.inline_layout(item)?;let origin=(x+(layout.padding.left+layout.border.left) as f64,y+(layout.padding.top+layout.border.top) as f64);
            for part in &item.inline {children.push(inline::accessible(part,&text,origin,scale,nodes));}
        }
        if item.inline.is_empty() && item.field.is_none() && item.text.is_some() && !item.label.is_empty() && !matches!(role,accesskit::Role::Label|accesskit::Role::Button) {
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
        if item.overflow==0{for child in &item.children {bounds=bounds.union(self.visual_bounds(child,x,y)?);}}
        Ok(bounds)
    }
    fn paint(
        &mut self,
        item: &mut Item,
        scene: &mut Scene,
        parent_x: f64,
        parent_y: f64,
        scale: f64,
        focus: Option<u64>,
    ) -> Result<(), String> {
        let layout = self.tree.layout(item.id).map_err(|e| e.to_string())?;
        let (x, y) = (
            parent_x + layout.location.x as f64,
            parent_y + layout.location.y as f64,
        );
        let id = item.identity;
        let transform = Affine::scale(scale);
        if item.opacity<1. {
            scene.push_layer(
                Fill::NonZero,
                vello::peniko::BlendMode::default(),
                item.opacity,
                transform,
                &self.visual_bounds(item,parent_x,parent_y)?,
            );
        }
        let rect=vello::kurbo::Rect::new(x,y,x+layout.size.width as f64,y+layout.size.height as f64);
        let outer=geometry::rounded(rect,item.radius);let inner=geometry::inner(rect,item.radius,item.border);
        let mut hits=self.clip_stack.clone();let mut hit=outer.clone();hit.apply_affine(transform);hits.push(hit);if !item.text_style.pointer_events{hits.push(vello::kurbo::BezPath::new());}self.hit_regions.insert(id,hits);self.cursors.insert(id,if !item.semantics.href.is_empty()&&item.text_style.cursor==0{3}else if (item.field.is_some()||item.role==Kind::Text&&item.text_style.selectable) && item.text_style.cursor==0{1}else{item.text_style.cursor});
        if item.shadow_width>0. {
            let half=item.shadow_width/2.;
            scene.stroke(&vello::kurbo::Stroke::new(item.shadow_width).with_join(vello::kurbo::Join::Miter),transform,paint_color(item.shadow_color),None,
                &geometry::rounded(rect.inflate(half,half),geometry::outset(item.radius,half)));
        }
        if let Some(color)=item.background {scene.fill(Fill::NonZero,transform,paint_color(color),None,&outer);}
        if item.border.iter().any(|v|*v>0.) {
            let mut border=outer.clone();border.extend(inner.elements().iter().copied());
            scene.fill(Fill::EvenOdd,transform,paint_color(item.border_color),None,&border);
        }
        if item.overflow!=0 {scene.push_layer(Fill::NonZero,vello::peniko::BlendMode::default(),1.,transform,&inner);let mut clip=inner.clone();clip.apply_affine(transform);self.clip_stack.push(clip);}
        let content_width=(layout.size.width-layout.padding.left-layout.padding.right-layout.border.left-layout.border.right).max(0.);
        let content_height=(layout.size.height-layout.padding.top-layout.padding.bottom-layout.border.top-layout.border.bottom).max(0.);
        let origin=(x+(layout.padding.left+layout.border.left) as f64,y+(layout.padding.top+layout.border.top) as f64);
        if let Some(editor)=self.editors.get_mut(&id) {
            editor.configure(item.text_style,content_width);
            editor.origin=origin;let viewport=(content_width as f64,content_height as f64);if editor.viewport!=viewport{editor.reveal=true;}editor.viewport=viewport;
            let mut text=editor.text.layout(&mut self.fonts,&mut self.layouts).clone();
            if focus==Some(id) && editor.reveal{editor.ensure_cursor_visible();editor.reveal=false;}
            else {editor.scroll.0=editor.scroll.0.min((text.width() as f64-editor.viewport.0).max(0.));editor.scroll.1=editor.scroll.1.min((text.height() as f64-editor.viewport.1).max(0.));}
            let clip=vello::kurbo::Rect::new(origin.0,origin.1,origin.0+content_width as f64,origin.1+content_height as f64);
            scene.push_layer(Fill::NonZero,vello::peniko::BlendMode::default(),1.,transform,&clip);
            let offset=(origin.0-editor.scroll.0,origin.1-editor.scroll.1);
            if focus==Some(id) {
                for (selection,_) in editor.text.selection_geometry(){
                    scene.fill(Fill::NonZero,transform,paint_color(item.ring.opacity(0.25)),None,&vello::kurbo::Rect::new(selection.x0+offset.0,selection.y0+offset.1,selection.x1+offset.0,selection.y1+offset.1));
                }
            }
            if editor.text.raw_text().is_empty() {
                if let Some(placeholder)=&mut item.text {if item.role==Kind::Input{placeholder.break_all_lines(None);}else{layout_text(placeholder,content_width,false);}paint_text(scene,placeholder,paint_color(item.placeholder_color.unwrap_or_else(||item.color.opacity(0.6))),transform,origin);}
            }else{paint_text(scene,&mut text,paint_color(item.color),transform,offset);}
            if focus==Some(id) && !editor.disabled {
                if let Some(caret)=editor.text.cursor_geometry(1.){scene.fill(Fill::NonZero,transform,paint_color(item.color),None,&vello::kurbo::Rect::new(caret.x0+offset.0,caret.y0+offset.1,caret.x1+offset.0,caret.y1+offset.1));}
                if let Some(preedit)=editor.text.raw_compose(){
                    // Preedit text stays separate from the committed application value.
                    let selection=parley::editing::Selection::new(parley::editing::Cursor::from_byte_index(&text,preedit.start,parley::Affinity::Downstream),parley::editing::Cursor::from_byte_index(&text,preedit.end,parley::Affinity::Upstream));
                    selection.geometry_with(&text,|r,_|scene.fill(Fill::NonZero,transform,paint_color(item.color),None,&vello::kurbo::Rect::new(r.x0+offset.0,r.y1+offset.1-1.,r.x1+offset.0,r.y1+offset.1)));
                }
            }
            scene.pop_layer();
        }else if let Some(text) = &mut item.text {
            layout_text(text,content_width,matches!(item.role,Kind::Button|Kind::DisabledButton)&&item.semantics.href.is_empty());
            text.align(match item.align {1=>Alignment::Center,2=>Alignment::End,_=>Alignment::Start},AlignmentOptions::default());
            if item.role==Kind::Text && item.text_style.selectable {
                self.live_readonly.insert(id);let selected=self.readonly.entry(id).or_insert_with(||inline::Selectable::new(item.raw_text.clone(),text.clone(),origin));selected.sync(&item.raw_text,text,origin);
                if self.selected_text==Some(id){selected.paint(scene,transform,item.ring);}
            }
            paint_text(scene,text,paint_color(item.color),transform,origin);
            for part in &item.inline {inline::paint(part,text,scene,origin,transform,&self.clip_stack,&mut self.hit_regions,&mut self.cursors,self.focus);}
        }
        let (dx,dy)=self.scroll_offset(id);
        for child in &mut item.children {
            self.paint(child, scene, x-dx, y-dy, scale, focus)?;
        }
        if item.overflow!=0{self.clip_stack.pop();scene.pop_layer();}
        if let Some(color)=item.outline.or_else(||(!item.suppress_outline && self.focus==Some(id) && (matches!(item.role,Kind::Button|Kind::Input|Kind::Textarea)||item.semantics.tab_index.is_some())).then_some(item.ring)) {
            scene.stroke(&vello::kurbo::Stroke::new(2.).with_join(vello::kurbo::Join::Miter), transform, paint_color(color), None,
                &geometry::rounded(rect.inflate(3.,3.),geometry::outset(item.radius,3.)));
        }
        if item.opacity<1. {
            scene.pop_layer();
        }
        Ok(())
    }
}

fn associate_controls(root:&Node,nodes:&mut [(accesskit::NodeId,accesskit::Node)])->std::collections::BTreeMap<u64,u64> {
    fn collect<'a>(node:&'a Node,all:&mut std::collections::BTreeMap<u64,&'a Node>) {if let Some(node)=node.visible_root(){all.insert(node.identity.get(),node);for child in &node.children{collect(child,all);}}}
    fn labelable(node:&Node)->bool {matches!(node.kind,Kind::Button|Kind::DisabledButton|Kind::Input|Kind::Textarea)}
    fn first(node:&Node)->Option<u64> {let node=node.visible_root()?;if labelable(node){return Some(node.identity.get());}node.children.iter().find_map(first)}
    let mut all=std::collections::BTreeMap::new();collect(root,&mut all);
    let mut ids=std::collections::BTreeMap::new();for (&id,node) in &all {if !node.semantics.id.is_empty(){ids.entry(node.semantics.id.as_str()).or_insert(id);}}
    let mut labels=std::collections::BTreeMap::<u64,Vec<accesskit::NodeId>>::new();let mut targets=std::collections::BTreeMap::new();
    for (&id,node) in all.iter().filter(|(_,node)|node.tag()=="label") {
        let target=if node.semantics.html_for.is_empty(){first(node)}else{ids.get(node.semantics.html_for.as_str()).copied().filter(|id|all.get(id).is_some_and(|node|labelable(node)))};
        if let Some(target)=target{targets.insert(id,target);labels.entry(target).or_default().push(accesskit::NodeId(id));}
    }
    for (id,accessible) in nodes {
        let Some(node)=all.get(&id.0) else{continue;};
        if node.semantics.label.is_empty(){if let Some(labels)=labels.get(&id.0){accessible.set_labelled_by(labels.clone());}}
        let described=node.semantics.described_by.split_ascii_whitespace().filter_map(|name|ids.get(name).copied()).map(accesskit::NodeId).collect::<Vec<_>>();
        if !described.is_empty(){accessible.set_described_by(described);}
    }
    targets
}

fn paint_text(scene:&mut Scene,text:&mut Layout<Brush>,color:Color,transform:Affine,origin:(f64,f64)) {
    for line in text.lines(){for run in line.items(){if let PositionedLayoutItem::GlyphRun(glyph_run)=run {
        let font_run=glyph_run.run();let coordinates=font_run.normalized_coords().iter().map(|coord|coord.to_bits()).collect::<Vec<_>>();
        scene.draw_glyphs(font_run.font()).font_size(font_run.font_size()).normalized_coords(&coordinates).transform(transform*Affine::translate(origin)).brush(glyph_run.style().brush.color.map(paint_color).unwrap_or(color)).draw(Fill::NonZero,glyph_run.positioned_glyphs().map(|glyph|vello::Glyph{id:glyph.id,x:glyph.x,y:glyph.y}));
        let style=glyph_run.style();let brush=style.brush;let metrics=font_run.font_metrics();
        for (enabled,y) in [(brush.underline,glyph_run.baseline()-metrics.underline_offset),(brush.strike,glyph_run.baseline()-metrics.strikethrough_offset)] {
            if enabled {scene.stroke(&vello::kurbo::Stroke::new(metrics.underline_size.max(1.) as f64),transform,brush.color.map(paint_color).unwrap_or(color),None,&vello::kurbo::Line::new((origin.0+glyph_run.offset() as f64,origin.1+y as f64),(origin.0+(glyph_run.offset()+glyph_run.advance()) as f64,origin.1+y as f64)));}
        }
    }}}
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
fn focusable(node:&accesskit::Node)->bool {node.supports_action(accesskit::Action::Focus) && !node.is_disabled() && !node.is_hidden()}
struct App {
    location:String,
    reading_positions:std::collections::BTreeMap<String,std::collections::BTreeMap<u64,(f64,f64)>>,
    proxy: winit::event_loop::EventLoopProxy<accesskit::ActionRequest>,
    pointer: (f64,f64),
    pressed: Option<u64>,
    dragging_text:bool,
    dragging_selection:bool,
    focus: Option<u64>,
    shift: bool,
    modifiers: ModifiersState,
    focus_visible: bool,
    clipboard:Option<arboard::Clipboard>,
    key_pressed: Option<u64>,
    close_after_frame: bool,
    presented: bool,
    #[cfg(feature = "acceptance")]
    resize_step: usize,
    deadline: Option<std::time::Instant>,
    node: Node,
    title: String,
    options:super::window::Options,
    context: RenderContext,
    painter: Painter,
    active: Option<Active>,
    error: Option<String>,
}
impl App {
    fn clickable(&self,id:u64)->bool {self.active.as_ref().is_some_and(|active|active.snapshot.lock().unwrap().nodes.iter().any(|(node_id,node)|node_id.0==id&&node.supports_action(accesskit::Action::Click)))}
    fn link(&self,id:u64)->bool {self.active.as_ref().is_some_and(|active|active.snapshot.lock().unwrap().nodes.iter().any(|(node_id,node)|node_id.0==id&&node.role()==accesskit::Role::Link))}
    fn visible_hit(&self,id:u64)->bool {self.painter.hit_regions.get(&id).is_some_and(|paths|paths.iter().all(|path|path.contains(vello::kurbo::Point::new(self.pointer.0,self.pointer.1))))}
    fn controls(&self)->Vec<(u64,accesskit::Rect)> {
        self.active.as_ref().map(|active|{
            let snapshot=active.snapshot.lock().unwrap();
            self.painter.control_order.iter().filter_map(|id|snapshot.nodes.iter().find(|(node_id,node)|node_id.0==*id&&focusable(node)).and_then(|(_,node)|node.bounds()).map(|bounds|(*id,bounds))).collect()
        }).unwrap_or_default()
    }
    fn pointer_nodes(&self)->std::collections::BTreeSet<u64> {
        self.active.as_ref().map(|active|active.snapshot.lock().unwrap().nodes.iter().filter_map(|(id,node)|node.bounds().filter(|r|self.pointer.0>=r.x0 && self.pointer.0<r.x1 && self.pointer.1>=r.y0 && self.pointer.1<r.y1 && self.visible_hit(id.0)).map(|_|id.0)).collect()).unwrap_or_default()
    }
    fn pressed_nodes(&self)->std::collections::BTreeSet<u64> {
        let mut ids=std::collections::BTreeSet::new();let Some(mut current)=self.pressed.or(self.key_pressed) else{return ids;};
        let Some(active)=&self.active else{return ids;};let snapshot=active.snapshot.lock().unwrap();
        let parents=snapshot.nodes.iter().flat_map(|(id,node)|node.children().iter().map(move|child|(child.0,id.0))).collect::<std::collections::BTreeMap<_,_>>();
        while ids.insert(current) {let Some(parent)=parents.get(&current) else{break;};current=*parent;}ids
    }
    fn hit(&self)->Option<u64> {
        self.controls().into_iter().rev().find(|(id,r)|self.pointer.0>=r.x0 && self.pointer.0<r.x1 && self.pointer.1>=r.y0 && self.pointer.1<r.y1 && self.visible_hit(*id)).map(|(id,_)|id).or_else(||self.painter.label_targets.iter().find(|(label,target)|self.visible_hit(**label) && self.controls().iter().any(|(id,_)|id==*target)).map(|(_,target)|*target))
    }
    fn activate(&mut self,id:u64,event_loop:&ActiveEventLoop) {
        if let Some((serial,value))=self.node.copy_node_mut(id).and_then(Node::begin_copy){
            let result=self.clipboard().and_then(|clipboard|clipboard.set_text(value).map_err(|error|format!("Clipboard unavailable: {error}")));
            if let Some(node)=self.node.copy_node_mut(id){node.finish_copy(serial,result);}
            if let Some(active)=&self.active{active.window.request_redraw();}return;
        }
        match self.node.activate(id).and_then(|_|self.node.refresh()) {
            Ok(_)=>if let Some(active)=&self.active {active.window.request_redraw();},
            Err(error)=>self.fail(event_loop,error),
        }
    }

    fn set_focus(&mut self,next:Option<u64>) {
        if self.focus!=next {
            self.painter.viewport.reveal_focus=next.is_some();
            if let Some(editor)=self.focus.and_then(|id|self.painter.editors.get_mut(&id)){editor.cancel_composition(&mut self.painter.fonts,&mut self.painter.layouts);}
            self.focus=next;self.key_pressed=None;if let Some(editor)=next.and_then(|id|self.painter.editors.get_mut(&id)){editor.reveal=true;}
        }
    }
    fn pointer_edit(&mut self,id:u64,extend:bool) {
        let scale=self.active.as_ref().map(|active|active.window.scale_factor()).unwrap_or(1.);
        if let Some(editor)=self.painter.editors.get_mut(&id){editor.pointer((self.pointer.0/scale,self.pointer.1/scale),extend,&mut self.painter.fonts,&mut self.painter.layouts);}
    }
    fn publish_edit(&mut self,id:u64)->Result<(),String> {
        let Some(editor)=self.painter.editors.get(&id) else{return Ok(());};
        let value=editor.committed();self.node.input(id,&value)?;self.node.refresh()?;if let Some(actual)=self.node.control_value(id){self.painter.editors.get_mut(&id).unwrap().sync(actual);}Ok(())
    }
    fn clipboard(&mut self)->Result<&mut arboard::Clipboard,String> {
        if self.clipboard.is_none(){self.clipboard=Some(arboard::Clipboard::new().map_err(|error|format!("Clipboard unavailable: {error}"))?);}
        Ok(self.clipboard.as_mut().unwrap())
    }
    fn ime(&mut self,event:winit::event::Ime)->Result<(),String> {
        let Some(id)=self.focus else{return Ok(());};
        let Some(editor)=self.painter.editors.get_mut(&id).filter(|editor|!editor.disabled && !editor.read_only) else{return Ok(());};
        match event {
            winit::event::Ime::Preedit(value,cursor)=>editor.preedit(&value,cursor,&mut self.painter.fonts,&mut self.painter.layouts),
            winit::event::Ime::Commit(value)=>{let before=editor.snapshot();editor.insert(&value,&mut self.painter.fonts,&mut self.painter.layouts);self.publish_edit(id)?;self.painter.editors.get_mut(&id).unwrap().record(before);Ok(())},
            winit::event::Ime::Disabled=>{editor.cancel_composition(&mut self.painter.fonts,&mut self.painter.layouts);Ok(())},
            _=>Ok(()),
        }
    }
    fn text_key(&mut self,event:&winit::event::KeyEvent)->Result<(),String> {
        let snapshot=self.focus.and_then(|id|self.painter.editors.get(&id).map(|editor|(id,editor.snapshot())));
        let command=if cfg!(target_os="macos"){self.modifiers.super_key()}else{self.modifiers.control_key()};
        let history=command && matches!(&event.logical_key,Key::Character(value) if matches!(value.to_lowercase().as_str(),"z"|"y"));
        self.text_key_inner(event)?;
        if !history{if let Some((id,before))=snapshot{if let Some(editor)=self.painter.editors.get_mut(&id){editor.record(before);}}}Ok(())
    }
    fn text_key_inner(&mut self,event:&winit::event::KeyEvent)->Result<(),String> {
        if event.state!=ElementState::Pressed{return Ok(());}
        let Some(id)=self.focus else{return Ok(());};
        let command=if cfg!(target_os="macos"){self.modifiers.super_key()}else{self.modifiers.control_key()};
        let word=if cfg!(target_os="macos"){self.modifiers.alt_key()}else{self.modifiers.control_key()};
        let line_command=cfg!(target_os="macos") && command;
        let Some(editor)=self.painter.editors.get_mut(&id).filter(|editor|!editor.disabled) else{return Ok(());};
        if editor.navigation(&event.logical_key,self.shift,word,line_command,&mut self.painter.fonts,&mut self.painter.layouts){return Ok(());}
        if editor.text.is_composing(){return Ok(());}
        let letter=match &event.logical_key{Key::Character(value)=>value.to_lowercase(),_=>String::new()};
        if command && letter=="a" {editor.text.driver(&mut self.painter.fonts,&mut self.painter.layouts).select_all();return Ok(());}
        if command && matches!(letter.as_str(),"c"|"x") {
            if letter=="x" && editor.read_only{return Ok(());}
            let Some(selected)=editor.text.selected_text().map(str::to_owned) else{return Ok(());};
            if let Err(error)=self.clipboard().and_then(|clipboard|clipboard.set_text(selected).map_err(|error|error.to_string())){eprintln!("Clipboard copy failed: {error}");return Ok(());}
            if letter=="x" {self.painter.editors.get_mut(&id).unwrap().text.driver(&mut self.painter.fonts,&mut self.painter.layouts).delete_selection();self.publish_edit(id)?;}
            return Ok(());
        }
        if editor.read_only{return Ok(());}
        if command && matches!(letter.as_str(),"z"|"y") {
            editor.undo(letter=="y" || self.shift,&mut self.painter.fonts,&mut self.painter.layouts);self.publish_edit(id)?;return Ok(());
        }
        if command && letter=="v" {
            let pasted=match self.clipboard().and_then(|clipboard|clipboard.get_text().map_err(|error|error.to_string())){Ok(value)=>value,Err(error)=>{eprintln!("Clipboard paste failed: {error}");return Ok(());}};
            self.painter.editors.get_mut(&id).unwrap().insert(&pasted,&mut self.painter.fonts,&mut self.painter.layouts);self.publish_edit(id)?;return Ok(());
        }
        match &event.logical_key {
            Key::Named(NamedKey::Backspace)=>{let mut driver=editor.text.driver(&mut self.painter.fonts,&mut self.painter.layouts);if line_command{driver.select_to_line_start();driver.delete_selection();}else if word{driver.backdelete_word();}else{driver.backdelete();}},
            Key::Named(NamedKey::Delete)=>{let mut driver=editor.text.driver(&mut self.painter.fonts,&mut self.painter.layouts);if word{driver.delete_word();}else{driver.delete();}},
            Key::Named(NamedKey::Enter) if editor.multiline()=>editor.insert("\n",&mut self.painter.fonts,&mut self.painter.layouts),
            _ if !command && !self.modifiers.control_key() && !self.modifiers.super_key()=>if let Some(text)=&event.text {if !text.chars().any(char::is_control){editor.insert(text,&mut self.painter.fonts,&mut self.painter.layouts);}},
            _=>{},
        }
        self.publish_edit(id)
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl ToString) {
        self.error = Some(error.to_string());
        self.active = None;
        event_loop.exit();
    }
    fn open(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let attributes=Window::default_attributes().with_title(&self.title).with_visible(false)
            .with_inner_size(LogicalSize::new(self.options.width,self.options.height))
            .with_min_inner_size(LogicalSize::new(self.options.min_width,self.options.min_height));
        #[cfg(target_os="macos")]
        let attributes=if self.options.titlebar==super::window::Titlebar::Custom {
            use winit::platform::macos::WindowAttributesExtMacOS;
            attributes.with_fullsize_content_view(true).with_titlebar_transparent(true).with_title_hidden(true)
        }else{attributes};
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|e| e.to_string())?,
        );
        let size = window.inner_size();
        self.painter.viewport.size=Some((size.width as f64/window.scale_factor(),size.height as f64/window.scale_factor()));
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
        if let Some(path)=super::navigation::current(){
            if path!=self.location {
                let reading_nodes=self.node.reading_nodes();for positions in self.reading_positions.values_mut(){positions.retain(|id,_|reading_nodes.contains(id));}
                if !self.location.is_empty(){self.reading_positions.insert(self.location.clone(),self.painter.scroll_order.iter().filter(|id|reading_nodes.contains(id)).filter_map(|id|self.painter.scrolls.get(id).map(|region|(*id,region.viewport.offset))).collect());}
                let restored=self.reading_positions.get(&path);
                if let Some(positions)=restored{for (id,(x,y)) in positions{if let Some(region)=self.painter.scrolls.get_mut(id){region.viewport.scroll_to(*x,*y);}}}
                fn target(node:&Node,fragment:Option<&str>)->Option<u64>{let node=node.visible_root()?;if fragment.map_or(node.tag()=="h1",|fragment|node.semantics.id==fragment){return Some(node.identity.get());}node.children.iter().find_map(|child|target(child,fragment))}
                self.node.ensure_identities();self.painter.reveal_target=if restored.is_some(){None}else{path.split_once('#').and_then(|(_,fragment)|target(&self.node,Some(fragment)))};self.location=path;
            }
        }
        self.painter.hover=self.pointer_nodes();self.painter.pressed_nodes=self.pressed_nodes();
        let cursor=self.painter.cursors.iter().rev().find(|(id,_)|self.visible_hit(**id)).map(|(_,cursor)|*cursor).unwrap_or(0);
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
        let cursor=match cursor {1=>winit::window::CursorIcon::Text,2=>winit::window::CursorIcon::NotAllowed,3=>winit::window::CursorIcon::Pointer,_=>winit::window::CursorIcon::Default};
        active.window.set_cursor(cursor);
        let scale = active.window.scale_factor();
        let viewport=Some((size.width as f64/scale,size.height as f64/scale));
        if self.painter.viewport.size!=viewport{self.painter.viewport.reveal_focus=self.focus.is_some();}
        self.painter.viewport.size=viewport;
        let visible_focus=self.focus.filter(|id|self.focus_visible || self.painter.editors.contains_key(id));
        let (scene, mut snapshot) =
            self.painter
                .scene_focused(&self.node, (size.width as f64 / scale) as f32, scale, self.focus,visible_focus)?;
        if let Some((_, root)) = snapshot.nodes.iter_mut().find(|(id, _)| id.0 == 0) {
            root.set_label(self.title.clone());
        }
        if self.focus.is_some_and(|id|!snapshot.nodes.iter().any(|(node_id,node)|node_id.0==id && focusable(node))) {self.focus=None;self.key_pressed=None;}
        snapshot.focus=accesskit::NodeId(self.focus.unwrap_or(0));
        let editor=self.focus.and_then(|id|self.painter.editors.get(&id)).filter(|editor|!editor.disabled && !editor.read_only);
        active.window.set_ime_allowed(editor.is_some());
        if let Some(editor)=editor {
            let area=editor.text.ime_cursor_area();
            active.window.set_ime_cursor_area(winit::dpi::LogicalPosition::new(editor.origin.0-editor.scroll.0+area.x0,editor.origin.1-editor.scroll.1+area.y0),LogicalSize::new((area.x1-area.x0).max(1.),(area.y1-area.y0).max(1.)));
        }
        let mut observed=Vec::new();self.node.observed_nodes(false,&mut observed);let visibility=self.painter.visibility(&observed,&snapshot.nodes,scale);
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
        let mut changed=false;for (id,visible) in visibility{changed|=self.node.visibility_changed(id,visible)?;}
        if changed{self.node.refresh()?;active.window.request_redraw();}
        Ok(true)
    }
}
impl ApplicationHandler<accesskit::ActionRequest> for App {
    fn user_event(&mut self,event_loop:&ActiveEventLoop,request:accesskit::ActionRequest) {
        if request.target_tree!=accesskit::TreeId::ROOT {return;}
        let scale=self.active.as_ref().map(|active|active.window.scale_factor()).unwrap_or(1.);
        if request.action==accesskit::Action::ScrollIntoView {
            if self.painter.scroll_chains.contains_key(&request.target_node.0){self.painter.reveal_target=Some(request.target_node.0);if let Some(active)=&self.active{active.window.request_redraw();}}
            return;
        }
        if self.painter.scroll_action(&request,scale){if let Some(active)=&self.active{active.window.request_redraw();}return;}
        if !self.controls().iter().any(|(id,_)|*id==request.target_node.0) {return;}
        match request.action {
            accesskit::Action::Click if self.clickable(request.target_node.0)=>self.activate(request.target_node.0,event_loop),
            accesskit::Action::SetValue=>if let Some(accesskit::ActionData::Value(value))=request.data {
                if let Err(error)=self.node.input(request.target_node.0,&value).and_then(|_|self.node.refresh()){self.fail(event_loop,error);}else if let Some(active)=&self.active{active.window.request_redraw();}
            },
            accesskit::Action::Focus=>{self.set_focus(Some(request.target_node.0));self.focus_visible=true;if let Some(active)=&self.active {active.window.request_redraw();}},
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
            WindowEvent::CursorMoved{position,..}=>{self.pointer=(position.x,position.y);if self.dragging_text{if let Some(id)=self.pressed{self.pointer_edit(id,true);}}if self.dragging_selection{self.select_text(true);}if let Some(active)=&self.active {active.window.request_redraw();}},
            WindowEvent::CursorLeft{..}=>{self.pointer=(-1.,-1.);if let Some(active)=&self.active {active.window.request_redraw();}},
            WindowEvent::ModifiersChanged(modifiers)=>{self.modifiers=modifiers.state();self.shift=self.modifiers.shift_key();},
            WindowEvent::Focused(false)=>{self.pressed=None;self.dragging_selection=false;self.painter.selected_text=None;self.set_focus(None);},
            WindowEvent::MouseInput{state,button:MouseButton::Left,..}=>{
                if state==ElementState::Pressed && self.options.titlebar==super::window::Titlebar::Custom && self.node.window_drag_at(|id|self.visible_hit(id)) {
                    if let Some(active)=&self.active {if let Err(error)=active.window.drag_window(){self.fail(event_loop,error);}}
                    return;
                }
                if state==ElementState::Pressed {self.pressed=self.hit();self.set_focus(self.pressed);self.focus_visible=false;self.dragging_text=self.pressed.is_some_and(|id|self.painter.editors.contains_key(&id) && self.visible_hit(id));if self.dragging_text{self.pointer_edit(self.pressed.unwrap(),self.shift);}
                    let previous_selection=self.painter.selected_text;self.dragging_selection=false;self.painter.selected_text=None;
                    if !self.pressed.is_some_and(|id|self.clickable(id)||self.painter.editors.contains_key(&id)){self.painter.selected_text=self.pointer_nodes().iter().rev().find(|id|self.painter.live_readonly.contains(id)).copied();self.dragging_selection=self.painter.selected_text.is_some();self.select_text(self.shift&&previous_selection==self.painter.selected_text);}
                }
                else if let Some(id)=self.pressed.take() {self.dragging_text=false;self.dragging_selection=false;if self.hit()==Some(id) && self.clickable(id){self.activate(id,event_loop);}}
                if state==ElementState::Released{self.dragging_selection=false;}
                if let Some(active)=&self.active {active.window.request_redraw();}
            },
            WindowEvent::KeyboardInput{event,..}=>{
                self.focus_visible=true;
                if event.logical_key==Key::Named(NamedKey::Tab) {
                    self.painter.selected_text=None;
                    let ids=self.controls().into_iter().map(|(id,_)|id).filter(|id|!self.painter.tab_excluded.contains(id)).collect::<Vec<_>>();
                    let mut next=self.focus;keyboard(&mut next,&mut self.key_pressed,&ids,self.shift,&event.logical_key,event.state,event.repeat);self.set_focus(next);
                }else if self.readonly_key(&event) {
                }else if self.focus.is_some_and(|id|self.painter.editors.contains_key(&id)) {
                    if event.logical_key==Key::Named(NamedKey::Escape) {if let Some(editor)=self.focus.and_then(|id|self.painter.editors.get_mut(&id)){editor.cancel_composition(&mut self.painter.fonts,&mut self.painter.layouts);}}
                    else if let Err(error)=self.text_key(&event){self.fail(event_loop,error);}
                }else{
                    if event.state==ElementState::Pressed {
                        self.painter.scroll_key(self.focus,&event.logical_key,self.shift);
                    }
                    let ids=self.controls().into_iter().map(|(id,_)|id).filter(|id|self.clickable(*id) && !(event.logical_key==Key::Named(NamedKey::Space) && self.link(*id))).collect::<Vec<_>>();
                    if let Some(id)=keyboard(&mut self.focus,&mut self.key_pressed,&ids,self.shift,&event.logical_key,event.state,event.repeat){self.activate(id,event_loop);}
                }
                if let Some(active)=&self.active {active.window.request_redraw();}
            },
            WindowEvent::Ime(event)=>{
                if let Err(error)=self.ime(event){self.fail(event_loop,error);}
                if let Some(active)=&self.active {active.window.request_redraw();}
            },
            WindowEvent::MouseWheel{delta,..}=>{
                let (mut dx,mut dy)=match delta {winit::event::MouseScrollDelta::LineDelta(x,y)=>(x as f64*32.,y as f64*32.),winit::event::MouseScrollDelta::PixelDelta(p)=>{let scale=self.active.as_ref().unwrap().window.scale_factor();(p.x/scale,p.y/scale)}};
                if let Some(id)=self.hit() && let Some(editor)=self.painter.editors.get_mut(&id) {
                    let before=editor.scroll;
                    let text=editor.text.layout(&mut self.painter.fonts,&mut self.painter.layouts);
                    editor.scroll.0=(editor.scroll.0-dx).clamp(0.,(text.width() as f64-editor.viewport.0).max(0.));editor.scroll.1=(editor.scroll.1-dy).clamp(0.,(text.height() as f64-editor.viewport.1).max(0.));
                    dx-=before.0-editor.scroll.0;dy-=before.1-editor.scroll.1;
                }
                let target=self.painter.scroll_order.iter().rev().find(|id|self.painter.scrolls.get(id).is_some_and(|region|region.wheel)&&self.visible_hit(**id)).copied();
                self.painter.scroll_chain(target,-dx,-dy);
                if let Some(active)=&self.active{active.window.request_redraw();}
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
    run_configured(node,title,"{}")
}
pub fn run_configured(node:Node,title:String,options:&str)->Result<(),String> {
    run_window(node,title,false,options)
}
#[cfg(feature = "acceptance")]
pub fn verify_window(node: Node, title: String) -> Result<(), String> {
    run_window(node, title, true,"{}")
}
#[cfg(feature = "acceptance")]
pub fn verify_window_configured(node:Node,title:String,options:&str)->Result<(),String> {run_window(node,title,true,options)}
fn run_window(mut node: Node, title: String, close_after_frame: bool,options:&str) -> Result<(), String> {
    let options=super::window::Options::parse(options)?;
    #[cfg(not(target_os="macos"))]
    if options.titlebar==super::window::Titlebar::Custom{return Err("Custom titlebars currently require macOS; use titlebar: native on this platform".into());}
    node.refresh()?;
    let event_loop = EventLoop::<accesskit::ActionRequest>::with_user_event().build().map_err(|e| e.to_string())?;
    let mut app = App {
        location:String::new(),
        reading_positions:Default::default(),
        proxy: event_loop.create_proxy(), pointer:(-1.,-1.),pressed:None,dragging_text:false,dragging_selection:false,focus:None,shift:false,modifiers:ModifiersState::empty(),focus_visible:false,clipboard:None,key_pressed:None,
        close_after_frame,
        presented: false,
        #[cfg(feature = "acceptance")]
        resize_step: 0,
        deadline: close_after_frame
            .then(|| std::time::Instant::now() + std::time::Duration::from_secs(20)),
        node,
        title,
        options,
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
    fn application_theme_visibility_and_header_hits_share_native_node_identities() {
        let mode=crate::Signal::new(String::from("light"));let hidden=crate::Signal::new(false);let test=hidden.handle();
        let field=Node::input_control(None,Some("draft".into()),"{}","w-48 h-12".into(),None).unwrap();
        let panel=Node::view(vec![crate::Child::Node(field)],"gap-4".into()).unwrap().with_hidden(move||test.get()).unwrap();
        let disabled=Node::button_children(vec![],"w-24 h-12".into(),true,None).unwrap();
        let mut root=Node::view(vec![crate::Child::Node(panel),crate::Child::Node(disabled)],"gap-4 p-4".into()).unwrap()
            .with_semantics(r#"{"windowDragRegion":true}"#).unwrap().with_theme_binding(include_str!("theme.json"),mode.text_binding(Clone::clone)).unwrap();
        root.refresh().unwrap();let field=root.children[0].children[0].identity.get();let button=root.children[1].identity.get();
        let mut painter=Painter::new();let (_,first)=painter.scene(&root,640.,1.,Some(field)).unwrap();let bounds=first.nodes.iter().find(|(id,_)|id.0==button).unwrap().1.bounds().unwrap();
        let point=vello::kurbo::Point::new(bounds.x0+2.,bounds.y0+2.);
        assert!(!root.window_drag_at(|id|painter.hit_regions.get(&id).is_some_and(|paths|paths.iter().all(|path|path.contains(point)))));
        root.input(field,"kept 🦀").unwrap();mode.set("dark".into()).unwrap();root.refresh().unwrap();
        let (_,dark)=painter.scene(&root,640.,1.,Some(field)).unwrap();assert_eq!(dark.nodes.iter().find(|(id,_)|id.0==field).unwrap().1.value(),Some("kept 🦀"));
        let item=painter.node(&root,TextStyle::default()).unwrap();assert_eq!(item.background,Some(crate::theme::Theme::default().color("dark","background").unwrap()));
        hidden.set(true).unwrap();root.refresh().unwrap();let (_,hidden_tree)=painter.scene(&root,640.,1.,None).unwrap();assert!(!hidden_tree.nodes.iter().any(|(id,_)|id.0==field));
        hidden.set(false).unwrap();root.refresh().unwrap();let (_,restored)=painter.scene(&root,480.,2.,Some(field)).unwrap();assert_eq!(restored.nodes.iter().find(|(id,_)|id.0==field).unwrap().1.value(),Some("kept 🦀"));
    }
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
mod conditional_tests {
    use super::*;
    use crate::{Child,Signal};
    #[test]
    fn empty_live_text_runs_create_no_layout_gap_and_keep_adjacent_controls() {
        let text=Signal::new(String::new());let read=text.handle();let update=text.handle();
        let input=Node::input_control(None,Some("stable".into()),"{}",String::new(),None).unwrap();
        let mut root=Node::view(vec![Child::Text(crate::TextPart::Signal(read.text_binding(Clone::clone))),Child::Node(input)],"flex-row gap-4".into()).unwrap().own_state(text).unwrap();root.refresh().unwrap();let stable=root.children[1].identity.get();let mut painter=Painter::new();
        let (_,empty)=painter.scene(&root,600.,1.,Some(stable)).unwrap();assert_eq!(root.visible_children().count(),1);assert_eq!(empty.nodes.iter().find(|(id,_)|id.0==stable).unwrap().1.bounds().unwrap().x0,0.);
        update.set("Ready".into()).unwrap();root.refresh().unwrap();let (_,present)=painter.scene(&root,600.,1.,Some(stable)).unwrap();assert_eq!(root.visible_children().count(),2);assert!(present.nodes.iter().find(|(id,_)|id.0==stable).unwrap().1.bounds().unwrap().x0>16.);
        update.set(String::new()).unwrap();root.refresh().unwrap();let (_,removed)=painter.scene(&root,600.,1.,Some(stable)).unwrap();assert_eq!(removed.nodes.iter().find(|(id,_)|id.0==stable).unwrap().1.bounds().unwrap().x0,0.);assert_eq!(painter.editors.len(),1);
        drop(root);assert!(update.get().is_err());
    }
    #[test]
    fn conditional_layout_and_accessibility_keep_retained_editor_identity_and_ime() {
        let show=Signal::new(true);let select=show.handle();
        let branch=Node::branch(move||select.get(),||Ok(Some(Node::input_control(None,Some("branch".into()),"{}",String::new(),None)?.with_semantics(r#"{"label":"Branch"}"#)?)),||Ok(None)).unwrap();
        let stable=Node::input_control(None,Some("stable".into()),"{}",String::new(),None).unwrap().with_semantics(r#"{"id":"stable","label":"Stable"}"#).unwrap();
        let mut root=Node::view(vec![Child::Node(branch),Child::Node(stable)],"flex-row gap-4".into()).unwrap();root.refresh().unwrap();
        let old=root.children[0].visible_root().unwrap().identity.get();let stable=root.children[1].identity.get();let mut painter=Painter::new();
        let (_,first)=painter.scene(&root,600.,1.,Some(stable)).unwrap();let bounds=first.nodes.iter().find(|(id,_)|id.0==stable).unwrap().1.bounds().unwrap();assert_eq!(bounds.x0,256.);
        for id in [old,stable]{let editor=painter.editors.get_mut(&id).unwrap();editor.text.driver(&mut painter.fonts,&mut painter.layouts).select_byte_range(1,3);editor.preedit("漢",Some((3,3)),&mut painter.fonts,&mut painter.layouts).unwrap();}
        let preedit=painter.editors[&stable].text.raw_text().to_owned();show.set(false).unwrap();root.refresh().unwrap();
        let (_,hidden)=painter.scene(&root,600.,1.,Some(stable)).unwrap();assert!(!painter.editors.contains_key(&old));assert!(!hidden.nodes.iter().any(|(id,_)|id.0==old));
        let stable_accessible=hidden.nodes.iter().find(|(id,_)|id.0==stable).unwrap();assert_eq!(stable_accessible.1.bounds().unwrap().x0,0.);assert_eq!(stable_accessible.1.label(),Some("Stable"));
        assert_eq!(painter.editors[&stable].text.raw_text(),preedit);assert!(painter.editors[&stable].text.is_composing());
        show.set(true).unwrap();root.refresh().unwrap();let fresh=root.children[0].visible_root().unwrap().identity.get();assert!(fresh>stable);
        let (_,shown)=painter.scene(&root,600.,1.,Some(stable)).unwrap();assert!(shown.nodes.iter().any(|(id,_)|id.0==fresh));assert_eq!(painter.editors[&fresh].text.raw_text(),"branch");assert!(!painter.editors[&fresh].text.is_composing());assert!(painter.editors[&stable].text.is_composing());
        let mut focus=Some(stable);let mut pressed=None;keyboard(&mut focus,&mut pressed,&[fresh,stable],false,&Key::Named(NamedKey::Tab),ElementState::Pressed,false);assert_eq!(focus,Some(fresh));
    }
    #[test]
    fn conditional_root_can_disappear_without_a_layout_or_accessibility_placeholder() {
        let show=Signal::new(true);let select=show.handle();let mut root=Node::branch(move||select.get(),||Ok(Some(Node::input_control(None,None,"{}",String::new(),None)?)),||Ok(None)).unwrap();
        root.refresh().unwrap();let mut painter=Painter::new();painter.scene(&root,600.,1.,None).unwrap();assert_eq!(painter.editors.len(),1);
        show.set(false).unwrap();root.refresh().unwrap();let (_,snapshot)=painter.scene(&root,600.,1.,None).unwrap();assert_eq!(snapshot.nodes.len(),1);assert_eq!(snapshot.nodes[0].0,accesskit::NodeId(0));assert!(painter.editors.is_empty());
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
  assert_eq!(button.background,Some(super::super::theme::ResolvedColor::rgb8(0x12,0x34,0x56)));assert_eq!(button.radius,[13.;4]);assert_eq!(button.border,[1.;4]);assert_eq!(button.align,1);
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

#[cfg(test)] mod card_style_tests {
 use super::*;
 #[test] fn inherited_spacing_typography_and_clips_affect_real_native_layout() {
  let text=Node::new(Kind::Text,"text-sm leading-snug font-medium font-serif".into(),"Title".into(),vec![]).unwrap();
  let button=Node::button("Clipped".into(),"w-64 h-32 shrink-0".into(),false,||Ok(())).unwrap();
  let root=Node::new(Kind::View,"w-16 h-16 overflow-hidden rounded-xl border-t [--panel:--spacing(3)] px-(--panel)".into(),String::new(),vec![text,button]).unwrap();
  let mut painter=Painter::new();let item=painter.node(&root,TextStyle::default()).unwrap();let style=painter.tree.style(item.id).unwrap();assert_eq!(style.padding.left,length(12.));assert_eq!(style.border.top,length(1.));assert_eq!(style.border.bottom,length(0.));assert_eq!(item.overflow,1);
  let mut text=item.children[0].text.clone().unwrap();text.break_all_lines(None);assert!((text.height()-14.*1.375).abs()<0.02,"{}",text.height());
  let (_,tree)=painter.scene(&root,64.,1.,None).unwrap();assert!(tree.nodes.iter().any(|(id,n)|id.0==1 && n.clips_children()));
  let clips=&painter.hit_regions[&3];assert!(clips.len()>=2);assert!(!clips.iter().all(|path|path.contains(vello::kurbo::Point::new(80.,40.))));
  painter.scene(&root,64.,2.,None).unwrap();let clips=&painter.hit_regions[&3];assert!(clips.iter().all(|path|path.contains(vello::kurbo::Point::new(40.,80.))));assert!(!clips.iter().all(|path|path.contains(vello::kurbo::Point::new(160.,80.))));
 }
}

#[cfg(test)]
mod editable_tests {
 use super::*;
 use crate::{Child,TextPart};
 fn field(value:&str,multiline:bool)->Node {Node::input_control(None,Some(value.into()),&format!(r#"{{"multiline":{multiline},"required":true}}"#),"w-full border border-input rounded-lg px-2.5 py-1 outline-none focus-visible:ring-3 placeholder:text-muted-foreground".into(),None).unwrap()}
 #[test]
 fn editable_accessibility_keeps_labels_descriptions_values_and_control_actions() {
  let label=Node::view(vec![Child::Text(TextPart::Literal("Full name".into()))],String::new()).unwrap().with_semantics(r#"{"tag":"label","htmlFor":"name"}"#).unwrap();
  let input=field("Ada",false).with_semantics(r#"{"id":"name","describedBy":"hint missing","invalid":"true"}"#).unwrap();
  let hint=Node::new(Kind::Text,String::new(),"Use your full name".into(),vec![]).unwrap().with_semantics(r#"{"id":"hint"}"#).unwrap();
  let textarea=field("One\nTwo",true).with_read_only(||Ok(true)).unwrap();
  let root=Node::new(Kind::View,"p-4 gap-2".into(),String::new(),vec![label,input,hint,textarea]).unwrap();let mut painter=Painter::new();let (_,tree)=painter.scene(&root,640.,2.,Some(4)).unwrap();
  let input=&tree.nodes.iter().find(|(id,_)|id.0==4).unwrap().1;
  assert_eq!(input.role(),accesskit::Role::TextInput);assert_eq!(input.value(),Some("Ada"));assert_eq!(input.labelled_by(),&[accesskit::NodeId(2)]);assert_eq!(input.described_by(),&[accesskit::NodeId(5)]);assert!(input.is_required());assert_eq!(input.invalid(),Some(accesskit::Invalid::True));assert!(input.supports_action(accesskit::Action::SetValue));
  let textarea=&tree.nodes.iter().find(|(id,_)|id.0==6).unwrap().1;assert_eq!(textarea.role(),accesskit::Role::MultilineTextInput);assert!(textarea.is_read_only());assert!(textarea.supports_action(accesskit::Action::Focus));assert!(!textarea.supports_action(accesskit::Action::SetValue));
  assert_eq!(painter.label_targets.get(&2),Some(&4));assert_eq!(painter.editors.len(),2);assert_eq!(painter.editors[&4].text.raw_text(),"Ada");
 }
 #[test]
 fn native_editing_keeps_controlled_application_values_and_updates_the_rendered_editor() {
  use std::{cell::RefCell,rc::Rc};
  let state=Rc::new(RefCell::new(String::from("Ada")));let read=Rc::downgrade(&state);let write=read.clone();
  let mut input=Node::input_control(Some(Box::new(move||Ok(read.upgrade().ok_or("disposed")?.borrow().clone()))),None,"{}","w-full h-8".into(),Some(Box::new(move|value,_,_|{*write.upgrade().ok_or("disposed")?.borrow_mut()=value.to_owned();Ok(())}))).unwrap();
  let mut painter=Painter::new();painter.scene(&input,160.,1.,Some(1)).unwrap();let editor=painter.editors.get_mut(&1).unwrap();editor.text.driver(&mut painter.fonts,&mut painter.layouts).select_all();let before=editor.snapshot();editor.insert("日本🙂",&mut painter.fonts,&mut painter.layouts);editor.record(before);
  assert!(input.input(1,&editor.committed()).unwrap());input.refresh().unwrap();assert_eq!(*state.borrow(),"日本🙂");painter.scene(&input,160.,1.,Some(1)).unwrap();assert_eq!(painter.editors[&1].text.raw_text(),"日本🙂");
  let editor=painter.editors.get_mut(&1).unwrap();editor.undo(false,&mut painter.fonts,&mut painter.layouts);assert_eq!(editor.committed(),"Ada");editor.undo(true,&mut painter.fonts,&mut painter.layouts);assert_eq!(editor.committed(),"日本🙂");
  drop(input);drop(state);painter.editors.clear();
 }
 #[test]
 fn native_utility_states_keep_fractional_spacing_breakpoints_and_placeholder_colors() {
  let mut theme=super::super::theme::Theme::default();theme.spacing.md=20.;
  let input=field("",false).with_theme(&serde_json::to_string(&theme).unwrap(),"dark").unwrap();let mut input=input;
  input.class_name="w-full h-8 px-2.5 text-base md:text-sm placeholder:text-muted-foreground outline-none aria-invalid:border-destructive disabled:opacity-50 dark:bg-input/30".into();input.semantics.invalid="true".into();input.apply_disabled(true);
  let mut painter=Painter::new();painter.viewport_width=800.;let item=painter.node(&input,TextStyle::default()).unwrap();let layout=painter.tree.style(item.id).unwrap();assert_eq!(layout.padding.left,length(12.5));assert_eq!(item.text_style.size,14.);assert_eq!(item.opacity,0.5);assert!(item.suppress_outline);assert_eq!(item.placeholder_color,Some(theme.color("dark","muted-foreground").unwrap()));assert_eq!(item.border_color,theme.color("dark","destructive").unwrap());assert!((item.background.unwrap().alpha()-0.3).abs()<0.00001);
  painter.viewport_width=767.;let item=painter.node(&input,TextStyle::default()).unwrap();assert_eq!(item.text_style.size,16.);
 }
}
#[cfg(test)]
mod label_editable_tests {
 use super::*;
 use crate::{Child,TextPart};
 #[test]
 fn implicit_label_names_exclude_the_associated_editable_value() {
  let input=Node::input_control(None,Some("Ada".into()),"{}",String::new(),None).unwrap();let label=Node::view(vec![Child::Text(TextPart::Literal("Name".into())),Child::Node(input)],String::new()).unwrap().with_semantics(r#"{"tag":"label"}"#).unwrap();assert_eq!(label.content_text(),"Name");
  let mut painter=Painter::new();let (_,tree)=painter.scene(&label,320.,1.,None).unwrap();let node=|id|&tree.nodes.iter().find(|(n,_)|n.0==id).unwrap().1;assert_eq!(node(1).label(),Some("Name"));assert_eq!(node(3).labelled_by(),&[accesskit::NodeId(1)]);assert_eq!(node(3).value(),Some("Ada"));
 }
}

#[cfg(test)]mod semantic_content_tests {
 use super::*;
 fn view(classes:&str,semantics:&str,children:Vec<Node>)->Node{Node::view(children.into_iter().map(super::super::Child::Node).collect(),classes.into()).unwrap().with_semantics(semantics).unwrap()}
 fn key(value:&str)->Node{view("inline-flex flex-row h-5 w-fit min-w-5 items-center justify-center gap-1 rounded-sm px-1 font-sans text-xs font-medium bg-muted pointer-events-none",r#"{"tag":"kbd","slot":"kbd"}"#,vec![Node::text(vec![super::super::TextPart::Literal(value.into())],String::new()).unwrap()])}
 fn rect(tree:&accesskit::TreeUpdate,id:u64)->accesskit::Rect{tree.nodes.iter().find(|(node,_)|node.0==id).unwrap().1.bounds().unwrap()}
 #[test]fn keycap_fit_content_and_vertical_separator_keep_layout_and_noninteractive_semantics(){
  let separator=view("shrink-0 bg-border data-horizontal:h-px data-horizontal:w-full data-vertical:w-px data-vertical:self-stretch",r#"{"role":"separator","orientation":"vertical"}"#,vec![]);
  let root=view("flex-row items-center h-12 gap-4","",vec![key("Ctrl"),separator,key("K")]);let mut painter=Painter::new();let (_,tree)=painter.scene(&root,320.,1.,None).unwrap();
  let first=rect(&tree,2);let separator=rect(&tree,4);let last=rect(&tree,5);assert_eq!(first.height(),20.);assert!(first.width()>=20.&&first.width()<100.);assert_eq!(last.height(),20.);assert_eq!(separator.width(),1.);assert_eq!(separator.height(),48.);assert!(first.x1<separator.x0&&separator.x1<last.x0);
  let node=&tree.nodes.iter().find(|(id,_)|id.0==4).unwrap().1;assert_eq!(node.role(),accesskit::Role::Splitter);assert_eq!(node.orientation(),Some(accesskit::Orientation::Vertical));assert!(!node.supports_action(accesskit::Action::Focus));assert!(painter.hit_regions[&2].iter().any(|path|path.elements().is_empty()));
 }
 #[test]fn horizontal_separator_and_custom_theme_scale_remain_independent_of_pixel_border(){
  let separator=view("shrink-0 bg-border data-horizontal:h-px data-horizontal:w-full",r#"{"role":"separator"}"#,vec![]);
  let mut theme=serde_json::to_value(super::super::theme::Theme::default()).unwrap();theme["spacing"]["md"]=20.into();theme["typography"]["base"]=20.into();
  let root=view("p-4 gap-4","",vec![key("A"),separator]).with_theme(&theme.to_string(),"dark").unwrap();let mut painter=Painter::new();let (_,tree)=painter.scene(&root,320.,1.,None).unwrap();assert_eq!(rect(&tree,2).height(),25.);assert_eq!(rect(&tree,4).height(),1.);assert_eq!(rect(&tree,4).width(),280.);
  let item=painter.node(&root,TextStyle::default()).unwrap();assert_eq!(item.children[0].text_style.size,15.);assert_eq!(item.children[0].children[0].text_style.size,15.);
 }
}

#[cfg(test)] mod viewport_tests {
 use super::*;
 #[test] fn composed_button_label_keeps_measurement_and_paint_in_agreement_on_resize() {
  let mut painter=Painter::new();
  for size in [12.,14.,15.,16.,17.,18.,20.,24.] {
  let mut theme=super::super::theme::Theme::default();theme.typography.base=size;
  let root=Node::new(Kind::View,"p-4".into(),String::new(),vec![Node::button_children(vec![crate::Child::Text(crate::TextPart::Literal("Toggle second counter".into()))],"rounded text-center px-4 py-2".into(),false,None).unwrap()]).unwrap().with_theme(&serde_json::to_string(&theme).unwrap(),"light").unwrap();
  for width in [640.,800.,480.,160.,220.,960.,640.] {
   let (_,tree)=painter.scene(&root,width,1.,None).unwrap();
   let bounds=tree.nodes.iter().find(|(id,_)|id.0==3).unwrap().1.bounds().unwrap();
   let item=painter.node(&root,TextStyle::default()).unwrap();let mut text=item.children[0].children[0].text.clone().unwrap();
   layout_text(&mut text,bounds.width() as f32,false);
   assert!(text.height() as f64<=bounds.height()+0.02,"font={size}, width={width}, label bounds={bounds:?}, painted height={}",text.height());
  }
  }
 }
 #[test] fn window_scroll_keeps_hit_accessibility_and_editing_coordinates_together() {
  let spacer=Node::new(Kind::View,"h-64 shrink-0".into(),String::new(),vec![]).unwrap();
  let button=Node::button("Below the fold".into(),String::new(),false,||Ok(())).unwrap();
  let input=Node::input_control(None,Some("Editable".into()),"{}","w-full h-8".into(),None).unwrap();
  let root=Node::new(Kind::View,"p-4 gap-4".into(),String::new(),vec![spacer,button,input]).unwrap();
  let mut painter=Painter::new();painter.viewport.size=Some((320.,120.));
  let (_,initial)=painter.scene(&root,320.,2.,None).unwrap();
  let button=root.children[1].identity.get();let input=root.children[2].identity.get();
  let rect=|tree:&accesskit::TreeUpdate,id|tree.nodes.iter().find(|(node,_)|node.0==id).unwrap().1.bounds().unwrap();
  let before=rect(&initial,button);assert!(before.y0>240.);assert!(painter.viewport.maximum().1>0.);
  painter.viewport.reveal_focus=true;
  let (_,focused)=painter.scene(&root,320.,2.,Some(button)).unwrap();let after=rect(&focused,button);
  assert!(after.y0>=0. && after.y1<=240.01);assert_eq!(after.y0,before.y0-painter.viewport.offset.1*2.);
  let point=vello::kurbo::Point::new((after.x0+after.x1)/2.,(after.y0+after.y1)/2.);
  assert!(painter.hit_regions[&button].iter().all(|path|path.contains(point)));
  assert!(!painter.hit_regions[&button].iter().all(|path|path.contains(vello::kurbo::Point::new(point.x,241.))));
  let window=&focused.nodes.iter().find(|(id,_)|id.0==0).unwrap().1;
  assert_eq!(window.scroll_y(),Some(painter.viewport.offset.1*2.));assert!(window.clips_children());assert!(window.supports_action(accesskit::Action::SetScrollOffset));
  painter.viewport.reveal_focus=true;let (_,focused)=painter.scene(&root,320.,2.,Some(input)).unwrap();let bounds=rect(&focused,input);
  assert!(bounds.y0>=0. && bounds.y1<=240.01);assert_eq!(painter.editors[&input].origin.1,bounds.y0/2.);
  painter.viewport.scroll_to(1e6,1e6);assert_eq!(painter.viewport.offset,painter.viewport.maximum());
  painter.viewport.size=Some((320.,1200.));painter.scene(&root,320.,2.,None).unwrap();assert_eq!(painter.viewport.offset,(0.,0.));
  assert!(!painter.viewport.scroll_to(f64::NAN,f64::INFINITY));
 }

}

#[cfg(test)]
mod navigation_semantic_tests {
 use super::*;
 #[test]
 fn native_links_and_markdown_structures_keep_accessibility_roles(){
  let location=crate::navigation::location(String::from,Clone::clone).unwrap();
  crate::navigation::navigate("/docs").unwrap();
  let link=crate::navigation::link(vec![crate::Child::Text(crate::TextPart::Literal("Docs".into()))],String::new(),"/docs").unwrap();
  let item=Node::view(vec![crate::Child::Text(crate::TextPart::Literal("Item".into()))],String::new()).unwrap().with_semantics(r#"{"tag":"li"}"#).unwrap();
  let list=Node::view(vec![crate::Child::Node(item)],String::new()).unwrap().with_semantics(r#"{"tag":"ul"}"#).unwrap();
  let cell=Node::view(vec![crate::Child::Text(crate::TextPart::Literal("Value".into()))],String::new()).unwrap().with_semantics(r#"{"tag":"td"}"#).unwrap();
  let row=Node::view(vec![crate::Child::Node(cell)],"flex-row".into()).unwrap().with_semantics(r#"{"tag":"tr"}"#).unwrap();
  let table=Node::view(vec![crate::Child::Node(row)],String::new()).unwrap().with_semantics(r#"{"tag":"table"}"#).unwrap();
  let mut root=Node::view(vec![crate::Child::Node(link),crate::Child::Node(list),crate::Child::Node(table)],String::new()).unwrap().own_state(location).unwrap();root.refresh().unwrap();
  let (_,snapshot)=Painter::new().scene(&root,640.,1.,None).unwrap();
  for role in [accesskit::Role::Link,accesskit::Role::List,accesskit::Role::ListItem,accesskit::Role::Table,accesskit::Role::Row,accesskit::Role::Cell]{assert!(snapshot.nodes.iter().any(|(_,node)|node.role()==role),"{role:?}");}
  let link=&snapshot.nodes.iter().find(|(_,node)|node.role()==accesskit::Role::Link).unwrap().1;assert_eq!(link.label(),Some("Docs"));assert_eq!(link.aria_current(),Some(accesskit::AriaCurrent::Page));
 }
}

#[cfg(test)]mod aspect_ratio_tests {
 use super::*;
 #[test]fn aspect_ratio_geometry_matches_preferred_dimensions_and_constraints(){
  let mut observed=Vec::new();
  for container in [0,1,2] {
  for contents in [false,true] {
  for (index,(classes,width,height)) in [("w-full aspect-square",320.,320.),("w-full aspect-video",320.,180.),("w-full aspect-[3/4]",120.,160.),("w-full aspect-[1.5]",317.5,317.5/1.5),("w-full aspect-video h-32",320.,128.),("w-full aspect-video min-h-64",320.,256.),("w-full aspect-square max-h-48",320.,192.)].into_iter().enumerate() {
   let children=if contents{vec![super::super::Child::Node(Node::text(vec![super::super::TextPart::Literal("Content".into())],String::new()).unwrap())]}else{vec![]};let node=Node::view(children,format!("relative min-w-0 shrink-0 {classes}")).unwrap();let root=if container==0{node}else{Node::view(vec![super::super::Child::Node(node)],if container==1{"".into()}else{"grid grid-cols-1 items-start".into()}).unwrap()};let identity=if container==0{1}else{2};let mut painter=Painter::new();
   for scale in [1.,1.5,2.] {let (_,tree)=painter.scene(&root,width,scale,None).unwrap();let rect=tree.nodes.iter().find(|(id,_)|id.0==identity).unwrap().1.bounds().unwrap();assert!((rect.width()/scale-width as f64).abs()<0.02,"{classes}: {rect:?}");assert!((rect.height()/scale-height as f64).abs()<0.02,"{classes}: {rect:?}");if scale==1. && !contents && container==0 {observed.push(serde_json::json!({"name":(["square","video","portrait","fractional-width","explicit-height","minimum-height","maximum-height"][index]),"width":rect.width(),"height":rect.height()}));}}
  }
  }
  }
  println!("ASPECT_GEOMETRY {}",serde_json::json!({"cases":observed}));
 }
}

#[cfg(test)]mod positioning_tests {
 use super::*;
 #[test] fn parent_positioning_has_matching_flow_bounds_and_source_order(){
  let cases=[("absolute top-2 right-2 w-8 h-8",214.,10.,32.,32.),("absolute inset-x-4 top-1/2 h-8",18.,96.,220.,32.),("relative top-2 left-1/4 w-8 h-8 shrink-0",73.,26.,32.,32.),("absolute -left-2 -top-2 w-8 h-8",-6.,-6.,32.,32.)];
  let mut observed=Vec::new();for display in ["flex","grid"]{for (index,(classes,x,y,width,height)) in cases.into_iter().enumerate(){
   let child=Node::view(vec![],classes.into()).unwrap();let root=Node::view(vec![crate::Child::Node(child)],format!("{display} h-48 border-2 p-4")).unwrap();root.ensure_identities();let identity=root.children[0].identity.get();let mut painter=Painter::new();
   for scale in [1.,1.5,2.]{let (_,tree)=painter.scene(&root,256.,scale,None).unwrap();let rect=tree.nodes.iter().find(|(id,_)|id.0==identity).unwrap().1.bounds().unwrap();for (actual,expected) in [(rect.x0,x),(rect.y0,y),(rect.width(),width),(rect.height(),height)]{assert!((actual/scale-expected).abs()<0.05,"{display} {classes}: {rect:?}");}if scale==1.&&display=="flex"{observed.push(serde_json::json!({"index":index,"x":rect.x0,"y":rect.y0,"width":rect.width(),"height":rect.height()}));}}
  }}println!("POSITION_GEOMETRY {}",serde_json::json!({"cases":observed}));
 }
}
