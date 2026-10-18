//! The event loop owns the surface, renderer and window. No global window registry.
use crate::{Kind, Node, style::Utility};
use parley::{FontContext, FontWeight, Layout, LayoutContext, PositionedLayoutItem, StyleProperty};
use std::sync::{Arc, Mutex};
use taffy::prelude::*;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Color, Fill};
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, RenderParams, Renderer, RendererOptions, Scene};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[derive(Clone, Copy)]
struct TextStyle {
    size: f32,
    bold: bool,
    color: [u8; 3],
}
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.,
            bold: false,
            color: [23, 32, 51],
        }
    }
}
struct Item {
    role: Kind,
    label: String,
    id: NodeId,
    text: Option<Layout<()>>,
    color: [u8; 3],
    background: Option<[u8; 3]>,
    radius: f64,
    disabled: bool,
    children: Vec<Item>,
}
struct Painter {
    fonts: FontContext,
    layouts: LayoutContext<()>,
    tree: TaffyTree<Layout<()>>,
}
impl Painter {
    fn new() -> Self {
        Self {
            fonts: FontContext::new(),
            layouts: LayoutContext::new(),
            tree: TaffyTree::new(),
        }
    }
    fn node(&mut self, node: &Node, mut inherited: TextStyle) -> Result<Item, String> {
        let mut style = Style {
            flex_direction: FlexDirection::Column,
            ..Style::default()
        };
        let (mut background, mut radius) = (None, 0.);
        if node.kind == Kind::DisabledButton {
            style.padding = Rect {
                left: length(16.),
                right: length(16.),
                top: length(8.),
                bottom: length(8.),
            };
        }
        for token in node.class_name.split_whitespace() {
            match Utility::parse(token).expect("validated utility") {
                Utility::Row => style.flex_direction = FlexDirection::Row,
                Utility::Column => style.flex_direction = FlexDirection::Column,
                Utility::Gap(v) => {
                    style.gap = Size {
                        width: length(v),
                        height: length(v),
                    }
                }
                Utility::Padding(v) => {
                    style.padding = Rect {
                        left: length(v),
                        right: length(v),
                        top: length(v),
                        bottom: length(v),
                    }
                }
                Utility::FontSize(v) => inherited.size = v,
                Utility::Bold => inherited.bold = true,
                Utility::Foreground(v) => inherited.color = v,
                Utility::Background(v) => background = Some(v),
                Utility::Rounded(v) => radius = v as f64,
            }
        }
        let children = node
            .children
            .iter()
            .map(|child| self.node(child, inherited))
            .collect::<Result<Vec<_>, _>>()?;
        let (text, id) = if node.kind == Kind::View {
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
            label: node.text.clone(),
            id,
            text,
            color: inherited.color,
            background,
            radius,
            disabled: node.kind == Kind::DisabledButton,
            children,
        })
    }
    fn scene(
        &mut self,
        node: &Node,
        width: f32,
        scale: f64,
    ) -> Result<(Scene, accesskit::TreeUpdate), String> {
        self.tree.clear();
        let mut root = self.node(node, TextStyle::default())?;
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
        self.paint(&mut root, &mut scene, 0., 0., scale)?;
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
        let mut node = accesskit::Node::new(match item.role {
            Kind::View => accesskit::Role::GenericContainer,
            Kind::Text => accesskit::Role::Label,
            Kind::DisabledButton => accesskit::Role::Button,
        });
        if !item.label.is_empty() {
            node.set_label(item.label.clone());
        }
        if item.disabled {
            node.set_disabled();
        }
        node.set_bounds(accesskit::Rect::new(
            x * scale,
            y * scale,
            (x + layout.size.width as f64) * scale,
            (y + layout.size.height as f64) * scale,
        ));
        let children = item
            .children
            .iter()
            .map(|child| self.accessible(child, nodes, next, x, y, scale))
            .collect::<Result<Vec<_>, _>>()?;
        node.set_children(children);
        nodes.push((id, node));
        Ok(id)
    }
    fn paint(
        &self,
        item: &mut Item,
        scene: &mut Scene,
        parent_x: f64,
        parent_y: f64,
        scale: f64,
    ) -> Result<(), String> {
        let layout = self.tree.layout(item.id).map_err(|e| e.to_string())?;
        let (x, y) = (
            parent_x + layout.location.x as f64,
            parent_y + layout.location.y as f64,
        );
        let transform = Affine::scale(scale);
        if item.disabled {
            scene.push_layer(
                Fill::NonZero,
                vello::peniko::BlendMode::default(),
                0.6,
                transform,
                &vello::kurbo::Rect::new(
                    x,
                    y,
                    x + layout.size.width as f64,
                    y + layout.size.height as f64,
                ),
            );
        }
        let alpha = 1.;
        if let Some([r, g, b]) = item.background {
            scene.fill(
                Fill::NonZero,
                transform,
                Color::from_rgba8(r, g, b, (alpha * 255.) as u8),
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
        if let Some(text) = &mut item.text {
            text.break_all_lines(Some(
                (layout.size.width - layout.padding.left - layout.padding.right).max(0.),
            ));
            let [r, g, b] = item.color;
            let color = Color::from_rgba8(r, g, b, (alpha * 255.) as u8);
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
                                        x + layout.padding.left as f64,
                                        y + layout.padding.top as f64,
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
            self.paint(child, scene, x, y, scale)?;
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
impl accesskit::ActionHandler for NoActions {
    fn do_action(&mut self, _: accesskit::ActionRequest) {}
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
struct App {
    close_after_frame: bool,
    presented: bool,
    deadline: Option<std::time::Instant>,
    node: Node,
    title: String,
    context: RenderContext,
    painter: Painter,
    active: Option<Active>,
    error: Option<String>,
}
impl App {
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
        )?;
        if let Some((_, root)) = snapshot.nodes.iter_mut().find(|(id, _)| id.0 == 0) {
            root.set_label(self.title.clone());
        }
        let snapshot = Arc::new(Mutex::new(snapshot));
        let accessibility = accesskit_winit::Adapter::with_direct_handlers(
            event_loop,
            &window,
            InitialTree(snapshot.clone()),
            NoActions,
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
                .scene(&self.node, (size.width as f64 / scale) as f32, scale)?;
        if let Some((_, root)) = snapshot.nodes.iter_mut().find(|(id, _)| id.0 == 0) {
            root.set_label(self.title.clone());
        }
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
                    base_color: Color::WHITE,
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
impl ApplicationHandler for App {
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
            WindowEvent::CloseRequested => {
                self.active = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => match self.draw() {
                Ok(true) if self.close_after_frame => {
                    self.presented = true;
                    self.active = None;
                    event_loop.exit();
                }
                Err(error) => self.fail(event_loop, error),
                _ => {}
            },
            WindowEvent::Resized(_)
            | WindowEvent::ScaleFactorChanged { .. }
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
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    let mut app = App {
        close_after_frame,
        presented: false,
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
        let (_, one) = painter.scene(&root, 400., 1.).unwrap();
        let (_, two) = painter.scene(&root, 400., 2.).unwrap();
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
