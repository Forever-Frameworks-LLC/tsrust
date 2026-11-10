//! GPUI Kit renderer for the retained Node model. One root view owns the tree.
//! Every event updates Node state, refreshes its live bindings and notifies the
//! view; `render` turns the current tree into GPUI elements. Buttons without
//! classes and text inputs use GPUI Kit components; classed buttons use Kit's
//! unstyled native button behavior. Utilities resolve against the active theme.
use super::{Kind, Node, kit::Kit, style::{self, Length, State, Utility}, theme::{ResolvedColor, Theme}, window::Options};
use gpui_kit::component::Disableable;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonRounded, ButtonVariants};
use gpui_kit::component::input::{Input, InputContentType, InputEvent, InputState, MaskPattern, TabSize, Textarea, TextareaState, WrappingIndent};
use gpui_kit::*;
use std::collections::HashMap;

pub fn run(node: Node, title: String) -> Result<(), String> {run_configured(node, title, "{}")}

/// Ends the application on an unhandled UI error. GPUI's run loop exits the process when it quits instead of
/// returning, so the error is printed and the process exits with status 1 here, as an uncaught error would.
fn fail(failure: &Failure, error: String) -> ! {
    eprintln!("Uncaught {error}");
    *failure.borrow_mut() = Some(error);
    std::process::exit(1)
}

pub fn run_configured(mut node: Node, title: String, options: &str) -> Result<(), String> {
    let options = Options::parse(options)?;
    node.refresh()?;
    node.ensure_identities();
    let failure = std::rc::Rc::new(std::cell::RefCell::new(None::<String>));
    let reported = failure.clone();
    application().with_assets(assets::Assets).run(move |cx| {
        init(cx);
        #[cfg(feature = "gpui-http")]
        cx.set_http_client(std::sync::Arc::new(RemoteImages::new()));
        gpui_kit::component::set_locale(&super::i18n::locale());
        // The application ends with its last window, as a single-window app does.
        cx.on_window_closed(|cx, _| if cx.windows().is_empty() {cx.quit();}).detach();
        if let Err(error) = open(node, title, options, reported.clone(), cx) {fail(&reported, error);}
    });
    match failure.take() {Some(error) => Err(error), None => Ok(())}
}

/// GPUI fetches remote image sources through the application's HTTP client.
#[cfg(feature = "gpui-http")]
struct RemoteImages {client: reqwest::Client, runtime: tokio::runtime::Runtime}

#[cfg(feature = "gpui-http")]
impl RemoteImages {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("HTTP runtime");
        Self {client: reqwest::Client::new(), runtime}
    }
}

#[cfg(feature = "gpui-http")]
impl http_client::HttpClient for RemoteImages {
    fn user_agent(&self) -> Option<&http_client::http::HeaderValue> {None}
    fn proxy(&self) -> Option<&http_client::Url> {None}
    fn send(&self, request: http_client::Request<http_client::AsyncBody>) -> std::pin::Pin<Box<dyn std::future::Future<Output = http_client::Result<http_client::Response<http_client::AsyncBody>>> + Send>> {
        let mut outgoing = self.client.request(reqwest::Method::from_bytes(request.method().as_str().as_bytes()).unwrap_or(reqwest::Method::GET), request.uri().to_string());
        for (name, value) in request.headers() {outgoing = outgoing.header(name.as_str(), value.as_bytes());}
        let task = self.runtime.spawn(async move {
            let response = outgoing.send().await?;
            let mut builder = http_client::http::Response::builder().status(response.status().as_u16());
            for (name, value) in response.headers() {builder = builder.header(name.as_str(), value.as_bytes());}
            let bytes = response.bytes().await?;
            http_client::Result::Ok(builder.body(http_client::AsyncBody::from(bytes.to_vec()))?)
        });
        Box::pin(async move {task.await.map_err(|error| http_client::anyhow!(error.to_string()))?})
    }
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
    open_window(window, cx, move |_, cx| cx.new(|cx| Root {viewport:(options.width as f32,options.height as f32), revision: u64::MAX, command_owner: commands::Owner::new(), styles: HashMap::new(), focus: cx.focus_handle(), node, inputs: HashMap::new(), input_options: HashMap::new(), states: HashMap::new(), dialog: None, sheet: None, accordions: HashMap::new(), failure}))
        .map(|_| ()).map_err(|error| format!("Failed to open the window: {error}"))
}

/// Mount-time input options: assistive text input settings GPUI reads when the field is created.
macro_rules! configured {($state:expr, $options:expr) => {{
    let options: &std::collections::BTreeMap<String, String> = $options;
    let option = |key: &str| options.get(key).map(String::as_str);
    let mut state = $state;
    if let Some(action) = option("enterKeyHint") {
        state = state.input_action(match action {"done" => TextInputAction::Done, "go" => TextInputAction::Go, "next" => TextInputAction::Next, "previous" => TextInputAction::Previous, "search" => TextInputAction::Search, "send" => TextInputAction::Send, _ => TextInputAction::Enter});
    }
    if let Some(correct) = option("autoCorrect") {state = state.autocorrect(correct == "true");}
    if let Some(replace) = option("replaceable") {state = state.replaceable(replace == "true");}
    if let Some(capitalize) = option("autoCapitalize") {
        state = state.autocapitalize(match capitalize {"words" => Autocapitalize::Words, "sentences" => Autocapitalize::Sentences, "characters" => Autocapitalize::Characters, _ => Autocapitalize::None});
    }
    state
}}}

