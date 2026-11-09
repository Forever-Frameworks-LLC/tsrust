//! GPUI Kit renderer for the retained Node model. One root view owns the tree.
//! Every event updates Node state, refreshes its live bindings and notifies the
//! view; `render` turns the current tree into GPUI elements. Buttons without
//! classes and text inputs use GPUI Kit components; classed views and buttons use
//! the shared utility classes, resolved against the active Tsrust theme.
use super::{Kind, Node, kit::Kit, style::{self, Length, State, Utility}, theme::{ResolvedColor, Theme}, window::Options};
use gpui_kit::component::Disableable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;
use std::collections::HashMap;

pub fn run(node: Node, title: String) -> Result<(), String> {run_configured(node, title, "{}")}

pub fn run_configured(mut node: Node, title: String, options: &str) -> Result<(), String> {
    let options = Options::parse(options)?;
    node.refresh()?;
    node.ensure_identities();
    let failure = std::rc::Rc::new(std::cell::RefCell::new(None::<String>));
    let reported = failure.clone();
    application().with_assets(assets::Assets).run(move |cx| {
        init(cx);
        // The application ends with its last window, as a single-window app does.
        cx.on_window_closed(|cx, _| if cx.windows().is_empty() {cx.quit();}).detach();
        if let Err(error) = open(node, title, options, reported.clone(), cx) {
            *reported.borrow_mut() = Some(error);
            cx.quit();
        }
    });
    match failure.take() {Some(error) => Err(error), None => Ok(())}
}

type Failure = std::rc::Rc<std::cell::RefCell<Option<String>>>;

/// Open one window over a Node tree. Fonts load first; the default font applies app-wide.
fn open(node: Node, title: String, options: Options, failure: Failure, cx: &mut App) -> Result<(), String> {
    for path in &options.fonts {
        let bytes = std::fs::read(path).map_err(|error| format!("Cannot read font {path}: {error}"))?;
        cx.text_system().add_fonts(vec![std::borrow::Cow::Owned(bytes)]).map_err(|error| format!("Cannot load font {path}: {error}"))?;
    }
    if !options.font.is_empty() {gpui_kit::component::Theme::global_mut(cx).font_family = options.font.clone().into();}
    let bounds = Bounds::centered(None, size(px(options.width as f32), px(options.height as f32)), cx);
    let window = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(options.min_width as f32), px(options.min_height as f32))),
        titlebar: Some(TitlebarOptions {title: Some(title.into()), ..Default::default()}),
        ..Default::default()
    };
    open_window(window, cx, move |_, cx| cx.new(|_| Root {node, inputs: HashMap::new(), states: HashMap::new(), dialog: None, sheet: None, accordions: HashMap::new(), failure}))
        .map(|_| ()).map_err(|error| format!("Failed to open the window: {error}"))
}

struct Root {
    node: Node,
    inputs: HashMap<u64, (Entity<InputState>, Subscription)>,
    /// Kit component state entities (select, sliders, pickers), by node.
    states: HashMap<u64, KitState>,
    /// The node whose dialog or sheet is open, if any.
    dialog: Option<u64>,
    sheet: Option<u64>,
    /// Open sections of uncontrolled accordions.
    accordions: HashMap<u64, std::collections::BTreeSet<usize>>,
    failure: Failure,
}

impl Root {
    /// A failed handler or binding ends the application with its message, as an
    /// uncaught exception ends a backend program.
    fn settle(&mut self, result: Result<bool, String>, cx: &mut Context<Self>) {
        match result.and_then(|_| self.node.refresh()) {
            Ok(_) => cx.notify(),
            Err(error) => {*self.failure.borrow_mut() = Some(error); cx.quit();}
        }
    }

