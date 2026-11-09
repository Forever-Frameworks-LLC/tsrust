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
use gpui_kit::component::{Placement, Selectable, WindowExt};

enum KitState {
    Select(Entity<SelectState<Vec<SharedString>>>),
    Number(Entity<InputState>),
    Otp(Entity<OtpState>),
    Date(Entity<DatePickerState>),
    Color(Entity<ColorPickerState>),
    Slider(Entity<SliderState>),
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
            if let Some(kit) = &node.kit && matches!(kit.name.as_str(), "Select" | "NumberInput" | "OtpInput" | "DatePicker" | "ColorPicker" | "Slider" | "Dialog" | "Sheet") {
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
                    KitState::Date(_) | KitState::Otp(_) => {}
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
            // Item and slot components render through their container.
            name => div().child(SharedString::from(format!("{name} must be placed inside its container component"))).into_any_element(),
        }
    }
}