/// Options any input state re-applies when its props change.
macro_rules! apply_input_options {($state:expr, $options:expr, $placeholder:expr, $window:expr, $cx:expr) => {{
    let options: &std::collections::BTreeMap<String, String> = $options;
    let option = |key: &str| options.get(key).map(String::as_str);
    let flag = |key: &str| option(key) == Some("true");
    $state.set_placeholder($placeholder, $window, $cx);
    $state.set_clean_on_escape(flag("cleanOnEscape"));
    if let Some(submit) = option("submitOnEnter") {$state.set_submit_on_enter(submit == "true", $cx);}
    $state.set_context_menu_enabled(option("contextMenu") != Some("false"));
    $state.set_show_whitespaces(flag("showWhitespace"), $window, $cx);
    $state.set_text_align(match option("textAlign") {Some("center") => TextAlign::Center, Some("right") => TextAlign::Right, _ => TextAlign::Left}, $cx);
    let lines = |key: &str| option(key).and_then(|value| value.parse::<usize>().ok());
    $state.set_scroll_beyond_last_line(lines("scrollBeyondLastLine"), $window, $cx);
    $state.set_cursor_surrounding_lines(lines("cursorSurroundingLines"), $window, $cx);
    let mask = match (option("mask"), option("thousandsSeparator"), option("fractionDigits")) {
        (Some(pattern), _, _) if !pattern.is_empty() => MaskPattern::from(pattern),
        (_, None, None) => MaskPattern::None,
        (_, separator, fraction) => MaskPattern::Number {separator: separator.and_then(|separator| separator.chars().next()), fraction: fraction.and_then(|digits| digits.parse().ok())},
    };
    $state.set_mask_pattern(mask, $window, $cx);
}}}

/// Paste decisions run before Kit inserts text; redraw after the synchronous decision.
macro_rules! paste_options {($view:expr,$node:expr,$this:expr,$cx:expr)=>{{
    let mut view=$view;
    if let Some(predicate)=$node.kit.as_deref().and_then(|kit|kit.predicates.get("paste")).cloned() {
        let failure=$this.failure.clone();let scope=$this.command_owner.context();let root=$cx.entity().downgrade();
        view=view.on_paste(move|clipboard,_,cx| {
            let _scope=scope();
            let paths=clipboard.entries().iter().filter_map(|entry|if let ClipboardEntry::ExternalPaths(paths)=entry {Some(&paths.0)}else{None}).flatten().map(|path|path.to_string_lossy().into_owned()).collect::<Vec<_>>();
            let image_count=clipboard.entries().iter().filter(|entry|matches!(entry,ClipboardEntry::Image(_))).count();
            let payload=serde_json::json!({"text":clipboard.text(),"paths":paths,"imageCount":image_count}).to_string();
            let consumed=super::kit::predicate(&predicate,&payload).unwrap_or_else(|error|fail(&failure,error));
            let root=root.clone();cx.defer(move|cx|{let _=root.update(cx,|root,cx|root.settle(Ok(true),cx));});
            consumed
        });
    }
    view
}}}

/// Input and Textarea element options from a field's props.
macro_rules! input_options {($input:expr, $node:expr, $line:tt, $this:expr, $cx:expr) => {{
    let node: &Node = $node;
    let id = node.identity.get();
    let option = |name: &str| node.option(name).unwrap_or_default();
    let flag = |name: &str| option(name) == "true";
    let read_only = node.editable.as_ref().is_some_and(|field| field.read_only);
    let mut input = $input.disabled(node.is_disabled()).readonly(read_only);
    if option("appearance") == "false" {input = input.appearance(false);}
    if option("bordered") == "false" {input = input.bordered(false);}
    input = match option("size") {"xsmall" => input.with_size(KitSize::XSmall), "small" => input.with_size(KitSize::Small), "large" => input.with_size(KitSize::Large), _ => input};
    if let Ok(index) = option("tabIndex").parse::<isize>() {input = input.tab_index(index);}
    if !node.semantics.label.is_empty() {input = input.aria_label(node.semantics.label.clone());}
    if !node.semantics.id.is_empty() {input = input.accessibility_id(node.semantics.id.clone());}
    input = match option("role") {"searchbox" => input.role(Role::SearchInput), "combobox" => input.role(Role::ComboBox), "spinbutton" => input.role(Role::SpinButton), "none" => input.role(None::<Role>), _ => input};
    if let Ok(height) = option("height").parse::<f32>() {input = input.h(px(height));}
    let root = $cx.entity().downgrade();
    if node.kit.as_deref().is_some_and(|kit| kit.handles("tokenClick")) {
        let root = root.clone();
        input = input.on_token_click(move |event, _, cx| {let token = event.token().id().to_string(); let _ = root.update(cx, |root, cx| root.emit(id, "tokenClick", token, cx));});
    }
    if node.kit.as_deref().is_some_and(|kit| kit.handles("tokenHover")) {
        input = input.on_token_hover(move |event, _, cx| {let token = if event.is_hovered() {event.token().id().to_string()} else {String::new()}; let _ = root.update(cx, |root, cx| root.emit(id, "tokenHover", token, cx));});
    }
    input=paste_options!(input,node,$this,$cx);
    input_line_options!(input, $line, option, flag)
}}}

/// Options only a single-line Input has.
macro_rules! input_line_options {($input:expr, true, $option:expr, $flag:expr) => {{
    let (option, flag) = ($option, $flag);
    let mut input = $input.cleanable(flag("cleanable")).selected(flag("selected"));
    if option("focusBordered") == "false" {input = input.focus_bordered(false);}
    if flag("maskToggle") || (option("type") == "password" && option("maskToggle") != "false") {input = input.mask_toggle();}
    if let Some(kind) = content_type(option("type"), option("autoComplete")) {input = input.content_type(kind);}
    if let Some(icon) = icon_name(option("prefix")) {input = input.prefix(Icon::new(icon).small());}
    if let Some(icon) = icon_name(option("suffix")) {input = input.suffix(Icon::new(icon).small());}
    if option("height") == "full" {input = input.h_full();}
    input
}}; ($input:expr, false, $option:expr, $flag:expr) => {$input}}