    /// Keep one Kit input state per visible editable node and push controlled
    /// values into it when the Node value no longer matches the edited text.
    fn sync_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Multiline fields use the same single-line Kit input until Textarea
        // (TextareaState) is wrapped with the component catalog.
        fn collect(node: &Node, out: &mut Vec<(u64, String, String)>) {
            let Some(node) = node.visible_root() else {return};
            if let Some(field) = &node.editable {out.push((node.identity.get(), field.value.clone(), field.placeholder.clone()));}
            for child in &node.children {collect(child, out);}
        }
        let mut fields = Vec::new();
        collect(&self.node, &mut fields);
        self.inputs.retain(|id, _| fields.iter().any(|field| field.0 == *id));
        for (id, value, placeholder) in fields {
            if let Some((state, _)) = self.inputs.get(&id) {
                if state.read(cx).value().as_ref() != value {state.update(cx, |state, cx| state.set_value(value, window, cx));}
                continue;
            }
            let state = cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder(placeholder);
                state.set_value(value, window, cx);
                state
            });
            let subscription = cx.subscribe_in(&state, window, move |this, state, event, _, cx| {
                if matches!(event, InputEvent::Change) {
                    let value = state.read(cx).value().to_string();
                    let result = this.node.input(id, &value);
                    this.settle(result, cx);
                }
            });
            self.inputs.insert(id, (state, subscription));
        }
    }

    fn element(&self, node: &Node, theme: &Theme, mode: &str, cx: &mut Context<Self>) -> AnyElement {
        let (theme, mode) = node.theme.as_ref().map_or((theme, mode), |(theme, mode)| (theme, mode.as_str()));
        let id = node.identity.get();
        if node.editable.is_some() {
            let Some((state, _)) = self.inputs.get(&id) else {return div().into_any_element()};
            // Like a browser input, an unsized field keeps a usable intrinsic width.
            return styled(div().min_w(px(240.)), &node.class_name, theme, mode, node.is_disabled()).child(Input::new(state).disabled(node.is_disabled())).into_any_element();
        }
        if let Some(kit) = &node.kit {
            let element = self.kit_element(node, kit, cx);
            if node.class_name.is_empty() {return element;}
            return styled(div(), &node.class_name, theme, mode, false).child(element).into_any_element();
        }
        let children = node.visible_children().map(|child| self.element(child, theme, mode, cx)).collect::<Vec<_>>();
        if let Some(copy) = &node.copy {
            return Clipboard::new(ElementId::Integer(id)).value(SharedString::from(copy.value.clone())).into_any_element();
        }
        match node.kind {
            Kind::Button | Kind::DisabledButton if node.class_name.is_empty() && !node.semantics.href.is_empty() => {
                gpui_kit::component::link::Link::new(ElementId::Integer(id)).disabled(node.is_disabled()).child(SharedString::from(node.content_text()))
                    .on_click(cx.listener(move |this, _, _, cx| {let result = this.node.activate(id); this.settle(result, cx);})).into_any_element()
            }
            Kind::Button | Kind::DisabledButton if node.class_name.is_empty() => {
                let button = Button::new(ElementId::Integer(id)).label(node.content_text()).disabled(node.is_disabled());
                let button = match node.semantics.variant.as_str() {"ghost" => button.ghost(), "danger" | "destructive" => button.danger(), "outline" => button.outline(), _ => button.primary()};
                button.on_click(cx.listener(move |this, _, _, cx| {let result = this.node.activate(id); this.settle(result, cx);})).into_any_element()
            }
            Kind::Button | Kind::DisabledButton => {
                let element = styled(div(), &node.class_name, theme, mode, node.is_disabled()).children(children).id(ElementId::Integer(id));
                if node.is_disabled() {return element.into_any_element();}
                element.cursor_pointer().on_click(cx.listener(move |this, _, _, cx| {let result = this.node.activate(id); this.settle(result, cx);})).into_any_element()
            }
            // Text is one inline run; its literal and bound parts render as one string.
            Kind::Text => styled(div(), &node.class_name, theme, mode, false).child(SharedString::from(node.content_text())).into_any_element(),
            _ if children.is_empty() => styled(div(), &node.class_name, theme, mode, false).child(SharedString::from(node.text.clone())).into_any_element(),
            _ => {
                // GPUI keeps scroll offsets per element id; overflow-auto views need one.
                // Their children keep their natural size instead of shrinking to fit.
                let scrolls = style::resolved(&node.class_name, theme, mode).iter().any(|(_, rule)| matches!(rule, Utility::Overflow(2)));
                if !scrolls {return styled(div(), &node.class_name, theme, mode, false).children(children).into_any_element();}
                let children = children.into_iter().map(|child| div().flex_shrink_0().child(child));
                styled(div(), &node.class_name, theme, mode, false).children(children).id(ElementId::Integer(id)).overflow_scroll().into_any_element()
            }
        }
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.node.ensure_identities();
        self.sync_inputs(window, cx);
        self.sync_kit(window, cx);
        self.run_commands(window, cx);
        let theme = Theme::default();
        let body = self.node.visible_root().map(|node| self.element(node, &theme, "light", cx));
        div().size_full().children(body)
    }
}

fn color(value: ResolvedColor) -> Hsla {
    let [r, g, b, a] = value.components();
    Rgba {r, g, b, a}.into()
}

fn length(value: Length) -> gpui_kit::Length {
    match value {
        Length::Pixels(value) => px(value).into(),
        Length::Percent(value) => relative(value).into(),
        Length::Viewport(_) => relative(1.).into(),
        Length::Auto | Length::None | Length::Fit => gpui_kit::Length::Auto,
    }
}

/// Apply the resolved utility classes; hover rules become a GPUI hover style.
fn styled(mut element: Div, classes: &str, theme: &Theme, mode: &str, disabled: bool) -> Div {
    let rules = style::resolved(classes, theme, mode);
    let flex = rules.iter().any(|(_, rule)| matches!(rule, Utility::Row | Utility::Column | Utility::Display(false) | Utility::InlineFlex));
    if flex {element = element.flex();}
    for (state, rule) in &rules {
        if *state == State::Base || *state == State::Disabled && disabled {apply(element.style(), *rule);}
    }
    if rules.iter().any(|(state, _)| *state == State::Hover) && !disabled {
        let hover = rules.iter().filter(|(state, _)| *state == State::Hover).map(|(_, rule)| *rule).collect::<Vec<_>>();
        element = element.hover(move |mut style| {for rule in &hover {apply(&mut style, *rule);} style});
    }
    element
}

