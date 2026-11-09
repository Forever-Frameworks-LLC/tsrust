//! GPUI Kit renderer for the retained Node model. One root view owns the tree.
//! Every event updates Node state, refreshes its live bindings and notifies the
//! view; `render` turns the current tree into GPUI elements. Buttons without
//! classes and text inputs use GPUI Kit components; classed views and buttons use
//! the shared utility classes, resolved against the active Tsrust theme.
use super::{Kind, Node, kit::Kit, style::{self, Length, State, Utility}, theme::{ResolvedColor, Theme}, window::Options};
use gpui_kit::component::Disableable;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonRounded, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
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
        gpui_kit::component::set_locale(&super::i18n::locale());
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
        // A custom titlebar draws the app's own TitleBar over a transparent native one.
        titlebar: Some(match options.titlebar {
            super::window::Titlebar::Custom => TitlebarOptions {title: Some(title.into()), ..gpui_kit::component::TitleBar::title_bar_options()},
            super::window::Titlebar::Native => TitlebarOptions {title: Some(title.into()), ..Default::default()},
        }),
        ..Default::default()
    };
    open_window(window, cx, move |_, cx| cx.new(|_| Root {node, inputs: HashMap::new(), states: HashMap::new(), dialog: None, sheet: None, accordions: HashMap::new(), failure}))
        .map(|_| ()).map_err(|error| format!("Failed to open the window: {error}"))
}

/// The Kit state behind a std:ui text field: one line, or a multiline textarea.
enum Field {Line(Entity<InputState>), Text(Entity<TextareaState>)}