/// Platform autofill hint for a field: its type, else its HTML autocomplete token.
fn content_type(kind: &str, autocomplete: &str) -> Option<InputContentType> {
    use InputContentType::*;
    Some(match (kind, autocomplete) {
        ("password", "new-password") | (_, "new-password") => NewPassword,
        ("password", _) | (_, "current-password") | (_, "password") => Password,
        ("email", _) | (_, "email") => EmailAddress, ("url", _) | (_, "url") => Url, ("tel", _) | (_, "tel") => TelephoneNumber,
        (_, "name") => Name, (_, "honorific-prefix") => NamePrefix, (_, "given-name") => GivenName, (_, "additional-name") => MiddleName, (_, "family-name") => FamilyName,
        (_, "honorific-suffix") => NameSuffix, (_, "nickname") => Nickname, (_, "organization-title") => JobTitle, (_, "organization") => OrganizationName,
        (_, "street-address") => FullStreetAddress, (_, "address-level2") => AddressCity, (_, "address-level1") => AddressState, (_, "address-level3") => Sublocality,
        (_, "country-name") => CountryName, (_, "postal-code") => PostalCode, (_, "cc-number") => CreditCardNumber, (_, "cc-name") => CreditCardName,
        (_, "cc-given-name") => CreditCardGivenName, (_, "cc-additional-name") => CreditCardMiddleName, (_, "cc-family-name") => CreditCardFamilyName,
        (_, "cc-csc") => CreditCardSecurityCode, (_, "cc-exp") => CreditCardExpiration, (_, "cc-exp-month") => CreditCardExpirationMonth, (_, "cc-exp-year") => CreditCardExpirationYear,
        (_, "cc-type") => CreditCardType, (_, "username") => Username, (_, "one-time-code") => OneTimeCode, (_, "bday") => Birthdate, (_, "bday-day") => BirthdateDay,
        (_, "bday-month") => BirthdateMonth, (_, "bday-year") => BirthdateYear, (_, "location") => Location,
        _ => return None,
    })
}

/// One controlInput action on any input state.
macro_rules! control_input {
($state:expr, $action:expr, $value:expr, $label:expr, $start:expr, $end:expr, $window:expr, $cx:expr) => {
    control_input!($state,$action,$value,$label,$start,$end,$window,$cx,{
        let label=$label;
        let token=gpui_kit::component::input::InlineToken::new($value.clone(),$value.clone());
        let token=if label.is_empty(){token}else{token.with_label(label)};
        let _=$state.replace_with_token(token,$window,$cx);
    })
};
($state:expr, $action:expr, $value:expr, $label:expr, $start:expr, $end:expr, $window:expr, $cx:expr, $token:expr) => {{
    let length = $state.value().len();
    let range = $start.min(length)..$end.max($start).min(length);
    match $action.as_str() {
        "focus" => $state.focus($window, $cx),
        "selectAll" => $state.select_all($window, $cx),
        "unselect" => $state.unselect($window, $cx),
        "clear" => $state.clean($window, $cx),
        "insert" => $state.insert($value, $window, $cx),
        "replace" => $state.replace($value, $window, $cx),
        "setValue" => $state.replace_all($value, $window, $cx),
        "setCursor" => $state.set_selected_range(range.start..range.start, $cx),
        "setSelection" => $state.set_selected_range(range, $cx),
        "addSelection" => $state.add_selection(range, $cx),
        "toggleMasked" => $state.toggle_masked($window, $cx),
        "scrollTo" => $state.set_scroll_offset(point(px($start as f32), px($end as f32)), $cx),
        "insertToken" => $token,
        "openSearch" => $state.open_search(false, $cx),
        "openReplace" => $state.open_search(true, $cx),
        "search" => {$state.open_search(false, $cx); $state.set_search_query($value, true, $cx);}
        "nextMatch" => {let _ = $state.next_search_match($cx);}
        "previousMatch" => {let _ = $state.previous_search_match($cx);}
        "replaceMatch" => {let _ = $state.replace_current_search_match(&$value, $window, $cx);}
        "replaceAllMatches" => {let _ = $state.replace_all_search_matches(&$value, $window, $cx);}
        "closeSearch" => $state.close_search($cx),
        _ => {}
    }
}}}

/// The Kit state behind a std:ui text field: one line, or a multiline textarea.
enum Field {Line(Entity<InputState>), Text(Entity<TextareaState>)}

struct Root {
    viewport: (f32,f32),
    revision: u64,
    command_owner: commands::Owner,
    styles: HashMap<u64, Vec<(State, Utility)>>,
    node: Node,
    inputs: HashMap<u64, (Field, Subscription)>,
    /// Options last applied to each input's Kit state, so changed props re-apply once.
    input_options: HashMap<u64, std::collections::BTreeMap<String, String>>,
    /// Kit component state entities (select, sliders, pickers), by node.
    states: HashMap<u64, KitState>,
    /// The node whose dialog or sheet is open, if any.
    dialog: Option<u64>,
    sheet: Option<u64>,
    /// Open sections of uncontrolled accordions.
    accordions: HashMap<u64, std::collections::BTreeSet<usize>>,
    failure: Failure,
    /// Keeps the root on the action dispatch path when no control has focus.
    focus: FocusHandle,
}