fn apply(style: &mut StyleRefinement, rule: Utility) {
    let edge = |edges: &mut EdgesRefinement<DefiniteLength>, side: u8, value: f32| {
        let value = Some(px(value).into());
        match side {0 => edges.top = value, 1 => edges.right = value, 2 => edges.bottom = value, _ => edges.left = value}
    };
    match rule {
        Utility::Row => style.flex_direction = Some(FlexDirection::Row),
        Utility::Column => style.flex_direction = Some(FlexDirection::Column),
        Utility::Display(grid) => style.display = Some(if grid {Display::Grid} else {Display::Flex}),
        Utility::InlineFlex => style.display = Some(Display::Flex),
        Utility::Hidden => style.display = Some(Display::None),
        Utility::GapAxis(0, value) => style.gap.height = Some(px(value).into()),
        Utility::GapAxis(_, value) => style.gap.width = Some(px(value).into()),
        Utility::Padding(side, value) => edge(&mut style.padding, side, value),
        Utility::Margin(side, value) => {
            let value = Some(px(value).into());
            match side {0 => style.margin.top = value, 1 => style.margin.right = value, 2 => style.margin.bottom = value, _ => style.margin.left = value}
        }
        Utility::Dimension(property, value) => {
            let value = Some(length(value));
            match property {
                0 => style.size.width = value, 1 => style.size.height = value,
                2 => style.min_size.width = value, 3 => style.min_size.height = value,
                4 => style.max_size.width = value, _ => style.max_size.height = value,
            }
        }
        Utility::Grow(value) => style.flex_grow = Some(value),
        Utility::Shrink(value) => style.flex_shrink = Some(value),
        Utility::BasisZero => style.flex_basis = Some(px(0.).into()),
        Utility::Wrap(wrap) => style.flex_wrap = Some(if wrap {FlexWrap::Wrap} else {FlexWrap::NoWrap}),
        Utility::Items(value) => style.align_items = Some([AlignItems::FlexStart, AlignItems::Center, AlignItems::Stretch][value.min(2) as usize]),
        Utility::Justify(value) => style.justify_content = Some([JustifyContent::FlexStart, JustifyContent::Center, JustifyContent::SpaceBetween][value.min(2) as usize]),
        Utility::SelfAlign(0, value) => style.align_self = Some([AlignSelf::FlexStart, AlignSelf::Center, AlignSelf::FlexEnd, AlignSelf::Stretch, AlignSelf::Stretch][value.min(4) as usize]),
        Utility::FontSize(value) => style.text.font_size = Some(px(value).into()),
        Utility::Weight(value) => style.text.font_weight = Some(FontWeight(value)),
        Utility::Leading(value) => style.text.line_height = Some(relative(value).into()),
        Utility::DefaultLeading(value) => style.text.line_height = Some(px(value).into()),
        Utility::Align(value) => style.text.text_align = Some([TextAlign::Left, TextAlign::Center, TextAlign::Right][value.min(2) as usize]),
        Utility::Foreground(value) => style.text.color = Some(color(value)),
        Utility::Background(value) => style.background = Some(color(value).into()),
        Utility::BorderColor(value) => style.border_color = Some(color(value)),
        Utility::EdgeBorder(side, value) => {
            let value = Some(px(value).into());
            match side {0 => style.border_widths.top = value, 1 => style.border_widths.right = value, 2 => style.border_widths.bottom = value, _ => style.border_widths.left = value}
        }
        Utility::Corner(corner, value) => {
            let value = Some(px(value).into());
            match corner {0 => style.corner_radii.top_left = value, 1 => style.corner_radii.top_right = value, 2 => style.corner_radii.bottom_right = value, _ => style.corner_radii.bottom_left = value}
        }
        Utility::Overflow(value) => {
            let value = [Overflow::Visible, Overflow::Hidden, Overflow::Scroll][value.min(2) as usize];
            style.overflow.x = Some(value);
            style.overflow.y = Some(value);
        }
        Utility::Opacity(value) => style.opacity = Some(value),
        Utility::Relative => style.position = Some(Position::Relative),
        Utility::Absolute => style.position = Some(Position::Absolute),
        Utility::Inset(side, value) => {
            let value = Some(length(value));
            match side {0 => style.inset.top = value, 1 => style.inset.right = value, 2 => style.inset.bottom = value, _ => style.inset.left = value}
        }
        Utility::AspectRatio(value) => style.aspect_ratio = value,
        // font-sans keeps the window font; serif and mono use each platform's standard families.
        Utility::FontFamily(1) => style.text.font_family = Some(if cfg!(target_os = "linux") {"DejaVu Serif"} else {"Georgia"}.into()),
        Utility::FontFamily(2) => style.text.font_family = Some(if cfg!(target_os = "macos") {"Menlo"} else if cfg!(windows) {"Consolas"} else {"DejaVu Sans Mono"}.into()),
        // Text decoration, rings, cursors, tables and grids have no direct GPUI
        // style field; the classes stay valid and render without the effect.
        _ => {}
    }
}