struct Root {
    node: Node,
    inputs: HashMap<u64, (Field, Subscription)>,
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
        fn collect(node: &Node, out: &mut Vec<(u64, String, String, bool)>) {
            let Some(node) = node.visible_root() else {return};
            if let Some(field) = &node.editable {out.push((node.identity.get(), field.value.clone(), field.placeholder.clone(), field.multiline));}
            for child in &node.children {collect(child, out);}
        }
        let mut fields = Vec::new();
        collect(&self.node, &mut fields);
        self.inputs.retain(|id, _| fields.iter().any(|field| field.0 == *id));
        for (id, value, placeholder, multiline) in fields {
            match self.inputs.get(&id) {
                Some((Field::Line(state), _)) => {if state.read(cx).value().as_ref() != value {state.update(cx, |state, cx| state.set_value(value, window, cx));} continue}
                Some((Field::Text(state), _)) => {if state.read(cx).value().as_ref() != value {state.update(cx, |state, cx| state.set_value(value, window, cx));} continue}
                None => {}
            }
            // Edits update the Node and refresh the tree like any handler.
            let entry = if multiline {
                let state = cx.new(|cx| {let mut state = TextareaState::new(window, cx).rows(4).placeholder(placeholder); state.set_value(value, window, cx); state});
                let subscription = cx.subscribe_in(&state, window, move |this, state, event, _, cx| {
                    if matches!(event, InputEvent::Change) {let value = state.read(cx).value().to_string(); let result = this.node.input(id, &value); this.settle(result, cx);}
                });
                (Field::Text(state), subscription)
            } else {
                let state = cx.new(|cx| {let mut state = InputState::new(window, cx).placeholder(placeholder); state.set_value(value, window, cx); state});
                let subscription = cx.subscribe_in(&state, window, move |this, state, event, _, cx| {
                    if matches!(event, InputEvent::Change) {let value = state.read(cx).value().to_string(); let result = this.node.input(id, &value); this.settle(result, cx);}
                });
                (Field::Line(state), subscription)
            };
            self.inputs.insert(id, entry);
        }
    }

    /// A GPUI Kit Button for an unclassed core Button, with its Kit options.
    fn kit_button(&self, node: &Node, cx: &mut Context<Self>) -> Button {
        let id = node.identity.get();
                // Kit button options (variant, size, icon, states) arrive as node options.
                let option = |name: &str| node.option(name).unwrap_or_default();
                let flag = |name: &str| option(name) == "true";
                let label = node.content_text();
                let mut button = Button::new(ElementId::Integer(id)).disabled(node.is_disabled());
                if !label.is_empty() {button = button.label(label);}
                let variant = if option("variant").is_empty() {node.semantics.variant.as_str()} else {option("variant")};
                button = match variant {
                    "secondary" => button.secondary(), "ghost" => button.ghost(), "danger" | "destructive" => button.danger(),
                    "warning" => button.warning(), "success" => button.success(), "info" => button.info(), "link" => button.link(), "text" => button.text(),
                    "outline" => button.outline(), "primary" => button.primary(), _ => button,
                };
                if flag("outline") {button = button.outline();}
                if flag("compact") {button = button.compact();}
                if let Some(icon) = icon_name(option("icon")) {button = button.icon(icon);}
                button = match option("size") {"xsmall" => button.with_size(KitSize::XSmall), "small" => button.with_size(KitSize::Small), "large" => button.with_size(KitSize::Large), _ => button};
                button = button.loading(flag("loading")).selected(flag("selected")).dropdown_caret(flag("caret"));
                if !option("tooltip").is_empty() {button = button.tooltip(option("tooltip").to_owned());}
                button = button.tooltip_placement(match option("tooltipPlacement") {"bottom" => Placement::Bottom, "left" => Placement::Left, "right" => Placement::Right, _ => Placement::Top});
                if let Ok(delay) = option("tooltipDelay").parse::<u64>() {button = button.tooltip_show_delay(std::time::Duration::from_millis(delay));}
                button = match option("rounded") {"none" => button.rounded(ButtonRounded::None), "small" => button.rounded(ButtonRounded::Small), "large" => button.rounded(ButtonRounded::Large), _ => button};
                if let Some(icon) = icon_name(option("loadingIcon")) {button = button.loading_icon(icon);}
                if flag("spinner") {button = button.icon(gpui_kit::component::spinner::Spinner::new());}
                if let Ok(value) = option("progress").parse::<f32>() {button = button.icon(gpui_kit::component::progress::ProgressCircle::new(ElementId::Name(format!("{id}-progress").into())).value(value));}
                button = button.toggled(flag("toggled"));
                button = match option("role") {
                    "link" => button.role(Role::Link), "menuitem" => button.role(Role::MenuItem), "tab" => button.role(Role::Tab),
                    "checkbox" => button.role(Role::CheckBox), "switch" => button.role(Role::Switch), "none" => button.role(None::<Role>), _ => button,
                };
                if !node.semantics.label.is_empty() {button = button.accessibility_label(node.semantics.label.clone());}
                if !node.semantics.id.is_empty() {button = button.accessibility_id(node.semantics.id.clone());}
                if let Ok(index) = option("tabIndex").parse::<isize>() {button = button.tab_index(index);}
                if option("tabStop") == "false" {button = button.tab_stop(false);}
                if !option("color").is_empty() {
                    let custom = ButtonCustomVariant::new(cx);
                    let custom = match theme_color(option("color"), cx) {Some(color) => custom.color(color), None => custom};
                    let custom = match theme_color(option("foreground"), cx) {Some(color) => custom.foreground(color), None => custom};
                    let custom = match theme_color(option("hoverColor"), cx) {Some(color) => custom.hover(color), None => custom};
                    let custom = match theme_color(option("activeColor"), cx) {Some(color) => custom.active(color), None => custom};
                    button = button.custom(custom);
                }
                if node.kit.as_deref().is_some_and(|kit| kit.handles("hover")) {
                    let root = cx.entity().downgrade();
                    button = button.on_hover(move |hovered, _, cx| {let hovered = *hovered; let _ = root.update(cx, |root, cx| root.emit(id, "hover", hovered.to_string(), cx));});
                }
                button.on_click(cx.listener(move |this, _, _, cx| {let result = this.node.activate(id); this.settle(result, cx);}))
    }

    fn element(&self, node: &Node, theme: &Theme, mode: &str, cx: &mut Context<Self>) -> AnyElement {
        let (theme, mode) = node.theme.as_ref().map_or((theme, mode), |(theme, mode)| (theme, mode.as_str()));
        let id = node.identity.get();
        if node.editable.is_some() {
            let control = match self.inputs.get(&id) {
                Some((Field::Line(state), _)) => Input::new(state).disabled(node.is_disabled()).into_any_element(),
                Some((Field::Text(state), _)) => Textarea::new(state).disabled(node.is_disabled()).into_any_element(),
                None => return div().into_any_element(),
            };
            // Like a browser input, an unsized field keeps a usable intrinsic width.
            return styled(div().min_w(px(240.)), &node.class_name, theme, mode, node.is_disabled()).child(control).into_any_element();
        }
        if let Some(kit) = node.kit.as_deref().filter(|kit| !kit.name.is_empty()) {
            let element = self.kit_element(node, kit, cx);
            if node.class_name.is_empty() {return element;}
            // The class wrapper can shrink in flex layouts, so scroll areas inside it scroll.
            return styled(div().min_h_0().min_w_0(), &node.class_name, theme, mode, false).child(element).into_any_element();
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
            Kind::Button | Kind::DisabledButton if node.class_name.is_empty() => self.kit_button(node, cx).into_any_element(),
            Kind::Button | Kind::DisabledButton => {
                let element = styled(div(), &node.class_name, theme, mode, node.is_disabled()).children(children).id(ElementId::Integer(id));
                if node.is_disabled() {return element.into_any_element();}
                element.cursor_pointer().on_click(cx.listener(move |this, _, _, cx| {let result = this.node.activate(id); this.settle(result, cx);})).into_any_element()
            }
            // Vector art draws as an SVG image with colors resolved for this theme.
            Kind::Svg => {
                let (width, height) = node.svg_size().unwrap_or((24., 24.));
                let current = style::resolved(&node.class_name, theme, mode).iter().find_map(|(_, rule)| match rule {Utility::Foreground(value) => Some(*value), _ => None}).or_else(|| theme.color(mode, "foreground").ok());
                let base = styled(div().w(px(width as f32)).h(px(height as f32)).flex_shrink_0(), &node.class_name, theme, mode, false);
                match current.and_then(|current| node.svg_markup(current, theme, mode)) {
                    Some(markup) => base.child(img(std::sync::Arc::new(Image::from_bytes(ImageFormat::Svg, markup.into_bytes()))).size_full()).into_any_element(),
                    None => base.into_any_element(),
                }
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
        let theme = utility_theme(cx);
        let body = self.node.visible_root().map(|node| self.element(node, &theme, mode(cx), cx));
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