impl Root {
    fn scope(&self)->(super::navigation::Scope, commands::Scope) {(self.node.navigation.enter(),self.command_owner.enter())}
    fn activate(&mut self,id:u64,cx:&mut Context<Self>) {
        let _scope=self.scope();
        let result=self.node.activate(id);self.settle(result,cx);
    }
    fn reply(&mut self,reply:commands::Reply,value:&str,cx:&mut Context<Self>) {
        let _scope=self.scope();
        let result=reply(value);self.settle(result.map(|_|true),cx);
    }
    /// A failed handler or binding ends the application with its message, as an
    /// uncaught exception ends a backend program.
    fn settle(&mut self, result: Result<bool, String>, cx: &mut Context<Self>) {
        match result.and_then(|_| self.node.refresh()) {
            Ok(_) => {
                let revision=super::state::revision();
                if self.revision != revision {
                    // shortcut: redraw live windows on shared writes; track subscribers if window counts make this costly.
                    cx.refresh_windows();
                }
                self.revision=revision;
                cx.notify();
            }
            Err(error) => fail(&self.failure, error),
        }
    }

    /// Keep one Kit input state per visible editable node and push controlled
    /// values into it when the Node value no longer matches the edited text.
    /// Input options (masking, masks, rows, search...) apply at creation and whenever they change.
    fn sync_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        type Options = std::collections::BTreeMap<String, String>;
        fn collect(node: &Node, out: &mut Vec<(u64, String, String, bool, Options)>) {
            let Some(node) = node.visible_root() else {return};
            if let Some(field) = &node.editable {
                out.push((node.identity.get(), field.value.clone(), field.placeholder.clone(), field.multiline, node.kit.as_deref().map(|kit| kit.props.clone()).unwrap_or_default()));
            }
            for child in &node.children {collect(child, out);}
        }
        let mut fields = Vec::new();
        collect(&self.node, &mut fields);
        self.inputs.retain(|id, _| fields.iter().any(|field| field.0 == *id));
        self.input_options.retain(|id, _| fields.iter().any(|field| field.0 == *id));
        for (id, value, placeholder, multiline, options) in fields {
            if !self.inputs.contains_key(&id) {
                // Edits update the Node and refresh the tree like any handler; focus, blur and Enter are events.
                let entry = if multiline {
                    let state = cx.new(|cx| configured!(TextareaState::new(window, cx).rows(4).placeholder(placeholder.clone()), &options));
                    let subscription = cx.subscribe_in(&state, window, move |this, state, event, _, cx| {let value = state.read(cx).value().to_string(); this.input_event(id, event, value, cx);});
                    (Field::Text(state), subscription)
                } else {
                    let validator=find(&self.node,id).and_then(|node|node.kit.as_deref()).and_then(|kit|kit.predicates.get("validate")).cloned();
                    let failure=self.failure.clone();let scope=self.command_owner.context();
                    let state = cx.new(|cx| {
                        let state=configured!(InputState::new(window,cx).placeholder(placeholder.clone()),&options);
                        match validator {Some(predicate)=>state.validate(move|value,_| {let _scope=scope();super::kit::predicate(&predicate,value).unwrap_or_else(|error|fail(&failure,error))}),None=>state}
                    });
                    let subscription = cx.subscribe_in(&state, window, move |this, state, event, _, cx| {let value = state.read(cx).value().to_string(); this.input_event(id, event, value, cx);});
                    (Field::Line(state), subscription)
                };
                self.inputs.insert(id, entry);
            }
            let changed = self.input_options.get(&id) != Some(&options);
            match self.inputs.get(&id) {
                Some((Field::Line(state), _)) => state.update(cx, |state, cx| {
                    if state.value().as_ref() != value {state.set_value(value, window, cx);}
                    if changed {
                        apply_input_options!(state, &options, placeholder, window, cx);
                        // Masking, loading and patterns belong to single-line fields.
                        let flag = |key: &str| options.get(key).is_some_and(|value| value == "true");
                        state.set_masked(flag("masked") || options.get("type").is_some_and(|kind| kind == "password"), window, cx);
                        state.set_loading(flag("loading"), window, cx);
                        if let Some(pattern) = options.get("pattern").and_then(|pattern| regex::Regex::new(&format!("^(?:{pattern})$")).ok()) {state.set_pattern(pattern, window, cx);}
                    }
                }),
                Some((Field::Text(state), _)) => state.update(cx, |state, cx| {
                    if state.value().as_ref() != value {state.set_value(value, window, cx);}
                    if changed {
                        apply_input_options!(state, &options, placeholder, window, cx);
                        let number = |key: &str| options.get(key).and_then(|value| value.parse::<usize>().ok());
                        match (number("minRows"), number("maxRows")) {
                            (Some(min), Some(max)) => state.set_auto_grow(min, max.max(min), cx),
                            (Some(min), None) => state.set_auto_grow(min, usize::MAX, cx),
                            _ => if let Some(rows) = number("rows") {state.set_rows(rows, cx);},
                        }
                        if let Some(wrap) = options.get("softWrap") {state.set_soft_wrap(wrap == "true", window, cx);}
                        if let Some(searchable) = options.get("searchable") {state.set_searchable(searchable == "true", cx);}
                        if let Some(bounce) = options.get("scrollBounce") {state.set_scroll_bounce(bounce == "true", cx);}
                        if let Some(indent) = options.get("wrappingIndent") {state.set_wrapping_indent(if indent == "none" {WrappingIndent::None} else {WrappingIndent::Same}, window, cx);}
                        if let Some(size) = number("tabSize") {state.set_tab_size(TabSize {tab_size: size, hard_tabs: options.get("hardTabs").is_some_and(|hard| hard == "true")}, cx);}
                    }
                }),
                None => {}
            }
            if changed {self.input_options.insert(id, options);}
        }
    }

    /// Deliver a Kit input event: edits update the field, focus, blur and Enter reach handlers.
    fn input_event(&mut self, id: u64, event: &InputEvent, value: String, cx: &mut Context<Self>) {
        let _scope=self.scope();
        let result = match event {
            InputEvent::Change => self.node.input(id, &value),
            InputEvent::Focus => self.node.kit_event(id, "focus", ""),
            InputEvent::Blur => self.node.kit_event(id, "blur", ""),
            InputEvent::PressEnter {secondary, ..} => self.node.kit_event(id, "submit", &secondary.to_string()),
        };
        self.settle(result, cx);
    }

    /// The edited field whose semantic id matches, for controlInput and readInput.
    fn input_by_id(&self, target: &str) -> Option<u64> {
        fn find_id(node: &Node, target: &str) -> Option<u64> {
            let node = node.visible_root()?;
            if node.editable.is_some() && node.semantics.id == target || node.kit.as_deref().is_some_and(|kit|kit.name=="CodeEditor" && kit.prop("id")==Some(target)) {return Some(node.identity.get());}
            node.children.iter().find_map(|child| find_id(child, target))
        }
        find_id(&self.node, target)
    }

    /// Run one controlInput action on a field's Kit state.
    fn control_input(&mut self, payload: &serde_json::Value, window: &mut Window, cx: &mut Context<Self>) {
        let text = |key: &str| payload.get(key).and_then(|value| value.as_str()).unwrap_or_default().to_owned();
        let Some(id) = self.input_by_id(&text("id")) else {return};
        let (action, value) = (text("action"), text("value"));
        let number = |key: &str| payload.get(key).and_then(|value| value.as_u64()).unwrap_or(0) as usize;
        let (start, end) = (number("start"), number("end"));
        match self.inputs.get(&id) {
            Some((Field::Line(state), _)) => state.update(cx, |state, cx| control_input!(state, action.clone(), value.clone(), text("label"), start, end, window, cx)),
            Some((Field::Text(state), _)) => state.update(cx, |state, cx| control_input!(state, action.clone(), value.clone(), text("label"), start, end, window, cx)),
            None => if let Some(KitState::Editor(state,_))=self.states.get(&id) {
                let failure=self.failure.clone();
                state.update(cx, |state,cx|control_input!(state,action,value,text("label"),start,end,window,cx,fail(&failure,"CodeEditor does not support inline tokens; use Input or Textarea".into())));
            }
        }
    }

    /// A field's current text, selection and scroll position as JSON, for readInput.
    fn read_input(&self, id: &str, window: &Window, cx: &App) -> String {
        let snapshot = |value: String, unmasked: String, selected: String, range: std::ops::Range<usize>, reversed: bool, offset: Point<Pixels>, rows: Option<std::ops::Range<usize>>, focused: bool| serde_json::json!({
            "found": true, "value": value, "unmaskedValue": unmasked, "selectedText": selected, "selectionStart": range.start, "selectionEnd": range.end,
            "selectionReversed": reversed, "scrollX": f32::from(offset.x), "scrollY": f32::from(offset.y),
            "firstVisibleRow": rows.as_ref().map_or(0, |rows| rows.start), "lastVisibleRow": rows.map_or(0, |rows| rows.end), "focused": focused,
        });
        let target=self.input_by_id(id);
        match target.and_then(|id| self.inputs.get(&id)) {
            Some((Field::Line(state), _)) => {let state = state.read(cx); snapshot(state.value().to_string(), state.unmask_value().to_string(), state.selected_text().to_string(), state.selected_range(), state.is_selection_reversed(), state.scroll_offset(), state.visible_row_range(), state.focus_handle(cx).contains_focused(window, cx))}
            Some((Field::Text(state), _)) => {let state = state.read(cx); snapshot(state.value().to_string(), state.unmask_value().to_string(), state.selected_text().to_string(), state.selected_range(), state.is_selection_reversed(), state.scroll_offset(), state.visible_row_range(), state.focus_handle(cx).contains_focused(window, cx))}
            None => match target.and_then(|id|self.states.get(&id)) {
                Some(KitState::Editor(state,_)) => {
                    let state=state.read(cx);
                    let mut result=snapshot(state.value().to_string(),state.unmask_value().to_string(),state.selected_text().to_string(),state.selected_range(),state.is_selection_reversed(),state.scroll_offset(),state.visible_row_range(),state.focus_handle(cx).contains_focused(window,cx));
                    result["language"]=state.language_name().to_string().into();result
                }
                _=>serde_json::json!({"found": false}),
            },
        }.to_string()
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
                if !option("tooltip").is_empty() {
                    button = if option("tooltipCommand").is_empty() {button.tooltip(option("tooltip").to_owned())}
                        else {button.tooltip_with_action(option("tooltip").to_owned(), &TsrustCommand {name: option("tooltipCommand").to_owned()}, None)};
                }
                button = button.tooltip_placement(match option("tooltipPlacement") {"bottom" => Placement::Bottom, "left" => Placement::Left, "right" => Placement::Right, _ => Placement::Top});
                if let Ok(delay) = option("tooltipDelay").parse::<u64>() {button = button.tooltip_show_delay(std::time::Duration::from_millis(delay));}
                button = match option("rounded") {"none" => button.rounded(ButtonRounded::None), "small" => button.rounded(ButtonRounded::Small), "large" => button.rounded(ButtonRounded::Large), _ => button};
                if let Some(icon) = icon_name(option("loadingIcon")) {button = button.loading_icon(icon);}
                if flag("spinner") {button = button.icon(gpui_kit::component::spinner::Spinner::new());}
                if let Ok(value) = option("progress").parse::<f32>() {button = button.icon(gpui_kit::component::progress::ProgressCircle::new(ElementId::Name(format!("{id}-progress").into())).value(value));}
                if !option("toggled").is_empty() {button = button.toggled(flag("toggled"));}
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
                button.on_click(cx.listener(move |this, _, _, cx| {this.activate(id, cx);}))
    }

    fn element(&self, node: &Node, theme: &Theme, mode: &str, cx: &mut Context<Self>) -> AnyElement {
        let (theme, mode) = node.theme.as_ref().map_or((theme, mode), |(theme, mode)| (theme, mode.as_str()));
        let id = node.identity.get();
        if node.editable.is_some() {
            let control = match self.inputs.get(&id) {
                Some((Field::Line(state), _)) => input_options!(Input::new(state), node, true, self, cx).into_any_element(),
                Some((Field::Text(state), _)) => input_options!(Textarea::new(state), node, false, self, cx).into_any_element(),
                None => return div().into_any_element(),
            };
            // Like a browser input, an unsized field keeps a usable intrinsic width.
            return styled_rules(div().min_w(px(240.)), self.rules(node), node.is_disabled(), node.identity.get()).child(control).into_any_element();
        }
        if let Some(kit) = node.kit.as_deref().filter(|kit| !kit.name.is_empty()) {
            let element = self.kit_element(node, kit, cx);
            if node.class_name.is_empty() || STYLED_KIT.contains(&kit.name.as_str()) {return element;}
            // The class wrapper can shrink in flex layouts, so scroll areas inside it scroll.
            return styled_rules(div().min_h_0().min_w_0(), self.rules(node), node.is_disabled(), node.identity.get()).child(element).into_any_element();
        }
        let children = node.visible_children().map(|child| self.element(child, theme, mode, cx)).collect::<Vec<_>>();
        if let Some(copy) = &node.copy {
            return Clipboard::new(ElementId::Integer(id)).value(SharedString::from(copy.value.clone())).into_any_element();
        }
        match node.kind {
            Kind::Button | Kind::DisabledButton if node.class_name.is_empty() && !node.semantics.href.is_empty() => {
                gpui_kit::component::link::Link::new(ElementId::Integer(id)).disabled(node.is_disabled()).child(SharedString::from(node.content_text()))
                    .on_click(cx.listener(move |this, _, _, cx| {this.activate(id, cx);})).into_any_element()
            }
            Kind::Button | Kind::DisabledButton if node.class_name.is_empty() => self.kit_button(node, cx).into_any_element(),
            Kind::Button | Kind::DisabledButton => {
                let option=|name:&str|node.option(name).unwrap_or_default();
                let mut button=gpui_kit::base::Button::new(ElementId::Integer(id)).cursor_pointer().disabled(node.is_disabled()).selected(option("selected")=="true");
                let label=if node.semantics.label.is_empty(){node.content_text()}else{node.semantics.label.clone()};
                if !label.is_empty() {button=button.accessibility_label(label);}
                if !node.semantics.id.is_empty() {button=button.accessibility_id(node.semantics.id.clone());}
                button=match option("role") {"link"=>button.role(Role::Link),"menuitem"=>button.role(Role::MenuItem),"tab"=>button.role(Role::Tab),"checkbox"=>button.role(Role::CheckBox),"switch"=>button.role(Role::Switch),"none"=>button.role(None::<Role>),_ if !node.semantics.href.is_empty()=>button.role(Role::Link),_=>button};
                if let Ok(index)=option("tabIndex").parse::<isize>() {button=button.tab_index(index);}
                if option("tabStop")=="false" {button=button.tab_stop(false);}
                if !option("toggled").is_empty() {button=button.aria_toggled(if option("toggled")=="true" {gpui_kit::accesskit::Toggled::True}else{gpui_kit::accesskit::Toggled::False});}
                if let Some(icon)=icon_name(option("icon")) {button=button.child(Icon::new(icon));}
                if !option("tooltip").is_empty() {let text=option("tooltip").to_owned();button=button.tooltip(move|window,cx|gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window,cx));}
                if node.kit.as_deref().is_some_and(|kit|kit.handles("hover")) {
                    let root=cx.entity().downgrade();button=button.on_hover(move|hovered,_,cx|{let value=hovered.to_string();let _=root.update(cx,|root,cx|root.emit(id,"hover",value,cx));});
                }
                let loading=option("loading")=="true";
                styled_control(button,self.rules(node),node.is_disabled()).children(children)
                    .on_click(cx.listener(move|this,_,_,cx|{if !loading {this.activate(id,cx);}})).into_any_element()
            }
            // Vector art draws as an SVG image with colors resolved for this theme.
            Kind::Svg => {
                let (width, height) = node.svg_size().unwrap_or((24., 24.));
                let current = style::state_rules(self.rules(node), State::Base, false).iter().find_map(|rule| match rule {Utility::Foreground(value) => Some(*value), _ => None}).or_else(|| theme.color(mode, "foreground").ok());
                let base = styled_rules(div().w(px(width as f32)).h(px(height as f32)).flex_shrink_0(), self.rules(node), false, node.identity.get());
                match current.and_then(|current| node.svg_markup(current, theme, mode)) {
                    Some(markup) => base.child(img(std::sync::Arc::new(Image::from_bytes(ImageFormat::Svg, markup.into_bytes()))).size_full()).into_any_element(),
                    None => base.into_any_element(),
                }
            }
            // Text is one inline run; its literal and bound parts render as one string.
            Kind::Text => styled_rules(div(), self.rules(node), false, node.identity.get()).child(SharedString::from(node.content_text())).into_any_element(),
            _ if children.is_empty() => styled_rules(div(), self.rules(node), false, node.identity.get()).child(SharedString::from(node.text.clone())).into_any_element(),
            _ => {
                // GPUI keeps scroll offsets per element id; overflow-auto views need one.
                // Their children keep their natural size instead of shrinking to fit.
                let scrolls = style::state_rules(self.rules(node), State::Base, false).iter().any(|rule| matches!(rule, Utility::Overflow(2) | Utility::OverflowAxis(_,2)));
                if !scrolls {return styled_rules(div(), self.rules(node), false, node.identity.get()).children(children).into_any_element();}
                let children = children.into_iter().map(|child| div().flex_shrink_0().child(child));
                styled_rules(div(), self.rules(node), false, node.identity.get()).children(children).into_any_element()
            }
        }
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _scope=self.scope();
        self.viewport=(f32::from(window.viewport_size().width),f32::from(window.viewport_size().height));
        let revision=super::state::revision();
        if self.revision != revision {
            if let Err(error)=self.node.refresh() {fail(&self.failure,error);}
            self.revision=revision;
        }
        self.node.ensure_identities();
        self.sync_inputs(window, cx);
        self.sync_kit(window, cx);
        self.run_commands(window, cx);
        let theme = utility_theme(cx);
        self.styles.clear();
        collect_styles(&self.node, &mut Vec::new(), &theme, mode(cx), &Default::default(), self.viewport, &mut self.styles);
        let body = self.node.visible_root().map(|node| self.element(node, &theme, mode(cx), cx));
        if window.focused(cx).is_none() {self.focus.focus(window, cx);}
        div().size_full().track_focus(&self.focus).on_action(cx.listener(|this, command: &TsrustCommand, _, cx| this.run_command(&command.name, cx))).children(body)
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

