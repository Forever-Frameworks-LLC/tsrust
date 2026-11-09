// GPUI Kit component arms for `std:ui` Kit nodes (generated wrappers in
// kit.tsr, spec in kit.json). Props arrive as text; item components are read
// from their container's children; stateful components keep one Kit state
// entity per node; overlay content renders through a weak handle to the root.
use gpui_kit::component::accordion::Accordion;
use gpui_kit::component::alert::Alert;
use gpui_kit::component::avatar::Avatar;
use gpui_kit::component::badge::Badge;
use gpui_kit::component::breadcrumb::{Breadcrumb, BreadcrumbItem};
use gpui_kit::component::button::{DropdownButton, Toggle};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::collapsible::Collapsible;
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_kit::component::description_list::DescriptionList;
use gpui_kit::component::empty::{Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyTitle};
use gpui_kit::component::group_box::GroupBox;
use gpui_kit::component::hover_card::HoverCard;
use gpui_kit::component::input::NumberInput;
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::Label;
use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_kit::component::input::{OtpEvent, OtpInput, OtpState};
use gpui_kit::component::pagination::Pagination;
use gpui_kit::component::popover::Popover;
use gpui_kit::component::progress::{Progress, ProgressCircle};
use gpui_kit::component::radio::{Radio, RadioGroup};
use gpui_kit::component::rating::Rating;
use gpui_kit::component::resizable::{h_resizable, resizable_panel, v_resizable};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::separator::Separator;
use gpui_kit::component::shimmer::ShimmerText;
use gpui_kit::component::sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem};
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::slider::{Slider, SliderEvent, SliderState, SliderValue};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::stepper::{Stepper, StepperItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_kit::component::tag::Tag;
use gpui_kit::component::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::bubble::{Bubble, BubbleVariant};
use gpui_kit::component::button::{ButtonGroup, ToggleGroup};
use gpui_kit::component::calendar::{Calendar, CalendarEvent, CalendarState, Date};
use gpui_kit::component::carousel::{Carousel, CarouselContent, CarouselEvent, CarouselItem, CarouselNext, CarouselPrevious, CarouselState};
use gpui_kit::component::chart::{AreaChart, BarChart, LineChart, PieChart};
use gpui_kit::component::combobox::{Combobox, ComboboxEvent, ComboboxState};
use gpui_kit::component::command::{Command as CommandPalette, CommandItem, CommandState};
use gpui_kit::component::form::{field, h_form, v_form};
use gpui_kit::component::input::{Editor, EditorState};
use gpui_kit::component::list::ListItem;
use gpui_kit::component::message::{Message, MessageAlignment, MessageContent, MessageHeader};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::searchable_list::SearchableVec;
use gpui_kit::component::status_bar::StatusBar;
use gpui_kit::component::text::TextView;
use gpui_kit::component::time_field::{HourCycle, TimeField, TimeFieldEvent, TimeFieldState, TimePrecision};
use gpui_kit::component::toolbar::Toolbar;
use gpui_kit::component::tree::{TreeItem, TreeState, tree};
use gpui_kit::component::{IndexPath, ThemeMode, TitleBar, VirtualListScrollHandle, v_virtual_list};
use gpui_kit::component::Theme as KitTheme;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::{Icon, Placement, Selectable, Sizable, Size as KitSize, WindowExt};
use super::commands::{self, Command};

enum KitState {
    Select(Entity<SelectState<Vec<SharedString>>>),
    Number(Entity<InputState>),
    Otp(Entity<OtpState>),
    Date(Entity<DatePickerState>),
    Color(Entity<ColorPickerState>),
    Slider(Entity<SliderState>),
    Calendar(Entity<CalendarState>),
    Time(Entity<TimeFieldState>),
    Combobox(Entity<ComboboxState<SearchableVec<SharedString>>>),
    Command(Entity<CommandState>),
    Carousel(Entity<CarouselState>),
    Editor(Entity<EditorState>),
    Tree(Entity<TreeState>, String),
    List(VirtualListScrollHandle),
}

/// A popover trigger wrapping arbitrary content; Kit triggers must be selectable.
struct Trigger {element: AnyElement, selected: bool}
impl Selectable for Trigger {
    fn selected(mut self, selected: bool) -> Self {self.selected = selected; self}
    fn is_selected(&self) -> bool {self.selected}
}
impl RenderOnce for Trigger {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {self.element}
}
impl IntoElement for Trigger {
    type Element = ViewElement<Self>;
    fn into_element(self) -> Self::Element {ViewElement::new(self)}
}

fn find(node: &Node, id: u64) -> Option<&Node> {
    if node.identity.get() == id {return Some(node);}
    node.children.iter().find_map(|child| find(child, id))
}

/// Kit components that read their item children; their own children are not drawn directly.
fn items<'a>(node: &'a Node, name: &'a str) -> impl Iterator<Item = (&'a Node, &'a Kit)> + 'a {
    node.visible_children().filter_map(move |child| child.kit.as_deref().filter(|kit| kit.name == name).map(|kit| (child, kit)))
}

fn hex(color: Hsla) -> String {
    let rgba = color.to_rgb();
    let channel = |value: f32| (value.clamp(0., 1.) * 255.).round() as u8;
    format!("#{:02x}{:02x}{:02x}", channel(rgba.r), channel(rgba.g), channel(rgba.b))
}

fn parse_hex(text: &str) -> Option<Hsla> {
    let digits = text.strip_prefix('#')?;
    (digits.len() == 6).then(|| u32::from_str_radix(digits, 16).ok()).flatten().map(|value| rgb(value).into())
}

/// Keystroke parsing needs 'static text; each distinct shortcut is kept once.
fn keystroke(text: &str) -> Option<Keystroke> {
    thread_local! {static KEYS: std::cell::RefCell<HashMap<String, &'static str>> = std::cell::RefCell::new(HashMap::new());}
    let text = KEYS.with(|keys| *keys.borrow_mut().entry(text.to_owned()).or_insert_with(|| Box::leak(text.to_owned().into_boxed_str())));
    Keystroke::parse(text).ok()
}

impl Root {
    /// Deliver a component event to its node, then refresh and redraw.
    fn emit(&mut self, id: u64, event: &str, value: String, cx: &mut Context<Self>) {
        let result = self.node.kit_event(id, event, &value);
        self.settle(result, cx);
    }

    fn content(&self, node: &Node, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = Theme::default();
        node.visible_children().map(|child| self.element(child, &theme, "light", cx)).collect()
    }

    /// Children of the node with this identity, for overlay content built later.
    fn content_of(&self, id: u64, cx: &mut Context<Self>) -> Vec<AnyElement> {
        match find(&self.node, id) {Some(node) => self.content(node, cx), None => Vec::new()}
    }

    /// Create Kit state entities for stateful components, push changed props
    /// into them, and open or close dialogs and sheets to match their props.
    fn sync_kit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        fn collect(node: &Node, out: &mut Vec<(u64, String, std::collections::BTreeMap<String, String>)>) {
            let Some(node) = node.visible_root() else {return};
            if let Some(kit) = &node.kit && matches!(kit.name.as_str(), "Select" | "NumberInput" | "OtpInput" | "DatePicker" | "ColorPicker" | "Slider" | "Dialog" | "Sheet" | "Calendar" | "TimeField" | "Combobox" | "Command" | "Carousel" | "CodeEditor" | "Tree" | "List") {
                out.push((node.identity.get(), kit.name.clone(), kit.props.clone()));
            }
            for child in &node.children {collect(child, out);}
        }
        let mut found = Vec::new();
        collect(&self.node, &mut found);
        self.states.retain(|id, _| found.iter().any(|(other, _, _)| other == id));
        let mut dialog = None;
        let mut sheet = None;
        for (id, name, props) in found {
            let text = |key: &str| props.get(key).cloned().unwrap_or_default();
            let number = |key: &str| props.get(key).and_then(|value| value.parse::<f32>().ok());
            match name.as_str() {
                "Dialog" => {if props.get("open").map(String::as_str) == Some("true") {dialog = Some((id, text("title"), text("okText"), text("cancelText")));} continue}
                "Sheet" => {if props.get("open").map(String::as_str) == Some("true") {sheet = Some((id, text("title"), text("side")));} continue}
                _ => {}
            }
            if let Some(state) = self.states.get(&id) {
                match state {
                    KitState::Select(state) => {
                        let value = text("value");
                        if state.read(cx).selected_value().map(|current| current.as_ref() != value).unwrap_or(!value.is_empty()) {
                            state.update(cx, |state, cx| state.set_selected_value(&SharedString::from(value), window, cx));
                        }
                    }
                    KitState::Number(state) => {
                        if let Some(value) = number("value") && state.read(cx).value().parse::<f32>().ok() != Some(value) {
                            state.update(cx, |state, cx| state.set_value(value.to_string(), window, cx));
                        }
                    }
                    KitState::Slider(state) => {
                        if let Some(value) = number("value") && state.read(cx).value() != SliderValue::Single(value) {
                            state.update(cx, |state, cx| state.set_value(value, window, cx));
                        }
                    }
                    KitState::Color(state) => {
                        if let Some(color) = parse_hex(&text("value")) && state.read(cx).value().map(hex) != Some(text("value").to_lowercase()) {
                            state.update(cx, |state, cx| state.set_value(color, window, cx));
                        }
                    }
                    KitState::Editor(state) => {
                        let value = text("value");
                        if state.read(cx).value().as_ref() != value {state.update(cx, |state, cx| state.set_value(value, window, cx));}
                    }
                    KitState::Tree(state, shape) => {
                        // Rebuild the items when the TreeItem children change.
                        let current = tree_shape(&self.node, id);
                        if *shape != current {
                            let items = find(&self.node, id).map(tree_items).unwrap_or_default();
                            state.update(cx, |state, cx| state.set_items(items, cx));
                            if let Some(KitState::Tree(_, shape)) = self.states.get_mut(&id) {*shape = current;}
                        }
                    }
                    KitState::Date(_) | KitState::Otp(_) | KitState::Calendar(_) | KitState::Time(_) | KitState::Combobox(_) | KitState::Command(_) | KitState::Carousel(_) | KitState::List(_) => {}
                }
                continue;
            }
            let state = match name.as_str() {
                "Select" => {
                    let items = text("items").split('\n').filter(|item| !item.is_empty()).map(|item| SharedString::from(item.to_owned())).collect::<Vec<_>>();
                    let selected = items.iter().position(|item| item.as_ref() == text("value"));
                    let state = cx.new(|cx| SelectState::new(items, selected.map(gpui_kit::component::IndexPath::new), window, cx));
                    cx.subscribe_in(&state, window, move |this, _, event: &SelectEvent<Vec<SharedString>>, _, cx| {
                        let SelectEvent::Confirm(value) = event;
                        this.emit(id, "change", value.as_ref().map(ToString::to_string).unwrap_or_default(), cx);
                    }).detach();
                    KitState::Select(state)
                }
                "NumberInput" => {
                    let state = cx.new(|cx| {
                        let mut state = InputState::new(window, cx).placeholder(text("placeholder"));
                        if let Some(value) = number("value") {state.set_value(value.to_string(), window, cx);}
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, state, event: &InputEvent, _, cx| {
                        if matches!(event, InputEvent::Change) && let Ok(value) = state.read(cx).value().parse::<f64>() {this.emit(id, "change", value.to_string(), cx);}
                    }).detach();
                    KitState::Number(state)
                }
                "OtpInput" => {
                    let length = number("length").map_or(6, |value| value as usize);
                    let masked = text("masked") == "true";
                    let state = cx.new(|cx| OtpState::new(length, window, cx).masked(masked));
                    cx.subscribe_in(&state, window, move |this, state, event: &OtpEvent, _, cx| {
                        if matches!(event, OtpEvent::Change | OtpEvent::Complete) {let value = state.read(cx).value().to_string(); this.emit(id, "change", value, cx);}
                    }).detach();
                    KitState::Otp(state)
                }
                "DatePicker" => {
                    let initial = chrono::NaiveDate::parse_from_str(&text("value"), "%Y-%m-%d").ok();
                    let state = cx.new(|cx| {
                        let mut state = DatePickerState::new(window, cx);
                        if let Some(date) = initial {state.set_date(date, window, cx);}
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, _, event: &DatePickerEvent, _, cx| {
                        let DatePickerEvent::Change(value) = event;
                        this.emit(id, "change", value.format("%Y-%m-%d").map(|date| date.to_string()).unwrap_or_default(), cx);
                    }).detach();
                    KitState::Date(state)
                }
                "ColorPicker" => {
                    let initial = parse_hex(&text("value"));
                    let state = cx.new(|cx| {
                        let state = ColorPickerState::new(window, cx);
                        match initial {Some(color) => state.default_value(color), None => state}
                    });
                    cx.subscribe_in(&state, window, move |this, _, event: &ColorPickerEvent, _, cx| {
                        let ColorPickerEvent::Change(value) = event;
                        this.emit(id, "change", value.map(hex).unwrap_or_default(), cx);
                    }).detach();
                    KitState::Color(state)
                }
                "Calendar" => {
                    let initial = chrono::NaiveDate::parse_from_str(&text("value"), "%Y-%m-%d").ok();
                    let state = cx.new(|cx| {
                        let mut state = CalendarState::new(window, cx);
                        if let Some(date) = initial {state.set_date(date, window, cx);}
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, _, event: &CalendarEvent, _, cx| {
                        let CalendarEvent::Selected(date) = event;
                        if let Date::Single(Some(date)) = date {this.emit(id, "change", date.format("%Y-%m-%d").to_string(), cx);}
                    }).detach();
                    KitState::Calendar(state)
                }
                "TimeField" => {
                    let initial = chrono::NaiveTime::parse_from_str(&text("value"), "%H:%M:%S").or_else(|_| chrono::NaiveTime::parse_from_str(&text("value"), "%H:%M")).ok();
                    let (seconds, twelve) = (text("seconds") == "true", text("twelveHour") == "true");
                    let state = cx.new(|cx| {
                        let mut state = TimeFieldState::new(window, cx);
                        if seconds {state = state.precision(TimePrecision::Second);}
                        if twelve {state = state.hour_cycle(HourCycle::H12);}
                        if let Some(time) = initial {state.set_time(time, window, cx);}
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, _, event: &TimeFieldEvent, _, cx| {
                        let TimeFieldEvent::Change(time) = event;
                        this.emit(id, "change", time.format("%H:%M:%S").to_string(), cx);
                    }).detach();
                    KitState::Time(state)
                }
                "Combobox" => {
                    let items = text("items").split('\n').filter(|item| !item.is_empty()).map(|item| SharedString::from(item.to_owned())).collect::<Vec<_>>();
                    let selected = items.iter().position(|item| item.as_ref() == text("value")).map(IndexPath::new).into_iter().collect::<Vec<_>>();
                    let state = cx.new(|cx| ComboboxState::new(SearchableVec::new(items), selected, window, cx).searchable(true));
                    cx.subscribe_in(&state, window, move |this, _, event: &ComboboxEvent<SearchableVec<SharedString>>, _, cx| {
                        if let ComboboxEvent::Confirm(values) | ComboboxEvent::Change(values) = event {this.emit(id, "change", values.first().map(ToString::to_string).unwrap_or_default(), cx);}
                    }).detach();
                    KitState::Combobox(state)
                }
                "Command" => KitState::Command(cx.new(|cx| CommandState::new(window, cx))),
                "Carousel" => {
                    let count = find(&self.node, id).map(|node| items(node, "CarouselSlide").count()).unwrap_or(0);
                    let looping = text("loop") == "true";
                    let state = cx.new(|_| CarouselState::new(count).with_looping(looping));
                    cx.subscribe_in(&state, window, move |this, _, event: &CarouselEvent, _, cx| {
                        let CarouselEvent::Change(index) = event;
                        this.emit(id, "change", index.to_string(), cx);
                    }).detach();
                    KitState::Carousel(state)
                }
                "CodeEditor" => {
                    let (language, value, read_only) = (text("language"), text("value"), text("readOnly") == "true");
                    let state = cx.new(|cx| {
                        let mut state = EditorState::new(window, cx).language(language);
                        state.set_value(value, window, cx);
                        state
                    });
                    let _ = read_only;
                    cx.subscribe_in(&state, window, move |this, state, event: &InputEvent, _, cx| {
                        if matches!(event, InputEvent::Change) {let value = state.read(cx).value().to_string(); this.emit(id, "change", value, cx);}
                    }).detach();
                    KitState::Editor(state)
                }
                "Tree" => {
                    let items = find(&self.node, id).map(tree_items).unwrap_or_default();
                    let state = cx.new(|cx| TreeState::new(cx).items(items));
                    KitState::Tree(state, tree_shape(&self.node, id))
                }
                "List" => KitState::List(VirtualListScrollHandle::new()),
                _ => {
                    let (min, max, step) = (number("min").unwrap_or(0.), number("max").unwrap_or(100.), number("step").unwrap_or(1.));
                    let initial = number("value").unwrap_or(min);
                    let state = cx.new(|cx| {
                        let mut state = SliderState::new().min(min).max(max).step(step);
                        state.set_value(initial, window, cx);
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, _, event: &SliderEvent, _, cx| {
                        if let SliderEvent::Change(SliderValue::Single(value)) = event {this.emit(id, "change", value.to_string(), cx);}
                    }).detach();
                    KitState::Slider(state)
                }
            };
            self.states.insert(id, state);
        }
        let root = cx.entity().downgrade();
        match (dialog, self.dialog) {
            (Some((id, title, ok, cancel)), current) if current != Some(id) => {
                self.dialog = Some(id);
                let root = root.clone();
                window.open_dialog(cx, move |dialog, _, cx| {
                    let body = root.update(cx, |root, cx| root.content_of(id, cx)).unwrap_or_default();
                    let mut dialog = dialog.title(title.clone()).child(div().flex().flex_col().gap_2().children(body));
                    let close = root.clone();
                    dialog = dialog.on_close(move |_, _, cx| {let _ = close.update(cx, |root, cx| {root.dialog = None; root.emit(id, "close", String::new(), cx)});});
                    if !ok.is_empty() || !cancel.is_empty() {
                        let props = gpui_kit::component::dialog::DialogButtonProps::default().ok_text(if ok.is_empty() {"OK".to_owned()} else {ok.clone()}).cancel_text(if cancel.is_empty() {"Cancel".to_owned()} else {cancel.clone()}).show_cancel(!cancel.is_empty());
                        let confirm = root.clone();
                        dialog = dialog.button_props(props).on_ok(move |_, _, cx| {let _ = confirm.update(cx, |root, cx| root.emit(id, "ok", String::new(), cx)); true});
                        // A plain Kit dialog draws no buttons from its props; compose the footer.
                        let mut footer = gpui_kit::component::dialog::DialogFooter::new();
                        if !cancel.is_empty() {footer = footer.child(gpui_kit::component::dialog::DialogClose::new().child(gpui_kit::component::button::Button::new("tsrust-dialog-cancel").outline().label(cancel.clone())));}
                        if !ok.is_empty() {footer = footer.child(gpui_kit::component::dialog::DialogAction::new().child(gpui_kit::component::button::Button::new("tsrust-dialog-ok").primary().label(ok.clone())));}
                        dialog = dialog.footer(footer);
                    }
                    dialog
                });
            }
            (None, Some(_)) => {self.dialog = None; window.close_dialog(cx);}
            _ => {}
        }
        match (sheet, self.sheet) {
            (Some((id, title, side)), current) if current != Some(id) => {
                self.sheet = Some(id);
                let placement = match side.as_str() {"left" => Placement::Left, "top" => Placement::Top, "bottom" => Placement::Bottom, _ => Placement::Right};
                window.open_sheet_at(placement, cx, move |sheet, _, cx| {
                    let body = root.update(cx, |root, cx| root.content_of(id, cx)).unwrap_or_default();
                    let close = root.clone();
                    sheet.title(title.clone()).child(div().flex().flex_col().gap_2().children(body))
                        .on_close(move |_, _, cx| {let _ = close.update(cx, |root, cx| {root.sheet = None; root.emit(id, "close", String::new(), cx)});})
                });
            }
            (None, Some(_)) => {self.sheet = None; window.close_sheet(cx);}
            _ => {}
        }
    }

    /// Run queued application commands; replies refresh the tree like handlers.
    fn run_commands(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        commands::set_dark(matches!(window.appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark));
        // GPUI Kit's own strings (calendar months, placeholders) follow the app locale.
        if super::i18n::take_changed() {gpui_kit::component::set_locale(&super::i18n::locale());}
        for command in commands::take() {
            match command {
                Command::Plain {name, payload} => {
                    let value: serde_json::Value = serde_json::from_str(&payload).unwrap_or_default();
                    let text = |key: &str| value.get(key).and_then(|value| value.as_str()).unwrap_or_default().to_owned();
                    match name.as_str() {
                        "notify" => {
                            let message = text("message");
                            let note = match text("variant").as_str() {"success" => Notification::success(message), "warning" => Notification::warning(message), "error" => Notification::error(message), _ => Notification::info(message)};
                            let note = match text("delivery").as_str() {"system" => note.system(), "both" => note.in_app_and_system(), _ => note};
                            let title = text("title");
                            window.push_notification(if title.is_empty() {note} else {note.title(title)}, cx);
                        }
                        "set-title" => window.set_window_title(&text("title")),
                        "theme" => match text("mode").as_str() {
                            "dark" => KitTheme::change(ThemeMode::Dark, Some(window), cx),
                            "light" => KitTheme::change(ThemeMode::Light, Some(window), cx),
                            _ => KitTheme::sync_system_appearance(Some(window), cx),
                        },
                        "minimize" => window.minimize_window(),
                        "zoom" => window.zoom_window(),
                        "fullscreen" => window.toggle_fullscreen(),
                        "close" => window.remove_window(),
                        "quit" => cx.quit(),
                        "open-url" => cx.open_url(&text("url")),
                        _ => cx.write_to_clipboard(ClipboardItem::new_string(text("text"))),
                    }
                }
                Command::Reply {name, payload, reply} => {
                    let value: serde_json::Value = serde_json::from_str(&payload).unwrap_or_default();
                    let flag = |key: &str| value.get(key).and_then(|value| value.as_bool()).unwrap_or(false);
                    if name == "read-clipboard" {
                        let text = cx.read_from_clipboard().and_then(|item| item.text()).unwrap_or_default();
                        let result = reply(&text);
                        self.settle(result.map(|_| true), cx);
                        continue;
                    }
                    let receiver = if name == "pick-files" {
                        let directories = flag("directories");
                        let prompt = cx.prompt_for_paths(PathPromptOptions {files: !directories, directories, multiple: flag("multiple"), prompt: None});
                        Box::pin(async move {prompt.await.ok().and_then(Result::ok).flatten().unwrap_or_default().iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join("\n")}) as std::pin::Pin<Box<dyn std::future::Future<Output = String>>>
                    } else {
                        let directory = std::env::current_dir().unwrap_or_default();
                        let suggested = value.get("name").and_then(|name| name.as_str()).unwrap_or_default().to_owned();
                        let prompt = cx.prompt_for_new_path(&directory, Some(&suggested));
                        Box::pin(async move {prompt.await.ok().and_then(Result::ok).flatten().map(|path| path.display().to_string()).unwrap_or_default()})
                    };
                    cx.spawn(async move |this, cx| {
                        let chosen = receiver.await;
                        let _ = this.update(cx, |root, cx| {let result = reply(&chosen); root.settle(result.map(|_| true), cx);});
                    }).detach();
                }
                Command::Window {mut node, title, options} => {
                    let result = node.refresh().and_then(|_| {node.ensure_identities(); Options::parse(&options)}).and_then(|options| open(node, title, options, self.failure.clone(), cx));
                    if let Err(error) = result {*self.failure.borrow_mut() = Some(error); cx.quit();}
                }
            }
        }
    }

    fn kit_element(&self, node: &Node, kit: &Kit, cx: &mut Context<Self>) -> AnyElement {
        let id = node.identity.get();
        let element_id = ElementId::Integer(id);
        let text = |name: &str| SharedString::from(kit.text(name));
        let root = cx.entity().downgrade();
        // A plain event handler for Kit callbacks that run with &mut App.
        let event = |event: &'static str| {
            let root = root.clone();
            move |value: String, cx: &mut App| {let _ = root.update(cx, |root, cx| root.emit(id, event, value, cx));}
        };
        match kit.name.as_str() {
            "Icon" => match icon_name(&kit.text("name")) {
                Some(name) => {
                    let size = match kit.prop("size") {Some("xs") => KitSize::XSmall, Some("sm") => KitSize::Small, Some("lg") => KitSize::Large, _ => KitSize::Medium};
                    Icon::new(name).with_size(size).into_any_element()
                }
                None => div().into_any_element(),
            },
            "Animate" => {
                let kind = kit.text("kind");
                let mut animation = Animation::new(std::time::Duration::from_millis(kit.index("duration").unwrap_or(300) as u64)).with_easing(ease_in_out);
                if kit.flag("repeat") {animation = animation.repeat();}
                let key = ElementId::Name(format!("animate-{id}-{}", kit.text("key")).into());
                div().relative().children(self.content(node, cx)).with_animation(key, animation, move |element, delta| match kind.as_str() {
                    "fade-out" => element.opacity(1. - delta),
                    "slide-up" => element.top(px((1. - delta) * 16.)).opacity(delta),
                    "slide-down" => element.top(px((delta - 1.) * 16.)).opacity(delta),
                    "slide-left" => element.left(px((1. - delta) * 16.)).opacity(delta),
                    "slide-right" => element.left(px((delta - 1.) * 16.)).opacity(delta),
                    "pulse" => element.opacity(0.4 + 0.6 * (1. - (2. * delta - 1.).abs())),
                    _ => element.opacity(delta),
                }).into_any_element()
            }
            "Badge" => {
                let mut badge = Badge::new();
                if kit.flag("dot") {badge = badge.dot();}
                if let Some(count) = kit.index("count") {badge = badge.count(count);}
                if let Some(max) = kit.index("max") {badge = badge.max(max);}
                badge.children(self.content(node, cx)).into_any_element()
            }
            "Tag" => {
                let tag = match kit.prop("variant") {
                    Some("primary") => Tag::primary(), Some("danger") => Tag::danger(), Some("success") => Tag::success(),
                    Some("warning") => Tag::warning(), Some("info") => Tag::info(), _ => Tag::secondary(),
                };
                let tag = if kit.flag("outline") {tag.outline()} else {tag};
                tag.children(self.content(node, cx)).into_any_element()
            }
            "Label" => Label::new(text("text")).secondary(text("secondary")).into_any_element(),
            "Kbd" => match keystroke(&kit.text("keys")) {
                Some(keys) => {let kbd = Kbd::new(keys); if kit.flag("outline") {kbd.outline().into_any_element()} else {kbd.into_any_element()}}
                None => div().child(text("keys")).into_any_element(),
            },
            "Separator" => {
                let separator = if kit.prop("orientation") == Some("vertical") {Separator::vertical()} else {Separator::horizontal()};
                let separator = if kit.flag("dashed") {separator.dashed()} else {separator};
                match kit.prop("label") {Some(label) if !label.is_empty() => separator.label(label.to_owned()).into_any_element(), _ => separator.into_any_element()}
            }
            "Skeleton" => if kit.flag("secondary") {Skeleton::new().secondary().into_any_element()} else {Skeleton::new().into_any_element()},
            "Spinner" => Spinner::new().into_any_element(),
            "Progress" => Progress::new(element_id).value(kit.number("value").unwrap_or_default() as f32).loading(kit.flag("loading")).into_any_element(),
            "ProgressCircle" => ProgressCircle::new(element_id).value(kit.number("value").unwrap_or_default() as f32).loading(kit.flag("loading")).into_any_element(),
            "Avatar" => {
                let mut avatar = Avatar::new().name(text("name"));
                if let Some(source) = kit.prop("src").filter(|source| !source.is_empty()) {avatar = avatar.src(SharedString::from(source.to_owned()));}
                avatar.into_any_element()
            }
            "Alert" => {
                let message = text("message");
                let mut alert = match kit.prop("variant") {
                    Some("success") => Alert::success(element_id, message), Some("warning") => Alert::warning(element_id, message),
                    Some("error") => Alert::error(element_id, message), _ => Alert::info(element_id, message),
                };
                if let Some(title) = kit.prop("title").filter(|title| !title.is_empty()) {alert = alert.title(title.to_owned());}
                if kit.flag("banner") {alert = alert.banner();}
                if kit.handles("close") {let close = event("close"); alert = alert.on_close(move |_, _, cx| close(String::new(), cx));}
                alert.into_any_element()
            }
            "ShimmerText" => ShimmerText::new(text("text")).id(element_id).into_any_element(),
            "Empty" => Empty::new()
                .header(EmptyHeader::new().title(EmptyTitle::new().child(text("title"))).description(EmptyDescription::new().child(text("description"))))
                .content(EmptyContent::new().children(self.content(node, cx))).into_any_element(),
            "GroupBox" => GroupBox::new().title(text("title")).children(self.content(node, cx)).into_any_element(),
            "DescriptionList" => {
                let list = if kit.flag("vertical") {DescriptionList::vertical()} else {DescriptionList::new()};
                let mut list = list.columns(kit.index("columns").unwrap_or(3)).bordered(kit.flag("bordered"));
                for (_, item) in items(node, "DescriptionItem") {list = list.item(item.text("label"), item.text("value"), item.index("span").unwrap_or(1));}
                list.into_any_element()
            }
            "Breadcrumb" => {
                let mut trail = Breadcrumb::new();
                for (child, item) in items(node, "BreadcrumbItem") {
                    let mut entry = BreadcrumbItem::new(item.text("label")).disabled(item.flag("disabled"));
                    if item.handles("click") {
                        let child = child.identity.get();
                        let root = root.clone();
                        entry = entry.on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(child, "click", String::new(), cx));});
                    }
                    trail = trail.child(entry);
                }
                trail.into_any_element()
            }
            "Clipboard" => Clipboard::new(element_id).value(text("value")).into_any_element(),
            "Checkbox" => {
                let mut checkbox = Checkbox::new(element_id).checked(kit.flag("checked")).disabled(kit.flag("disabled"));
                if let Some(label) = kit.prop("label") {checkbox = checkbox.label(label.to_owned());}
                if kit.handles("change") {checkbox = checkbox.on_click(cx.listener(move |this, checked: &bool, _, cx| this.emit(id, "change", checked.to_string(), cx)));}
                checkbox.into_any_element()
            }
            "Switch" => {
                let mut switch = Switch::new(element_id).checked(kit.flag("checked")).disabled(kit.flag("disabled"));
                if let Some(label) = kit.prop("label") {switch = switch.label(label.to_owned());}
                if kit.handles("change") {switch = switch.on_click(cx.listener(move |this, checked: &bool, _, cx| this.emit(id, "change", checked.to_string(), cx)));}
                switch.into_any_element()
            }
            "RadioGroup" => {
                let group = if kit.flag("vertical") {RadioGroup::vertical(element_id)} else {RadioGroup::horizontal(element_id)};
                let selected = kit.number("selected").filter(|index| *index >= 0.).map(|index| index as usize);
                let mut group = group.selected_index(selected).disabled(kit.flag("disabled"));
                for (child, item) in items(node, "Radio") {group = group.child(Radio::new(ElementId::Integer(child.identity.get())).label(item.text("label")).disabled(item.flag("disabled")));}
                if kit.handles("change") {group = group.on_click(cx.listener(move |this, index: &usize, _, cx| this.emit(id, "change", index.to_string(), cx)));}
                group.into_any_element()
            }
            "Toggle" => {
                let mut toggle = Toggle::new(element_id).label(text("label")).checked(kit.flag("checked"));
                if kit.handles("change") {toggle = toggle.on_click(cx.listener(move |this, checked: &bool, _, cx| this.emit(id, "change", checked.to_string(), cx)));}
                toggle.into_any_element()
            }
            "Rating" => {
                let mut rating = Rating::new(element_id).value(kit.index("value").unwrap_or(0)).max(kit.index("max").unwrap_or(5)).disabled(kit.flag("disabled"));
                if kit.handles("change") {rating = rating.on_click(cx.listener(move |this, value: &usize, _, cx| this.emit(id, "change", value.to_string(), cx)));}
                rating.into_any_element()
            }
            "Pagination" => {
                let mut pages = Pagination::new(element_id).current_page(kit.index("page").unwrap_or(1)).total_pages(kit.index("total").unwrap_or(1));
                if kit.flag("compact") {pages = pages.compact();}
                if kit.handles("change") {pages = pages.on_click(cx.listener(move |this, page: &usize, _, cx| this.emit(id, "change", page.to_string(), cx)));}
                pages.into_any_element()
            }
            "Stepper" => {
                let mut stepper = Stepper::new(element_id).selected_index(kit.index("selected").unwrap_or(0));
                if kit.flag("vertical") {stepper = stepper.vertical();}
                stepper.items(items(node, "StepperItem").map(|(_, item)| StepperItem::new().disabled(item.flag("disabled")).child(SharedString::from(item.text("label"))))).into_any_element()
            }
            "Tabs" => {
                let mut bar = TabBar::new(element_id).selected_index(kit.index("selected").unwrap_or(0));
                bar = match kit.prop("variant") {Some("underline") => bar.underline(), Some("pill") => bar.pill(), Some("outline") => bar.outline(), Some("segmented") => bar.segmented(), _ => bar};
                for (index, (_, item)) in items(node, "Tab").enumerate() {
                    let mut tab = Tab::new().label(item.text("label")).disabled(item.flag("disabled"));
                    if kit.handles("change") {tab = tab.on_click(cx.listener(move |this, _, _, cx| this.emit(id, "change", index.to_string(), cx)));}
                    bar = bar.child(tab);
                }
                bar.into_any_element()
            }
            "Accordion" => {
                let mut accordion = Accordion::new(element_id).multiple(kit.flag("multiple")).bordered(kit.prop("bordered") != Some("false"));
                // Without onChange the renderer keeps the open sections itself.
                let controlled = kit.handles("change");
                let open_sections = self.accordions.get(&id);
                for (index, (child, item)) in items(node, "AccordionItem").enumerate() {
                    let body = self.content(child, cx);
                    let open = match open_sections {Some(sections) if !controlled => sections.contains(&index), _ => item.flag("open")};
                    let (title, disabled) = (item.text("title"), item.flag("disabled"));
                    accordion = accordion.item(move |entry| entry.title(SharedString::from(title)).open(open).disabled(disabled).child(div().children(body)));
                }
                accordion = accordion.on_toggle_click(cx.listener(move |this, open: &[usize], _, cx| {
                    if controlled {this.emit(id, "change", open.iter().map(ToString::to_string).collect::<Vec<_>>().join(","), cx);}
                    else {this.accordions.insert(id, open.iter().copied().collect()); cx.notify();}
                }));
                accordion.into_any_element()
            }
            "Collapsible" => Collapsible::new().open(kit.flag("open")).content(div().children(self.content(node, cx))).into_any_element(),
            "Select" => match self.states.get(&id) {
                Some(KitState::Select(state)) => Select::new(state).placeholder(text("placeholder")).disabled(kit.flag("disabled")).into_any_element(),
                _ => div().into_any_element(),
            },
            "NumberInput" => match self.states.get(&id) {
                Some(KitState::Number(state)) => NumberInput::new(state).placeholder(text("placeholder")).into_any_element(),
                _ => div().into_any_element(),
            },
            "OtpInput" => match self.states.get(&id) {Some(KitState::Otp(state)) => OtpInput::new(state).into_any_element(), _ => div().into_any_element()},
            "DatePicker" => match self.states.get(&id) {
                Some(KitState::Date(state)) => DatePicker::new(state).placeholder(text("placeholder")).into_any_element(),
                _ => div().into_any_element(),
            },
            "ColorPicker" => match self.states.get(&id) {
                Some(KitState::Color(state)) => {let picker = ColorPicker::new(state); match kit.prop("label") {Some(label) if !label.is_empty() => picker.label(label.to_owned()).into_any_element(), _ => picker.into_any_element()}}
                _ => div().into_any_element(),
            },
            "Slider" => match self.states.get(&id) {
                Some(KitState::Slider(state)) => Slider::new(state).disabled(kit.flag("disabled")).into_any_element(),
                _ => div().into_any_element(),
            },
            "Tooltip" => {
                let tip = text("text");
                div().id(element_id).children(self.content(node, cx)).tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx)).into_any_element()
            }
            "Popover" | "HoverCard" => {
                let slot = |name: &'static str| items(node, name).next().map(|(child, _)| child);
                let trigger = slot(if kit.name == "Popover" {"PopoverTrigger"} else {"HoverCardTrigger"}).map(|child| self.content(child, cx)).unwrap_or_default();
                let body = slot(if kit.name == "Popover" {"PopoverContent"} else {"HoverCardContent"}).map(|child| child.identity.get());
                let render = move |cx: &mut App| match body {Some(body) => root.update(cx, |root, cx| root.content_of(body, cx)).unwrap_or_default(), None => Vec::new()};
                let trigger = div().children(trigger);
                if kit.name == "Popover" {
                    Popover::new(element_id).trigger(Trigger {element: trigger.into_any_element(), selected: false}).content(move |_, _, cx| div().flex().flex_col().gap_2().children(render(cx))).into_any_element()
                } else {
                    HoverCard::new(element_id).trigger(trigger).content(move |_, _, cx| div().flex().flex_col().gap_2().children(render(cx))).into_any_element()
                }
            }
            "Dialog" | "Sheet" => div().into_any_element(),
            "DropdownMenu" | "ContextMenu" => {
                let entries = items(node, "MenuItem").map(|(child, item)| (child.identity.get(), item.text("label"), item.flag("disabled"), item.flag("checked"))).collect::<Vec<_>>();
                let build = move |mut menu: gpui_kit::component::menu::PopupMenu| {
                    for (child, label, disabled, checked) in &entries {
                        let (child, root) = (*child, root.clone());
                        menu = menu.item(PopupMenuItem::new(label.clone()).disabled(*disabled).checked(*checked).on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(child, "select", String::new(), cx));}));
                    }
                    menu
                };
                if kit.name == "DropdownMenu" {
                    DropdownButton::new(element_id).button(gpui_kit::component::button::Button::new(ElementId::Name(format!("{id}-button").into())).label(text("label")))
                        .dropdown_menu(move |menu, _, _| build(menu)).into_any_element()
                } else {
                    let theme = Theme::default();
                    let content = node.visible_children().filter(|child| child.kit.as_ref().is_none_or(|kit| kit.name != "MenuItem")).map(|child| self.element(child, &theme, "light", cx)).collect::<Vec<_>>();
                    div().id(element_id).children(content).context_menu(move |menu, _, _| build(menu)).into_any_element()
                }
            }
            "Sidebar" => {
                let mut sidebar = Sidebar::new(element_id).collapsed(kit.flag("collapsed"));
                for (group_node, group) in items(node, "SidebarGroup") {
                    let entries = items(group_node, "SidebarItem").map(|(child, item)| {
                        let mut entry = SidebarMenuItem::new(item.text("label")).active(item.flag("active"));
                        if item.handles("click") {
                            let (child, root) = (child.identity.get(), root.clone());
                            entry = entry.on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(child, "click", String::new(), cx));});
                        }
                        entry
                    }).collect::<Vec<_>>();
                    sidebar = sidebar.child(SidebarGroup::new(group.text("label")).child(SidebarMenu::new().children(entries)));
                }
                sidebar.into_any_element()
            }
            "Resizable" => {
                let mut group = if kit.flag("vertical") {v_resizable(element_id)} else {h_resizable(element_id)};
                for (child, panel) in items(node, "ResizablePanel") {
                    let mut entry = resizable_panel().child(div().size_full().children(self.content(child, cx)));
                    if let Some(size) = panel.number("size") {entry = entry.size(px(size as f32));}
                    group = group.child(entry);
                }
                group.into_any_element()
            }
            "Table" => {
                let mut table = Table::new();
                for (section, kit) in node.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))) {
                    let rows = items(section, "TableRow").map(|(row, _)| {
                        let mut cells = TableRow::new();
                        for (cell, cell_kit) in row.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))) {
                            let content = div().children(self.content(cell, cx));
                            let span = cell_kit.index("colSpan").unwrap_or(1);
                            cells = if cell_kit.name == "TableHead" {cells.child(TableHead::new().col_span(span).child(content))} else {cells.child(TableCell::new().col_span(span).child(content))};
                        }
                        cells
                    }).collect::<Vec<_>>();
                    if kit.name == "TableHeader" {
                        let mut header = TableHeader::new();
                        for row in rows {header = header.child(row);}
                        table = table.child(header);
                    } else {
                        let mut body = TableBody::new();
                        for row in rows {body = body.child(row);}
                        table = table.child(body);
                    }
                }
                table.into_any_element()
            }
            "Markdown" => TextView::markdown(element_id, text("content")).into_any_element(),
            "Html" => TextView::html(element_id, text("content")).into_any_element(),
            "Image" => {
                let fit = match kit.prop("fit") {Some("cover") => ObjectFit::Cover, Some("fill") => ObjectFit::Fill, Some("none") => ObjectFit::None, _ => ObjectFit::Contain};
                let alt = text("alt");
                let fallback = alt.clone();
                // Existing files load from disk; other sources are URLs or embedded asset names.
                let source = kit.text("src");
                let source: ImageSource = if std::path::Path::new(&source).is_file() {std::path::PathBuf::from(source).into()} else {SharedString::from(source).into()};
                img(source).id(element_id).size_full().object_fit(fit)
                    .with_loading(move || div().child(alt.clone()).into_any_element())
                    .with_fallback(move || div().child(fallback.clone()).into_any_element()).into_any_element()
            }
            "TitleBar" => TitleBar::new().children(self.content(node, cx)).into_any_element(),
            "StatusBar" => {
                let mut bar = StatusBar::new();
                for child in node.visible_children() {
                    let element = div().flex().items_center().gap_2().children(self.content(child, cx));
                    bar = match child.kit.as_ref().map(|kit| kit.name.as_str()) {
                        Some("StatusBarLeft") => bar.left(element),
                        Some("StatusBarRight") => bar.right(element),
                        _ => {let theme = Theme::default(); bar.child(self.element(child, &theme, "light", cx))}
                    };
                }
                bar.into_any_element()
            }
            "Toolbar" => {
                let mut toolbar = Toolbar::new(element_id);
                for child in self.content(node, cx) {toolbar = toolbar.content(child);}
                toolbar.into_any_element()
            }
            "Form" => {
                let form = if kit.flag("horizontal") {h_form()} else {v_form()};
                let mut form = form.columns(kit.index("columns").unwrap_or(1).max(1));
                for (child, item) in items(node, "Field") {
                    let mut entry = field().label(item.text("label")).required(item.flag("required")).child(div().children(self.content(child, cx)));
                    if let Some(description) = item.prop("description").filter(|text| !text.is_empty()) {entry = entry.description(description.to_owned());}
                    form = form.child(entry);
                }
                form.into_any_element()
            }
            "ScrollArea" => {
                let area = div().id(element_id).size_full().children(self.content(node, cx).into_iter().map(|child| div().flex_shrink_0().child(child)));
                if kit.flag("horizontal") {area.flex().overflow_x_scrollbar().into_any_element()} else {area.overflow_y_scrollbar().into_any_element()}
            }
            "Message" => {
                let author = text("author");
                let bubble = Bubble::new().with_variant(if kit.prop("variant") == Some("primary") {BubbleVariant::Filled} else {BubbleVariant::Secondary}).children(self.content(node, cx));
                let mut message = Message::new().alignment(if kit.prop("align") == Some("end") {MessageAlignment::End} else {MessageAlignment::Start});
                if !author.is_empty() {message = message.avatar(Avatar::new().name(author.clone()).size_8()).header(MessageHeader::new().child(author).child(text("time")));}
                message.content(MessageContent::new().bubble(bubble)).into_any_element()
            }
            "Chart" => {
                let labels = kit.text("labels");
                let values = kit.text("values");
                let data = labels.split('\n').zip(values.split('\n')).map(|(label, value)| (SharedString::from(label.to_owned()), value.trim().parse::<f64>().unwrap_or_default())).collect::<Vec<_>>();
                match kit.prop("kind") {
                    Some("bar") => BarChart::new(data).band(|point: &(SharedString, f64)| point.0.clone()).value(|point: &(SharedString, f64)| point.1).into_any_element(),
                    Some("area") => AreaChart::new(data).x(|point: &(SharedString, f64)| point.0.clone()).y(|point: &(SharedString, f64)| point.1).into_any_element(),
                    Some("pie") => {
                        // Each slice takes the next theme chart color.
                        let theme = cx.theme();
                        let palette = [theme.chart_1, theme.chart_2, theme.chart_3, theme.chart_4, theme.chart_5];
                        let data = data.into_iter().enumerate().map(|(index, (label, value))| (label, value, palette[index % palette.len()])).collect::<Vec<_>>();
                        PieChart::new(data).value(|point: &(SharedString, f64, Hsla)| point.1 as f32).color(|point: &(SharedString, f64, Hsla)| point.2).outer_radius(100.).into_any_element()
                    }
                    _ => LineChart::new(data).x(|point: &(SharedString, f64)| point.0.clone()).y(|point: &(SharedString, f64)| point.1).into_any_element(),
                }
            }
            "List" => match self.states.get(&id) {
                Some(KitState::List(handle)) => {
                    let count = kit.text("items").split('\n').filter(|item| !item.is_empty()).count();
                    let sizes = std::rc::Rc::new(vec![size(px(1.), px(32.)); count]);
                    v_virtual_list(cx.entity(), element_id, sizes, move |root: &mut Root, range: std::ops::Range<usize>, _, cx| {
                        let Some(kit) = find(&root.node, id).and_then(|node| node.kit.as_deref()) else {return Vec::new()};
                        let rows = kit.text("items").split('\n').filter(|item| !item.is_empty()).map(str::to_owned).collect::<Vec<_>>();
                        let selected = kit.number("selected").filter(|index| *index >= 0.).map(|index| index as usize);
                        range.filter_map(|index| rows.get(index).map(|row| {
                            ListItem::new(ElementId::Name(format!("{id}-row-{index}").into())).selected(selected == Some(index)).child(SharedString::from(row.clone()))
                                .on_click(cx.listener(move |this, _, _, cx| this.emit(id, "select", index.to_string(), cx)))
                        })).collect()
                    }).track_scroll(handle).into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Tree" => match self.states.get(&id) {
                Some(KitState::Tree(state, _)) => tree(state, move |index, entry, selected, _, _| {
                    let item_id = entry.item().id.to_string();
                    let root = root.clone();
                    ListItem::new(index).selected(selected).pl(px(12. * entry.depth() as f32 + 8.)).child(entry.item().label.clone())
                        .on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(id, "select", item_id.clone(), cx));})
                }).into_any_element(),
                _ => div().into_any_element(),
            },
            "Calendar" => match self.states.get(&id) {
                Some(KitState::Calendar(state)) => Calendar::new(state).number_of_months(kit.index("months").unwrap_or(1).max(1)).into_any_element(),
                _ => div().into_any_element(),
            },
            "TimeField" => match self.states.get(&id) {Some(KitState::Time(state)) => TimeField::new(state).into_any_element(), _ => div().into_any_element()},
            "Combobox" => match self.states.get(&id) {
                Some(KitState::Combobox(state)) => Combobox::new(state).placeholder(text("placeholder")).into_any_element(),
                _ => div().into_any_element(),
            },
            "Command" => match self.states.get(&id) {
                Some(KitState::Command(state)) => {
                    let entries = items(node, "CommandItem").map(|(child, item)| (child.identity.get(), item.text("label"), item.flag("disabled"))).collect::<Vec<_>>();
                    let targets = entries.iter().map(|(child, _, _)| *child).collect::<Vec<_>>();
                    let palette = CommandPalette::new(state).placeholder(text("placeholder"))
                        .items(entries.into_iter().map(|(_, label, disabled)| CommandItem::new().label(label).disabled(disabled)));
                    let root = root.clone();
                    palette.on_confirm(move |index: IndexPath, _, cx| {
                        if let Some(target) = targets.get(index.row) {let target = *target; let _ = root.update(cx, |root, cx| root.emit(target, "select", String::new(), cx));}
                    }).into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Carousel" => match self.states.get(&id) {
                Some(KitState::Carousel(state)) => {
                    let mut content = CarouselContent::new(state);
                    for (index, (child, _)) in items(node, "CarouselSlide").enumerate() {
                        content = content.child(CarouselItem::new(ElementId::Name(format!("{id}-slide-{index}").into()), index, state).child(div().size_full().children(self.content(child, cx))));
                    }
                    Carousel::new(element_id, state).child(content).child(CarouselPrevious::new(state)).child(CarouselNext::new(state)).into_any_element()
                }
                _ => div().into_any_element(),
            },
            "CodeEditor" => match self.states.get(&id) {
                Some(KitState::Editor(state)) => Editor::new(state).h_full().readonly(kit.flag("readOnly")).into_any_element(),
                _ => div().into_any_element(),
            },
            "ToggleGroup" => {
                let toggles = items(node, "Toggle").map(|(child, item)| (child.identity.get(), item.text("label"), item.flag("checked"), item.handles("change"))).collect::<Vec<_>>();
                let targets = toggles.iter().map(|(child, _, checked, handles)| (*child, *checked, *handles)).collect::<Vec<_>>();
                let mut group = ToggleGroup::new(element_id).children(toggles.into_iter().map(|(child, label, checked, _)| Toggle::new(ElementId::Integer(child)).label(label).checked(checked)));
                if kit.flag("segmented") {group = group.segmented();}
                group.on_click(cx.listener(move |this, states: &Vec<bool>, _, cx| {
                    for ((target, before, handles), now) in targets.iter().zip(states) {
                        if *handles && before != now {this.emit(*target, "change", now.to_string(), cx);}
                    }
                })).into_any_element()
            }
            "ButtonGroup" => {
                let buttons = items(node, "GroupButton").map(|(child, item)| (child.identity.get(), item.text("label"), item.flag("selected"))).collect::<Vec<_>>();
                let targets = buttons.iter().map(|(child, _, _)| *child).collect::<Vec<_>>();
                let mut group = ButtonGroup::new(element_id).children(buttons.into_iter().map(|(child, label, selected)| gpui_kit::component::button::Button::new(ElementId::Integer(child)).label(label).selected(selected)));
                if kit.flag("outline") {group = group.outline();}
                group.on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
                    for index in clicked {if let Some(target) = targets.get(*index) {this.emit(*target, "click", String::new(), cx);}}
                })).into_any_element()
            }
            // Item and slot components render through their container.
            name => div().child(SharedString::from(format!("{name} must be placed inside its container component"))).into_any_element(),
        }
    }
}

/// TreeItem children as GPUI Kit tree items, recursively.
fn tree_items(node: &Node) -> Vec<TreeItem> {
    items(node, "TreeItem").map(|(child, item)| {
        let mut entry = TreeItem::new(item.text("id"), item.text("label")).expanded(item.flag("expanded"));
        for sub in tree_items(child) {entry = entry.child(sub);}
        entry
    }).collect()
}

/// A fingerprint of a tree's items, to rebuild the Kit state only when they change.
fn tree_shape(root: &Node, id: u64) -> String {
    fn walk(node: &Node, out: &mut String) {
        for (child, item) in items(node, "TreeItem") {out.push_str(&item.text("id")); out.push('\u{1}'); out.push_str(&item.text("label")); out.push('['); walk(child, out); out.push(']');}
    }
    let mut out = String::new();
    if let Some(node) = find(root, id) {walk(node, &mut out);}
    out
}