fn styled_rules(element: Div, rules: &[(State, Utility)], disabled: bool, id:u64) -> Stateful<Div> {
    styled_control(element.id(ElementId::Integer(id)),rules,disabled)
}

fn styled_control<T:Styled+StatefulInteractiveElement>(mut element:T,rules:&[(State,Utility)],disabled:bool)->T {
    let flex = rules.iter().any(|(_, rule)| matches!(rule, Utility::Row | Utility::Column | Utility::Display(false) | Utility::InlineFlex));
    if flex {element = element.flex();}
    for (state, rule) in rules {
        if *state == State::Base || *state == State::Disabled && disabled {apply(element.style(), *rule);}
    }
    apply_effects(element.style(), &style::state_rules(rules,State::Base,disabled));
    if !disabled {
        for state in [State::Hover,State::Focus,State::Active] {
            let local=rules.iter().filter(|(current,_)|*current==state).map(|(_,rule)|*rule).collect::<Vec<_>>();
            if local.is_empty() {continue;}
            let effects=style::state_rules(rules,state,false);
            let refinement=move |mut style:StyleRefinement| {
                for rule in &local {apply(&mut style,*rule);}
                if local.iter().any(|rule|matches!(rule,Utility::Ring(_) | Utility::RingWidth(_) | Utility::RingShadowColor(_) | Utility::OutlineNone | Utility::Shadow(_) | Utility::ShadowColor(_))) {
                    apply_shadow_effects(&mut style,&effects);
                }
                if local.iter().any(|rule|matches!(rule,Utility::GradientAngle(_) | Utility::GradientStop(_,_))) {apply_gradient_effects(&mut style,&effects);}
                style
            };
            element=match state {State::Hover=>element.hover(refinement),State::Focus=>element.focus(refinement),State::Active=>element.active(refinement),_=>unreachable!()};
        }
    }
    element
}

fn collect_styles<'a>(node: &'a Node, ancestors: &mut Vec<&'a Node>, theme: &Theme, mode: &str, inherited: &std::collections::BTreeMap<String, f32>, viewport: (f32,f32), output: &mut HashMap<u64, Vec<(State, Utility)>>) {
    let (theme, mode) = node.theme.as_ref().map_or((theme, mode), |(theme, mode)| (theme, mode.as_str()));
    let (rules, variables) = style::resolved_scoped_node_at_width(node, ancestors, theme, mode, inherited, viewport.0);
    output.insert(node.identity.get(), rules.into_iter().map(|(state,rule)|(state,rule.in_viewport(viewport.0,viewport.1))).collect());
    ancestors.push(node);
    for child in node.visible_children() {collect_styles(child, ancestors, theme, mode, &variables, viewport, output);}
    ancestors.pop();
}

impl Root {
    fn rules(&self, node: &Node) -> &[(State, Utility)] {self.styles.get(&node.identity.get()).map(Vec::as_slice).unwrap_or_default()}
    fn node_style(&self, node: &Node) -> StyleRefinement {
        let mut surface = styled_rules(div(), self.rules(node), node.is_disabled(), node.identity.get());
        surface.style().clone()
    }
}

fn apply_effects(style: &mut StyleRefinement, rules: &[Utility]) {
    apply_shadow_effects(style,rules);
    apply_gradient_effects(style,rules);
}

fn apply_shadow_effects(style:&mut StyleRefinement,rules:&[Utility]) {
    if !rules.iter().any(|rule|matches!(rule,Utility::Ring(_) | Utility::RingWidth(_) | Utility::RingShadowColor(_) | Utility::OutlineNone | Utility::Shadow(_) | Utility::ShadowColor(_))) {return;}
    style.box_shadow=Some(Vec::new());
    for rule in rules {if matches!(rule,Utility::Shadow(_)) {apply(style,*rule);}}
    let mut ring_width=0.;
    let mut ring_color=hsla(0.6,0.7,0.5,1.);
    for rule in rules {
        match rule {
            Utility::Ring(value)=>{ring_width=2.;ring_color=color(*value);}
            Utility::RingWidth(value)=>ring_width=*value,
            Utility::RingShadowColor(value)=>ring_color=color(*value),
            Utility::OutlineNone=>ring_width=0.,
            Utility::ShadowColor(value)=>{for shadow in style.box_shadow.as_mut().unwrap() {shadow.color=color(*value);}},
            _=>{}
        }
    }
    if ring_width>0. {style.box_shadow.as_mut().unwrap().push(BoxShadow {color:ring_color,offset:point(px(0.),px(0.)),blur_radius:px(0.),spread_radius:px(ring_width),inset:false});}
}

fn apply_gradient_effects(style:&mut StyleRefinement,rules:&[Utility]) {
    let mut angle=None;
    let mut from=hsla(0.,0.,0.,0.);
    let mut to=from;
    for rule in rules {match rule {Utility::GradientAngle(value)=>angle=Some(*value),Utility::GradientStop(0,value)=>from=color(*value),Utility::GradientStop(2,value)=>to=color(*value),_=>{}}}
    if let Some(angle)=angle {style.background=Some(linear_gradient(angle as f32,linear_color_stop(from,0.),linear_color_stop(to,1.)).into());}
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
        Utility::Basis(value) => style.flex_basis = Some(length(value)),
        Utility::MarginAuto(side) => {
            let value = Some(gpui_kit::Length::Auto);
            match side {0 => style.margin.top = value, 1 => style.margin.right = value, 2 => style.margin.bottom = value, _ => style.margin.left = value}
        }
        Utility::Tracks(axis, count) if count < 254 => {
            let value = Some(GridTemplate {repeat: count as u16, min_size: GridTemplateMinSize::Zero});
            if axis == 0 {style.grid_cols = value;} else {style.grid_rows = value;}
        }
        Utility::Placement(property, count) => {
            let location = style.grid_location_mut();
            match property {0 => location.column.start = GridPlacement::Line(count as i16), 1 => location.column.end = GridPlacement::Span(count), 2 => location.row.start = GridPlacement::Line(count as i16), _ => location.row.end = GridPlacement::Span(count)}
        }
        Utility::AlignContent(value) => style.align_content = Some([AlignContent::FlexStart, AlignContent::Center, AlignContent::FlexEnd, AlignContent::SpaceBetween, AlignContent::SpaceAround, AlignContent::SpaceEvenly, AlignContent::Stretch][value.min(6) as usize]),
        Utility::Wrap(wrap) => style.flex_wrap = Some(if wrap {FlexWrap::Wrap} else {FlexWrap::NoWrap}),
        Utility::Items(value) => style.align_items = Some([AlignItems::FlexStart, AlignItems::Center, AlignItems::Stretch, AlignItems::FlexEnd, AlignItems::Baseline][value.min(4) as usize]),
        Utility::Justify(value) => style.justify_content = Some([JustifyContent::FlexStart, JustifyContent::Center, JustifyContent::SpaceBetween, JustifyContent::FlexEnd, JustifyContent::SpaceAround, JustifyContent::SpaceEvenly][value.min(5) as usize]),
        Utility::SelfAlign(0, value) => style.align_self = Some([AlignSelf::FlexStart, AlignSelf::Center, AlignSelf::FlexEnd, AlignSelf::Stretch, AlignSelf::Stretch][value.min(4) as usize]),
        Utility::FontSize(value) => style.text.font_size = Some(px(value).into()),
        Utility::Weight(value) => style.text.font_weight = Some(FontWeight(value)),
        Utility::Italic(value) => style.text.font_style = Some(if value {FontStyle::Italic} else {FontStyle::Normal}),
        Utility::Underline(value) => style.text.underline = value.then(|| UnderlineStyle {thickness: px(1.), ..Default::default()}),
        Utility::LineThrough(value) => style.text.strikethrough = value.then(|| StrikethroughStyle {thickness: px(1.), ..Default::default()}),
        Utility::WhiteSpace(value) => style.text.white_space = Some(if matches!(value, style::WhiteSpace::NoWrap | style::WhiteSpace::Pre) {gpui_kit::WhiteSpace::Nowrap} else {gpui_kit::WhiteSpace::Normal}),
        Utility::TextOverflow(value) => style.text.text_overflow = (value == 1).then(|| TextOverflow::Truncate("…".into())),
        Utility::LineClamp(value) => style.text.line_clamp = (value > 0).then_some(value as usize),
        Utility::Visible(value) => style.visibility = Some(if value {Visibility::Visible} else {Visibility::Hidden}),
        Utility::BorderDashed(value) => style.border_style = Some(if value {BorderStyle::Dashed} else {BorderStyle::Solid}),
        Utility::Cursor(value) => style.mouse_cursor = Some(match value {
            1 => CursorStyle::IBeam, 2 => CursorStyle::OperationNotAllowed, 3 => CursorStyle::PointingHand,
            5 => CursorStyle::OpenHand, 6 => CursorStyle::ClosedHand, 7 => CursorStyle::ClosedHand, 8 => CursorStyle::Crosshair,
            9 => CursorStyle::ResizeColumn, 10 => CursorStyle::ResizeRow, 11 => CursorStyle::ResizeLeftRight, 12 => CursorStyle::ResizeUpDown, 13 => CursorStyle::ContextualMenu, 14 => CursorStyle::DragCopy,
            _ => CursorStyle::Arrow,
        }),
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
        Utility::OverflowAxis(axis, value) => {
            let value = Some([Overflow::Visible, Overflow::Hidden, Overflow::Scroll][value.min(2) as usize]);
            if axis == 0 {style.overflow.x = value;} else {style.overflow.y = value;}
        }
        Utility::Shadow(index) => style.box_shadow = Some(Utility::shadow_layers(index).iter().map(|&(y, blur, spread, alpha)| BoxShadow {
            color: hsla(0., 0., 0., alpha), offset: point(px(0.), px(y)), blur_radius: px(blur), spread_radius: px(spread), inset: false,
        }).collect()),
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
