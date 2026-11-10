// GPUI Kit component arms for `std:ui` Kit nodes (generated wrappers in
// kit.tsr, spec in kit.json). Props arrive as text; item components are read
// from their container's children; stateful components keep one Kit state
// entity per node; overlay content renders through a weak handle to the root.
use gpui_kit::component::accordion::Accordion;
use gpui_kit::component::alert::Alert;
use gpui_kit::component::avatar::{Avatar, AvatarGroup};
use gpui_kit::component::badge::Badge;
use gpui_kit::component::breadcrumb::{Breadcrumb, BreadcrumbItem};
use gpui_kit::component::button::{ButtonVariant, DropdownButton, Toggle};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::collapsible::Collapsible;
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState, ColorSelect};
use gpui_kit::component::description_list::DescriptionList;
use gpui_kit::component::empty::{Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle};
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants};
use gpui_kit::component::hover_card::HoverCard;
use gpui_kit::component::input::NumberInput;
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::label::{HighlightsMatch, Label};
use gpui_kit::component::menu::{AppMenuBar, ContextMenuExt, DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::input::{OtpEvent, OtpInput, OtpState};
use gpui_kit::component::pagination::Pagination;
use gpui_kit::component::popover::Popover;
use gpui_kit::component::progress::{Progress, ProgressCircle};
use gpui_kit::component::radio::{Radio, RadioGroup};
use gpui_kit::component::rating::Rating;
use gpui_kit::component::resizable::{h_resizable, resizable_panel, v_resizable};
use gpui_kit::component::select::{Select, SelectEvent, SelectGroup, SelectItem, SelectState};
use gpui_kit::component::separator::Separator;
use gpui_kit::component::shimmer::{ShimmerStyle, ShimmerText};
use gpui_kit::component::sidebar::{Sidebar, SidebarCollapsible, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem, SidebarToggleButton};
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::slider::{Slider, SliderEvent, SliderState, SliderValue};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::stepper::{Stepper, StepperItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tab::{Tab, TabBar, TabVariant};
use gpui_kit::component::table::{Column, ColumnSort, DataTable, Table, TableBody, TableCell, TableDelegate, TableEvent, TableHead, TableHeader, TableRow, TableState};
use gpui_kit::component::attachment::{Attachment, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentStatus, AttachmentTitle};
use gpui_kit::component::diff::{Diff, DiffFile, DiffMode, DiffState, DiffHoverHighlight, DiffHunkSeparator, DiffChangeIndicator, DiffInlineUnit, DiffLinePosition, DiffLineRange, DiffSide, DiffParseError, DiffAnnotation, DiffDecoration, DiffGutterMarker};
use gpui_kit::component::input::{InputGroup, InputGroupAddon, InputGroupAddonAlignment, InputGroupButton, InputGroupText};
use gpui_kit::component::marker::{Marker, MarkerAlignment, MarkerContent, MarkerIcon, MarkerLoadingStyle, MarkerVariant};
use gpui_kit::component::message_scroller::{MessageScroller, MessageScrollerState};
use gpui_kit::component::dock::{BasePanel, DockArea, DockLayout, DockSkin, Panel as DockPanelView, PanelEvent, panel_handle};
use gpui_kit::component::questionnaire::{Questionnaire, QuestionnaireActions, QuestionnaireChoice, QuestionnaireChoiceDefinition, QuestionnaireChoices, QuestionnaireDescription, QuestionnaireError, QuestionnaireEvent, QuestionnaireItem, QuestionnaireItemDefinition, QuestionnaireNext, QuestionnairePrevious, QuestionnaireProgress, QuestionnaireSkip, QuestionnaireState, QuestionnaireSubmit, QuestionnaireTitle};
use gpui_kit::component::setting::{RenderOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings};
use gpui_kit::component::tag::Tag;
use gpui_kit::component::ColorName;
use gpui_kit::component::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::bubble::{Bubble, BubbleContent, BubbleGroup, BubbleReactionSide, BubbleReactions, BubbleVariant};
use gpui_kit::component::button::{ButtonGroup, ToggleGroup};
use gpui_kit::component::calendar::{Calendar, CalendarEvent, CalendarState, Date, Matcher};
use gpui_kit::component::carousel::{Carousel, CarouselContent, CarouselEvent, CarouselItem, CarouselNext, CarouselPrevious, CarouselState};
use gpui_kit::component::chart::{AreaChart, BarChart, CandlestickChart, LineChart, PieChart, RadarChart, SankeyChart};
use gpui_kit::component::plot::shape::{Area, Bar, Line, SankeyLink};
use gpui_kit::component::plot::{AxisText, Curve, Grid as PlotGrid, Plot as KitPlot, PlotAxis, PlotElement, axis_gutter};
use gpui_kit::component::speech::{SpeechButton, SpeechEvent, SpeechState, SpeechWaveform};
// The system recognizer exists only with Kit speech, selected for programs that use SpeechButton.
#[cfg(feature = "gpui-speech")]
use gpui_kit::component::speech::SystemRecognizer;
use gpui_kit::component::combobox::{Combobox, ComboboxEvent, ComboboxState};
use gpui_kit::component::command::{Command as CommandPalette, CommandGroup, CommandItem, CommandState};
use gpui_kit::component::form::{field, h_form, v_form};
use gpui_kit::component::input::{Editor, EditorState};
use gpui_kit::component::list::{List, ListDelegate, ListEvent, ListItem, ListState};
use gpui_kit::component::message::{Message, MessageAlignment, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::searchable_list::SearchableVec;
use gpui_kit::component::status_bar::StatusBar;
use gpui_kit::component::text::TextView;
use gpui_kit::component::time_field::{HourCycle, TimeField, TimeFieldEvent, TimeFieldState, TimePrecision};
use gpui_kit::component::toolbar::{Toolbar, ToolbarGroup};
use gpui_kit::component::tree::{TreeEvent, TreeItem, TreeState, tree};
use gpui_kit::component::{IndexPath, ThemeMode, TitleBar};
use gpui_kit::base::component_traits::Collapsible as _;
use gpui_kit::component::Theme as KitTheme;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::{Icon, Placement, Selectable, Sizable, Size as KitSize, StyledExt, WindowExt};
use super::commands::{self, Command};

/// One List row: a ListItem (or ListSeparator) node with its searchable text.
#[derive(Clone)]
struct ListRow {id: u64, text: String, separator: bool, disabled: bool}

/// A List's delegate: rows come from ListItem children, sections from ListGroup children. Root syncs the rows;
/// each row renders its own children through Root.
struct TsrustList {
    root: WeakEntity<Root>,
    list: u64,
    sections: Vec<(String, Vec<ListRow>)>,
    query: String,
    empty: Option<u64>,
    loading: bool,
    has_more: bool,
    threshold: usize,
    selected: Option<IndexPath>,
    values: HashMap<u64, String>,
}

impl TsrustList {
    /// The rows in a section that match the search query (separators hide while searching).
    fn visible(&self, section: usize) -> Vec<&ListRow> {
        let query = self.query.to_lowercase();
        self.sections.get(section).map(|(_, rows)| rows.iter().filter(|row| if query.is_empty() {true} else {!row.separator && row.text.to_lowercase().contains(&query)}).collect()).unwrap_or_default()
    }
    fn row(&self, index: IndexPath) -> Option<ListRow> {self.visible(index.section).get(index.row).map(|row| (*row).clone())}
}

/// A List node's sections: loose rows first, then one per ListGroup (heading, rows).
fn list_sections(node: &Node) -> Vec<(String, Vec<ListRow>)> {
    let row = |child: &Node, kit: &Kit| {
        let keywords = kit.text("keywords").replace('\n', " ");
        ListRow {id: child.identity.get(), text: format!("{} {} {keywords}", child.content_text(), kit.text("value")), separator: kit.name == "ListSeparator", disabled: kit.flag("disabled")}
    };
    let mut sections = vec![(String::new(), Vec::new())];
    for (child, kit) in node.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))) {
        match kit.name.as_str() {
            "ListItem" | "ListSeparator" => sections[0].1.push(row(child, kit)),
            "ListGroup" => sections.push((kit.text("heading"), child.visible_children().filter_map(|item| item.kit.as_deref().filter(|kit| matches!(kit.name.as_str(), "ListItem" | "ListSeparator")).map(|kit| row(item, kit))).collect())),
            _ => {}
        }
    }
    // Kit measures one header height from section 0, so loose rows only form a section when present.
    if sections.len() > 1 && sections[0].1.is_empty() {sections.remove(0);}
    sections
}

/// Row `value` props by row identity.
fn list_values(node: &Node) -> HashMap<u64, String> {
    let mut values = HashMap::new();
    fn walk(node: &Node, values: &mut HashMap<u64, String>) {
        for child in node.visible_children() {
            match child.kit.as_deref() {
                Some(kit) if kit.name == "ListItem" => {if let Some(value) = kit.prop("value") {values.insert(child.identity.get(), value.to_owned());}}
                Some(kit) if kit.name == "ListGroup" => walk(child, values),
                _ => {}
            }
        }
    }
    walk(node, &mut values);
    values
}

/// The IndexPath of the flat row index (counting every section's rows in order).
fn list_path(sections: &[(String, Vec<ListRow>)], mut index: usize) -> Option<IndexPath> {
    for (section, (_, rows)) in sections.iter().enumerate() {
        if index < rows.len() {return Some(IndexPath::new(index).section(section));}
        index -= rows.len();
    }
    None
}

/// What List events report for a row: its `value` prop, else its flat index.
fn list_value(list: &TsrustList, index: IndexPath) -> String {
    let Some(row) = list.row(index) else {return String::new()};
    let flat = list.sections.iter().take(index.section).map(|(_, rows)| rows.len()).sum::<usize>() + list.sections[index.section].1.iter().position(|other| other.id == row.id).unwrap_or(0);
    list.values.get(&row.id).cloned().unwrap_or_else(|| flat.to_string())
}

impl ListDelegate for TsrustList {
    type Item = ListItem;
    fn perform_search(&mut self, query: &str, _: &mut Window, cx: &mut Context<ListState<Self>>) -> Task<()> {
        self.query = query.to_owned();
        cx.notify();
        Task::ready(())
    }
    fn sections_count(&self, _: &App) -> usize {self.sections.len().max(1)}
    fn items_count(&self, section: usize, _: &App) -> usize {self.visible(section).len()}
    fn render_item(&mut self, index: IndexPath, _: &mut Window, cx: &mut Context<ListState<Self>>) -> Option<ListItem> {
        let row = self.row(index)?;
        self.root.update(cx, |root, cx| root.list_item(row.id, cx)).ok().flatten()
    }
    fn render_section_header(&mut self, section: usize, _: &mut Window, cx: &mut Context<ListState<Self>>) -> Option<impl IntoElement> {
        // Kit sizes every header like section 0's, so all sections get one once any group has a heading.
        if self.sections.iter().all(|(heading, _)| heading.is_empty()) {return None;}
        let heading = self.sections.get(section).map(|(heading, _)| heading.clone()).filter(|heading| !heading.is_empty()).unwrap_or_else(|| "\u{a0}".into());
        Some(div().px_3().pt_2().pb_1().text_xs().font_weight(FontWeight::MEDIUM).text_color(cx.theme().muted_foreground).child(heading))
    }
    fn render_empty(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) -> impl IntoElement {
        match self.empty {
            Some(empty) => div().size_full().flex().items_center().justify_center().children(self.root.update(cx, |root, cx| root.content_of(empty, cx)).unwrap_or_default()).into_any_element(),
            None => div().size_full().flex().items_center().justify_center().text_color(cx.theme().muted_foreground.opacity(0.6)).child(Icon::new(IconName::Inbox).size_12()).into_any_element(),
        }
    }
    fn loading(&self, _: &App) -> bool {self.loading}
    fn set_selected_index(&mut self, index: Option<IndexPath>, _: &mut Window, cx: &mut Context<ListState<Self>>) {self.selected = index; cx.notify();}
    fn has_more(&self, _: &App) -> bool {self.has_more}
    fn load_more_threshold(&self) -> usize {self.threshold}
    fn load_more(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) {
        // Deferred: load_more runs inside the List's render, so Root is reached after it.
        let (root, list) = (self.root.clone(), self.list);
        cx.defer(move |cx| {let _ = root.update(cx, |root, cx| root.emit(list, "loadMore", String::new(), cx));});
    }
}

enum KitState {
    /// A Select's state and the option shape it was built from; grouped options use SelectGroups.
    Select(Entity<SelectState<SearchableVec<Choice>>>, String),
    SelectGroups(Entity<SelectState<SearchableVec<SelectGroup<Choice>>>>, String),
    Number(Entity<InputState>),
    Otp(Entity<OtpState>),
    Date(Entity<DatePickerState>),
    Color(Entity<ColorPickerState>),
    Slider(Entity<SliderState>),
    Calendar(Entity<CalendarState>),
    Time(Entity<TimeFieldState>),
    /// A Combobox's state and option shape; grouped options use ComboboxGroups.
    Combobox(Entity<ComboboxState<SearchableVec<Choice>>>, String),
    ComboboxGroups(Entity<ComboboxState<SearchableVec<SelectGroup<Choice>>>>, String),
    Command(Entity<CommandState>),
    AppMenuBar(Entity<AppMenuBar>, Vec<ApplicationMenu>),
    Carousel(Entity<CarouselState>),
    Editor(Entity<EditorState>, std::collections::BTreeMap<String,String>),
    Tree(Entity<TreeState>, String),
    List(Entity<ListState<TsrustList>>),
    Diff(Entity<DiffState>, String),
    Table(Entity<TableState<Grid>>, String),
    Scroller(Entity<MessageScrollerState>, usize),
    Dock(Entity<DockArea>),
    Questionnaire(Entity<QuestionnaireState>),
    Speech(Entity<SpeechState>),
    /// A running Ticker's timer task (dropping it stops the ticks) and the props it was started with.
    Ticker(Option<Task<()>>, String),
}

/// One Plot mark: a line, area or bar series over the Plot's labels.
struct Mark {label: String, kind: &'static str, values: Vec<f64>, color: Hsla, curve: Curve, dots: bool, width: f32}

/// A Plot: marks share one band x scale over the labels and one linear y scale.
struct PlotTipRow {label:String,value:String,color:Option<Hsla>,value_color:Option<Hsla>}
struct PlotTip {glide:bool,progress:Option<f32>,title:Option<String>,series:bool,gap:f32,cross:String,dots:bool,appearance:bool,rows:Vec<PlotTipRow>,style:StyleRefinement}
struct Marks {id:u64,tooltip:Option<PlotTip>,labels: Vec<SharedString>, marks: Vec<Mark>, axis: bool, grid: bool, border: Hsla, muted: Hsla}

impl IntoElement for Marks {
    type Element = PlotElement<Self>;
    fn into_element(self) -> Self::Element {PlotElement::new(self)}
}

impl KitPlot for Marks {
    fn id(&self)->Option<ElementId> {self.tooltip.as_ref().map(|_|ElementId::Name(format!("plot-{}",self.id).into()))}
    fn tooltip_state(&self,position:Point<Pixels>,bounds:Bounds<Pixels>,_:&App)->Option<gpui_kit::component::plot::TooltipState> {
        let count=self.labels.len();
        let width=bounds.size.width.as_f32();
        let height=(bounds.size.height.as_f32()-if self.axis {axis_gutter(px(12.))}else{0.}).max(0.);
        if count==0 || width<=0. || height<=0. || position.y.as_f32()>=height {return None;}
        let step=width/count as f32;
        let index=super::kit::plot_index(position.x.as_f32(),width,count)?;
        let x=step*(index as f32+0.5);
        let (low,high)=self.marks.iter().flat_map(|mark|mark.values.iter().copied()).fold((0f64,0f64),|(low,high),value|(low.min(value),high.max(value)));
        let bar_count=self.marks.iter().filter(|mark|mark.kind=="bars").count();
        let mut bar_index=0;
        let dots=self.marks.iter().filter_map(|mark| {
            let x=if mark.kind=="bars" {let x=super::kit::plot_bar_x(x,step,bar_index,bar_count);bar_index+=1;x}else{x};
            mark.values.get(index).map(|value|point(px(x),px(super::kit::plot_y(*value,low,high,height))))
        }).collect();
        Some(gpui_kit::component::plot::TooltipState::new(index,point(px(x),position.y),dots))
    }
    fn tooltip(&self,state:&gpui_kit::component::plot::TooltipState,cursor:Point<Pixels>,bounds:Bounds<Pixels>,_:&mut Window,_:&mut App)->Option<AnyElement> {
        use gpui_kit::component::plot::tooltip::{Tooltip as PlotTooltip,CrossLine,Dot};
        let options=self.tooltip.as_ref()?;
        let mut tooltip=PlotTooltip::new(cursor,bounds.size).glide(options.glide).gap(px(options.gap)).appearance(options.appearance).refine_style(&options.style);
        if let Some(progress)=options.progress {tooltip=tooltip.progress(progress);}
        let title=options.title.as_ref().map(String::as_str).unwrap_or_else(||self.labels.get(state.index).map_or("",SharedString::as_ref));
        if !title.is_empty() {tooltip=tooltip.title(title.to_owned());}
        if options.series {for mark in &self.marks {if let Some(value)=mark.values.get(state.index) {tooltip=tooltip.row(mark.color,mark.label.clone(),value.to_string());}}}
        for row in &options.rows {
            tooltip=match row.color {Some(color)=>tooltip.row(color,row.label.clone(),row.value.clone()),None=>tooltip.plain_row(row.label.clone(),row.value.clone())};
            if let Some(color)=row.value_color {tooltip=tooltip.value_color(color);}
        }
        let cross=CrossLine::new(state.cross_line);
        tooltip=match options.cross.as_str() {"none"=>tooltip,"horizontal"=>tooltip.cross_line(cross.horizontal()),"both"=>tooltip.cross_line(cross.both()),_=>tooltip.cross_line(cross)};
        if options.dots {
            let colors=self.marks.iter().filter(|mark|mark.values.get(state.index).is_some()).map(|mark|mark.color);
            tooltip=tooltip.dots(state.dots.iter().zip(colors).map(|(point,color)|Dot::new(*point).fill(color).stroke(color)));
        }
        Some(tooltip.into_any_element())
    }
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let count = self.labels.len();
        if count == 0 {return;}
        let width = bounds.size.width.as_f32();
        let height = (bounds.size.height.as_f32() - if self.axis {axis_gutter(px(12.))} else {0.}).max(0.);
        let (low, high) = self.marks.iter().flat_map(|mark| mark.values.iter().copied()).fold((0f64, 0f64), |(low, high), value| (low.min(value), high.max(value)));
        let y = move |value: f64| super::kit::plot_y(value,low,high,height);
        let step = width / count as f32;
        let center = move |index: usize| step * (index as f32 + 0.5);
        if self.grid {
            PlotGrid::new().y((0..=4).map(|level| px(height * level as f32 / 4.))).stroke(self.border).dash_array(&[px(4.), px(2.)]).paint(&bounds, window);
        }
        let bar_series = self.marks.iter().filter(|mark| mark.kind == "bars").count().max(1);
        let mut bar_index = 0;
        for mark in &self.marks {
            let points = (0..count.min(mark.values.len())).map(|index| (center(index), y(mark.values[index]))).collect::<Vec<_>>();
            match mark.kind {
                "area" => Area::new().data(points).x(|point: &(f32, f32)| Some(point.0)).y0(y(low.max(0.).min(high))).y1(|point: &(f32, f32)| Some(point.1))
                    .fill(mark.color.opacity(0.3)).stroke(mark.color).curve(mark.curve).paint(&bounds, window),
                "bars" => {
                    let band = step * 0.7 / bar_series as f32;
                    let offset = super::kit::plot_bar_x(0.,step,bar_index,bar_series)-band/2.;
                    let (base, color) = (y(0f64.max(low).min(high)), mark.color);
                    Bar::new().data(points).band_width(band).cross(move |point: &(f32, f32)| Some(point.0 + offset)).base(move |_| base).value(|point: &(f32, f32)| Some(point.1))
                        .fill(move |_, _, _| color).paint(&bounds, window, cx);
                    bar_index += 1;
                }
                _ => {
                    let mut line = Line::new().data(points).x(|point: &(f32, f32)| Some(point.0)).y(|point: &(f32, f32)| Some(point.1)).stroke(mark.color).stroke_width(mark.width).curve(mark.curve);
                    if mark.dots {line = line.dot().dot_size(8.).dot_fill(mark.color);}
                    line.paint(&bounds, window);
                }
            }
        }
        if self.axis {
            let labels = self.labels.iter().enumerate().map(|(index, label)| AxisText::new(label.clone(), center(index), self.muted).font_size(px(12.))).collect::<Vec<_>>();
            PlotAxis::new().stroke(self.border).x(height).x_label(labels).paint(&bounds, window, cx);
        }
    }
}

/// The std:ui palette (light or dark) matching GPUI Kit's active theme, so
/// utility colors such as text-muted-foreground follow dark mode.
fn mode(cx: &App) -> &'static str {if cx.theme().mode.is_dark() {"dark"} else {"light"}}

/// The std:ui utility palette built from the active GPUI Kit theme, so classes
/// such as text-muted-foreground and border-border follow any Kit theme. Cached
/// until the Kit colors change.
fn utility_theme(cx: &App) -> Theme {
    thread_local! {static CACHE: std::cell::RefCell<Option<(String, Theme)>> = const {std::cell::RefCell::new(None)};}
    let kit = cx.theme();
    let colors = [
        ("background", kit.background), ("foreground", kit.foreground), ("card", kit.group_box), ("cardForeground", kit.group_box_foreground),
        ("primary", kit.primary), ("primaryForeground", kit.primary_foreground), ("secondary", kit.secondary), ("secondaryForeground", kit.secondary_foreground),
        ("muted", kit.muted), ("mutedForeground", kit.muted_foreground), ("accent", kit.accent), ("accentForeground", kit.accent_foreground),
        ("destructive", kit.danger), ("destructiveForeground", kit.danger_foreground), ("border", kit.border), ("input", kit.input), ("ring", kit.ring),
    ].map(|(name, color)| (name, hex(color)));
    let key = colors.iter().map(|(_, color)| color.as_str()).collect::<String>();
    CACHE.with(|cache| {
        if let Some((cached, theme)) = cache.borrow().as_ref() && *cached == key {return theme.clone();}
        let mut config: serde_json::Value = serde_json::from_str(super::theme::DEFAULT_JSON).expect("bundled theme");
        for mode in ["light", "dark"] {for (name, color) in &colors {config[mode][*name] = color.clone().into();}}
        let theme = Theme::parse(&config.to_string()).unwrap_or_default();
        *cache.borrow_mut() = Some((key, theme.clone()));
        theme
    })
}

/// The one GPUI action behind every Shortcut: it carries the command name, so
/// key bindings, tooltips and menus can refer to a command by name.
#[derive(Action, Clone, PartialEq, serde::Deserialize)]
#[action(namespace = tsrust, no_json)]
struct TsrustCommand {name: String}

// Bind a Shortcut's keys once per key and command pair.
thread_local! {
    /// The keys most recently bound to each Shortcut command, for Kbd command lookup.
    static COMMAND_KEYS: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

fn bind_shortcut(keys: &str, command: &str, cx: &mut App) {
    thread_local! {static BOUND: std::cell::RefCell<std::collections::HashSet<(String, String)>> = std::cell::RefCell::new(Default::default());}
    if keys.is_empty() || command.is_empty() || !BOUND.with(|bound| bound.borrow_mut().insert((keys.to_owned(), command.to_owned()))) {return;}
    match KeyBinding::load(&platform_keys(keys), Box::new(TsrustCommand {name: command.to_owned()}), None, false, None, cx) {
        Ok(binding) => {COMMAND_KEYS.with(|bound| bound.borrow_mut().insert(command.to_owned(), keys.to_owned())); cx.bind_keys([binding])}
        Err(error) => eprintln!("Shortcut {command}: invalid keys {keys}: {error:?}"),
    }
}

/// Keystrokes with `secondary` resolved: cmd on macOS, ctrl elsewhere.
fn platform_keys(keys: &str) -> String {
    keys.split(' ').map(|chord| chord.split('-').map(|part| if part == "secondary" {if cfg!(target_os = "macos") {"cmd"} else {"ctrl"}} else {part}).collect::<Vec<_>>().join("-")).collect::<Vec<_>>().join(" ")
}

/// Menu children that describe entries rather than render content.
const MENU_PARTS: &[&str] = &["MenuItem", "MenuLabel", "MenuSeparator", "Submenu"];

/// A menu entry captured from MenuItem, MenuLabel, MenuSeparator and Submenu nodes,
/// so menu builders (which run later, per open) need no access to the tree.
#[derive(Clone, PartialEq)]
enum MenuEntry {
    Item {id: u64, label: String, icon: Option<IconName>, disabled: bool, checked: bool, command: String, href: String, custom: bool, handles: bool},
    Label(String),
    Separator,
    Submenu {label: String, icon: Option<IconName>, entries: Vec<MenuEntry>},
}

#[derive(Clone, PartialEq)]
struct ApplicationMenu {label: String, disabled: bool, entries: Vec<MenuEntry>}

fn editor_options(state:&mut EditorState, options:&std::collections::BTreeMap<String,String>, window:&mut Window, cx:&mut Context<EditorState>) {
    let text=|key:&str|options.get(key).cloned().unwrap_or_default();
    let flag=|key:&str,default:bool|options.get(key).map_or(default,|value|value=="true");
    apply_input_options!(state,options,text("placeholder"),window,cx);
    state.set_disabled(flag("disabled",false),cx);
    state.set_readonly(flag("readOnly",false),cx);
    state.set_line_number(flag("lineNumbers",true),window,cx);
    state.set_folding(flag("folding",true),window,cx);
    state.set_auto_close(flag("autoClose",true),window,cx);
    state.set_smart_indent(flag("smartIndent",true),window,cx);
    state.set_indent_guides(flag("indentGuides",true),window,cx);
    if let Some(value)=options.get("softWrap") {state.set_soft_wrap(value=="true",window,cx);}
    if let Some(value)=options.get("searchable") {state.set_searchable(value=="true",cx);}
    if let Some(value)=options.get("scrollBounce") {state.set_scroll_bounce(value=="true",cx);}
    if let Some(value)=options.get("wrappingIndent") {state.set_wrapping_indent(if value=="none" {WrappingIndent::None}else{WrappingIndent::Same},window,cx);}
    if options.contains_key("tabSize") || options.contains_key("hardTabs") {state.set_tab_size(TabSize{tab_size:options.get("tabSize").and_then(|value|value.parse::<usize>().ok()).unwrap_or(2),hard_tabs:flag("hardTabs",false)},cx);}
}

fn application_menus(node: &Node) -> Vec<ApplicationMenu> {
    items(node, "AppMenu").map(|(child, kit)| ApplicationMenu {label: kit.text("label"), disabled: kit.flag("disabled"), entries: menu_entries(child)}).collect()
}

fn install_application_menus(menus: &[ApplicationMenu], cx: &mut App) -> Result<(), String> {
    fn entries(values: &[MenuEntry], parent_disabled:bool) -> Result<Vec<gpui_kit::MenuItem>, String> {
        values.iter().map(|entry| Ok(match entry {
            MenuEntry::Separator => gpui_kit::MenuItem::separator(),
            MenuEntry::Label(label) => gpui_kit::MenuItem::action(label.clone(), NoAction).disabled(true),
            MenuEntry::Submenu {label, entries: children, ..} => gpui_kit::MenuItem::submenu(gpui_kit::Menu::new(label.clone()).items(entries(children,parent_disabled)?)),
            MenuEntry::Item {label, disabled, checked, command, custom, handles, href, ..} => {
                if *custom || *handles || !href.is_empty() {
                    return Err("AppMenuBar items require text and a Shortcut command; custom rows, onSelect and href belong in popup menus".into());
                }
                // Native menu validation uses action availability; NoAction keeps disabled rows inert.
                let disabled=parent_disabled || *disabled || command.is_empty();
                let item=if disabled {gpui_kit::MenuItem::action(label.clone(),NoAction)}else{gpui_kit::MenuItem::action(label.clone(),TsrustCommand {name:command.clone()})};
                item.checked(*checked).disabled(disabled)
            }
        })).collect()
    }
    let menus=menus.iter().map(|menu| Ok(gpui_kit::Menu::new(menu.label.clone()).disabled(menu.disabled).items(entries(&menu.entries,menu.disabled)?))).collect::<Result<Vec<_>, String>>()?;
    if cfg!(target_os="macos") {cx.set_menus(menus);}
    else {gpui_kit::base::GlobalState::global_mut(cx).set_app_menus(menus.into_iter().map(gpui_kit::Menu::owned).collect());}
    Ok(())
}

fn menu_entries(node: &Node) -> Vec<MenuEntry> {
    node.visible_children().filter_map(|child| {
        let kit = child.kit.as_deref()?;
        Some(match kit.name.as_str() {
            "MenuItem" => {
                // Text-only children are the label (<MenuItem>Copy</MenuItem>); other children draw a custom row.
                let plain = child.visible_children().all(|part| part.kind == Kind::Text && part.kit.as_deref().is_none_or(|kit| kit.name.is_empty()));
                let label = if plain && kit.text("label").is_empty() {child.content_text().trim().to_owned()} else {kit.text("label")};
                MenuEntry::Item {id: child.identity.get(), label, icon: icon_name(&kit.text("icon")), disabled: kit.flag("disabled"), checked: kit.flag("checked"),
                    command: kit.text("command"), href: kit.text("href"), custom: !plain, handles: kit.handles("select")}
            }
            "MenuLabel" => MenuEntry::Label(kit.text("text")),
            "MenuSeparator" => MenuEntry::Separator,
            "Submenu" => MenuEntry::Submenu {label: kit.text("label"), icon: icon_name(&kit.text("icon")), entries: menu_entries(child)},
            _ => return None,
        })
    }).collect()
}

/// Menu-wide options from DropdownMenu or ContextMenu props.
#[derive(Clone, Copy)]
struct MenuOptions {min: Option<f32>, max: Option<f32>, height: Option<f32>, scrollable: bool, right: bool, link_icon: bool}

impl MenuOptions {
    fn new(kit: &Kit) -> Self {
        Self {min: kit.number("minWidth").map(|value| value as f32), max: kit.number("maxWidth").map(|value| value as f32), height: kit.number("maxHeight").map(|value| value as f32),
            scrollable: kit.flag("scrollable"), right: kit.prop("checkSide") == Some("right"), link_icon: kit.prop("externalLinkIcon") != Some("false")}
    }
    fn apply(self, mut menu: gpui_kit::component::menu::PopupMenu) -> gpui_kit::component::menu::PopupMenu {
        if let Some(width) = self.min {menu = menu.min_w(px(width));}
        if let Some(width) = self.max {menu = menu.max_w(px(width));}
        if let Some(height) = self.height {menu = menu.max_h(px(height));}
        menu.scrollable(self.scrollable).check_side(if self.right {gpui_kit::component::Side::Right} else {gpui_kit::component::Side::Left}).external_link_icon(self.link_icon)
    }
}

/// A menu built without a menu context: submenus flatten into labelled sections.
fn flat_menu(mut menu: gpui_kit::component::menu::PopupMenu, entries: &[MenuEntry], root: &WeakEntity<Root>) -> gpui_kit::component::menu::PopupMenu {
    for entry in entries {
        menu = match entry.clone() {
            MenuEntry::Separator => menu.separator(),
            MenuEntry::Label(text) => menu.label(text),
            MenuEntry::Submenu {label, entries, ..} => flat_menu(menu.separator().label(label), &entries, root),
            MenuEntry::Item {id, label, icon, disabled, checked, handles, ..} => {
                let mut item = PopupMenuItem::new(label).disabled(disabled).checked(checked);
                if let Some(icon) = icon {item = item.icon(icon);}
                if handles {let root = root.clone(); item = item.on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(id, "select", String::new(), cx));});}
                menu.item(item)
            }
        };
    }
    menu
}

fn build_menu(mut menu: gpui_kit::component::menu::PopupMenu, entries: &[MenuEntry], root: &WeakEntity<Root>, window: &mut Window, cx: &mut Context<gpui_kit::component::menu::PopupMenu>) -> gpui_kit::component::menu::PopupMenu {
    for entry in entries {
        menu = match entry.clone() {
            MenuEntry::Separator => menu.separator(),
            MenuEntry::Label(text) => menu.label(text),
            MenuEntry::Submenu {label, icon, entries} => {
                let root = root.clone();
                menu.submenu_with_icon(icon.map(Icon::new), label, window, cx, move |menu, window, cx| build_menu(menu, &entries, &root, window, cx))
            }
            MenuEntry::Item {id, label, icon, disabled, checked, command, href, custom, handles} => {
                let mut item = if custom {
                    let root = root.clone();
                    PopupMenuItem::element(move |_, cx| root.update(cx, |root, cx| div().children(root.content_of(id, cx))).unwrap_or_else(|_| div()))
                } else if !href.is_empty() {
                    PopupMenuItem::link(label, href)
                } else {
                    PopupMenuItem::new(label)
                };
                if let Some(icon) = icon {item = item.icon(icon);}
                if !command.is_empty() {item = item.action(Box::new(TsrustCommand {name: command}));}
                item = item.disabled(disabled).checked(checked);
                if handles {
                    let root = root.clone();
                    item = item.on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(id, "select", String::new(), cx));});
                }
                menu.item(item)
            }
        };
    }
    menu
}

/// Kit components whose className refines the component itself instead of a wrapper.
pub(crate) const STYLED_KIT: &[&str] = &["CodeEditor", "Tag", "Label", "Kbd", "Separator", "Skeleton", "Progress", "ProgressCircle", "Avatar", "AvatarGroup", "Alert", "ShimmerText", "Empty", "GroupBox", "Table", "List", "Tree", "Carousel", "Message", "MessageGroup", "Bubble", "BubbleGroup", "Marker", "Attachment", "AttachmentGroup"];

/// The size prop shared by sizable Kit components.
/// A checkable control's state: `checked` when the caller controls it, else `defaultChecked`.
fn checked(kit: &Kit) -> bool {kit.prop("checked").map_or(kit.flag("defaultChecked"), |value| value == "true")}

/// A popup anchor corner by name; top-left by default.
fn anchor_named(name: Option<&str>) -> gpui_kit::Anchor {
    match name {Some("top-right") => gpui_kit::Anchor::TopRight, Some("bottom-left") => gpui_kit::Anchor::BottomLeft, Some("bottom-right") => gpui_kit::Anchor::BottomRight, Some("top-center") => gpui_kit::Anchor::TopCenter, Some("bottom-center") => gpui_kit::Anchor::BottomCenter, Some("left-center") => gpui_kit::Anchor::LeftCenter, Some("right-center") => gpui_kit::Anchor::RightCenter, _ => gpui_kit::Anchor::TopLeft}
}

/// DropdownMenu's button options on its trigger, whether a plain or split button.
fn menu_button<T: ButtonVariants + Sizable + Selectable + Disableable>(button: T, kit: &Kit, cx: &App) -> T {
    let mut button = if kit.prop("disabled").is_some() {button.disabled(kit.flag("disabled"))} else {button};
    if kit.prop("selected").is_some() {button = button.selected(kit.flag("selected"));}
    if let Some(size) = kit_size(kit) {button = button.with_size(size);}
    button = match kit.prop("variant") {
        Some("primary") => button.primary(), Some("secondary") => button.secondary(), Some("danger") => button.danger(), Some("warning") => button.warning(),
        Some("success") => button.success(), Some("info") => button.info(), Some("ghost") => button.ghost(), Some("link") => button.link(), Some("text") => button.text(), _ => button,
    };
    if let Some(color) = theme_color(&kit.text("color"), cx) {
        let custom = gpui_kit::component::button::ButtonCustomVariant::new(cx).color(color);
        let custom = match theme_color(&kit.text("foreground"), cx) {Some(color) => custom.foreground(color), None => custom};
        let custom = match theme_color(&kit.text("hoverColor"), cx) {Some(color) => custom.hover(color), None => custom};
        let custom = match theme_color(&kit.text("activeColor"), cx) {Some(color) => custom.active(color), None => custom};
        button = button.custom(custom);
    }
    button
}

/// The props that restart a Ticker.
fn ticker_shape(props: &std::collections::BTreeMap<String, String>) -> String {format!("{}\u{1}{}", props.get("interval").map_or("", String::as_str), props.get("running").map_or("", String::as_str))}

/// Starts a Ticker's timer: every `interval` ms (default 16, about 60 per second) it reports the frame count.
fn ticker(id: u64, props: &std::collections::BTreeMap<String, String>, cx: &mut Context<Root>) -> Option<Task<()>> {
    if props.get("running").map(String::as_str) == Some("false") {return None;}
    let interval = std::time::Duration::from_millis(props.get("interval").and_then(|value| value.parse::<f64>().ok()).map_or(16, |ms| ms.max(1.) as u64));
    Some(cx.spawn(async move |this, cx| {
        let mut frame: u64 = 0;
        loop {
            cx.background_executor().timer(interval).await;
            frame += 1;
            if this.update(cx, |root, cx| root.emit(id, "tick", frame.to_string(), cx)).is_err() {break;}
        }
    }))
}

/// A chat part's alignment: `align="end"` puts it at the trailing edge.
fn message_alignment(kit: &Kit) -> MessageAlignment {if kit.prop("align") == Some("end") {MessageAlignment::End} else {MessageAlignment::Start}}

/// A Bubble's look from its `variant` prop.
fn bubble_variant(kit: &Kit) -> BubbleVariant {
    match kit.prop("variant") {
        Some("secondary") => BubbleVariant::Secondary, Some("muted") => BubbleVariant::Muted, Some("tinted") => BubbleVariant::Tinted,
        Some("outline") => BubbleVariant::Outline, Some("ghost") => BubbleVariant::Ghost, Some("destructive") => BubbleVariant::Destructive, _ => BubbleVariant::Filled,
    }
}

/// Shimmer timing and color from `shimmerDuration` (ms) and `shimmerColor`.
fn shimmer_style(kit: &Kit, cx: &App) -> Option<ShimmerStyle> {
    let (duration, color) = (kit.index("shimmerDuration"), kit.prop("shimmerColor").and_then(|color| theme_color(color, cx)));
    if duration.is_none() && color.is_none() {return None;}
    let mut style = ShimmerStyle::new();
    if let Some(duration) = duration {style = style.duration(std::time::Duration::from_millis(duration as u64));}
    if let Some(color) = color {style = style.highlight_color(color);}
    Some(style)
}

fn kit_size(kit: &Kit) -> Option<KitSize> {
    match kit.prop("size") {Some("xsmall") => Some(KitSize::XSmall), Some("small") => Some(KitSize::Small), Some("large") => Some(KitSize::Large), Some("medium") => Some(KitSize::Medium), _ => None}
}

/// A GPUI Kit theme color by name (primary, danger, magenta, chart-1, muted-foreground, …) or #rrggbb.
fn theme_color(name: &str, cx: &App) -> Option<Hsla> {
    let theme = cx.theme();
    Some(match name.replace('-', "_").as_str() {
        "" => return None,
        "primary" => theme.primary, "primary_foreground" => theme.primary_foreground, "secondary" => theme.secondary, "secondary_foreground" => theme.secondary_foreground,
        "danger" | "destructive" => theme.danger, "danger_foreground" => theme.danger_foreground, "warning" => theme.warning, "warning_foreground" => theme.warning_foreground,
        "success" => theme.success, "success_foreground" => theme.success_foreground, "info" => theme.info, "info_foreground" => theme.info_foreground,
        "foreground" => theme.foreground, "background" => theme.background, "muted" => theme.muted, "muted_foreground" => theme.muted_foreground,
        "accent" => theme.accent, "accent_foreground" => theme.accent_foreground, "border" => theme.border, "ring" => theme.ring, "link" => theme.link,
        "red" => theme.red, "red_light" => theme.red_light, "green" => theme.green, "green_light" => theme.green_light, "blue" => theme.blue, "blue_light" => theme.blue_light,
        "yellow" => theme.yellow, "yellow_light" => theme.yellow_light, "magenta" => theme.magenta, "magenta_light" => theme.magenta_light, "cyan" => theme.cyan, "cyan_light" => theme.cyan_light,
        "chart_1" => theme.chart_1, "chart_2" => theme.chart_2, "chart_3" => theme.chart_3, "chart_4" => theme.chart_4, "chart_5" => theme.chart_5,
        // Palette names (blue-500) and literal colors (#ff6600, oklch(...)) work as in utility classes.
        other => return parse_hex(other).or_else(|| super::theme::fixed_color(name).or_else(|| super::theme::fixed_color(&format!("[{name}]"))).map(color)),
    })
}

/// A Plot color: a theme color name or #rrggbb, else the next chart color.
fn plot_color(name: &str, index: usize, cx: &App) -> Hsla {
    let theme = cx.theme();
    let charts = [theme.chart_1, theme.chart_2, theme.chart_3, theme.chart_4, theme.chart_5];
    // Any theme, palette or literal color a color prop accepts; unset cycles through the chart colors.
    theme_color(name, cx).unwrap_or(charts[index % charts.len()])
}

/// One DockPanel: its content is the node's children, drawn through the root.
struct DockContent {root: WeakEntity<Root>, node: u64, title: SharedString, focus: FocusHandle}
impl EventEmitter<PanelEvent> for DockContent {}
impl Focusable for DockContent {
    fn focus_handle(&self, _: &App) -> FocusHandle {self.focus.clone()}
}
impl BasePanel for DockContent {
    fn panel_name(&self) -> &'static str {"TsrustDockPanel"}
}
impl DockPanelView for DockContent {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {self.title.clone()}
}
impl Render for DockContent {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let node = self.node;
        let content = self.root.update(cx, |root, cx| root.content_of(node, cx)).unwrap_or_default();
        div().size_full().p_2().children(content)
    }
}

/// DataTable rows: text cells shown in `order`, which sorting rearranges.
/// What rebuilds a DataTable's delegate: its rows, columns, parts and load state.
fn grid_shape(node: &Node) -> String {
    let kit = node.kit.as_deref().expect("DataTable");
    let mut out = ["rows", "columns", "sortable", "loading", "hasMore", "loadMoreThreshold"].map(|key| kit.text(key)).join("\u{1}");
    for (child, part) in node.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))) {
        out.push('\u{2}'); out.push_str(&part.name);
        for (key, value) in &part.props {out.push_str(key); out.push('='); out.push_str(value); out.push(';');}
        out.push_str(&child.identity.get().to_string());
    }
    out
}

/// One DataTable column from a DataTableColumn child.
#[derive(Clone, PartialEq)]
struct GridColumn {
    key: SharedString, label: SharedString, width: Option<f32>, min: Option<f32>, max: Option<f32>, sortable: bool, sort: String,
    align: String, fixed: bool, resizable: bool, movable: bool, selectable: bool, padding: Option<f32>,
}

/// A DataTable's delegate: string cells from the `rows` JSON, columns from DataTableColumn children (in display
/// order), a sort order over the rows, and the table's empty, loading, load-more and menu parts.
struct Grid {
    columns: Vec<GridColumn>, rows: Vec<Vec<SharedString>>, order: Vec<usize>,
    root: WeakEntity<Root>, table: u64, empty: Option<u64>, menu: Vec<MenuEntry>, loading: bool, has_more: bool, threshold: usize,
}

impl Grid {
    fn new(node: &Node, root: WeakEntity<Root>) -> Self {
        let kit = node.kit.as_deref().expect("DataTable");
        let mut columns = items(node, "DataTableColumn").enumerate().map(|(index, (_, column))| {
            let label = column.text("label");
            GridColumn {
                key: column.prop("key").map_or_else(|| index.to_string(), str::to_owned).into(), label: label.into(),
                width: column.number("width").map(|value| value as f32), min: column.number("minWidth").map(|value| value as f32), max: column.number("maxWidth").map(|value| value as f32),
                sortable: column.flag("sortable"), sort: column.text("defaultSort"), align: column.text("align"), fixed: column.text("fixed") == "left",
                resizable: column.prop("resizable") != Some("false"), movable: column.prop("movable") != Some("false"), selectable: column.prop("selectable") != Some("false"),
                padding: column.number("padding").map(|value| value as f32),
            }
        }).collect::<Vec<_>>();
        // Without DataTableColumn children, the `columns` prop names the columns.
        if columns.is_empty() {
            columns = kit.text("columns").split('\n').filter(|name| !name.is_empty()).enumerate().map(|(index, name)| GridColumn {
                key: index.to_string().into(), label: name.to_owned().into(), width: None, min: None, max: None, sortable: kit.flag("sortable"), sort: String::new(),
                align: String::new(), fixed: false, resizable: true, movable: true, selectable: true, padding: None,
            }).collect();
        }
        let rows = serde_json::from_str::<Vec<Vec<serde_json::Value>>>(&kit.text("rows")).unwrap_or_default().into_iter()
            .map(|row| row.into_iter().map(|cell| SharedString::from(match cell {serde_json::Value::String(text) => text, serde_json::Value::Null => String::new(), other => other.to_string()})).collect()).collect::<Vec<Vec<_>>>();
        Self {
            order: (0..rows.len()).collect(), columns, rows, root, table: node.identity.get(),
            empty: first_item(node, "DataTableEmpty").map(|empty| empty.identity.get()), menu: first_item(node, "DataTableMenu").map(menu_entries).unwrap_or_default(),
            loading: kit.flag("loading"), has_more: kit.flag("hasMore"), threshold: kit.index("loadMoreThreshold").unwrap_or(20),
        }
    }
    /// The column index into each row for a display column.
    fn source(&self, column: usize) -> usize {
        let key = &self.columns[column].key;
        key.parse::<usize>().ok().filter(|index| *index < self.rows.first().map_or(usize::MAX, Vec::len)).unwrap_or(column)
    }
    fn cell(&self, row: usize, column: usize) -> SharedString {self.order.get(row).and_then(|row| self.rows[*row].get(self.source(column))).cloned().unwrap_or_default()}
    fn emit(&self, event: &'static str, value: String, cx: &mut App) {
        let (root, table) = (self.root.clone(), self.table);
        cx.defer(move |cx| {let _ = root.update(cx, |root, cx| root.emit(table, event, value, cx));});
    }
}

impl TableDelegate for Grid {
    fn columns_count(&self, _: &App) -> usize {self.columns.len()}
    fn rows_count(&self, _: &App) -> usize {self.rows.len()}
    fn column(&self, column: usize, _: &App) -> Column {
        let spec = &self.columns[column];
        let mut entry = Column::new(spec.key.clone(), spec.label.clone()).resizable(spec.resizable).movable(spec.movable).selectable(spec.selectable);
        if spec.sortable {entry = entry.sortable();}
        match spec.sort.as_str() {"ascending" => entry = entry.ascending(), "descending" => entry = entry.descending(), _ => {}}
        match spec.align.as_str() {"center" => entry = entry.text_center(), "right" => entry = entry.text_right(), _ => {}}
        if let Some(width) = spec.width {entry = entry.width(px(width));}
        if let Some(width) = spec.min {entry = entry.min_width(px(width));}
        if let Some(width) = spec.max {entry = entry.max_width(px(width));}
        if spec.fixed {entry = entry.fixed_left();}
        match spec.padding {Some(padding) if padding == 0. => entry = entry.p_0(), Some(padding) => entry = entry.paddings(px(padding)), None => {}}
        entry
    }
    fn perform_sort(&mut self, column: usize, sort: ColumnSort, _: &mut Window, cx: &mut Context<TableState<Self>>) {
        self.order = (0..self.rows.len()).collect();
        let direction = match sort {ColumnSort::Ascending => "ascending", ColumnSort::Descending => "descending", _ => ""};
        self.emit("sort", format!("{}\n{direction}", self.columns[column].key), cx);
        if direction.is_empty() {return;}
        let (rows, source) = (&self.rows, self.source(column));
        // Numbers compare numerically; anything else compares as text.
        self.order.sort_by(|left, right| {
            let (left, right) = (rows[*left].get(source).map(|cell| cell.as_ref()).unwrap_or(""), rows[*right].get(source).map(|cell| cell.as_ref()).unwrap_or(""));
            match (left.trim().parse::<f64>(), right.trim().parse::<f64>()) {(Ok(left), Ok(right)) => left.total_cmp(&right), _ => left.cmp(right)}
        });
        if matches!(sort, ColumnSort::Descending) {self.order.reverse();}
    }
    fn context_menu(&mut self, row: usize, menu: gpui_kit::component::menu::PopupMenu, _: &mut Window, cx: &mut Context<TableState<Self>>) -> gpui_kit::component::menu::PopupMenu {
        if self.menu.is_empty() {return menu;}
        if let Some(index) = self.order.get(row) {self.emit("contextMenu", index.to_string(), cx);}
        flat_menu(menu, &self.menu, &self.root)
    }
    fn render_td(&mut self, row: usize, column: usize, _: &mut Window, _: &mut Context<TableState<Self>>) -> impl IntoElement {self.cell(row, column)}
    fn cell_label(&self, row: usize, column: usize, _: &App) -> Option<SharedString> {Some(self.cell(row, column))}
    fn move_column(&mut self, from: usize, to: usize, _: &mut Window, cx: &mut Context<TableState<Self>>) {
        let column = self.columns.remove(from);
        self.columns.insert(to, column);
        self.emit("columnsChange", self.columns.iter().map(|column| column.key.to_string()).collect::<Vec<_>>().join("\n"), cx);
    }
    fn render_empty(&mut self, _: &mut Window, cx: &mut Context<TableState<Self>>) -> impl IntoElement {
        let content = self.empty.and_then(|empty| self.root.update(cx, |root, cx| root.content_of(empty, cx)).ok());
        match content {
            Some(content) => div().size_full().flex().items_center().justify_center().children(content).into_any_element(),
            None => div().size_full().flex().items_center().justify_center().text_color(cx.theme().muted_foreground.opacity(0.6)).child(Icon::new(IconName::Inbox).size_12()).into_any_element(),
        }
    }
    fn loading(&self, _: &App) -> bool {self.loading}
    fn has_more(&self, _: &App) -> bool {self.has_more}
    fn load_more_threshold(&self) -> usize {self.threshold}
    fn load_more(&mut self, _: &mut Window, cx: &mut Context<TableState<Self>>) {self.emit("loadMore", String::new(), cx);}
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

/// Select element options from its props; a SelectEmpty child replaces the empty list.
macro_rules! select_options {($root:expr, $select:expr, $kit:expr, $empty:expr, $cx:expr) => {{
    let kit: &Kit = $kit;
    let mut select = $select.placeholder(kit.text("placeholder")).disabled(kit.flag("disabled")).cleanable(kit.flag("cleanable"));
    if let Some(label) = kit.prop("accessibilityLabel") {select = select.accessibility_label(label.to_owned());}
    if let Some(icon) = icon_name(&kit.text("icon")) {select = select.icon(icon);}
    if let Some(prefix) = kit.prop("titlePrefix") {select = select.title_prefix(prefix.to_owned());}
    if let Some(placeholder) = kit.prop("searchPlaceholder") {select = select.search_placeholder(placeholder.to_owned());}
    if let Some(bounce) = kit.prop("scrollBounce") {select = select.scroll_bounce(bounce == "true");}
    if kit.prop("appearance") == Some("false") {select = select.appearance(false);}
    if let Some(width) = kit.number("menuWidth") {select = select.menu_width(px(width as f32));}
    if let Some(height) = kit.number("menuMaxHeight") {select = select.menu_max_h(px(height as f32));}
    if let Some(size) = kit_size(kit) {select = select.with_size(size);}
    // Kit renders the empty state from &App, so a SelectEmpty child contributes its text.
    if let Some(empty) = $empty {
        let message = SharedString::from(find(&$root.node, empty).map(|node| node.content_text()).unwrap_or_default());
        select = select.empty(move |_, cx| div().flex().justify_center().p_4().text_color(cx.theme().muted_foreground).child(message.clone()));
    }
    select
}}}

/// Combobox element options; ComboboxTrigger and ComboboxFooter children render as its trigger and footer.
macro_rules! combobox_options {($root:expr, $combobox:expr, $kit:expr, $empty:expr, $trigger:expr, $footer:expr, $weak:expr, $cx:expr) => {{
    let kit: &Kit = $kit;
    let mut combobox = $combobox.placeholder(kit.text("placeholder")).disabled(kit.flag("disabled")).cleanable(kit.flag("cleanable"));
    if let Some(icon) = icon_name(&kit.text("icon")) {combobox = combobox.icon(icon);}
    if let Some(icon) = icon_name(&kit.text("checkIcon")) {combobox = combobox.check_icon(icon);}
    if let Some(placeholder) = kit.prop("searchPlaceholder") {combobox = combobox.search_placeholder(placeholder.to_owned());}
    if kit.prop("appearance") == Some("false") {combobox = combobox.appearance(false);}
    if let Some(width) = kit.number("menuWidth") {combobox = combobox.menu_width(px(width as f32));}
    if let Some(height) = kit.number("menuMaxHeight") {combobox = combobox.menu_max_h(px(height as f32));}
    if let Some(size) = kit_size(kit) {combobox = combobox.with_size(size);}
    if let Some(empty) = $empty {
        let message = SharedString::from(find(&$root.node, empty).map(|node| node.content_text()).unwrap_or_default());
        combobox = combobox.empty(move |_, cx| div().flex().justify_center().p_4().text_color(cx.theme().muted_foreground).child(message.clone()));
    }
    if let Some(trigger) = $trigger {
        let root = $weak.clone();
        combobox = combobox.render_trigger(move |_, _, cx| div().flex().items_center().gap_2().children(root.update(cx, |root, cx| root.content_of(trigger, cx)).unwrap_or_default()));
    }
    if let Some(footer) = $footer {
        let root = $weak.clone();
        combobox = combobox.footer(move |_, cx| div().flex().items_center().gap_2().p_1().children(root.update(cx, |root, cx| root.content_of(footer, cx)).unwrap_or_default()));
    }
    combobox
}}}

/// Marks notifications pushed with an id, so a later notify or dismissNotification with the same id finds them.
struct TsrustNote;

type NoteReply = std::rc::Rc<std::cell::RefCell<Option<commands::Reply>>>;

/// A Kit notification from a notify payload; with a reply, its action, click and close report back once.
fn notification(value: &serde_json::Value, events: Option<(WeakEntity<Root>, NoteReply)>) -> Notification {
    let text = |key: &str| value.get(key).and_then(|value| value.as_str()).unwrap_or_default().to_owned();
    let message = text("message");
    // Kit draws a custom icon only on an untyped notification; the variant's icon wins otherwise.
    let custom_icon = icon_name(&text("icon"));
    let mut note = match (text("variant").as_str(), custom_icon.is_some()) {
        (_, true) => Notification::new().message(message),
        ("success", _) => Notification::success(message), ("warning", _) => Notification::warning(message), ("error", _) => Notification::error(message), _ => Notification::info(message),
    };
    note = match text("delivery").as_str() {"system" => note.system(), "both" => note.in_app_and_system(), _ => note};
    if !text("title").is_empty() {note = note.title(text("title"));}
    if !text("id").is_empty() {note = note.id1::<TsrustNote>(ElementId::Name(text("id").into()));}
    if let Some(icon) = custom_icon {note = note.icon(icon);}
    if !text("placement").is_empty() {note = note.placement(anchor_named(Some(&text("placement"))));}
    if !text("autohide").is_empty() {note = note.autohide(text("autohide") == "true");}
    let Some((root, reply)) = events else {return note};
    let deliver = move |root: &WeakEntity<Root>, reply: &NoteReply, event: &str, cx: &mut App| {
        if let Some(reply) = reply.borrow_mut().take() {let event = event.to_owned(); let _ = root.update(cx, |root, cx| {root.reply(reply, &event, cx);});}
    };
    let (click_root, click_reply) = (root.clone(), reply.clone());
    note = note.on_click(move |_, _, cx| deliver(&click_root, &click_reply, "click", cx));
    let (close_root, close_reply) = (root.clone(), reply.clone());
    note = note.on_close(move |_, cx| deliver(&close_root, &close_reply, "close", cx));
    let action = text("action");
    if !action.is_empty() {
        note = note.action(move |_, _, _| {
            let (root, reply) = (root.clone(), reply.clone());
            gpui_kit::component::button::Button::new("tsrust-note-action").small().primary().label(action.clone()).on_click(move |_, _, cx| deliver(&root, &reply, "action", cx))
        });
    }
    note
}

/// Rendered parts of a Dialog or Sheet.
struct OverlayParts {title: Option<Vec<AnyElement>>, description: Option<Vec<AnyElement>>, body: Vec<AnyElement>, footer: Option<Vec<AnyElement>>}

/// A DatePicker value as text: a day (with THH:MM:SS when timed), start..end, or empty.
fn date_time_text(value: &gpui_kit::component::date_picker::DateTime, timed: bool) -> String {
    use gpui_kit::component::date_picker::DateTime;
    let format = |at: &Option<chrono::NaiveDateTime>| at.map(|at| at.format(if timed {"%Y-%m-%dT%H:%M:%S"} else {"%Y-%m-%d"}).to_string()).unwrap_or_default();
    match value {DateTime::Single(at) => format(at), DateTime::Range(None, None) => String::new(), DateTime::Range(start, end) => format!("{}..{}", format(start), format(end))}
}

/// Push a text value into a DatePicker: days, ranges, and date-times when timed.
fn set_picker_value(state: &mut DatePickerState, value: &str, range: bool, timed: bool, window: &mut Window, cx: &mut Context<DatePickerState>) {
    use gpui_kit::component::date_picker::DateTime;
    let at = |text: &str| chrono::NaiveDateTime::parse_from_str(text.trim(), "%Y-%m-%dT%H:%M:%S").or_else(|_| chrono::NaiveDateTime::parse_from_str(text.trim(), "%Y-%m-%dT%H:%M")).ok();
    if timed {
        let value = if range {let (start, end) = value.split_once("..").unwrap_or((value, "")); DateTime::Range(at(start), at(end))} else {DateTime::Single(at(value))};
        state.set_date_time(value, window, cx);
    } else {
        state.set_date(date_value(value, range), window, cx);
    }
}

/// A YYYY-MM-DD day.
fn day(text: &str) -> Option<chrono::NaiveDate> {chrono::NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()}

/// A calendar value: a day, or start..end for ranges.
fn date_value(text: &str, range: bool) -> Date {
    if range {let (start, end) = text.split_once("..").unwrap_or((text, "")); Date::Range(day(start), day(end))} else {Date::Single(day(text))}
}

/// A calendar value as text: a day, start..end, or empty.
fn date_text(date: &Date) -> String {
    let format = |day: &Option<chrono::NaiveDate>| day.map(|day| day.format("%Y-%m-%d").to_string()).unwrap_or_default();
    match date {Date::Single(single) => format(single), Date::Range(None, None) => String::new(), Date::Range(start, end) => format!("{}..{}", format(start), format(end))}
}

/// A weekday by name or number (0 = Sunday).
fn weekday(name: &str) -> Option<chrono::Weekday> {
    use chrono::Weekday::*;
    Some(match name.trim().to_lowercase().as_str() {"0" | "sun" | "sunday" => Sun, "1" | "mon" | "monday" => Mon, "2" | "tue" | "tuesday" => Tue, "3" | "wed" | "wednesday" => Wed, "4" | "thu" | "thursday" => Thu, "5" | "fri" | "friday" => Fri, "6" | "sat" | "saturday" => Sat, _ => return None})
}

/// Days a Calendar or DatePicker cannot select: before/after a day, a from..to range, listed weekdays or listed days.
fn date_matcher(props: &std::collections::BTreeMap<String, String>) -> Option<Matcher> {
    let text = |key: &str| props.get(key).cloned().unwrap_or_default();
    let (before, after, from, to) = (day(&text("disabledBefore")), day(&text("disabledAfter")), day(&text("disabledFrom")), day(&text("disabledTo")));
    let weekdays = text("disabledWeekdays").split([',', '\n']).filter_map(weekday).collect::<Vec<_>>();
    let days = text("disabledDates").split('\n').filter_map(day).collect::<Vec<_>>();
    if before.is_none() && after.is_none() && from.is_none() && to.is_none() && weekdays.is_empty() && days.is_empty() {return None;}
    Some(Matcher::custom(move |date: &chrono::NaiveDate| {
        use chrono::Datelike;
        before.is_some_and(|limit| *date < limit) || after.is_some_and(|limit| *date > limit)
            || ((from.is_some() || to.is_some()) && from.is_none_or(|start| *date >= start) && to.is_none_or(|end| *date <= end))
            || weekdays.contains(&date.weekday()) || days.contains(date)
    }))
}

fn parse_diff(source:&str)->Result<Vec<DiffFile>,DiffParseError> {
    Ok(DiffFile::parse(source)?.into_iter().map(|file| {
        let extension=std::path::Path::new(file.path().as_ref()).extension().and_then(|extension|extension.to_str());
        let language=extension.map(super::kit::source_language).filter(|language|Some(*language)!=extension).map(str::to_owned);
        match language {Some(language)=>file.with_language(language),None=>file}
    }).collect())
}
/// State setters preserve selection/scroll and skip unchanged values in Kit.
fn configure_diff(state: &mut DiffState, props: &std::collections::BTreeMap<String,String>, cx: &mut Context<DiffState>) {
    let text=|key:&str|props.get(key).map(String::as_str);
    let count=|key:&str,default:usize|text(key).and_then(|value|value.parse().ok()).unwrap_or(default);
    state.set_mode(if text("mode")==Some("split") {DiffMode::Split}else{DiffMode::Unified},cx);
    let context=match text("contextLines") {Some("-1")=>None,Some(value)=>value.parse().ok(),None=>Some(3)};
    state.set_context_lines(context,cx);
    state.set_expansion_lines(count("expansionLines",20),cx);
    state.set_min_collapsed_lines(count("minCollapsedLines",2),cx);
    state.set_inline_unit(match text("inlineUnit") {Some("none")=>None,Some("character")=>Some(DiffInlineUnit::Character),_=>Some(DiffInlineUnit::Word)},cx);
    state.set_inline_max_line_length(count("inlineMaxLineLength",1000),cx);
    state.set_syntax_max_line_length(count("syntaxMaxLineLength",1000),cx);
}
fn diff_side(side:DiffSide)->&'static str {match side {DiffSide::Original=>"original",DiffSide::Modified=>"modified"}}
fn diff_position(position:&DiffLinePosition)->String {
    serde_json::json!({"path":position.path().as_ref(),"side":diff_side(position.side()),"line":position.line()}).to_string()
}
fn diff_range(range:&DiffLineRange)->String {
    serde_json::json!({"path":range.path().as_ref(),"side":diff_side(range.side()),"start":range.start(),"endSide":diff_side(range.end_side()),"end":range.end()}).to_string()
}

/// A Select option: its label, value and disabled state.
#[derive(Clone)]
struct Choice {label: SharedString, value: SharedString, disabled: bool}

impl SelectItem for Choice {
    type Value = SharedString;
    fn title(&self) -> SharedString {self.label.clone()}
    fn value(&self) -> &SharedString {&self.value}
    fn disabled(&self) -> bool {self.disabled}
}

/// A Select's options: the `items` prop and SelectItem children, then SelectGroup children
/// (each titled by its label prop or SelectLabel child). The shape string detects changes.
fn choices(node: &Node, kit: &Kit) -> (Vec<Choice>, Vec<(SharedString, Vec<Choice>)>, String) {
    fn choice(item: &Kit) -> Choice {
        let label = item.text("label");
        let value = item.prop("value").map_or_else(|| label.clone(), str::to_owned);
        Choice {label: label.into(), value: value.into(), disabled: item.flag("disabled")}
    }
    let mut loose = kit.text("items").split('\n').filter(|item| !item.is_empty()).map(|item| Choice {label: item.to_owned().into(), value: item.to_owned().into(), disabled: false}).collect::<Vec<_>>();
    loose.extend(items(node, "SelectItem").map(|(_, item)| choice(item)));
    let groups = items(node, "SelectGroup").map(|(group, kit)| {
        let title = items(group, "SelectLabel").next().map_or_else(|| kit.text("label"), |(_, label)| label.text("text"));
        (SharedString::from(title), items(group, "SelectItem").map(|(_, item)| choice(item)).collect::<Vec<_>>())
    }).collect::<Vec<_>>();
    let mut shape = String::new();
    for (title, options) in std::iter::once((&SharedString::default(), &loose)).chain(groups.iter().map(|(title, options)| (title, options))) {
        shape.push_str(title);
        for option in options {shape.push_str(&format!("\u{1}{}\u{2}{}\u{3}{}", option.label, option.value, option.disabled));}
        shape.push('\u{4}');
    }
    (loose, groups, shape)
}

/// The first Dialog or Sheet part with this suffix, such as Content for DialogContent or SheetContent.
fn first_part<'a>(node: &'a Node, part: &str) -> Option<&'a Node> {
    node.visible_children().find(|child| child.kit.as_deref().is_some_and(|kit| kit.name.strip_prefix("AlertDialog").or_else(|| kit.name.strip_prefix("Dialog")).or_else(|| kit.name.strip_prefix("Sheet")) == Some(part)))
}

/// The nearest ancestor Kit component of this name above a node, with whether it is open.
fn ancestor_named(root: &Node, target: u64, name: &str) -> Option<(u64, bool)> {
    fn walk(node: &Node, target: u64, name: &str, nearest: Option<(u64, bool)>) -> Option<Option<(u64, bool)>> {
        if node.identity.get() == target {return Some(nearest);}
        let here = node.kit.as_deref().filter(|kit| kit.name == name).map(|kit| (node.identity.get(), kit.prop("open").or(kit.prop("defaultOpen")) == Some("true"))).or(nearest);
        node.children.iter().find_map(|child| walk(child, target, name, here))
    }
    walk(root, target, name, None).flatten()
}

/// A carousel's slides: CarouselItem (or CarouselSlide) children.
fn carousel_slides(node: &Node) -> Vec<&Node> {
    node.visible_children().filter(|child| child.kit.as_deref().is_some_and(|kit| kit.name == "CarouselItem" || kit.name == "CarouselSlide")).collect()
}

/// The first item component of this name among a node's children.
fn first_item<'a>(node: &'a Node, name: &'a str) -> Option<&'a Node> {items(node, name).next().map(|(child, _)| child)}

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
    let text = &platform_keys(text);
    thread_local! {static KEYS: std::cell::RefCell<HashMap<String, &'static str>> = std::cell::RefCell::new(HashMap::new());}
    let text = KEYS.with(|keys| *keys.borrow_mut().entry(text.to_owned()).or_insert_with(|| Box::leak(text.to_owned().into_boxed_str())));
    Keystroke::parse(text).ok()
}

impl Root {
    /// Deliver a component event to its node, then refresh and redraw.
    fn emit(&mut self, id: u64, event: &str, value: String, cx: &mut Context<Self>) {
        let _scope=self.scope();
        let result = self.node.kit_event(id, event, &value);
        self.settle(result, cx);
    }

    /// The style refinement for a utility class string.
    fn class_style(&self, classes: &str, cx: &mut Context<Self>) -> StyleRefinement {
        if classes.is_empty() {return StyleRefinement::default();}
        if let Err(error)=style::validate(classes) {fail(&self.failure,error);}
        let theme = utility_theme(cx);
        let rules=style::resolved_surface_at_width(classes,&theme,mode(cx),self.viewport.0).into_iter().map(|(state,rule)|(state,rule.in_viewport(self.viewport.0,self.viewport.1))).collect::<Vec<_>>();
        let mut surface = styled_rules(div(), &rules, false, 0);
        surface.style().clone()
    }

    /// Children of the node except the named item components.
    fn content_except(&self, node: &Node, skip: &[&str], cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = utility_theme(cx);
        node.visible_children().filter(|child| !child.kit.as_deref().is_some_and(|kit| skip.contains(&kit.name.as_str()))).map(|child| self.element(child, &theme, mode(cx), cx)).collect()
    }

    /// A CommandItem: label, icon, keywords, a Shortcut command's keys, check and custom content.
    fn command_item(&self, node: &Node, cx: &mut Context<Self>) -> CommandItem {
        let item = node.kit.as_deref().expect("CommandItem");
        let mut entry = CommandItem::new().label(item.text("label")).disabled(item.flag("disabled")).checked(item.flag("checked"));
        if let Some(icon) = icon_name(&item.text("icon")) {entry = entry.icon(icon);}
        let keywords = item.text("keywords").split('\n').filter(|word| !word.is_empty()).map(str::to_owned).collect::<Vec<_>>();
        if !keywords.is_empty() {entry = entry.keywords(keywords);}
        if !item.text("command").is_empty() {entry = entry.action(Box::new(TsrustCommand {name: item.text("command")}));}
        // Rich children (beyond text) replace the row's content.
        if node.visible_children().any(|part| part.kind != Kind::Text) {
            let (part, root) = (node.identity.get(), cx.entity().downgrade());
            entry = entry.child(move |_, cx| div().flex().items_center().gap_2().children(root.update(cx, |root, cx| root.content_of(part, cx)).unwrap_or_default()));
        }
        entry
    }

    /// A SidebarItem with its nested items, suffix and right-click menu.
    fn sidebar_item(&self, node: &Node, item: &Kit, collapsed: bool, cx: &mut Context<Self>) -> SidebarMenuItem {
        let id = node.identity.get();
        let mut entry = SidebarMenuItem::new(item.text("label")).active(item.flag("active")).collapsed(collapsed).disable(item.flag("disabled"))
            .default_open(item.flag("defaultOpen")).click_to_open(item.flag("clickToOpen")).click_to_toggle(item.flag("clickToToggle"))
            .label_style(self.class_style(&item.text("labelClassName"), cx));
        if let Some(icon) = icon_name(&item.text("icon")) {entry = entry.icon(icon);}
        if let Some(label) = item.prop("accessibilityLabel") {entry = entry.accessibility_label(label.to_owned());}
        let children = items(node, "SidebarItem").map(|(child, nested)| self.sidebar_item(child, nested, collapsed, cx)).collect::<Vec<_>>();
        if !children.is_empty() {entry = entry.children(children);}
        let root = cx.entity().downgrade();
        if let Some(suffix) = first_item(node, "SidebarItemSuffix") {
            let (suffix, root) = (suffix.identity.get(), root.clone());
            entry = entry.suffix(move |_, cx| div().flex().items_center().gap_1().children(root.update(cx, |root, cx| root.content_of(suffix, cx)).unwrap_or_default()));
        }
        let entries = std::rc::Rc::new(menu_entries(node));
        if !entries.is_empty() {
            let root = root.clone();
            entry = entry.context_menu(move |menu, _, _| flat_menu(menu, &entries, &root));
        }
        if item.handles("click") {entry = entry.on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(id, "click", String::new(), cx));});}
        entry
    }

    /// An InputGroupButton; MenuItem children make it open a dropdown menu.
    fn input_group_button(&self, node: &Node, cx: &mut Context<Self>) -> AnyElement {
        let id = node.identity.get();
        let kit = node.kit.as_deref().expect("InputGroupButton");
        let mut button = menu_button(InputGroupButton::new(ElementId::Integer(id)), kit, cx);
        if !kit.text("label").is_empty() {button = button.label(kit.text("label"));}
        if let Some(icon) = icon_name(&kit.text("icon")) {button = button.icon(icon);}
        if let Some(label) = kit.prop("accessibilityLabel") {button = button.accessibility_label(label.to_owned());}
        if let Some(tooltip) = kit.prop("tooltip").filter(|tooltip| !tooltip.is_empty()) {button = button.tooltip(tooltip.to_owned());}
        if let Some(delay) = kit.index("tooltipDelay") {button = button.tooltip_show_delay(std::time::Duration::from_millis(delay as u64));}
        if kit.flag("loading") {button = button.loading(true);}
        if let Some(icon) = icon_name(&kit.text("loadingIcon")) {button = button.loading_icon(icon);}
        if kit.flag("outline") {button = button.outline();}
        if let Some(index) = kit.number("tabIndex") {button = button.tab_index(index as isize);}
        if kit.flag("caret") {button = button.dropdown_caret(true);}
        let root = cx.entity().downgrade();
        if kit.handles("click") {
            let root = root.clone();
            button = button.on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(id, "click", String::new(), cx));});
        }
        let entries = std::rc::Rc::new(menu_entries(node));
        if entries.is_empty() {return button.into_any_element();}
        button.dropdown_menu(move |menu, window, cx| build_menu(menu, &entries, &root, window, cx)).into_any_element()
    }

    /// A Kit Avatar from an Avatar node's props.
    fn kit_avatar(kit: &Kit) -> Avatar {
        // An empty name would draw empty initials instead of the placeholder icon.
        let mut avatar = Avatar::new();
        if let Some(name) = kit.prop("name").filter(|name| !name.is_empty()) {avatar = avatar.name(SharedString::from(name.to_owned()));}
        if let Some(source) = kit.prop("src").filter(|source| !source.is_empty()) {avatar = avatar.src(SharedString::from(source.to_owned()));}
        if let Some(icon) = kit.prop("placeholder").and_then(icon_name) {avatar = avatar.placeholder(icon);}
        if let Some(size) = kit_size(kit) {avatar = avatar.with_size(size);}
        avatar
    }

    fn content(&self, node: &Node, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = utility_theme(cx);
        node.visible_children().map(|child| self.element(child, &theme, mode(cx), cx)).collect()
    }

    /// One List row from its ListItem (or ListSeparator) node: content, ListItemSuffix, check icon and events.
    fn list_item(&self, row: u64, cx: &mut Context<Self>) -> Option<ListItem> {
        let node = find(&self.node, row)?;
        let kit = node.kit.as_deref()?;
        let mut item = ListItem::new(ElementId::Name(format!("list-item-{row}").into()));
        if kit.name == "ListSeparator" {return Some(item.separator());}
        item = item.disabled(kit.flag("disabled")).confirmed(kit.flag("confirmed"));
        if let Some(label) = kit.prop("accessibilityLabel") {item = item.accessibility_label(label.to_owned());}
        // Kit draws the confirmed mark only when a check icon is set; default to a check.
        item = item.check_icon(icon_name(&kit.text("checkIcon")).unwrap_or(IconName::Check));
        let theme = utility_theme(cx);
        let mut suffix = None;
        let mut content = Vec::new();
        for child in node.visible_children() {
            if child.kit.as_deref().is_some_and(|kit| kit.name == "ListItemSuffix") {suffix = Some(child.identity.get()); continue;}
            content.push(self.element(child, &theme, mode(cx), cx));
        }
        item = item.child(div().flex().items_center().gap_2().min_w_0().children(content));
        let root = cx.entity().downgrade();
        if let Some(part) = suffix {
            let root = root.clone();
            item = item.suffix(move |_, cx| div().flex().items_center().gap_1().children(root.update(cx, |root, cx| root.content_of(part, cx)).unwrap_or_default()));
        }
        let emit = move |event: &'static str| {
            let root = root.clone();
            move |cx: &mut App| {let _ = root.update(cx, |root, cx| root.emit(row, event, String::new(), cx));}
        };
        let (click, down, enter) = (emit("click"), emit("mouseDown"), emit("hover"));
        Some(item.on_click(move |_, _, cx| click(cx)).on_mouse_down(MouseButton::Left, move |_, _, cx| down(cx)).on_mouse_enter(move |_, _, cx| enter(cx)))
    }

    /// Children of the node with this identity, for overlay content built later.
    fn content_of(&self, id: u64, cx: &mut Context<Self>) -> Vec<AnyElement> {
        match find(&self.node, id) {Some(node) => self.content(node, cx), None => Vec::new()}
    }

    fn slot_of(&self,id:u64,cx:&mut Context<Self>)->AnyElement {
        match find(&self.node,id) {
            Some(node)=>div().refine_style(&self.node_style(node)).children(self.content(node,cx)).into_any_element(),
            None=>div().into_any_element(),
        }
    }

    /// Run every mounted Shortcut bound to this command.
    fn run_command(&mut self, name: &str, cx: &mut Context<Self>) {
        fn targets(node: &Node, name: &str, out: &mut Vec<u64>) {
            let Some(node) = node.visible_root() else {return};
            if node.kit.as_deref().is_some_and(|kit| kit.name == "Shortcut" && kit.text("command") == name) {out.push(node.identity.get());}
            for child in &node.children {targets(child, name, out);}
        }
        let mut ids = Vec::new();
        targets(&self.node, name, &mut ids);
        for id in ids {self.emit(id, "trigger", String::new(), cx);}
    }

    /// The element for one visible child of a node, for row renderers built later.
    fn child_of(&self, id: u64, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let theme = utility_theme(cx);
        match find(&self.node, id).and_then(|node| node.visible_children().nth(index)) {Some(child) => self.element(child, &theme, mode(cx), cx), None => div().into_any_element()}
    }

    /// A Bubble: BubbleContent (or its loose children) and an optional BubbleReactions row.
    fn bubble(&self, node: &Node, kit: &Kit, cx: &mut Context<Self>) -> Bubble {
        let mut bubble = Bubble::new().with_variant(bubble_variant(kit)).alignment(message_alignment(kit)).refine_style(&self.node_style(node));
        let content = match first_item(node, "BubbleContent") {
            Some(part) => BubbleContent::new().refine_style(&self.node_style(part)).children(self.content(part, cx)),
            None => BubbleContent::new().children(self.content_except(node, &["BubbleReactions"], cx)),
        };
        bubble = bubble.content(content);
        if let Some((part, reactions)) = items(node, "BubbleReactions").next() {
            let side = if reactions.prop("side") == Some("top") {BubbleReactionSide::Top} else {BubbleReactionSide::Bottom};
            bubble = bubble.reactions(BubbleReactions::new().side(side).alignment(message_alignment(reactions)).refine_style(&self.node_style(part)).children(self.content(part, cx)));
        }
        bubble
    }

    /// An Attachment from its props (short form) or AttachmentMedia / AttachmentContent / AttachmentActions parts.
    fn attachment(&self, node: &Node, kit: &Kit, cx: &mut Context<Self>) -> Attachment {
        let id = node.identity.get();
        let status = match kit.prop("status") {
            Some("pending") => AttachmentStatus::Pending,
            Some("uploading") => AttachmentStatus::Uploading,
            Some("processing") => AttachmentStatus::Processing,
            Some("failed") => AttachmentStatus::Failed,
            _ => AttachmentStatus::Complete,
        };
        let size = kit_size(kit);
        let image_source = |source: String| -> ImageSource {if std::path::Path::new(&source).is_file() {std::path::PathBuf::from(source).into()} else {SharedString::from(source).into()}};
        let mut media = match items(node, "AttachmentMedia").next() {
            Some((part, media_kit)) => {
                let mut media = AttachmentMedia::new().refine_style(&self.node_style(part));
                match media_kit.prop("src") {
                    // With an image, children draw over it; otherwise they are the media (such as an icon).
                    Some(source) => {media = media.src(image_source(source.to_owned())); let overlay = self.content(part, cx); if !overlay.is_empty() {media = media.overlay(div().size_full().children(overlay));}}
                    None => media = media.children(self.content(part, cx)),
                }
                media
            }
            None => match kit.prop("image") {
                Some(source) => AttachmentMedia::new().src(image_source(source.to_owned())),
                None => AttachmentMedia::new().child(Icon::new(icon_name(&kit.text("icon")).unwrap_or(IconName::File))),
            },
        };
        if let Some(size) = size {media = media.with_size(size);}
        let (title_kit, title_text, description_text) = match items(node, "AttachmentContent").next() {
            Some((part, _)) => {
                let title = items(part, "AttachmentTitle").next();
                let description = first_item(part, "AttachmentDescription");
                (title.map(|(_, kit)| kit), title.map(|(node, _)| node.content_text()).unwrap_or_default(), description.map(Node::content_text).unwrap_or_default())
            }
            None => (None, kit.text("title"), kit.text("description")),
        };
        let mut title = AttachmentTitle::new(title_text).status(status);
        if let Some(style) = title_kit.and_then(|kit| shimmer_style(kit, cx)).or_else(|| shimmer_style(kit, cx)) {title = title.with_shimmer_style(style);}
        let mut content = AttachmentContent::new().title(title);
        if !description_text.is_empty() {content = content.description(AttachmentDescription::new(description_text).status(status));}
        let mut attachment = Attachment::new().id(ElementId::Integer(id)).status(status).media(media).content(content).refine_style(&self.node_style(node));
        if let Some(part) = first_item(node, "AttachmentActions") {attachment = attachment.actions(AttachmentActions::new().children(self.content(part, cx)));}
        if let Some(progress) = kit.number("progress") {attachment = attachment.progress(progress as f32);}
        if let Some(tooltip) = kit.prop("tooltip") {attachment = attachment.tooltip(tooltip.to_owned());}
        if kit.flag("vertical") {attachment = attachment.axis(Axis::Vertical);}
        if let Some(size) = size {attachment = attachment.with_size(size);}
        let root = cx.entity().downgrade();
        for (event, handled) in [("click", kit.handles("click")), ("remove", kit.handles("remove")), ("retry", kit.handles("retry"))] {
            if !handled {continue;}
            let root = root.clone();
            let handler = move |_: &ClickEvent, _: &mut Window, cx: &mut App| {let _ = root.update(cx, |root, cx| root.emit(id, event, String::new(), cx));};
            attachment = match event {"click" => attachment.on_click(handler), "remove" => attachment.on_remove(handler), _ => attachment.on_retry(handler)};
        }
        attachment
    }

    /// Create Kit state entities for stateful components, push changed props
    /// into them, and open or close dialogs and sheets to match their props.
    fn sync_kit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        fn collect(node: &Node, out: &mut Vec<(u64, String, std::collections::BTreeMap<String, String>)>) {
            let Some(node) = node.visible_root() else {return};
            if let Some(kit) = &node.kit && matches!(kit.name.as_str(), "AppMenuBar" | "AlertDialog" | "Select" | "NumberInput" | "OtpInput" | "DatePicker" | "ColorPicker" | "Slider" | "Dialog" | "Sheet" | "Calendar" | "TimeField" | "Combobox" | "Command" | "Carousel" | "CodeEditor" | "Tree" | "List" | "Diff" | "DataTable" | "MessageScroller" | "Dock" | "Questionnaire" | "SpeechButton" | "Ticker") {
                out.push((node.identity.get(), kit.name.clone(), kit.props.clone()));
            }
            for child in &node.children {collect(child, out);}
        }
        let mut found = Vec::new();
        collect(&self.node, &mut found);
        if self.states.values().any(|state|matches!(state,KitState::AppMenuBar(..))) && !found.iter().any(|(_,name,_)|name=="AppMenuBar") {
            if let Err(error)=install_application_menus(&[],cx) {fail(&self.failure,error);}
        }
        fn shortcuts(node: &Node, out: &mut Vec<(String, String)>) {
            let Some(node) = node.visible_root() else {return};
            if let Some(kit) = node.kit.as_deref().filter(|kit| kit.name == "Shortcut") {out.push((kit.text("keys"), kit.text("command")));}
            for child in &node.children {shortcuts(child, out);}
        }
        let mut keys = Vec::new();
        shortcuts(&self.node, &mut keys);
        for (keys, command) in keys {bind_shortcut(&keys, &command, cx);}
        self.states.retain(|id, _| found.iter().any(|(other, _, _)| other == id));
        let mut dialog = None;
        let mut sheet = None;
        for (id, name, props) in found {
            let text = |key: &str| props.get(key).cloned().unwrap_or_default();
            let number = |key: &str| props.get(key).and_then(|value| value.parse::<f32>().ok());
            match name.as_str() {
                "Dialog" | "AlertDialog" => {if props.get("open").or(props.get("defaultOpen")).map(String::as_str) == Some("true") {dialog = Some(id);} continue}
                "Sheet" => {if props.get("open").or(props.get("defaultOpen")).map(String::as_str) == Some("true") {sheet = Some((id, text("side")));} continue}
                _ => {}
            }
            if let Some(state) = self.states.get(&id) {
                match state {
                    KitState::AppMenuBar(bar, current) => {
                        let Some(node)=find(&self.node,id) else {continue};
                        let menus=application_menus(node);
                        if *current != menus {
                            if let Err(error)=install_application_menus(&menus,cx) {fail(&self.failure,error);}
                            bar.update(cx, |bar,cx|bar.reload(cx));
                            if let Some(KitState::AppMenuBar(_,current))=self.states.get_mut(&id) {*current=menus;}
                        }
                    }
                    KitState::Select(..) | KitState::SelectGroups(..) => {
                        let Some(node) = find(&self.node, id) else {continue};
                        let (loose, groups, shape) = choices(node, node.kit.as_deref().expect("Select"));
                        let value = props.get("value").cloned();
                        match self.states.get_mut(&id) {
                            Some(KitState::Select(state, current)) => {
                                if *current != shape && groups.is_empty() {*current = shape; state.update(cx, |state, cx| state.set_items(SearchableVec::new(loose), window, cx));}
                                if let Some(value) = value && state.read(cx).selected_value().map(|current| current.as_ref() != value).unwrap_or(!value.is_empty()) {
                                    state.update(cx, |state, cx| state.set_selected_value(&SharedString::from(value), window, cx));
                                }
                            }
                            Some(KitState::SelectGroups(state, current)) => {
                                if *current != shape {
                                    *current = shape;
                                    let mut data = SearchableVec::new(Vec::new());
                                    for (title, options) in groups {data.push(SelectGroup::new(title).items(options));}
                                    state.update(cx, |state, cx| state.set_items(data, window, cx));
                                }
                                if let Some(value) = value && state.read(cx).selected_value().map(|current| current.as_ref() != value).unwrap_or(!value.is_empty()) {
                                    state.update(cx, |state, cx| state.set_selected_value(&SharedString::from(value), window, cx));
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }
                    KitState::Combobox(..) | KitState::ComboboxGroups(..) => {
                        let Some(node) = find(&self.node, id) else {continue};
                        let (loose, groups, shape) = choices(node, node.kit.as_deref().expect("Combobox"));
                        let values = props.get("value").map(|value| value.split('\n').filter(|value| !value.is_empty()).map(SharedString::from).map(|value| SharedString::from(value.to_string())).collect::<Vec<_>>());
                        let query = props.get("query").cloned();
                        macro_rules! sync_combobox {($state:expr, $current:expr, $data:expr) => {{
                            if *$current != shape {*$current = shape; let data = $data; $state.update(cx, |state, cx| state.set_items(data, window, cx));}
                            if let Some(values) = values && $state.read(cx).selected_values() != values {$state.update(cx, |state, cx| state.set_selected_values(&values, window, cx));}
                            if let Some(query) = query && $state.read(cx).query(cx).as_ref() != query {$state.update(cx, |state, cx| state.set_query(query, window, cx));}
                        }}}
                        match self.states.get_mut(&id) {
                            Some(KitState::Combobox(state, current)) => if groups.is_empty() {sync_combobox!(state, current, SearchableVec::new(loose))},
                            Some(KitState::ComboboxGroups(state, current)) => sync_combobox!(state, current, {let mut data = SearchableVec::new(Vec::new()); for (title, options) in groups {data.push(SelectGroup::new(title).items(options));} data}),
                            _ => {}
                        }
                        continue;
                    }
                    KitState::Number(state) => {
                        let number=|key:&str|props.get(key).and_then(|value|value.parse::<f64>().ok());
                        let custom_step=find(&self.node,id).and_then(|node|node.kit.as_deref()).is_some_and(|kit|kit.number_step.is_some());
                        state.update(cx,|state,cx| {
                            if !custom_step {state.set_step(Some(gpui_kit::component::input::NumberStep::Fixed(number("step").unwrap_or(1.))),window,cx);}
                            state.set_min(number("min"),window,cx);state.set_max(number("max"),window,cx);
                        });
                        if let Some(value)=number("value") && state.read(cx).value().parse::<f64>().ok()!=Some(value) {state.update(cx,|state,cx|state.set_value(value.to_string(),window,cx));}
                    }
                    KitState::Slider(state) => {
                        let wanted = match (state.read(cx).value(), number("value"), number("endValue")) {
                            (SliderValue::Range(start, end), value, end_value) if value.is_some() || end_value.is_some() => Some(SliderValue::Range(value.unwrap_or(start), end_value.unwrap_or(end))),
                            (SliderValue::Single(_), Some(value), _) => Some(SliderValue::Single(value)),
                            _ => None,
                        };
                        if let Some(wanted) = wanted && state.read(cx).value() != wanted {state.update(cx, |state, cx| state.set_value(wanted, window, cx));}
                    }
                    KitState::List(state) => {
                        let Some(node) = find(&self.node, id) else {continue};
                        if let Some(query) = props.get("query") && state.read(cx).delegate().query != *query {let query = query.clone(); state.update(cx, |state, cx| state.set_query(&query, window, cx));}
                        let (sections, values, empty) = (list_sections(node), list_values(node), first_item(node, "ListEmpty").map(|empty| empty.identity.get()));
                        let (loading, has_more) = (text("loading") == "true", text("hasMore") == "true");
                        let threshold = number("loadMoreThreshold").map_or(20, |rows| rows as usize);
                        let (searchable, selectable) = (text("searchable") == "true", text("selectable") != "false");
                        state.update(cx, |state, cx| {
                            let list = state.delegate_mut();
                            (list.sections, list.values, list.empty, list.loading, list.has_more, list.threshold) = (sections, values, empty, loading, has_more, threshold);
                            state.set_searchable(searchable, cx);
                            state.set_selectable(selectable, cx);
                            cx.notify();
                        });
                        if let Some(index) = number("selected").filter(|index| *index >= 0.).map(|index| index as usize).and_then(|index| list_path(&state.read(cx).delegate().sections, index))
                            && state.read(cx).selected_index() != Some(index) {
                            state.update(cx, |state, cx| {state.set_selected_index(Some(index), window, cx); state.scroll_to_selected_item(window, cx);});
                        }
                    }
                    KitState::Carousel(state) => {
                        let count = find(&self.node, id).map(|node| carousel_slides(first_item(node, "CarouselContent").unwrap_or(node)).len()).unwrap_or(0);
                        if state.read(cx).item_count() != count {state.update(cx, |state, cx| state.set_item_count(count, cx));}
                        let looping = text("loop") == "true";
                        if state.read(cx).is_looping() != looping {state.update(cx, |state, cx| state.set_looping(looping, cx));}
                        let axis = if text("vertical") == "true" {Axis::Vertical} else {Axis::Horizontal};
                        if state.read(cx).axis() != axis {state.update(cx, |state, cx| state.set_axis(axis, cx));}
                        if let Some(selected) = number("selected").map(|index| index as usize) && state.read(cx).selected_index() != Some(selected) {state.update(cx, |state, cx| state.set_selected_index(selected, cx));}
                    }
                    KitState::Command(state) => {
                        if let Some(query) = props.get("query") && state.read(cx).query(cx).as_ref() != query {let query = query.clone(); state.update(cx, |state, cx| state.set_query(query, window, cx));}
                        let loading = text("loading") == "true";
                        if state.read(cx).is_loading() != loading {state.update(cx, |state, cx| state.set_loading(loading, window, cx));}
                    }
                    KitState::Otp(state) => {
                        let masked = text("masked") == "true";
                        if state.read(cx).is_masked() != masked {state.update(cx, |state, cx| state.set_masked(masked, window, cx));}
                        if let Some(value) = props.get("value") && state.read(cx).value().as_ref() != value {let value = value.clone(); state.update(cx, |state, cx| state.set_value(value, window, cx));}
                    }
                    KitState::Date(state) => {
                        let (range, timed) = (text("range") == "true", matches!(text("time").as_str(), "minute" | "second"));
                        if let Some(value) = props.get("value") && date_time_text(&state.read(cx).date_time(), timed) != *value {
                            let value = value.clone();
                            state.update(cx, |state, cx| set_picker_value(state, &value, range, timed, window, cx));
                        }
                    }
                    KitState::Calendar(state) => {
                        let range = text("range") == "true";
                        if let Some(value) = props.get("value") && date_text(&state.read(cx).date()) != *value {let date = date_value(value, range); state.update(cx, |state, cx| state.set_date(date, window, cx));}
                        if let Some(months) = number("months") && state.read(cx).number_of_months() != months as usize {state.update(cx, |state, cx| state.set_number_of_months(months as usize, window, cx));}
                        let view = match text("view").as_str() {"month" => Some(gpui_kit::base::CalendarView::Month), "year" => Some(gpui_kit::base::CalendarView::Year), "day" => Some(gpui_kit::base::CalendarView::Day), _ => None};
                        if let Some(view) = view && state.read(cx).view() != view {state.update(cx, |state, cx| {state.set_view(view); cx.notify();});}
                    }
                    KitState::Time(state) => {
                        let precision = if text("seconds") == "true" {TimePrecision::Second} else {TimePrecision::Minute};
                        let cycle = if text("twelveHour") == "true" {HourCycle::H12} else {HourCycle::H23};
                        state.update(cx, |state, cx| {state.set_precision(precision, window, cx); state.set_hour_cycle(cycle, window, cx);});
                        if let Some(value) = props.get("value") && let Ok(time) = chrono::NaiveTime::parse_from_str(value, "%H:%M:%S").or_else(|_| chrono::NaiveTime::parse_from_str(value, "%H:%M")) && state.read(cx).time() != time {
                            state.update(cx, |state, cx| state.set_time(time, window, cx));
                        }
                    }
                    KitState::Color(state) => {
                        if props.get("value").is_some_and(|value| value.is_empty()) && state.read(cx).value().is_some() {state.update(cx, |state, cx| state.clear_value(window, cx));}
                        if let Some(color) = theme_color(&text("value"), cx) && state.read(cx).value().map(hex) != Some(hex(color)) {
                            state.update(cx, |state, cx| state.set_value(color, window, cx));
                        }
                    }
                    KitState::Editor(state, current) => {
                        if let Some(value)=props.get("value") && state.read(cx).value().as_ref()!=value {state.update(cx,|state,cx|state.set_value(value.clone(),window,cx));}
                        if *current != props {
                            state.update(cx,|state,cx|editor_options(state,&props,window,cx));
                            if let Some(KitState::Editor(_,current))=self.states.get_mut(&id) {*current=props.clone();}
                        }
                    }
                    KitState::Tree(state, shape) => {
                        // Rebuild the items when the TreeItem children change.
                        let current = tree_shape(&self.node, id);
                        if *shape != current {
                            let items = find(&self.node, id).map(tree_items).unwrap_or_default();
                            state.update(cx, |state, cx| state.set_items(items, cx));
                            if let Some(KitState::Tree(_, shape)) = self.states.get_mut(&id) {*shape = current;}
                        }
                        if let Some(KitState::Tree(state, _)) = self.states.get(&id) && let Some(selected) = props.get("selected") {
                            let selected: SharedString = selected.clone().into();
                            if state.read(cx).selected_item().map(|item| &item.id) != Some(&selected) {
                                state.update(cx, |state, cx| tree_select(state, &selected, cx));
                            }
                        }
                    }
                    KitState::Diff(state, patch) => {
                        if *patch != text("patch") {
                            let files = parse_diff(&text("patch")).unwrap_or_else(|error|fail(&self.failure,format!("Diff patch: {error}")));
                            state.update(cx, |state, cx| state.set_files(files, cx));
                            if let Some(KitState::Diff(_, patch)) = self.states.get_mut(&id) {*patch = text("patch");}
                        }
                        if let Some(KitState::Diff(state, _)) = self.states.get(&id) {state.update(cx, |state,cx|configure_diff(state,&props,cx));}
                    }
                    KitState::Table(state, shape) => {
                        let Some(node) = find(&self.node, id) else {continue};
                        let current = grid_shape(node);
                        if *shape != current {
                            // Rows and columns changed: keep the sort direction by re-sorting the new rows.
                            let grid = Grid::new(node, cx.entity().downgrade());
                            state.update(cx, |state, cx| {*state.delegate_mut() = grid; state.refresh(cx); state.refresh_header_layout(cx);});
                            if let Some(KitState::Table(_, shape)) = self.states.get_mut(&id) {*shape = current;}
                        }
                        if let Some(KitState::Table(state, _)) = self.states.get(&id) && let Some(selected) = number("selectedRow").map(|row| row as usize) {
                            let display = state.read(cx).delegate().order.iter().position(|row| *row == selected);
                            if let Some(display) = display && state.read(cx).selected_row() != Some(display) {state.update(cx, |state, cx| {state.set_selected_row(display, cx); state.scroll_to_row(display, cx);});}
                        }
                    }
                    KitState::Scroller(state, count) => {
                        let current = find(&self.node, id).map(|node| node.visible_children().count()).unwrap_or(0);
                        if *count != current {
                            let previous = *count;
                            state.update(cx, |state, cx| if current > previous {state.append(current - previous, cx);} else {state.reset(current, cx);});
                            if let Some(KitState::Scroller(_, count)) = self.states.get_mut(&id) {*count = current;}
                        }
                    }
                    KitState::Ticker(_, shape) => {
                        // A changed interval or running flag restarts the timer.
                        if *shape != ticker_shape(&props) {
                            let task = ticker(id, &props, cx);
                            if let Some(KitState::Ticker(running, shape)) = self.states.get_mut(&id) {*running = task; *shape = ticker_shape(&props);}
                        }
                    }
                    KitState::Dock(_) | KitState::Questionnaire(_) | KitState::Speech(_) => {}
                }
                continue;
            }
            let state = match name.as_str() {
                "Select" => {
                    let Some(node) = find(&self.node, id) else {continue};
                    let (loose, groups, shape) = choices(node, node.kit.as_deref().expect("Select"));
                    let value = props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default();
                    let searchable = text("searchable") == "true";
                    let report = move |this: &mut Root, value: &Option<SharedString>, cx: &mut Context<Root>| this.emit(id, "change", value.as_ref().map(ToString::to_string).unwrap_or_default(), cx);
                    if groups.is_empty() {
                        let selected = loose.iter().position(|option| option.value.as_ref() == value).map(IndexPath::new);
                        let state = cx.new(|cx| SelectState::new(SearchableVec::new(loose), selected, window, cx).searchable(searchable));
                        cx.subscribe_in(&state, window, move |this, _, event: &SelectEvent<SearchableVec<Choice>>, _, cx| {let SelectEvent::Confirm(value) = event; report(this, value, cx);}).detach();
                        if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                        KitState::Select(state, shape)
                    } else {
                        let selected = groups.iter().enumerate().find_map(|(section, (_, options))| options.iter().position(|option| option.value.as_ref() == value).map(|row| IndexPath::new(row).section(section)));
                        let mut data = SearchableVec::new(Vec::new());
                        for (title, options) in groups {data.push(SelectGroup::new(title).items(options));}
                        let state = cx.new(|cx| SelectState::new(data, selected, window, cx).searchable(searchable));
                        cx.subscribe_in(&state, window, move |this, _, event: &SelectEvent<SearchableVec<SelectGroup<Choice>>>, _, cx| {let SelectEvent::Confirm(value) = event; report(this, value, cx);}).detach();
                        if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                        KitState::SelectGroups(state, shape)
                    }
                }
                "NumberInput" => {
                    let number=|key:&str|props.get(key).and_then(|value|value.parse::<f64>().ok());
                    let kit=find(&self.node,id).and_then(|node|node.kit.as_deref()).expect("NumberInput");
                    let validator=kit.predicates.get("validate").cloned();let step=kit.number_step.clone();
                    let failure=self.failure.clone();let scope=self.command_owner.context();
                    let state = cx.new(|cx| {
                        let mut state = InputState::new(window, cx).placeholder(text("placeholder"));
                        state = state.step(number("step").unwrap_or(1.));
                        if let Some(predicate)=validator {let failure=failure.clone();let scope=scope.clone();state=state.validate(move|value,_| {let _scope=scope();super::kit::predicate(&predicate,value).unwrap_or_else(|error|fail(&failure,error))});}
                        if let Some(step)=step {state=state.step_by(move|value,action,_| {let _scope=scope();let result=step.try_borrow_mut().map_err(|_|"NumberInput stepBy cannot reenter itself".to_owned()).and_then(|mut step|step(value,matches!(action,gpui_kit::component::input::StepAction::Increment)));result.unwrap_or_else(|error|fail(&failure,error))});}
                        if let Some(min) = number("min") {state = state.min(min);}
                        if let Some(max) = number("max") {state = state.max(max);}
                        if let Some(value) = number("value").or(number("defaultValue")) {state.set_value(value.to_string(), window, cx);}
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, state, event: &InputEvent, _, cx| {
                        if matches!(event, InputEvent::Change) && let Ok(value) = state.read(cx).value().parse::<f64>() {this.emit(id, "change", value.to_string(), cx);}
                    }).detach();
                    KitState::Number(state)
                }
                "OtpInput" => {
                    let length = number("length").map_or(6, |value| value as usize);
                    let initial = props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default();
                    let state = cx.new(|cx| OtpState::new(length, window, cx).masked(text("masked") == "true").default_value(initial));
                    cx.subscribe_in(&state, window, move |this, state, event: &OtpEvent, _, cx| {
                        let value = state.read(cx).value().to_string();
                        match event {
                            OtpEvent::Change => this.emit(id, "change", value, cx),
                            OtpEvent::Complete => {this.emit(id, "change", value.clone(), cx); this.emit(id, "complete", value, cx);}
                            OtpEvent::Focus => this.emit(id, "focus", String::new(), cx),
                            OtpEvent::Blur => this.emit(id, "blur", String::new(), cx),
                        }
                    }).detach();
                    if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                    KitState::Otp(state)
                }
                "DatePicker" => {
                    let range = text("range") == "true";
                    let time = match text("time").as_str() {"minute" => Some(TimePrecision::Minute), "second" => Some(TimePrecision::Second), _ => None};
                    let matcher = date_matcher(&props);
                    let value = props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default();
                    let state = cx.new(|cx| {
                        let mut state = if range {DatePickerState::range(window, cx)} else {DatePickerState::new(window, cx)};
                        if !text("format").is_empty() {state = state.date_format(text("format"));}
                        if let Some(time) = time {state = state.time_precision(time);}
                        if text("twelveHour") == "true" {state = state.hour_cycle(HourCycle::H12);}
                        if let Ok(default_time) = chrono::NaiveTime::parse_from_str(&text("defaultTime"), "%H:%M") {state = state.default_time(default_time);}
                        if let Some(months) = number("months") {state = state.number_of_months(months as usize);}
                        if let Some(first) = weekday(&text("firstDayOfWeek")) {state = state.first_day_of_week(first);}
                        if let Some(matcher) = matcher {state = state.disabled_matcher(matcher);}
                        if let (Some(start), Some(end)) = (number("yearStart"), number("yearEnd")) {state.set_year_range((start as i32, end as i32), cx);}
                        set_picker_value(&mut state, &value, range, time.is_some(), window, cx);
                        state
                    });
                    let timed = time.is_some();
                    cx.subscribe_in(&state, window, move |this, _, event: &DatePickerEvent, _, cx| {
                        let DatePickerEvent::Change(value) = event;
                        this.emit(id, "change", date_time_text(value, timed), cx);
                    }).detach();
                    KitState::Date(state)
                }
                "ColorPicker" => {
                    let initial = theme_color(&props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default(), cx);
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
                    let range = text("range") == "true";
                    let value = props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default();
                    let matcher = date_matcher(&props);
                    let state = cx.new(|cx| {
                        let mut state = CalendarState::new(window, cx);
                        if let Some(matcher) = matcher {state = state.disabled_matcher(matcher);}
                        if let (Some(start), Some(end)) = (number("yearStart"), number("yearEnd")) {state = state.year_range((start as i32, end as i32));}
                        state.set_date(date_value(&value, range), window, cx);
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, _, event: &CalendarEvent, _, cx| {
                        let CalendarEvent::Selected(date) = event;
                        this.emit(id, "change", date_text(date), cx);
                    }).detach();
                    KitState::Calendar(state)
                }
                "TimeField" => {
                    let source = props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default();
                    let initial = chrono::NaiveTime::parse_from_str(&source, "%H:%M:%S").or_else(|_| chrono::NaiveTime::parse_from_str(&source, "%H:%M")).ok();
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
                    if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                    KitState::Time(state)
                }
                "Combobox" => {
                    let Some(node) = find(&self.node, id) else {continue};
                    let (loose, groups, shape) = choices(node, node.kit.as_deref().expect("Combobox"));
                    // Selected values arrive one per line.
                    let chosen = props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default();
                    let chosen = chosen.split('\n').filter(|value| !value.is_empty()).map(str::to_owned).collect::<Vec<_>>();
                    let (multiple, searchable) = (text("multiple") == "true", text("searchable") != "false");
                    let report = move |this: &mut Root, values: &Vec<SharedString>, cx: &mut Context<Root>| this.emit(id, "change", values.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"), cx);
                    if groups.is_empty() {
                        let selected = loose.iter().enumerate().filter(|(_, option)| chosen.iter().any(|value| option.value.as_ref() == value)).map(|(row, _)| IndexPath::new(row)).collect::<Vec<_>>();
                        let state = cx.new(|cx| ComboboxState::new(SearchableVec::new(loose), selected, window, cx).multiple(multiple).searchable(searchable));
                        cx.subscribe_in(&state, window, move |this, _, event: &ComboboxEvent<SearchableVec<Choice>>, _, cx| {
                            if let ComboboxEvent::Change(values) = event {report(this, values, cx);}
                        }).detach();
                        if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                        KitState::Combobox(state, shape)
                    } else {
                        let mut selected = Vec::new();
                        for (section, (_, options)) in groups.iter().enumerate() {
                            for (row, option) in options.iter().enumerate() {if chosen.iter().any(|value| option.value.as_ref() == value) {selected.push(IndexPath::new(row).section(section));}}
                        }
                        let mut data = SearchableVec::new(Vec::new());
                        for (title, options) in groups {data.push(SelectGroup::new(title).items(options));}
                        let state = cx.new(|cx| ComboboxState::new(data, selected, window, cx).multiple(multiple).searchable(searchable));
                        cx.subscribe_in(&state, window, move |this, _, event: &ComboboxEvent<SearchableVec<SelectGroup<Choice>>>, _, cx| {
                            if let ComboboxEvent::Change(values) = event {report(this, values, cx);}
                        }).detach();
                        if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                        KitState::ComboboxGroups(state, shape)
                    }
                }
                "Command" => {
                    let state = cx.new(|cx| CommandState::new(window, cx));
                    if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                    KitState::Command(state)
                }
                "Carousel" => {
                    let count = find(&self.node, id).map(|node| carousel_slides(first_item(node, "CarouselContent").unwrap_or(node)).len()).unwrap_or(0);
                    let looping = text("loop") == "true";
                    let axis = if text("vertical") == "true" {Axis::Vertical} else {Axis::Horizontal};
                    let selected = number("selected").or(number("defaultSelected")).map_or(0, |index| index as usize);
                    let state = cx.new(|_| CarouselState::new(count).with_looping(looping).with_axis(axis).with_selected_index(selected));
                    cx.subscribe_in(&state, window, move |this, _, event: &CarouselEvent, _, cx| {
                        let CarouselEvent::Change(index) = event;
                        this.emit(id, "change", index.to_string(), cx);
                    }).detach();
                    KitState::Carousel(state)
                }
                "AppMenuBar" => {
                    let menus=find(&self.node,id).map(application_menus).unwrap_or_default();
                    if let Err(error)=install_application_menus(&menus,cx) {fail(&self.failure,error);}
                    KitState::AppMenuBar(AppMenuBar::new(cx),menus)
                }
                "CodeEditor" => {
                    let (language,value)=(text("language"),props.get("value").or(props.get("defaultValue")).cloned().unwrap_or_default());
                    let state = cx.new(|cx| {
                        let mut state = configured!(EditorState::new(window, cx).language(super::kit::source_language(&language).to_owned()),&props);
                        editor_options(&mut state,&props,window,cx);
                        state.set_value(value, window, cx);
                        if text("autoFocus")=="true" {state.focus(window,cx);}
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, state, event: &InputEvent, _, cx| {
                        match event {
                            InputEvent::Change => this.emit(id,"change",state.read(cx).value().to_string(),cx),
                            InputEvent::Focus => this.emit(id,"focus",String::new(),cx),
                            InputEvent::Blur => this.emit(id,"blur",String::new(),cx),
                            InputEvent::PressEnter{secondary,..} => this.emit(id,"submit",secondary.to_string(),cx),
                        }
                    }).detach();
                    KitState::Editor(state,props.clone())
                }
                "Diff" => {
                    let files = parse_diff(&text("patch")).unwrap_or_else(|error|fail(&self.failure,format!("Diff patch: {error}")));
                    let state = cx.new(|cx| {let mut state = DiffState::new(files, cx); configure_diff(&mut state,&props,cx); state});
                    KitState::Diff(state, text("patch"))
                }
                "DataTable" => {
                    let Some(node) = find(&self.node, id) else {continue};
                    let grid = Grid::new(node, cx.entity().downgrade());
                    let flag = |key: &str, default: bool| props.get(key).map_or(default, |value| value == "true");
                    let state = cx.new(|cx| TableState::new(grid, window, cx).sortable(flag("sortable", true)).row_selectable(flag("rowSelectable", true))
                        .loop_selection(flag("loopSelection", true)).col_movable(flag("columnMovable", true)).col_resizable(flag("columnResizable", true))
                        .select_on_modifier_click(flag("selectOnModifierClick", false)).col_selectable(flag("columnSelectable", false))
                        .cell_selectable(flag("cellSelectable", false)).row_header(flag("rowHeader", true)));
                    // Rows report their index in `rows`, whatever the sort; columns report their key.
                    cx.subscribe_in(&state, window, move |this, state, event: &TableEvent, _, cx| {
                        let grid = state.read(cx).delegate();
                        let row = |display: usize| grid.order.get(display).map_or(String::new(), usize::to_string);
                        let key = |column: usize| grid.columns.get(column).map_or(String::new(), |column| column.key.to_string());
                        let (event, value) = match event {
                            TableEvent::SelectRow(display) => ("select", row(*display)),
                            TableEvent::DoubleClickedRow(display) => ("activate", row(*display)),
                            TableEvent::SelectColumn(column) => ("selectColumn", key(*column)),
                            TableEvent::SelectCell(display, column) => ("selectCell", format!("{}\n{}", row(*display), key(*column))),
                            TableEvent::DoubleClickedCell(display, column) => ("activateCell", format!("{}\n{}", row(*display), key(*column))),
                            TableEvent::ColumnWidthsChanged(widths) => ("columnWidthsChange", widths.iter().map(|width| f32::from(*width).round().to_string()).collect::<Vec<_>>().join("\n")),
                            TableEvent::RightClickedRow(Some(display)) | TableEvent::RightClickedCell(display, _) => ("rightClick", row(*display)),
                            TableEvent::ClearSelection => ("clearSelection", String::new()),
                            _ => return,
                        };
                        this.emit(id, event, value, cx);
                    }).detach();
                    KitState::Table(state, grid_shape(node))
                }
                "Dock" => {
                    // The layout is built once; afterwards the user owns the arrangement.
                    let vertical = text("vertical") == "true";
                    let panels = find(&self.node, id).map(|node| items(node, "DockPanel").map(|(child, item)| (child.identity.get(), item.text("title"), item.number("size"))).collect::<Vec<_>>()).unwrap_or_default();
                    let (area, _) = DockSkin::dock_area(format!("dock-{id}"), None, window, cx);
                    let root = cx.entity();
                    let mut layout = if vertical {DockLayout::v_split()} else {DockLayout::h_split()};
                    for (node, title, size) in panels {
                        let weak = root.downgrade();
                        let panel = cx.new(|cx| {
                            cx.observe(&root, |_, _, cx| cx.notify()).detach();
                            DockContent {root: weak, node, title: title.into(), focus: cx.focus_handle()}
                        });
                        layout = layout.child(DockLayout::tabs().panel_view(panel_handle(panel), cx), size.map(|size| px(size as f32)));
                    }
                    area.update(cx, |area, cx| area.set_center(layout, window, cx));
                    KitState::Dock(area)
                }
                "Ticker" => {
                    let task = ticker(id, &props, cx);
                    KitState::Ticker(task, ticker_shape(&props))
                }
                "SpeechButton" => {
                    let locale = text("locale");
                    let state = cx.new(|cx| {
                        let state = SpeechState::new(cx);
                        #[cfg(feature = "gpui-speech")]
                        let state = if locale.is_empty() {state} else {state.recognizer(SystemRecognizer::new().locale(locale))};
                        #[cfg(not(feature = "gpui-speech"))]
                        let _ = locale;
                        state
                    });
                    cx.subscribe_in(&state, window, move |this, _, event: &SpeechEvent, _, cx| match event {
                        SpeechEvent::Partial(text) => this.emit(id, "partial", text.to_string(), cx),
                        SpeechEvent::Final(text) => this.emit(id, "final", text.to_string(), cx),
                        SpeechEvent::Error(error) => this.emit(id, "error", error.to_string(), cx),
                        _ => cx.notify(),
                    }).detach();
                    KitState::Speech(state)
                }
                "Questionnaire" => {
                    // Questions are read once; answers then live in the Kit state.
                    let mut questions = Vec::new();
                    let (mut seen, mut duplicate) = (std::collections::HashSet::new(), None);
                    for (child, question) in find(&self.node, id).map(|node| items(node, "Question").collect::<Vec<_>>()).unwrap_or_default() {
                        if !seen.insert(question.text("id")) {duplicate = Some(question.text("id"));}
                        let mut values = std::collections::HashSet::new();
                        let choices = items(child, "QuestionChoice").map(|(_, choice)| {
                            if !values.insert(choice.text("id")) {duplicate = Some(format!("{}/{}", question.text("id"), choice.text("id")));}
                            let definition = QuestionnaireChoiceDefinition::new(choice.text("id"), choice.text("label"));
                            if choice.text("description").is_empty() {definition} else {definition.with_description(choice.text("description"))}
                        }).collect::<Vec<_>>();
                        let mut definition = QuestionnaireItemDefinition::new(question.text("id"), question.text("title"))
                            .with_required(question.flag("required")).with_multiple(question.flag("multiple")).with_choices(choices);
                        if !question.text("description").is_empty() {definition = definition.with_description(question.text("description"));}
                        questions.push(definition);
                    }
                    if let Some(duplicate) = duplicate {
                        fail(&self.failure, format!("Questionnaire ids must be unique; {duplicate} repeats"));
                    }
                    let state = cx.new(|cx| QuestionnaireState::new(questions, cx).expect("question and choice ids are unique"));
                    cx.subscribe_in(&state, window, move |this, _, event: &QuestionnaireEvent, _, cx| {
                        if let QuestionnaireEvent::Submit(submission) = event {
                            let answers = submission.items().iter().map(|item| (item.name().to_string(), serde_json::Value::from(item.answer().choices().iter().map(ToString::to_string).collect::<Vec<_>>()))).collect::<serde_json::Map<_, _>>();
                            this.emit(id, "submit", serde_json::Value::Object(answers).to_string(), cx);
                        }
                    }).detach();
                    KitState::Questionnaire(state)
                }
                "MessageScroller" => {
                    let count = find(&self.node, id).map(|node| node.visible_children().count()).unwrap_or(0);
                    KitState::Scroller(cx.new(|cx| MessageScrollerState::new(count, cx)), count)
                }
                "Tree" => {
                    let items = find(&self.node, id).map(tree_items).unwrap_or_default();
                    let state = cx.new(|cx| TreeState::new(cx).items(items));
                    if let Some(selected) = props.get("selected").or(props.get("defaultSelected")).cloned() {
                        state.update(cx, |state, cx| tree_select(state, &selected.into(), cx));
                    }
                    cx.subscribe_in(&state, window, move |this, _, event: &TreeEvent, _, cx| match event {
                        TreeEvent::Expanded(item) => this.emit(id, "expand", item.to_string(), cx),
                        TreeEvent::Collapsed(item) => this.emit(id, "collapse", item.to_string(), cx),
                    }).detach();
                    if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                    KitState::Tree(state, tree_shape(&self.node, id))
                }
                "List" => {
                    let Some(node) = find(&self.node, id) else {continue};
                    let delegate = TsrustList {
                        root: cx.entity().downgrade(), list: id, sections: list_sections(node), query: String::new(),
                        empty: first_item(node, "ListEmpty").map(|empty| empty.identity.get()), loading: text("loading") == "true",
                        has_more: text("hasMore") == "true", threshold: number("loadMoreThreshold").map_or(20, |rows| rows as usize), selected: None, values: list_values(node),
                    };
                    let state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(text("searchable") == "true").selectable(text("selectable") != "false"));
                    let selected = number("selected").or(number("defaultSelected")).filter(|index| *index >= 0.).map(|index| index as usize);
                    if let Some(index) = selected.and_then(|index| list_path(&state.read(cx).delegate().sections, index)) {state.update(cx, |state, cx| state.set_selected_index(Some(index), window, cx));}
                    // Select follows the highlight (arrow keys, hover); Confirm is a click or Enter; Cancel is Escape.
                    cx.subscribe_in(&state, window, move |this, state, event: &ListEvent, _, cx| {
                        let report = |index: &IndexPath, cx: &App| list_value(state.read(cx).delegate(), *index);
                        // Disabled and separator rows never report.
                        let inert = |index: &IndexPath, cx: &App| state.read(cx).delegate().row(*index).is_none_or(|row| row.disabled || row.separator);
                        match event {
                            ListEvent::Select(index) | ListEvent::Confirm(index) if inert(index, cx) => {}
                            ListEvent::Select(index) => {let value = report(index, cx); this.emit(id, "select", value, cx);}
                            ListEvent::Confirm(index) => {let value = report(index, cx); this.emit(id, "confirm", value, cx);}
                            ListEvent::Cancel => this.emit(id, "cancel", String::new(), cx),
                        }
                    }).detach();
                    if text("autoFocus") == "true" {state.update(cx, |state, cx| state.focus(window, cx));}
                    KitState::List(state)
                }
                _ => {
                    let (min, max, step) = (number("min").unwrap_or(0.), number("max").unwrap_or(100.), number("step").unwrap_or(1.));
                    let start = number("value").or(number("defaultValue")).unwrap_or(min);
                    // A range slider has two thumbs: value is the start, endValue the end.
                    let initial = if text("range") == "true" {SliderValue::Range(start, number("endValue").or(number("defaultEndValue")).unwrap_or(max))} else {SliderValue::Single(start)};
                    let scale = if text("scale") == "logarithmic" {gpui_kit::component::slider::SliderScale::Logarithmic} else {gpui_kit::component::slider::SliderScale::Linear};
                    let state = cx.new(|_| SliderState::new().min(min).max(max).step(step).scale(scale).default_value(initial));
                    cx.subscribe_in(&state, window, move |this, _, event: &SliderEvent, _, cx| {
                        let (name, value) = match event {SliderEvent::Change(value) => ("change", *value), SliderEvent::Release(value) => ("commit", *value)};
                        match value {
                            SliderValue::Single(value) => this.emit(id, name, value.to_string(), cx),
                            SliderValue::Range(start, end) => {this.emit(id, name, start.to_string(), cx); if name == "change" {this.emit(id, "endChange", end.to_string(), cx);}}
                        }
                    }).detach();
                    KitState::Slider(state)
                }
            };
            self.states.insert(id, state);
        }
        let root = cx.entity().downgrade();
        match (dialog, self.dialog) {
            (Some(id), current) if current != Some(id) && find(&self.node, id).and_then(|node| node.kit.as_deref()).is_some_and(|kit| kit.name == "AlertDialog") => {
                self.dialog = Some(id);
                let root = root.clone();
                window.open_alert_dialog(cx, move |alert, _, cx| {
                    let Some((props, handles_ok, handles_cancel)) = root.upgrade().and_then(|root| find(&root.read(cx).node, id).and_then(|node| node.kit.as_deref()).map(|kit| (kit.props.clone(), kit.handles("ok"), kit.handles("cancel")))) else {return alert};
                    let text = |key: &str| props.get(key).cloned().unwrap_or_default();
                    let number = |key: &str| props.get(key).and_then(|value| value.parse::<f32>().ok());
                    let variant = |key: &str| match text(key).as_str() {
                        "primary" => Some(ButtonVariant::Primary), "secondary" => Some(ButtonVariant::Secondary), "danger" => Some(ButtonVariant::Danger), "info" => Some(ButtonVariant::Info),
                        "success" => Some(ButtonVariant::Success), "warning" => Some(ButtonVariant::Warning), "ghost" => Some(ButtonVariant::Ghost), "link" => Some(ButtonVariant::Link), "text" => Some(ButtonVariant::Text), "default" => Some(ButtonVariant::Default), _ => None,
                    };
                    let mut alert = alert.close_button(text("closeButton") == "true").keyboard(text("keyboard") != "false");
                    if !text("okText").is_empty() {alert = alert.ok_text(text("okText"));}
                    if !text("cancelText").is_empty() {alert = alert.cancel_text(text("cancelText")).show_cancel(true);}
                    if text("confirm") == "true" {alert = alert.confirm();}
                    if let Some(variant) = variant("okVariant") {alert = alert.ok_variant(variant);}
                    if let Some(variant) = variant("cancelVariant") {alert = alert.cancel_variant(variant);}
                    if let Some(width) = number("width") {alert = alert.width(px(width));}
                    if let Some(top) = number("marginTop") {alert = alert.margin_top(px(top));}
                    if text("centered") == "true" {alert = alert.centered();}
                    if text("entrance") == "fade" {alert = alert.entrance(gpui_kit::component::dialog::DialogEntrance::Fade);}
                    if let Some(icon) = icon_name(&text("icon")) {alert = alert.icon(Icon::new(icon).size_6());}
                    if let Ok(Some(parts)) = root.update(cx, |root, cx| root.overlay_parts(id, cx)) {
                        let OverlayParts {title, description, body, ..} = parts;
                        if let Some(title) = title {alert = alert.title(div().children(title));}
                        if let Some(description) = description {alert = alert.description(div().children(description));}
                        if !body.is_empty() {alert = alert.child(div().flex().flex_col().gap_2().children(body));}
                    }
                    let close = root.clone();
                    alert = alert.on_close(move |_, _, cx| {let _ = close.update(cx, |root, cx| {root.dialog = None; root.emit(id, "close", String::new(), cx); root.emit(id, "openChange", "false".into(), cx)});});
                    let confirm = root.clone();
                    alert = alert.on_ok(move |_, _, cx| {if handles_ok {let _ = confirm.update(cx, |root, cx| root.emit(id, "ok", String::new(), cx));} true});
                    let cancel = root.clone();
                    alert.on_cancel(move |_, _, cx| {if handles_cancel {let _ = cancel.update(cx, |root, cx| root.emit(id, "cancel", String::new(), cx));} true})
                });
            }
            (Some(id), current) if current != Some(id) => {
                self.dialog = Some(id);
                let root = root.clone();
                window.open_dialog(cx, move |dialog, _, cx| {
                    let Some(kit) = root.upgrade().and_then(|root| find(&root.read(cx).node, id).and_then(|node| node.kit.as_deref()).map(|kit| (kit.props.clone(), kit.handles("ok"), kit.handles("cancel")))) else {return dialog};
                    let (props, handles_ok, handles_cancel) = kit;
                    let text = |key: &str| props.get(key).cloned().unwrap_or_default();
                    let number = |key: &str| props.get(key).and_then(|value| value.parse::<f32>().ok());
                    let mut dialog = dialog.close_button(text("closeButton") != "false").overlay(text("overlay") != "false")
                        .overlay_closable(text("overlayClosable") != "false").keyboard(text("keyboard") != "false");
                    if let Some(width) = number("width") {dialog = dialog.width(px(width));}
                    if let Some(width) = number("maxWidth") {dialog = dialog.max_w(px(width));}
                    if let Some(top) = number("marginTop") {dialog = dialog.margin_top(px(top));}
                    if text("centered") == "true" {dialog = dialog.centered();}
                    if text("entrance") == "fade" {dialog = dialog.entrance(gpui_kit::component::dialog::DialogEntrance::Fade);}
                    let close = root.clone();
                    dialog = dialog.on_close(move |_, _, cx| {let _ = close.update(cx, |root, cx| {root.dialog = None; root.emit(id, "close", String::new(), cx); root.emit(id, "openChange", "false".into(), cx)});});
                    // The okText/cancelText shorthand draws Kit's own buttons.
                    let (ok, cancel) = (text("okText"), text("cancelText"));
                    if !ok.is_empty() || !cancel.is_empty() {
                        let props = gpui_kit::component::dialog::DialogButtonProps::default().ok_text(if ok.is_empty() {"OK".to_owned()} else {ok.clone()}).cancel_text(if cancel.is_empty() {"Cancel".to_owned()} else {cancel.clone()}).show_cancel(!cancel.is_empty());
                        dialog = dialog.button_props(props);
                        if handles_ok {let confirm = root.clone(); dialog = dialog.on_ok(move |_, _, cx| {let _ = confirm.update(cx, |root, cx| root.emit(id, "ok", String::new(), cx)); true});}
                        if handles_cancel {let cancel = root.clone(); dialog = dialog.on_cancel(move |_, _, cx| {let _ = cancel.update(cx, |root, cx| root.emit(id, "cancel", String::new(), cx)); true});}
                    }
                    let parts = root.clone();
                    let (ok, cancel) = (ok.clone(), cancel.clone());
                    let cancelled = root.clone();
                    dialog.content(move |content, _, cx| {
                        let Ok(Some(parts)) = parts.update(cx, |root, cx| root.overlay_parts(id, cx)) else {return content};
                        let OverlayParts {title, description, body, footer} = parts;
                        let mut content = content;
                        if title.is_some() || description.is_some() {
                            let mut header = gpui_kit::component::dialog::DialogHeader::new();
                            if let Some(title) = title {header = header.child(gpui_kit::component::dialog::DialogTitle::new().children(title));}
                            if let Some(description) = description {header = header.child(gpui_kit::component::dialog::DialogDescription::new().children(description));}
                            content = content.child(header);
                        }
                        content = content.child(div().flex().flex_col().gap_3().children(body));
                        match footer {
                            Some(footer) => content.child(gpui_kit::component::dialog::DialogFooter::new().children(footer)),
                            // The okText/cancelText shorthand: Kit's close and action buttons.
                            None if !ok.is_empty() || !cancel.is_empty() => {
                                let mut footer = gpui_kit::component::dialog::DialogFooter::new();
                                if !cancel.is_empty() {
                                    let cancelled = cancelled.clone();
                                    let button = gpui_kit::component::button::Button::new("tsrust-dialog-cancel").outline().label(cancel.clone())
                                        .on_click(move |_, _, cx| {let _ = cancelled.update(cx, |root, cx| root.emit(id, "cancel", String::new(), cx));});
                                    footer = footer.child(gpui_kit::component::dialog::DialogClose::new().child(button));
                                }
                                if !ok.is_empty() {footer = footer.child(gpui_kit::component::dialog::DialogAction::new().child(gpui_kit::component::button::Button::new("tsrust-dialog-ok").primary().label(ok.clone())));}
                                content.child(footer)
                            }
                            None => content,
                        }
                    })
                });
            }
            (None, Some(_)) => {self.dialog = None; window.close_dialog(cx);}
            _ => {}
        }
        match (sheet, self.sheet) {
            (Some((id, side)), current) if current != Some(id) => {
                self.sheet = Some(id);
                let placement = match side.as_str() {"left" => Placement::Left, "top" => Placement::Top, "bottom" => Placement::Bottom, _ => Placement::Right};
                window.open_sheet_at(placement, cx, move |sheet, _, cx| {
                    let Ok(Some(parts)) = root.update(cx, |root, cx| root.overlay_parts(id, cx)) else {return sheet};
                    let OverlayParts {title, description, body, footer} = parts;
                    let props = root.upgrade().and_then(|root| find(&root.read(cx).node, id).and_then(|node| node.kit.as_deref()).map(|kit| kit.props.clone())).unwrap_or_default();
                    let number = |key: &str| props.get(key).and_then(|value| value.parse::<f32>().ok());
                    let close = root.clone();
                    let mut sheet = sheet.on_close(move |_, _, cx| {let _ = close.update(cx, |root, cx| {root.sheet = None; root.emit(id, "close", String::new(), cx); root.emit(id, "openChange", "false".into(), cx)});})
                        .overlay(props.get("overlay").map(String::as_str) != Some("false")).overlay_closable(props.get("overlayClosable").map(String::as_str) != Some("false"));
                    if let Some(size) = number("size") {sheet = sheet.size(px(size));}
                    if props.get("resizable").map(String::as_str) == Some("true") {sheet = sheet.resizable(true);}
                    if let Some(title) = title {
                        let muted = cx.theme().muted_foreground;
                        let description = description.map(|description| div().text_sm().font_weight(FontWeight::NORMAL).text_color(muted).children(description));
                        sheet = sheet.title(div().flex().flex_col().gap_1().children(title).children(description));
                    }
                    if let Some(footer) = footer {sheet = sheet.footer(div().flex().justify_end().gap_2().children(footer));}
                    sheet.child(div().flex().flex_col().gap_3().children(body))
                });
            }
            (None, Some(_)) => {self.sheet = None; window.close_sheet(cx);}
            _ => {}
        }
    }

    /// A Dialog or Sheet's parts: DialogContent (or the node itself) holds DialogHeader
    /// (DialogTitle, DialogDescription), body content and DialogFooter, whose DialogClose
    /// children close the overlay. The title prop stands in for DialogTitle.
    fn overlay_parts(&self, id: u64, cx: &mut Context<Self>) -> Option<OverlayParts> {
        let node = find(&self.node, id)?;
        let kit = node.kit.as_deref()?;
        let content = first_part(node, "Content").unwrap_or(node);
        let header = first_part(content, "Header").or_else(|| first_part(node, "Header"));
        let title = header.and_then(|header| first_part(header, "Title")).map(|title| self.content(title, cx))
            .or_else(|| (!kit.text("title").is_empty()).then(|| vec![SharedString::from(kit.text("title")).into_any_element()]));
        let description = header.and_then(|header| first_part(header, "Description")).map(|description| self.content(description, cx))
            .or_else(|| (!kit.text("description").is_empty()).then(|| vec![SharedString::from(kit.text("description")).into_any_element()]));
        let body = self.content_except(content, &["DialogTrigger", "DialogHeader", "DialogFooter", "DialogContent", "SheetTrigger", "SheetHeader", "SheetFooter", "SheetContent", "AlertDialogTrigger", "AlertDialogHeader", "AlertDialogContent"], cx);
        let footer = first_part(content, "Footer").map(|footer| {
            let theme = utility_theme(cx);
            footer.visible_children().map(|child| match child.kit.as_deref().map(|kit| kit.name.as_str()) {
                Some("DialogClose" | "SheetClose") => gpui_kit::component::dialog::DialogClose::new().children(self.content(child, cx)).into_any_element(),
                Some("DialogAction") => gpui_kit::component::dialog::DialogAction::new().children(self.content(child, cx)).into_any_element(),
                _ => self.element(child, &theme, mode(cx), cx),
            }).collect::<Vec<_>>()
        });
        Some(OverlayParts {title, description, body, footer})
    }

    /// Run queued application commands; replies refresh the tree like handlers.
    fn run_commands(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let _scope=self.scope();
        commands::set_dark(matches!(window.appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark));
        // GPUI Kit's own strings (calendar months, placeholders) follow the app locale.
        if super::i18n::take_changed() {gpui_kit::component::set_locale(&super::i18n::locale());}
        for command in commands::take(&self.command_owner) {
            match command {
                Command::Plain {name, payload} => {
                    let value: serde_json::Value = serde_json::from_str(&payload).unwrap_or_default();
                    let text = |key: &str| value.get(key).and_then(|value| value.as_str()).unwrap_or_default().to_owned();
                    match name.as_str() {
                        "notify" => {let note = notification(&value, None); window.push_notification(note, cx);}
                        "dismiss-notification" => window.remove_notification1::<TsrustNote>(ElementId::Name(text("id").into()), cx),
                        "clear-notifications" => window.clear_notifications(cx),
                        "set-title" => window.set_window_title(&text("title")),
                        "register-themes" => if let Err(error) = gpui_kit::component::ThemeRegistry::global_mut(cx).load_themes_from_str(&text("json")) {
                            window.push_notification(Notification::error(format!("Invalid themes: {error}")), cx);
                        },
                        "use-theme" => match gpui_kit::component::ThemeRegistry::global(cx).themes().get(text("name").as_str()).cloned() {
                            Some(config) => {
                                let mode = config.mode;
                                KitTheme::global_mut(cx).apply_config(&config);
                                KitTheme::change(mode, Some(window), cx);
                            }
                            None => window.push_notification(Notification::error(format!("Unknown theme {}", text("name"))), cx),
                        },
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
                        "input" => self.control_input(&value, window, cx),
                        "relaunch" => match super::update::relaunch() {Ok(()) => cx.quit(), Err(error) => window.push_notification(Notification::error(error), cx)},
                        _ => cx.write_to_clipboard(ClipboardItem::new_string(text("text"))),
                    }
                }
                Command::Reply {name, payload, reply} => {
                    let value: serde_json::Value = serde_json::from_str(&payload).unwrap_or_default();
                    let flag = |key: &str| value.get(key).and_then(|value| value.as_bool()).unwrap_or(false);
                    if name == "notify-events" {
                        // The first of the action button, a click or the close reaches the reply.
                        let reply = std::rc::Rc::new(std::cell::RefCell::new(Some(reply)));
                        let note = notification(&value, Some((cx.entity().downgrade(), reply)));
                        window.push_notification(note, cx);
                        continue;
                    }
                    if name == "read-input" {
                        let snapshot = self.read_input(value.get("id").and_then(|id| id.as_str()).unwrap_or_default(), window, cx);
                        self.reply(reply, &snapshot, cx);
                        continue;
                    }
                    if name == "read-clipboard" {
                        let text = cx.read_from_clipboard().and_then(|item| item.text()).unwrap_or_default();
                        self.reply(reply, &text, cx);
                        continue;
                    }
                    if name == "check-update" || name == "install-update" {
                        // Network and file work runs off the UI thread; the reply lands like a dialog's.
                        let work = cx.background_executor().spawn(async move {if name == "check-update" {super::update::check()} else {super::update::install()}});
                        cx.spawn(async move |this, cx| {
                            let outcome = work.await;
                            let _ = this.update(cx, |root, cx| {root.reply(reply, &outcome, cx);});
                        }).detach();
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
                        let _ = this.update(cx, |root, cx| {root.reply(reply, &chosen, cx);});
                    }).detach();
                }
                Command::Window {mut node, title, options} => {
                    let result = node.refresh().and_then(|_| {node.ensure_identities(); Options::parse(&options)}).and_then(|options| open(*node, title, options, self.failure.clone(), cx));
                    if let Err(error) = result {fail(&self.failure, error);}
                }
            }
        }
    }

    fn kit_element(&self, node: &Node, kit: &Kit, cx: &mut Context<Self>) -> AnyElement {
        let id = node.identity.get();
        let element_id = ElementId::Integer(id);
        let text = |name: &str| SharedString::from(kit.text(name));
        // className refines Styled Kit components directly (see STYLED_KIT); others get a wrapper.
        let style = self.node_style(node);
        let root = cx.entity().downgrade();
        // A plain event handler for Kit callbacks that run with &mut App.
        let event = |event: &'static str| {
            let root = root.clone();
            move |value: String, cx: &mut App| {let _ = root.update(cx, |root, cx| root.emit(id, event, value, cx));}
        };
        match kit.name.as_str() {
            "Icon" => {
                let icon = match (kit.prop("svg").filter(|svg| !svg.is_empty()), icon_name(&kit.text("name"))) {
                    (Some(svg), _) => Icon::empty().data(svg.as_bytes()),
                    (None, Some(name)) => Icon::new(name),
                    (None, None) => return div().into_any_element(),
                };
                let size = match kit.prop("size") {Some("xs") => KitSize::XSmall, Some("sm") => KitSize::Small, Some("lg") => KitSize::Large, _ => KitSize::Medium};
                let icon = icon.with_size(size);
                match kit.number("rotate") {Some(degrees) if degrees != 0. => icon.rotate(radians((degrees as f32).to_radians())).into_any_element(), _ => icon.into_any_element()}
            }
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
                if let Some(icon) = kit.prop("icon").and_then(icon_name) {badge = badge.icon(icon);}
                if let Some(color) = kit.prop("color").and_then(|color| theme_color(color, cx)) {badge = badge.color(color);}
                if let Some(size) = kit_size(kit) {badge = badge.with_size(size);}
                badge.children(self.content(node, cx)).into_any_element()
            }
            "Tag" => {
                let custom = kit.prop("color").and_then(|color| theme_color(color, cx));
                let mut tag = match (custom, kit.prop("palette").and_then(|name| ColorName::try_from(name).ok())) {
                    (Some(color), _) => {
                        let foreground = kit.prop("foreground").and_then(|color| theme_color(color, cx)).unwrap_or(cx.theme().foreground);
                        Tag::custom(color, foreground, kit.prop("borderColor").and_then(|color| theme_color(color, cx)).unwrap_or(color))
                    }
                    (None, Some(name)) => Tag::color(name),
                    _ => match kit.prop("variant") {
                        Some("primary") => Tag::primary(), Some("danger") => Tag::danger(), Some("success") => Tag::success(),
                        Some("warning") => Tag::warning(), Some("info") => Tag::info(), _ => Tag::secondary(),
                    },
                };
                if kit.flag("outline") {tag = tag.outline();}
                if kit.flag("roundedFull") {tag = tag.rounded_full();} else if let Some(radius) = kit.number("rounded") {tag = tag.rounded(px(radius as f32));}
                if let Some(size) = kit_size(kit) {tag = tag.with_size(size);}
                tag.refine_style(&style).children(self.content(node, cx)).into_any_element()
            }
            "Label" => {
                let mut label = Label::new(text("text")).secondary(text("secondary")).masked(kit.flag("masked"));
                if let Some(highlight) = kit.prop("highlight").filter(|highlight| !highlight.is_empty()) {
                    let highlight = SharedString::from(highlight.to_owned());
                    label = label.highlights(if kit.flag("highlightPrefix") {HighlightsMatch::Prefix(highlight)} else {HighlightsMatch::Full(highlight)});
                }
                label.refine_style(&style).into_any_element()
            }
            "Kbd" => {
                let keys = match kit.prop("command").filter(|command| !command.is_empty()) {
                    Some(command) => COMMAND_KEYS.with(|bound| bound.borrow().get(command).cloned()).unwrap_or_default(),
                    None => kit.text("keys"),
                };
                match keystroke(&keys) {
                    Some(keys) => {
                        let kbd = Kbd::new(keys).appearance(!kit.flag("plain")).refine_style(&style);
                        if kit.flag("outline") {kbd.outline().into_any_element()} else {kbd.into_any_element()}
                    }
                    None => div().child(SharedString::from(keys)).into_any_element(),
                }
            }
            "Separator" => {
                let separator = if kit.prop("orientation") == Some("vertical") {Separator::vertical()} else {Separator::horizontal()};
                let mut separator = (if kit.flag("dashed") {separator.dashed()} else {separator}).refine_style(&style);
                if let Some(color) = kit.prop("color").and_then(|color| theme_color(color, cx)) {separator = separator.color(color);}
                match kit.prop("label") {Some(label) if !label.is_empty() => separator.label(label.to_owned()).into_any_element(), _ => separator.into_any_element()}
            }
            "Skeleton" => if kit.flag("secondary") {Skeleton::new().secondary().refine_style(&style).into_any_element()} else {Skeleton::new().refine_style(&style).into_any_element()},
            "Spinner" => {
                let mut spinner = Spinner::new();
                if let Some(icon) = kit.prop("icon").and_then(icon_name) {spinner = spinner.icon(icon);}
                if let Some(color) = kit.prop("color").and_then(|color| theme_color(color, cx)) {spinner = spinner.color(color);}
                let easing: Box<dyn Fn(f32) -> f32> = match kit.prop("easing") {
                    Some("ease-in-out") => Box::new(ease_in_out), Some("quadratic") => Box::new(quadratic),
                    Some("ease-out-quint") => Box::new(ease_out_quint()), _ => Box::new(linear),
                };
                spinner = spinner.ease(easing);
                if let Some(size) = kit_size(kit) {spinner = spinner.with_size(size);}
                spinner.into_any_element()
            }
            "Progress" => {
                let mut progress = Progress::new(element_id).value(kit.number("value").unwrap_or_default() as f32).loading(kit.flag("loading"));
                if let Some(color) = kit.prop("color").and_then(|color| theme_color(color, cx)) {progress = progress.color(color);}
                if let Some(label) = kit.prop("accessibilityLabel") {progress = progress.accessibility_label(label.to_owned());}
                if let Some(size) = kit_size(kit) {progress = progress.with_size(size);}
                progress.refine_style(&style).into_any_element()
            }
            "ProgressCircle" => {
                let mut progress = ProgressCircle::new(element_id).value(kit.number("value").unwrap_or_default() as f32).loading(kit.flag("loading"));
                if let Some(color) = kit.prop("color").and_then(|color| theme_color(color, cx)) {progress = progress.color(color);}
                if let Some(label) = kit.prop("accessibilityLabel") {progress = progress.accessibility_label(label.to_owned());}
                if let Some(size) = kit_size(kit) {progress = progress.with_size(size);}
                progress.refine_style(&style).children(self.content(node, cx)).into_any_element()
            }
            "Avatar" => Self::kit_avatar(kit).refine_style(&style).into_any_element(),
            "AvatarGroup" => {
                let mut group = AvatarGroup::new().children(items(node, "Avatar").map(|(_, avatar)| Self::kit_avatar(avatar)));
                if let Some(limit) = kit.index("limit") {group = group.limit(limit);}
                if kit.flag("ellipsis") {group = group.ellipsis();}
                if let Some(size) = kit_size(kit) {group = group.with_size(size);}
                group.refine_style(&style).into_any_element()
            }
            "Alert" => {
                // shadcn-style AlertTitle and AlertDescription children win over the title and message props.
                let part = |name: &str| items(node, name).next().map(|(_, item)| item.text("text"));
                let message = SharedString::from(part("AlertDescription").unwrap_or_else(|| kit.text("message")));
                let mut alert = match kit.prop("variant") {
                    Some("default") => Alert::new(element_id, message),
                    Some("success") => Alert::success(element_id, message), Some("warning") => Alert::warning(element_id, message),
                    Some("error") => Alert::error(element_id, message), _ => Alert::info(element_id, message),
                };
                if let Some(title) = part("AlertTitle").or_else(|| kit.prop("title").map(str::to_owned)).filter(|title| !title.is_empty()) {alert = alert.title(title);}
                if let Some(icon) = kit.prop("icon").and_then(icon_name) {alert = alert.icon(icon);}
                if kit.flag("banner") {alert = alert.banner();}
                if kit.prop("visible") == Some("false") {alert = alert.visible(false);}
                if let Some(size) = kit_size(kit) {alert = alert.with_size(size);}
                if kit.handles("close") {let close = event("close"); alert = alert.on_close(move |_, _, cx| close(String::new(), cx));}
                alert.refine_style(&style).into_any_element()
            }
            "ShimmerText" => {
                let mut shimmer = ShimmerText::new(text("text")).id(element_id).reverse(kit.flag("reverse")).once(kit.flag("once"));
                if let Some(duration) = kit.index("duration") {shimmer = shimmer.duration(std::time::Duration::from_millis(duration as u64));}
                if let Some(color) = kit.prop("highlightColor").and_then(|color| theme_color(color, cx)) {shimmer = shimmer.highlight_color(color);}
                if let Some(spread) = kit.number("spread") {shimmer = shimmer.spread(spread as f32);}
                shimmer.refine_style(&style).into_any_element()
            }
            "Empty" => {
                // shadcn composition: EmptyHeader holds EmptyMedia, EmptyTitle and EmptyDescription; EmptyContent holds actions.
                // The title, description and icon props cover the short form.
                let parts = items(node, "EmptyHeader").next().map_or(node, |(header, _)| header);
                let mut header = EmptyHeader::new();
                match items(parts, "EmptyTitle").next() {
                    Some((title, _)) => header = header.title(EmptyTitle::new().children(self.content(title, cx))),
                    None if !kit.text("title").is_empty() => header = header.title(EmptyTitle::new().child(text("title"))),
                    None => {}
                }
                match items(parts, "EmptyDescription").next() {
                    Some((description, _)) => header = header.description(EmptyDescription::new().children(self.content(description, cx))),
                    None if !kit.text("description").is_empty() => header = header.description(EmptyDescription::new().child(text("description"))),
                    None => {}
                }
                if let Some((media, item)) = items(parts, "EmptyMedia").next() {
                    let variant = if item.flag("framed") || item.prop("variant") == Some("icon") {EmptyMediaVariant::Icon} else {EmptyMediaVariant::Default};
                    header = header.media(EmptyMedia::new().with_variant(variant).children(self.content(media, cx)));
                } else if let Some(icon) = kit.prop("icon").and_then(icon_name) {
                    header = header.media(EmptyMedia::new().with_variant(EmptyMediaVariant::Icon).child(Icon::new(icon)));
                }
                let content = match items(node, "EmptyContent").next() {Some((content, _)) => self.content(content, cx), None => self.content_except(node, &["EmptyMedia", "EmptyHeader", "EmptyContent"], cx)};
                Empty::new().refine_style(&style).header(header).content(EmptyContent::new().children(content)).into_any_element()
            }
            "GroupBox" => {
                let variant = match kit.prop("variant") {Some("fill") => GroupBoxVariant::Fill, Some("outline") => GroupBoxVariant::Outline, _ => GroupBoxVariant::Normal};
                let mut group = GroupBox::new().id(element_id).with_variant(variant)
                    .title_style(self.class_style(&kit.text("titleClassName"), cx)).content_style(self.class_style(&kit.text("contentClassName"), cx));
                let title = kit.text("title");
                if !title.is_empty() {group = group.title(SharedString::from(title));}
                if let Some((footer, _)) = items(node, "GroupBoxFooter").next() {group = group.footer(div().flex().gap_2().children(self.content(footer, cx)));}
                group.refine_style(&style).children(self.content_except(node, &["GroupBoxFooter"], cx)).into_any_element()
            }
            "DescriptionList" => {
                let mut list = if kit.flag("vertical") {DescriptionList::vertical()} else {DescriptionList::horizontal()};
                list = list.columns(kit.index("columns").unwrap_or(3)).bordered(kit.flag("bordered"));
                if let Some(width) = kit.number("labelWidth") {list = list.label_width(px(width as f32));}
                if let Some(size) = kit_size(kit) {list = list.with_size(size);}
                for child in node.visible_children() {
                    match child.kit.as_deref().map(|kit| kit.name.as_str()) {
                        Some("DescriptionSeparator") => list = list.separator(),
                        Some("DescriptionItem") => {
                            let item = child.kit.as_deref().expect("item");
                            // Rich children (anything beyond text) render as the value.
                            let rich = child.visible_children().any(|part| part.kind != Kind::Text);
                            let value: gpui_kit::component::description_list::DescriptionText = if rich {div().flex().items_center().gap_2().children(self.content(child, cx)).into_any_element().into()} else {SharedString::from(item.text("value")).into()};
                            list = list.child(gpui_kit::component::description_list::DescriptionItem::new(SharedString::from(item.text("label"))).value(value).span(item.index("span").unwrap_or(1)));
                        }
                        _ => {}
                    }
                }
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
            "Clipboard" => {
                let mut clipboard = Clipboard::new(element_id).value(text("value"));
                if let Some(tooltip) = kit.prop("tooltip").filter(|tooltip| !tooltip.is_empty()) {clipboard = clipboard.tooltip(tooltip.to_owned());}
                if let Some(delay) = kit.index("tooltipDelay") {clipboard = clipboard.tooltip_show_delay(std::time::Duration::from_millis(delay as u64));}
                if let Some(label) = kit.prop("accessibilityLabel") {clipboard = clipboard.accessibility_label(label.to_owned());}
                if let Some(size) = kit_size(kit) {clipboard = clipboard.with_size(size);}
                if kit.handles("copied") {let copied = event("copied"); clipboard = clipboard.on_copied(move |value, _, cx| copied(value.to_string(), cx));}
                clipboard.into_any_element()
            }
            "Checkbox" => {
                let mut checkbox = Checkbox::new(element_id).checked(checked(kit)).disabled(kit.flag("disabled"));
                if let Some(label) = kit.prop("label") {checkbox = checkbox.label(label.to_owned());}
                checkbox = checkbox.tab_stop(kit.prop("tabStop") != Some("false"));
                if let Some(tooltip) = kit.prop("tooltip").filter(|tooltip| !tooltip.is_empty()) {checkbox = checkbox.tooltip(tooltip.to_owned());}
                if let Some(delay) = kit.index("tooltipDelay") {checkbox = checkbox.tooltip_show_delay(std::time::Duration::from_millis(delay as u64));}
                if let Some(label) = kit.prop("accessibilityLabel") {checkbox = checkbox.accessibility_label(label.to_owned());}
                if let Some(index) = kit.number("tabIndex") {checkbox = checkbox.tab_index(index as isize);}
                if let Some(size) = kit_size(kit) {checkbox = checkbox.with_size(size);}
                match kit.prop("role") {
                    Some("switch") => checkbox = checkbox.role(Role::Switch), Some("menuitemcheckbox") => checkbox = checkbox.role(Role::MenuItemCheckBox),
                    Some("none") => checkbox = checkbox.role(None::<Role>), _ => {}
                }
                {checkbox = checkbox.on_change(cx.listener(move |this, checked: &bool, _, cx| this.emit(id, "change", checked.to_string(), cx)));}
                checkbox.into_any_element()
            }
            "Switch" => {
                let mut switch = Switch::new(element_id).checked(checked(kit)).disabled(kit.flag("disabled"));
                if let Some(label) = kit.prop("label") {switch = switch.label(label.to_owned());}
                switch = switch.tab_stop(kit.prop("tabStop") != Some("false"));
                if let Some(tooltip) = kit.prop("tooltip").filter(|tooltip| !tooltip.is_empty()) {switch = switch.tooltip(tooltip.to_owned());}
                if let Some(delay) = kit.index("tooltipDelay") {switch = switch.tooltip_show_delay(std::time::Duration::from_millis(delay as u64));}
                if let Some(label) = kit.prop("accessibilityLabel") {switch = switch.accessibility_label(label.to_owned());}
                if let Some(index) = kit.number("tabIndex") {switch = switch.tab_index(index as isize);}
                if let Some(size) = kit_size(kit) {switch = switch.with_size(size);}
                if let Some(color) = kit.prop("color").and_then(|color| theme_color(color, cx)) {switch = switch.color(color);}
                {switch = switch.on_change(cx.listener(move |this, checked: &bool, _, cx| this.emit(id, "change", checked.to_string(), cx)));}
                switch.into_any_element()
            }
            "Radio" => {
                let mut radio = Radio::new(element_id).label(text("label")).checked(checked(kit)).disabled(kit.flag("disabled"));
                radio = radio.tab_stop(kit.prop("tabStop") != Some("false"));
                if let Some(tooltip) = kit.prop("tooltip").filter(|tooltip| !tooltip.is_empty()) {radio = radio.tooltip(tooltip.to_owned());}
                if let Some(delay) = kit.index("tooltipDelay") {radio = radio.tooltip_show_delay(std::time::Duration::from_millis(delay as u64));}
                if let Some(label) = kit.prop("accessibilityLabel") {radio = radio.accessibility_label(label.to_owned());}
                if let Some(index) = kit.number("tabIndex") {radio = radio.tab_index(index as isize);}
                if let Some(size) = kit_size(kit) {radio = radio.with_size(size);}
                {radio = radio.on_change(cx.listener(move |this, checked: &bool, _, cx| this.emit(id, "change", checked.to_string(), cx)));}
                radio.into_any_element()
            }
            "RadioGroup" => {
                let group = if kit.flag("vertical") {RadioGroup::vertical(element_id)} else {RadioGroup::horizontal(element_id)};
                let selected = kit.number("selected").or(kit.number("defaultSelected")).filter(|index| *index >= 0.).map(|index| index as usize);
                let mut group = group.selected_index(selected).disabled(kit.flag("disabled"));
                for (child, kit) in items(node, "Radio") {
                    let mut radio = Radio::new(ElementId::Integer(child.identity.get())).label(kit.text("label")).disabled(kit.flag("disabled"));
                    radio = radio.tab_stop(kit.prop("tabStop") != Some("false"));
                    if let Some(tooltip) = kit.prop("tooltip").filter(|tooltip| !tooltip.is_empty()) {radio = radio.tooltip(tooltip.to_owned());}
                    if let Some(delay) = kit.index("tooltipDelay") {radio = radio.tooltip_show_delay(std::time::Duration::from_millis(delay as u64));}
                    if let Some(label) = kit.prop("accessibilityLabel") {radio = radio.accessibility_label(label.to_owned());}
                    if let Some(index) = kit.number("tabIndex") {radio = radio.tab_index(index as isize);}
                    if let Some(size) = kit_size(kit) {radio = radio.with_size(size);}
                    group = group.child(radio);
                }
                {group = group.on_click(cx.listener(move |this, index: &usize, _, cx| this.emit(id, "change", index.to_string(), cx)));}
                group.into_any_element()
            }
            "Toggle" => {
                let mut toggle = Toggle::new(element_id).checked(checked(kit)).disabled(kit.flag("disabled"));
                if !kit.text("label").is_empty() {toggle = toggle.label(text("label"));}
                if let Some(icon) = icon_name(&kit.text("icon")) {toggle = toggle.icon(icon);}
                if !kit.text("tooltip").is_empty() {toggle = toggle.tooltip(text("tooltip"));}
                if let Some(delay) = kit.number("tooltipDelay") {toggle = toggle.tooltip_show_delay(std::time::Duration::from_millis(delay as u64));}
                if let Some(size) = kit_size(kit) {toggle = toggle.with_size(size);}
                {toggle = toggle.on_click(cx.listener(move |this, checked: &bool, _, cx| this.emit(id, "change", checked.to_string(), cx)));}
                toggle.into_any_element()
            }
            "Rating" => {
                let mut rating = Rating::new(element_id).value(kit.index("value").or(kit.index("defaultValue")).unwrap_or(0)).max(kit.index("max").unwrap_or(5)).disabled(kit.flag("disabled"));
                if let Some(color) = kit.prop("color").and_then(|color| theme_color(color, cx)) {rating = rating.color(color);}
                if let Some(size) = kit_size(kit) {rating = rating.with_size(size);}
                {rating = rating.on_click(cx.listener(move |this, value: &usize, _, cx| this.emit(id, "change", value.to_string(), cx)));}
                rating.into_any_element()
            }
            "Pagination" => {
                // page is controlled; without it the pagination keeps its own page (from defaultPage).
                let page = kit.index("page").or(kit.index("defaultPage")).unwrap_or(1);
                let mut pages = Pagination::new(element_id).current_page(page).total_pages(kit.index("total").unwrap_or(1)).disabled(kit.flag("disabled"));
                if kit.flag("compact") {pages = pages.compact();}
                if let Some(visible) = kit.index("visiblePages") {pages = pages.visible_pages(visible);}
                if let Some(size) = kit_size(kit) {pages = pages.with_size(size);}
                pages.on_click(cx.listener(move |this, page: &usize, _, cx| this.emit(id, "change", page.to_string(), cx))).into_any_element()
            }
            "Stepper" => {
                let selected = kit.index("selected").or(kit.index("defaultSelected")).unwrap_or(0);
                let mut stepper = Stepper::new(element_id).selected_index(selected).disabled(kit.flag("disabled")).text_center(kit.flag("textCenter"));
                if kit.flag("vertical") {stepper = stepper.vertical();}
                if let Some(size) = kit_size(kit) {stepper = stepper.with_size(size);}
                let steps = items(node, "StepperItem").map(|(_, item)| {
                    let mut step = StepperItem::new().disabled(item.flag("disabled"));
                    if let Some(icon) = icon_name(&item.text("icon")) {step = step.icon(icon);}
                    if let Some(size) = kit_size(item) {step = step.with_size(size);}
                    step.child(SharedString::from(item.text("label")))
                }).collect::<Vec<_>>();
                stepper.items(steps).on_click(cx.listener(move |this, index: &usize, _, cx| this.emit(id, "change", index.to_string(), cx))).into_any_element()
            }
            "Tabs" => {
                // shadcn composition: Tab triggers (by value, defaulting to their index) and TabsContent panels.
                let tabs = items(node, "Tab").enumerate().map(|(index, (child, item))| (child, item, item.prop("value").map_or_else(|| index.to_string(), str::to_owned))).collect::<Vec<_>>();
                let by_value = |value: &str| tabs.iter().position(|(_, _, tab)| tab == value);
                let selected = kit.prop("value").and_then(by_value).or(kit.index("selected")).or(kit.prop("defaultValue").and_then(by_value)).unwrap_or(0);
                let variant = |name: Option<&str>| match name {Some("underline") => Some(TabVariant::Underline), Some("pill") => Some(TabVariant::Pill), Some("outline") => Some(TabVariant::Outline), Some("segmented") => Some(TabVariant::Segmented), Some("folder") => Some(TabVariant::Folder), Some("tab") => Some(TabVariant::Tab), _ => None};
                let mut bar = TabBar::new(element_id).selected_index(selected).menu(kit.flag("menu"));
                if let Some(variant) = variant(kit.prop("variant")) {bar = bar.with_variant(variant);}
                if let Some(width) = kit.number("maxWidth") {bar = bar.max_width(px(width as f32));}
                if let Some(size) = kit_size(kit) {bar = bar.with_size(size);}
                if let Some(prefix) = first_item(node, "TabsPrefix") {bar = bar.prefix(div().flex().items_center().gap_1().children(self.content(prefix, cx)));}
                if let Some(suffix) = first_item(node, "TabsSuffix") {bar = bar.suffix(div().flex().items_center().gap_1().children(self.content(suffix, cx)));}
                if let Some(space) = first_item(node, "TabsEmptySpace") {bar = bar.last_empty_space(div().flex().items_center().size_full().children(self.content(space, cx)));}
                for (child, item, _) in &tabs {
                    let mut tab = Tab::new().disabled(item.flag("disabled"));
                    if !item.text("label").is_empty() {tab = tab.label(item.text("label"));}
                    if let Some(label) = item.prop("accessibilityLabel") {tab = tab.aria_label(label.to_owned());}
                    if let Some(icon) = icon_name(&item.text("icon")) {tab = tab.icon(icon);}
                    if let Some(variant) = variant(item.prop("variant")) {tab = tab.with_variant(variant);}
                    if let Some(prefix) = first_item(child, "TabPrefix") {tab = tab.prefix(div().flex().items_center().gap_1().children(self.content(prefix, cx)));}
                    if let Some(suffix) = first_item(child, "TabSuffix") {tab = tab.suffix(div().flex().items_center().gap_1().children(self.content(suffix, cx)));}
                    bar = bar.child(tab);
                }
                let values = tabs.iter().map(|(_, _, value)| value.clone()).collect::<Vec<_>>();
                bar = bar.on_click(cx.listener(move |this, index: &usize, _, cx| if let Some(value) = values.get(*index) {this.emit(id, "change", value.clone(), cx);}));
                let current = tabs.get(selected).map(|(_, _, value)| value.clone()).unwrap_or_default();
                let panel = items(node, "TabsContent").enumerate().find(|(index, (_, item))| item.prop("value").map_or_else(|| index.to_string(), str::to_owned) == current).map(|(_, (panel, _))| self.content(panel, cx)).unwrap_or_default();
                if panel.is_empty() {bar.into_any_element()} else {div().flex().flex_col().gap_3().child(bar).children(panel).into_any_element()}
            }
            "Accordion" => {
                // shadcn composition: AccordionItem (value, defaulting to its index) with AccordionTrigger and AccordionContent.
                // value lists the open items one per line; without it the accordion keeps its own open items.
                let entries = items(node, "AccordionItem").enumerate().map(|(index, (child, item))| (child, item, item.prop("value").map_or_else(|| index.to_string(), str::to_owned))).collect::<Vec<_>>();
                let listed = kit.prop("value").or(kit.prop("defaultValue")).map(|value| value.split('\n').map(str::to_owned).collect::<Vec<_>>());
                let mut accordion = Accordion::new(element_id).multiple(kit.flag("multiple")).bordered(kit.prop("bordered") != Some("false")).disabled(kit.flag("disabled"));
                if let Some(size) = kit_size(kit) {accordion = accordion.with_size(size);}
                for (child, item, value) in &entries {
                    let open = match &listed {Some(values) => values.contains(value), None => item.flag("open")};
                    let trigger = items(child, "AccordionTrigger").next().map(|(trigger, _)| self.content(trigger, cx));
                    let body = match items(child, "AccordionContent").next() {Some((content, _)) => self.content(content, cx), None => self.content_except(child, &["AccordionTrigger"], cx)};
                    let (title, disabled, icon) = (item.text("title"), item.flag("disabled"), icon_name(&item.text("icon")));
                    let (title_style, content_style) = (self.class_style(&item.text("titleClassName"), cx), self.class_style(&item.text("contentClassName"), cx));
                    let hover = self.class_style(&item.text("hoverClassName"), cx);
                    let size = kit_size(item);
                    accordion = accordion.item(move |mut entry| {
                        entry = match trigger {Some(trigger) => entry.title(div().flex().items_center().gap_2().children(trigger)), None => entry.title(SharedString::from(title))};
                        if let Some(icon) = icon {entry = entry.icon(icon);}
                        if let Some(size) = size {entry = entry.with_size(size);}
                        entry.open(open).disabled(disabled).title_style(title_style).content_style(content_style).hover(move |_| hover).child(div().children(body))
                    });
                }
                let values = entries.iter().map(|(_, _, value)| value.clone()).collect::<Vec<_>>();
                accordion = accordion.on_toggle_click(cx.listener(move |this, open: &[usize], _, cx| {
                    this.emit(id, "change", open.iter().filter_map(|index| values.get(*index).cloned()).collect::<Vec<_>>().join("\n"), cx);
                }));
                accordion.into_any_element()
            }
            "Collapsible" => {
                // shadcn composition: CollapsibleTrigger toggles CollapsibleContent; other children always show.
                let open = kit.prop("open").or(kit.prop("defaultOpen")) == Some("true");
                let mut collapsible = Collapsible::new().open(open).motion_id(ElementId::Name(format!("{id}-motion").into()));
                for child in node.visible_children() {
                    match child.kit.as_deref().map(|kit| kit.name.as_str()) {
                        Some("CollapsibleTrigger") => collapsible = collapsible.child(div().id(ElementId::Integer(child.identity.get())).children(self.content(child, cx)).on_click(cx.listener(move |this, _, _, cx| this.emit(id, "openChange", (!open).to_string(), cx)))),
                        Some("CollapsibleContent") => collapsible = collapsible.content(div().w_full().flex().flex_col().items_stretch().gap_2().children(self.content(child, cx))),
                        _ => {let theme = utility_theme(cx); collapsible = collapsible.child(self.element(child, &theme, mode(cx), cx));}
                    }
                }
                collapsible.into_any_element()
            }
            "Select" => {
                let empty = items(node, "SelectEmpty").next().map(|(empty, _)| empty.identity.get());
                match self.states.get(&id) {
                    Some(KitState::Select(state, _)) => select_options!(self, Select::new(state), kit, empty, cx).into_any_element(),
                    Some(KitState::SelectGroups(state, _)) => select_options!(self, Select::new(state), kit, empty, cx).into_any_element(),
                    _ => div().into_any_element(),
                }
            }
            "NumberInput" => match self.states.get(&id) {
                Some(KitState::Number(state)) => {
                    let mut input = NumberInput::new(state).placeholder(text("placeholder")).disabled(kit.flag("disabled"));
                    let constrained=self.rules(node).iter().any(|(_,rule)|match rule {Utility::Dimension(0|2|4,Length::Auto|Length::None|Length::Fit)=>false,Utility::Dimension(0|2|4,_)=>true,Utility::Grow(value)=>*value>0.,_=>false});
                    input=if constrained {input.w_full()}else{input.w(px(240.))};
                    if let Some(icon) = icon_name(&kit.text("prefix")) {input = input.prefix(Icon::new(icon).small());}
                    if let Some(icon) = icon_name(&kit.text("suffix")) {input = input.suffix(Icon::new(icon).small());}
                    if kit.prop("appearance") == Some("false") {input = input.appearance(false);}
                    if let Some(label) = kit.prop("accessibilityLabel") {input = input.accessibility_label(label.to_owned());}
                    if let Some(size) = kit_size(kit) {input = input.with_size(size);}
                    input.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "OtpInput" => match self.states.get(&id) {
                Some(KitState::Otp(state)) => {
                    let mut input = OtpInput::new(state).disabled(kit.flag("disabled"));
                    if let Some(groups) = kit.index("groups") {input = input.groups(groups);}
                    if let Some(size) = kit_size(kit) {input = input.with_size(size);}
                    input.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "DatePicker" => match self.states.get(&id) {
                Some(KitState::Date(state)) => {
                    let mut picker = DatePicker::new(state).placeholder(text("placeholder")).cleanable(kit.flag("cleanable")).disabled(kit.flag("disabled"));
                    if let Some(months) = kit.index("months") {picker = picker.number_of_months(months.max(1));}
                    if kit.prop("appearance") == Some("false") {picker = picker.appearance(false);}
                    if let Some(size) = kit_size(kit) {picker = picker.with_size(size);}
                    // presets: [label, day] or [label, start, end] rows.
                    let presets = serde_json::from_str::<Vec<Vec<String>>>(&kit.text("presets")).unwrap_or_default().into_iter().filter_map(|row| match row.as_slice() {
                        [label, date] => day(date).map(|date| gpui_kit::component::date_picker::DateRangePreset::single(label.clone(), date)),
                        [label, start, end] => day(start).zip(day(end)).map(|(start, end)| gpui_kit::component::date_picker::DateRangePreset::range(label.clone(), start, end)),
                        _ => None,
                    }).collect::<Vec<_>>();
                    if !presets.is_empty() {picker = picker.presets(presets);}
                    picker.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "ColorPicker" => match self.states.get(&id) {
                Some(KitState::Color(state)) => {
                    let featured = kit.text("featuredColors").split('\n').filter_map(|name| theme_color(name, cx)).collect::<Vec<_>>();
                    // select draws a dropdown of swatches; otherwise the swatch button with the full picker.
                    if kit.flag("select") {
                        let mut select = ColorSelect::new(state).placeholder(text("placeholder"));
                        if !featured.is_empty() {select = select.featured_colors(featured);}
                        if let Some(label) = kit.prop("accessibilityLabel") {select = select.accessibility_label(label.to_owned());}
                        if let Some(size) = kit_size(kit) {select = select.with_size(size);}
                        select.into_any_element()
                    } else {
                        let mut picker = ColorPicker::new(state).anchor(anchor_named(kit.prop("anchor")));
                        if !featured.is_empty() {picker = picker.featured_colors(featured);}
                        if let Some(label) = kit.prop("label").filter(|label| !label.is_empty()) {picker = picker.label(label.to_owned());}
                        if let Some(icon) = icon_name(&kit.text("icon")) {picker = picker.icon(icon);}
                        if let Some(label) = kit.prop("accessibilityLabel") {picker = picker.accessibility_label(label.to_owned());}
                        if let Some(size) = kit_size(kit) {picker = picker.with_size(size);}
                        picker.into_any_element()
                    }
                }
                _ => div().into_any_element(),
            },
            "Slider" => match self.states.get(&id) {
                Some(KitState::Slider(state)) => {
                    let mut slider = Slider::new(state).disabled(kit.flag("disabled"));
                    if kit.flag("vertical") {slider = slider.vertical();}
                    if kit.flag("reverse") {slider = slider.reverse();}
                    if let Some(label) = kit.prop("accessibilityLabel") {slider = slider.accessibility_label(label.to_owned());}
                    slider.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Tooltip" => {
                // Children are the trigger; a TooltipContent child is a rich tooltip, else the text prop.
                let (tip, mut keys, command) = (text("text"), kit.text("keys"), kit.text("command"));
                // A command shows the keys its Shortcut bound, as Kbd does.
                if keys.is_empty() && !command.is_empty() {keys = COMMAND_KEYS.with(|bound| bound.borrow().get(&command).cloned()).unwrap_or_default();}
                let rich = items(node, "TooltipContent").next().map(|(child, _)| child.identity.get());
                let style = self.class_style(&kit.text("contentClassName"), cx);
                let root = root.clone();
                div().id(element_id).children(self.content_except(node, &["TooltipContent"], cx)).tooltip(move |window, cx| {
                    let mut tooltip = match rich {
                        Some(body) => {let root = root.clone(); Tooltip::element(move |_, cx| div().flex().flex_col().gap_1().children(root.update(cx, |root, cx| root.content_of(body, cx)).unwrap_or_default()))}
                        None => Tooltip::new(tip.clone()),
                    };
                    if let Some(keys) = keystroke(&keys) {tooltip = tooltip.key_binding(Some(Kbd::new(keys)));}
                    tooltip.refine_style(&style).build(window, cx)
                }).into_any_element()
            }
            "Popover" | "HoverCard" => {
                let slot = |name: &'static str| items(node, name).next().map(|(child, _)| child);
                let trigger = slot(if kit.name == "Popover" {"PopoverTrigger"} else {"HoverCardTrigger"}).map(|child| self.content(child, cx)).unwrap_or_default();
                let body = slot(if kit.name == "Popover" {"PopoverContent"} else {"HoverCardContent"}).map(|child| child.identity.get());
                let render = move |cx: &mut App| match body {Some(body) => root.update(cx, |root, cx| root.content_of(body, cx)).unwrap_or_default(), None => Vec::new()};
                let trigger = div().children(trigger);
                if kit.name == "Popover" {
                    let mut popover = Popover::new(element_id).trigger(Trigger {element: trigger.into_any_element(), selected: false})
                        .anchor(anchor_named(kit.prop("anchor"))).arrow(kit.flag("arrow")).overlay_closable(kit.prop("overlayClosable") != Some("false"))
                        .trigger_style(self.class_style(&kit.text("triggerClassName"), cx));
                    if let Some(offset) = kit.number("offset") {popover = popover.offset(px(offset as f32));}
                    if kit.prop("mouseButton") == Some("right") {popover = popover.mouse_button(MouseButton::Right);}
                    if kit.prop("appearance") == Some("false") {popover = popover.appearance(false);}
                    // open controls the popover; defaultOpen only starts it open.
                    popover = match kit.prop("open") {Some(open) => popover.open(open == "true"), None => popover.default_open(kit.flag("defaultOpen"))};
                    if kit.handles("openChange") {popover = popover.on_open_change(cx.listener(move |this, open: &bool, _, cx| this.emit(id, "openChange", open.to_string(), cx)));}
                    popover.content(move |_, _, cx| div().flex().flex_col().gap_2().children(render(cx))).into_any_element()
                } else {
                    let mut card = HoverCard::new(element_id).trigger(trigger).anchor(anchor_named(kit.prop("anchor"))).trigger_style(self.class_style(&kit.text("triggerClassName"), cx));
                    if let Some(delay) = kit.index("openDelay") {card = card.open_delay(std::time::Duration::from_millis(delay as u64));}
                    if let Some(delay) = kit.index("closeDelay") {card = card.close_delay(std::time::Duration::from_millis(delay as u64));}
                    if kit.prop("appearance") == Some("false") {card = card.appearance(false);}
                    if kit.handles("openChange") {card = card.on_open_change(cx.listener(move |this, open: &bool, _, cx| this.emit(id, "openChange", open.to_string(), cx)));}
                    card.content(move |_, _, cx| div().flex().flex_col().gap_2().children(render(cx))).into_any_element()
                }
            }
            "Dialog" | "Sheet" | "AlertDialog" => match first_part(node, "Trigger") {
                // The trigger opens the overlay; open, when set, controls it instead.
                Some(trigger) => div().id(element_id).children(self.content(trigger, cx)).on_click(cx.listener(move |this, _, _, cx| this.emit(id, "openChange", "true".into(), cx))).into_any_element(),
                None => div().into_any_element(),
            },
            "DropdownMenu" | "ContextMenu" => {
                let entries = std::rc::Rc::new(menu_entries(node));
                let options = MenuOptions::new(kit);
                let build = {
                    let root = root.clone();
                    move |menu: gpui_kit::component::menu::PopupMenu, window: &mut Window, cx: &mut Context<gpui_kit::component::menu::PopupMenu>| build_menu(options.apply(menu), &entries, &root, window, cx)
                };
                if kit.name == "DropdownMenu" {
                    let trigger = node.visible_children().find(|child| child.kit.as_deref().is_none_or(|kit| kit.name.is_empty()) && matches!(child.kind, Kind::Button | Kind::DisabledButton));
                    let mut button = match trigger {
                        Some(child) => self.kit_button(child, cx),
                        None => gpui_kit::component::button::Button::new(ElementId::Name(format!("{id}-button").into())).label(text("label")),
                    };
                    let anchor = match kit.prop("anchor") {Some("top-right") => gpui_kit::Anchor::TopRight, Some("bottom-left") => gpui_kit::Anchor::BottomLeft, Some("bottom-right") => gpui_kit::Anchor::BottomRight, Some("top-center") => gpui_kit::Anchor::TopCenter, Some("bottom-center") => gpui_kit::Anchor::BottomCenter, Some("left-center") => gpui_kit::Anchor::LeftCenter, Some("right-center") => gpui_kit::Anchor::RightCenter, _ => gpui_kit::Anchor::TopLeft};
                    // A split button runs its own action and opens the menu from its arrow; otherwise the whole button opens the menu.
                    if kit.flag("split") || kit.handles("click") {
                        if kit.handles("click") {let click = event("click"); button = button.on_click(move |_, _, cx| click(String::new(), cx));}
                        let mut dropdown = menu_button(DropdownButton::new(element_id).button(button), kit, cx);
                        if kit.flag("outline") {dropdown = dropdown.outline();}
                        if !kit.text("menuLabel").is_empty() {dropdown = dropdown.menu_accessibility_label(text("menuLabel"));}
                        dropdown.dropdown_menu_with_anchor(anchor, move |menu, window, cx| build(menu, window, cx)).into_any_element()
                    } else {
                        let mut button = menu_button(button, kit, cx).dropdown_caret(trigger.is_none() || kit.flag("caret"));
                        if kit.flag("outline") {button = button.outline();}
                        button.dropdown_menu_with_anchor(anchor, move |menu, window, cx| build(menu, window, cx)).into_any_element()
                    }
                } else {
                    let theme = utility_theme(cx);
                    let content = node.visible_children().filter(|child| child.kit.as_deref().is_none_or(|kit| !MENU_PARTS.contains(&kit.name.as_str()))).map(|child| self.element(child, &theme, mode(cx), cx)).collect::<Vec<_>>();
                    div().id(element_id).children(content).context_menu(move |menu, window, cx| build(menu, window, cx)).into_any_element()
                }
            }
            "Sidebar" => {
                let collapsed = kit.flag("collapsed");
                let side = if kit.prop("side") == Some("right") {gpui_kit::component::Side::Right} else {gpui_kit::component::Side::Left};
                let collapsible = match kit.prop("collapsible") {Some("offcanvas") => SidebarCollapsible::Offcanvas, Some("none") => SidebarCollapsible::None, _ => SidebarCollapsible::Icon};
                let mut sidebar = Sidebar::new(element_id).collapsed(collapsed).side(side).collapsible(collapsible);
                if let Some((header, item)) = items(node, "SidebarHeader").next() {sidebar = sidebar.header(SidebarHeader::new().selected(item.flag("selected")).collapsed(collapsed).children(self.content(header, cx)));}
                if let Some((footer, item)) = items(node, "SidebarFooter").next() {sidebar = sidebar.footer(SidebarFooter::new().selected(item.flag("selected")).collapsed(collapsed).children(self.content(footer, cx)));}
                for (group_node, group) in items(node, "SidebarGroup") {
                    let entries = items(group_node, "SidebarItem").map(|(child, item)| self.sidebar_item(child, item, collapsed, cx)).collect::<Vec<_>>();
                    sidebar = sidebar.child(SidebarGroup::new(group.text("label")).child(SidebarMenu::new().children(entries)));
                }
                sidebar.into_any_element()
            }
            "SidebarToggleButton" => {
                let side = if kit.prop("side") == Some("right") {gpui_kit::component::Side::Right} else {gpui_kit::component::Side::Left};
                let mut button = SidebarToggleButton::new().side(side).collapsed(kit.flag("collapsed"));
                if let Some(label) = kit.prop("accessibilityLabel") {button = button.accessibility_label(label.to_owned());}
                button.on_click(cx.listener(move |this, _, _, cx| this.emit(id, "click", String::new(), cx))).into_any_element()
            }
            "Resizable" => {
                let mut group = if kit.flag("vertical") {v_resizable(element_id)} else {h_resizable(element_id)};
                if let Some(size) = kit.number("size") {group = group.size(px(size as f32));}
                for (child, panel) in items(node, "ResizablePanel") {
                    let mut entry = resizable_panel().child(div().size_full().children(self.content(child, cx))).visible(panel.prop("visible") != Some("false"));
                    if let Some(size) = panel.number("size") {entry = entry.size(px(size as f32));}
                    if panel.number("minSize").is_some() || panel.number("maxSize").is_some() {entry = entry.size_range(px(panel.number("minSize").unwrap_or(0.) as f32)..px(panel.number("maxSize").unwrap_or(100_000.) as f32));}
                    if panel.flag("fixed") {entry = entry.fixed();}
                    group = group.child(entry);
                }
                if kit.handles("resize") {
                    let root = root.clone();
                    group = group.on_resize(move |state, _, cx| {
                        let sizes = state.read(cx).sizes().iter().map(|size| f32::from(*size).round().to_string()).collect::<Vec<_>>().join("\n");
                        let _ = root.update(cx, |root, cx| root.emit(id, "resize", sizes, cx));
                    });
                }
                group.into_any_element()
            }
            "Table" => {
                // shadcn composition: TableHeader / TableBody / TableFooter hold TableRow; rows hold TableHead / TableCell;
                // TableCaption sits below. Each part takes its own className; size propagates from the Table.
                let mut table = Table::new().refine_style(&style);
                if let Some(label) = kit.prop("accessibilityLabel") {table = table.accessibility_label(label.to_owned());}
                if let Some(size) = kit_size(kit) {table = table.with_size(size);}
                for (section, part) in node.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))) {
                    let section_style = self.node_style(section);
                    if part.name == "TableCaption" {
                        table = table.child(gpui_kit::component::table::TableCaption::new().refine_style(&section_style).children(self.content(section, cx)));
                        continue;
                    }
                    let rows = items(section, "TableRow").map(|(row, _)| {
                        let mut cells = TableRow::new().refine_style(&self.node_style(row));
                        for (cell, cell_kit) in row.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))) {
                            let cell_style = self.node_style(cell);
                            let content = self.content(cell, cx);
                            let span = cell_kit.index("colSpan").unwrap_or(1);
                            let align = cell_kit.prop("align");
                            cells = if cell_kit.name == "TableHead" {
                                let mut head = TableHead::new().col_span(span).refine_style(&cell_style).children(content);
                                match align {Some("center") => head = head.text_center(), Some("right") => head = head.text_right(), _ => {}}
                                cells.child(head)
                            } else {
                                let mut body = TableCell::new().col_span(span).refine_style(&cell_style).children(content);
                                match align {Some("center") => body = body.text_center(), Some("right") => body = body.text_right(), _ => {}}
                                cells.child(body)
                            };
                        }
                        cells
                    }).collect::<Vec<_>>();
                    table = match part.name.as_str() {
                        "TableHeader" => table.child(TableHeader::new().refine_style(&section_style).children(rows)),
                        "TableFooter" => table.child(gpui_kit::component::table::TableFooter::new().refine_style(&section_style).children(rows)),
                        _ => table.child(TableBody::new().refine_style(&section_style).children(rows)),
                    };
                }
                table.into_any_element()
            }
            "Markdown" => TextView::markdown(element_id, super::kit::highlighted_markdown(&kit.text("content"))).into_any_element(),
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
            "TitleBar" => {
                let mut bar = TitleBar::new().children(self.content(node, cx));
                if kit.handles("closeWindow") {bar = bar.on_close_window(cx.listener(move |this, _, _, cx| this.emit(id, "closeWindow", String::new(), cx)));}
                bar.into_any_element()
            }
            "WindowBorder" => {
                let mut border = gpui_kit::component::WindowBorder::new().children(self.content(node, cx));
                if let Some(size) = kit.number("shadowSize") {border = border.shadow_size(px(size as f32));}
                if let Some(size) = kit.number("resizeHitSize") {border = border.resize_hit_size(px(size as f32));}
                border.into_any_element()
            }
            "StatusBar" => {
                let mut bar = StatusBar::new();
                for child in node.visible_children() {
                    let element = div().flex().items_center().gap_2().children(self.content(child, cx));
                    bar = match child.kit.as_ref().map(|kit| kit.name.as_str()) {
                        Some("StatusBarLeft") => bar.left(element),
                        Some("StatusBarRight") => bar.right(element),
                        _ => {let theme = utility_theme(cx); bar.child(self.element(child, &theme, mode(cx), cx))}
                    };
                }
                bar.into_any_element()
            }
            "Toolbar" => {
                let mut toolbar = Toolbar::new(element_id).disabled(kit.flag("disabled"));
                if let Some(size) = kit_size(kit) {toolbar = toolbar.with_size(size);}
                let theme = utility_theme(cx);
                for child in node.visible_children() {
                    match child.kit.as_deref().filter(|item| item.name == "ToolbarGroup") {
                        Some(item) => {
                            let mut group = ToolbarGroup::new(ElementId::Integer(child.identity.get()));
                            if !item.text("label").is_empty() {group = group.label(item.text("label"));}
                            if let Some(size) = kit_size(item) {group = group.with_size(size);}
                            for part in self.content(child, cx) {group = group.content(part);}
                            toolbar = toolbar.content(group);
                        }
                        None => toolbar = toolbar.content(self.element(child, &theme, mode(cx), cx)),
                    }
                }
                toolbar.into_any_element()
            }
            "Form" => {
                let form = if kit.flag("horizontal") {h_form()} else {v_form()};
                let mut form = form.columns(kit.index("columns").unwrap_or(1).max(1));
                match kit.prop("labelLayout") {Some("horizontal") => form = form.label_layout(Axis::Horizontal), Some("vertical") => form = form.label_layout(Axis::Vertical), _ => {}}
                if let Some(width) = kit.number("labelWidth") {form = form.label_width(px(width as f32));}
                if let Some(size) = kit.number("labelTextSize") {form = form.label_text_size(rems(size as f32));}
                if let Some(size) = kit_size(kit) {form = form.with_size(size);}
                for (child, item) in items(node, "Field") {
                    let mut entry = field().required(item.flag("required")).label_indent(item.prop("labelIndent") != Some("false")).visible(item.prop("visible") != Some("false"));
                    // FieldLabel and FieldDescription children are rich alternatives to the label and description props.
                    entry = match first_item(child, "FieldLabel") {
                        Some(label) => {let (label, root) = (label.identity.get(), root.clone()); entry.label_fn(move |_, cx| div().children(root.update(cx, |root, cx| root.content_of(label, cx)).unwrap_or_default()))}
                        None => entry.label(item.text("label")),
                    };
                    match first_item(child, "FieldDescription") {
                        Some(description) => {let (description, root) = (description.identity.get(), root.clone()); entry = entry.description_fn(move |_, cx| div().children(root.update(cx, |root, cx| root.content_of(description, cx)).unwrap_or_default()));}
                        None => if let Some(description) = item.prop("description").filter(|text| !text.is_empty()) {entry = entry.description(description.to_owned());},
                    }
                    entry = match item.prop("align") {Some("start") => entry.items_start(), Some("end") => entry.items_end(), Some("center") => entry.items_center(), _ => entry};
                    if let Some(span) = item.index("colSpan") {entry = entry.col_span(span as u16);}
                    if let Some(start) = item.number("colStart") {entry = entry.col_start(start as i16);}
                    if let Some(end) = item.number("colEnd") {entry = entry.col_end(end as i16);}
                    form = form.child(entry.child(div().children(self.content_except(child, &["FieldLabel", "FieldDescription"], cx))));
                }
                if let Some(footer) = first_item(node, "FormFooter") {form = form.footer(div().flex().gap_2().children(self.content(footer, cx)));}
                form.into_any_element()
            }
            "ScrollArea" => {
                let area = div().id(element_id).size_full().children(self.content(node, cx).into_iter().map(|child| div().flex_shrink_0().child(child)));
                // Kit keys scroll state by call site unless given an id, so each area needs its own.
                let scroll = ElementId::Name(format!("scroll-area-{id}").into());
                if kit.flag("horizontal") {area.flex().overflow_x_scrollbar().id(scroll).into_any_element()} else {area.overflow_y_scrollbar().id(scroll).into_any_element()}
            }
            "Message" => {
                // shadcn composition: MessageAvatar, MessageHeader, MessageContent (text or a Bubble) and MessageFooter.
                // Without parts, author/time/variant props wrap the children in a bubble.
                let mut message = Message::new().id(element_id).alignment(message_alignment(kit)).with_stack_style(self.class_style(&kit.text("stackClassName"), cx)).refine_style(&style);
                if kit.prop("role") == Some("presentation") {message = message.role(gpui_kit::base::RoleOverride::Presentational);}
                let parts = ["MessageAvatar", "MessageHeader", "MessageContent", "MessageFooter"];
                let mut composed = false;
                for (part, part_kit) in node.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))).filter(|(_, kit)| parts.contains(&kit.name.as_str())) {
                    composed = true;
                    let part_style = self.node_style(part);
                    message = match part_kit.name.as_str() {
                        "MessageAvatar" => message.avatar_slot(MessageAvatar::new().refine_style(&part_style).children(self.content(part, cx))),
                        "MessageHeader" => {let mut header = MessageHeader::new().refine_style(&part_style).children(self.content(part, cx)); if let Some(inset) = part_kit.prop("inset") {header = header.content_inset(inset == "true");} message.header(header)}
                        "MessageFooter" => {let mut footer = MessageFooter::new().refine_style(&part_style).children(self.content(part, cx)); if let Some(inset) = part_kit.prop("inset") {footer = footer.content_inset(inset == "true");} message.footer(footer)}
                        _ => {
                            let mut content = MessageContent::new().refine_style(&part_style);
                            let only_bubble = part.visible_children().count() == 1 && items(part, "Bubble").count() == 1;
                            content = match items(part, "Bubble").next() {Some((bubble, bubble_kit)) if only_bubble => {
                                    // A bubble follows its message's side unless it sets its own.
                                    let mut message_bubble = self.bubble(bubble, bubble_kit, cx);
                                    if bubble_kit.prop("align").is_none() {message_bubble = message_bubble.alignment(message_alignment(kit));}
                                    content.bubble(message_bubble)
                                } _ => content.children(self.content(part, cx))};
                            message.content(content)
                        }
                    };
                }
                if !composed {
                    let author = text("author");
                    let bubble = Bubble::new().with_variant(if kit.prop("variant") == Some("primary") {BubbleVariant::Filled} else {bubble_variant(kit)}).alignment(message_alignment(kit)).children(self.content(node, cx));
                    if !author.is_empty() {message = message.avatar(Avatar::new().name(author.clone()).size_8()).header(MessageHeader::new().child(author).child(text("time")));}
                    message = message.content(MessageContent::new().bubble(bubble));
                }
                message.into_any_element()
            }
            "MessageGroup" => MessageGroup::new().refine_style(&style).children(self.content(node, cx)).into_any_element(),
            "Bubble" => self.bubble(node, kit, cx).into_any_element(),
            "BubbleGroup" => BubbleGroup::new().refine_style(&style).children(self.content(node, cx)).into_any_element(),
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
                    Some("radar") => {
                        let color = cx.theme().chart_1;
                        RadarChart::new(data).label(|point: &(SharedString, f64)| point.0.clone()).value(|point: &(SharedString, f64)| point.1).stroke(color).fill(color.opacity(0.2)).dot().into_any_element()
                    }
                    Some("candlestick") => {
                        let series = |name: &str| kit.text(name).split('\n').map(|value| value.trim().parse::<f64>().unwrap_or_default()).collect::<Vec<_>>();
                        let (open, high, low, close) = (series("open"), series("high"), series("low"), series("close"));
                        let candles = labels.split('\n').enumerate().map(|(index, label)| {
                            let at = |values: &Vec<f64>| values.get(index).copied().unwrap_or_default();
                            (SharedString::from(label.to_owned()), at(&open), at(&high), at(&low), at(&close))
                        }).collect::<Vec<_>>();
                        type Candle = (SharedString, f64, f64, f64, f64);
                        CandlestickChart::new(candles).x(|candle: &Candle| candle.0.clone()).open(|candle: &Candle| candle.1).high(|candle: &Candle| candle.2).low(|candle: &Candle| candle.3).close(|candle: &Candle| candle.4).into_any_element()
                    }
                    Some("sankey") => {
                        let nodes = labels.split('\n').filter(|label| !label.is_empty()).map(|label| SharedString::from(label.to_owned())).collect::<Vec<_>>();
                        let index = |name: &str| nodes.iter().position(|node| node.as_ref() == name);
                        let links = serde_json::from_str::<Vec<Vec<String>>>(&kit.text("links")).unwrap_or_default().into_iter().filter_map(|link| {
                            let (source, target) = (index(link.first()?)?, index(link.get(1)?)?);
                            Some(SankeyLink::new(source, target, link.get(2)?.trim().parse::<f64>().ok()?))
                        }).collect::<Vec<_>>();
                        SankeyChart::new(nodes, links).node_label(|node: &SharedString| node.clone()).into_any_element()
                    }
                    _ => LineChart::new(data).x(|point: &(SharedString, f64)| point.0.clone()).y(|point: &(SharedString, f64)| point.1).into_any_element(),
                }
            }
            "List" => match self.states.get(&id) {
                Some(KitState::List(state)) => {
                    let mut list = List::new(state).scrollbar_visible(kit.prop("scrollbarVisible") != Some("false")).refine_style(&style);
                    if let Some(bounce) = kit.prop("scrollBounce") {list = list.scroll_bounce(bounce == "true");}
                    if let Some(placeholder) = kit.prop("placeholder") {list = list.search_placeholder(placeholder.to_owned());}
                    if let Some(size) = kit_size(kit) {list = list.with_size(size);}
                    list.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Tree" => match self.states.get(&id) {
                Some(KitState::Tree(state, _)) => {
                    let mut icons = HashMap::new();
                    tree_icons(node, &mut icons);
                    let row = root.clone();
                    let mut view = tree(state, move |index, entry, selected, _, _| {
                        let item = entry.item();
                        let item_id = item.id.to_string();
                        let icon = icons.get(&item_id).and_then(|icon| icon_name(icon)).unwrap_or(if entry.is_folder() {if entry.is_expanded() {IconName::FolderOpen} else {IconName::Folder}} else {IconName::File});
                        let root = row.clone();
                        let mut line = ListItem::new(index).selected(selected).disabled(entry.is_disabled()).pl(px(12. * entry.depth() as f32 + 8.))
                            .child(div().flex().items_center().gap_2().child(Icon::new(icon).size_4()).child(item.label.clone()));
                        line.on_click(move |_, _, cx| {let _ = root.update(cx, |root, cx| root.emit(id, "select", item_id.clone(), cx));})
                    }).refine_style(&style);
                    if let Some(bounce) = kit.prop("scrollBounce") {view = view.scroll_bounce(bounce == "true");}
                    if let Some(menu) = first_item(node, "TreeMenu") {
                        let (entries, root) = (menu_entries(menu), root.clone());
                        view = view.context_menu(move |_, entry, menu, _, cx| {
                            let item = entry.item().id.to_string();
                            let target = root.clone();
                            cx.defer(move |cx| {let _ = target.update(cx, |root, cx| root.emit(id, "contextMenu", item, cx));});
                            flat_menu(menu, &entries, &root)
                        });
                    }
                    view.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Calendar" => match self.states.get(&id) {
                Some(KitState::Calendar(state)) => {
                    let mut calendar = Calendar::new(state).number_of_months(kit.index("months").unwrap_or(1).max(1));
                    if let Some(first) = weekday(&kit.text("firstDayOfWeek")) {calendar = calendar.first_day_of_week(first);}
                    if let Some(size) = kit_size(kit) {calendar = calendar.with_size(size);}
                    calendar.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "TimeField" => match self.states.get(&id) {
                Some(KitState::Time(state)) => {
                    let mut field = TimeField::new(state).invalid(kit.flag("invalid")).disabled(kit.flag("disabled"));
                    if let Some(size) = kit_size(kit) {field = field.with_size(size);}
                    field.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Combobox" => {
                let slot = |name: &str| items(node, name).next().map(|(child, _)| child.identity.get());
                let (empty, trigger, footer) = (slot("SelectEmpty"), slot("ComboboxTrigger"), slot("ComboboxFooter"));
                match self.states.get(&id) {
                    Some(KitState::Combobox(state, _)) => combobox_options!(self, Combobox::new(state), kit, empty, trigger, footer, root, cx).into_any_element(),
                    Some(KitState::ComboboxGroups(state, _)) => combobox_options!(self, Combobox::new(state), kit, empty, trigger, footer, root, cx).into_any_element(),
                    _ => div().into_any_element(),
                }
            }
            "Command" => match self.states.get(&id) {
                Some(KitState::Command(state)) => {
                    // Ungrouped CommandItems are section 0 (when present); each CommandGroup is the next section.
                    let mut palette = CommandPalette::new(state).searchable(kit.prop("searchable") != Some("false")).filterable(kit.prop("filterable") != Some("false"))
                        .bordered(kit.prop("bordered") != Some("false"));
                    if !kit.text("placeholder").is_empty() {palette = palette.placeholder(text("placeholder"));}
                    if let Some(height) = kit.number("maxHeight") {palette = palette.max_h(px(height as f32));}
                    if let Some(bounce) = kit.prop("scrollBounce") {palette = palette.scroll_bounce(bounce == "true");}
                    let mut loose = Vec::new();
                    let mut sections = Vec::new();
                    for child in node.visible_children() {
                        match child.kit.as_deref().map(|kit| kit.name.as_str()) {
                            Some("CommandItem") => {let item = self.command_item(child, cx); loose.push(child.identity.get()); palette = palette.item(item);}
                            Some("CommandGroup") => {
                                let group_kit = child.kit.as_deref().expect("group");
                                let mut group = CommandGroup::new();
                                if !group_kit.text("heading").is_empty() {group = group.label(group_kit.text("heading"));}
                                let mut ids = Vec::new();
                                for (item_node, _) in items(child, "CommandItem") {ids.push(item_node.identity.get()); group = group.item(self.command_item(item_node, cx));}
                                sections.push(ids);
                                palette = palette.group(group);
                            }
                            Some("CommandSeparator") => palette = palette.separator(),
                            _ => {}
                        }
                    }
                    if !loose.is_empty() {sections.insert(0, loose);}
                    let sections = std::rc::Rc::new(sections);
                    let target = {let sections = sections.clone(); move |index: IndexPath| sections.get(index.section).and_then(|section| section.get(index.row)).copied()};
                    for (slot, event) in [("CommandEmpty", 0), ("CommandHeader", 1), ("CommandFooter", 2)] {
                        let Some(part) = first_item(node, slot) else {continue};
                        let (part, root) = (part.identity.get(), root.clone());
                        let render = move |cx: &mut App| div().flex().flex_col().gap_1().children(root.update(cx, |root, cx| root.content_of(part, cx)).unwrap_or_default());
                        palette = match event {0 => palette.empty(move |_, _, cx| render(cx)), 1 => palette.header(move |_, _, cx| render(cx)), _ => palette.footer(move |_, _, cx| render(cx))};
                    }
                    let (confirm_root, confirm_target) = (root.clone(), target.clone());
                    // The item's own onSelect runs, then the Command's onSelect with the item's value (or label).
                    palette = palette.on_confirm(move |index: IndexPath, _, cx| {if let Some(item) = confirm_target(index) {let _ = confirm_root.update(cx, |root, cx| {
                        let value = find(&root.node, item).and_then(|node| node.kit.as_deref()).map(|kit| kit.prop("value").map_or_else(|| kit.text("label"), str::to_owned)).unwrap_or_default();
                        root.emit(item, "select", String::new(), cx);
                        root.emit(id, "select", value, cx);
                    });}});
                    let (select_root, select_target) = (root.clone(), target);
                    palette = palette.on_select(move |index: IndexPath, _, cx| {if let Some(item) = select_target(index) {let _ = select_root.update(cx, |root, cx| root.emit(item, "highlight", String::new(), cx));}});
                    if kit.handles("query") {let root = root.clone(); palette = palette.on_query(move |query, _, cx| {let query = query.to_owned(); let _ = root.update(cx, |root, cx| root.emit(id, "query", query, cx));});}
                    if kit.handles("cancel") {let root = root.clone(); palette = palette.on_cancel(move |_, cx| {let _ = root.update(cx, |root, cx| root.emit(id, "cancel", String::new(), cx));});}
                    palette.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Carousel" => match self.states.get(&id) {
                Some(KitState::Carousel(state)) => {
                    // shadcn composition: CarouselContent holds CarouselItem slides; CarouselPrevious, CarouselNext and
                    // CarouselPagination place the controls (prev/next by default).
                    let holder = first_item(node, "CarouselContent").unwrap_or(node);
                    let mut content = CarouselContent::new(state).track_style(self.class_style(&kit.text("trackClassName"), cx)).refine_style(&self.class_style(if std::ptr::eq(holder, node) {""} else {&holder.class_name}, cx));
                    for (index, slide) in carousel_slides(holder).into_iter().enumerate() {
                        let mut item = CarouselItem::new(ElementId::Name(format!("{id}-slide-{index}").into()), index, state).refine_style(&self.node_style(slide)).child(div().size_full().children(self.content(slide, cx)));
                        if let Some(label) = slide.kit.as_deref().and_then(|kit| kit.prop("accessibilityLabel")) {item = item.accessibility_label(label.to_owned());}
                        content = content.child(item);
                    }
                    let mut carousel = Carousel::new(element_id, state).refine_style(&style).child(content);
                    if let Some(label) = kit.prop("accessibilityLabel") {carousel = carousel.accessibility_label(label.to_owned());}
                    let controls = node.visible_children().filter_map(|child| child.kit.as_deref().map(|kit| (child, kit))).filter(|(_, kit)| matches!(kit.name.as_str(), "CarouselPrevious" | "CarouselNext" | "CarouselPagination")).collect::<Vec<_>>();
                    if controls.is_empty() {
                        carousel = carousel.child(CarouselPrevious::new(state)).child(CarouselNext::new(state));
                    }
                    let count = carousel_slides(holder).len();
                    for (part, control) in controls {
                        let part_style = self.node_style(part);
                        let label = control.prop("accessibilityLabel").map(str::to_owned);
                        let size = kit_size(control);
                        carousel = match control.name.as_str() {
                            "CarouselPrevious" => {let mut button = CarouselPrevious::new(state).refine_style(&part_style); if let Some(label) = label {button = button.accessibility_label(label);} if let Some(size) = size {button = button.with_size(size);} carousel.child(button)}
                            "CarouselNext" => {let mut button = CarouselNext::new(state).refine_style(&part_style); if let Some(label) = label {button = button.accessibility_label(label);} if let Some(size) = size {button = button.with_size(size);} carousel.child(button)}
                            _ => {
                                let mut dots = gpui_kit::component::carousel::CarouselPagination::new().refine_style(&part_style);
                                if let Some(label) = label {dots = dots.accessibility_label(label);}
                                for index in 0..count {
                                    let mut dot = gpui_kit::component::carousel::CarouselPaginationItem::new(ElementId::Name(format!("{id}-dot-{index}").into()), index, state);
                                    if let Some(size) = size {dot = dot.with_size(size);}
                                    dots = dots.child(dot);
                                }
                                carousel.child(dots)
                            }
                        };
                    }
                    carousel.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "CodeEditor" => match self.states.get(&id) {
                Some(KitState::Editor(state,_)) => {
                    let mut editor=Editor::new(state).h_full().readonly(kit.flag("readOnly")).disabled(kit.flag("disabled")).refine_style(&self.node_style(node));
                    if let Some(value)=kit.prop("appearance") {editor=editor.appearance(value=="true");}
                    if let Some(value)=kit.prop("bordered") {editor=editor.bordered(value=="true");}
                    if let Some(value)=kit.prop("tabIndex").and_then(|value|value.parse::<isize>().ok()) {editor=editor.tab_index(value);}
                    if let Some(value)=kit.prop("accessibilityLabel") {editor=editor.aria_label(value.to_owned());}
                    editor=match kit.prop("role") {Some("searchbox")=>editor.role(Role::SearchInput),Some("combobox")=>editor.role(Role::ComboBox),Some("spinbutton")=>editor.role(Role::SpinButton),Some("none")=>editor.role(None::<Role>),_=>editor};
                    if let Some(size)=kit_size(kit) {editor=editor.with_size(size);}
                    paste_options!(editor,node,self,cx).into_any_element()
                }
                _ => div().into_any_element(),
            },
            "ToggleGroup" => {
                let toggles = items(node, "Toggle").map(|(child, item)| {
                    let mut toggle = Toggle::new(ElementId::Integer(child.identity.get())).checked(checked(item)).disabled(item.flag("disabled"));
                    if !item.text("label").is_empty() {toggle = toggle.label(item.text("label"));}
                    if let Some(icon) = icon_name(&item.text("icon")) {toggle = toggle.icon(icon);}
                    if !item.text("tooltip").is_empty() {toggle = toggle.tooltip(item.text("tooltip"));}
                    (child.identity.get(), checked(item), toggle)
                }).collect::<Vec<_>>();
                let targets = toggles.iter().map(|(child, checked, _)| (*child, *checked)).collect::<Vec<_>>();
                let mut group = ToggleGroup::new(element_id).children(toggles.into_iter().map(|(_, _, toggle)| toggle)).disabled(kit.flag("disabled"));
                if kit.flag("segmented") {group = group.segmented();}
                if let Some(size) = kit_size(kit) {group = group.with_size(size);}
                group.on_click(cx.listener(move |this, states: &Vec<bool>, _, cx| {
                    // Every toggle hears its change, so uncontrolled toggles keep their own state.
                    for ((target, before), now) in targets.iter().zip(states) {
                        if before != now {this.emit(*target, "change", now.to_string(), cx);}
                    }
                })).into_any_element()
            }
            "ButtonGroup" => {
                // Children are core Buttons (full options) or GroupButton shorthands.
                let mut buttons = Vec::new();
                let mut targets = Vec::new();
                for child in node.visible_children() {
                    let child_id = child.identity.get();
                    if let Some(item) = child.kit.as_deref().filter(|kit| kit.name == "GroupButton") {
                        let mut button = gpui_kit::component::button::Button::new(ElementId::Integer(child_id)).label(item.text("label")).selected(item.flag("selected")).disabled(item.flag("disabled"));
                        if let Some(icon) = icon_name(&item.text("icon")) {button = button.icon(icon);}
                        buttons.push(button);
                        targets.push((child_id, true, item.flag("selected")));
                    } else if matches!(child.kind, Kind::Button | Kind::DisabledButton) {
                        buttons.push(self.kit_button(child, cx));
                        targets.push((child_id, false, child.option("selected") == Some("true")));
                    }
                }
                let mut group = ButtonGroup::new(element_id).children(buttons).multiple(kit.flag("multiple")).disabled(kit.flag("disabled"));
                if kit.flag("outline") {group = group.outline();}
                if kit.flag("compact") {group = group.compact();}
                if kit.flag("vertical") {group = group.layout(gpui_kit::Axis::Vertical);}
                if let Some(size) = kit_size(kit) {group = group.with_size(size);}
                group = match kit.prop("variant") {
                    Some("primary") => group.primary(), Some("secondary") => group.secondary(), Some("danger") => group.danger(), Some("warning") => group.warning(),
                    Some("success") => group.success(), Some("info") => group.info(), Some("ghost") => group.ghost(), Some("link") => group.link(), Some("text") => group.text(), _ => group,
                };
                if let Some(color) = theme_color(&kit.text("color"), cx) {
                    let custom = gpui_kit::component::button::ButtonCustomVariant::new(cx).color(color);
                    let custom = match theme_color(&kit.text("foreground"), cx) {Some(color) => custom.foreground(color), None => custom};
                    let custom = match theme_color(&kit.text("hoverColor"), cx) {Some(color) => custom.hover(color), None => custom};
                    let custom = match theme_color(&kit.text("activeColor"), cx) {Some(color) => custom.active(color), None => custom};
                    group = group.custom(custom);
                }
                group.on_click(cx.listener(move |this, selected: &Vec<usize>, _, cx| {
                    // The clicked child is the one whose selection changed (or the only one selected).
                    let before = targets.iter().enumerate().filter(|(_, target)| target.2).map(|(index, _)| index).collect::<Vec<_>>();
                    let clicked = selected.iter().chain(before.iter()).find(|index| selected.contains(index) != before.contains(index)).or(selected.first()).copied();
                    if let Some((child, shorthand, _)) = clicked.and_then(|index| targets.get(index)).copied() {
                        if shorthand {this.emit(child, "click", String::new(), cx);} else {this.activate(child, cx);}
                    }
                    this.emit(id, "click", selected.iter().map(ToString::to_string).collect::<Vec<_>>().join(","), cx);
                })).into_any_element()
            }
            "InputGroup" => {
                // shadcn composition: one Input or Textarea child and InputGroupAddon children.
                let mut group = InputGroup::new(element_id).disabled(kit.flag("disabled")).readonly(kit.flag("readOnly")).invalid(kit.flag("invalid"));
                if let Some(size) = kit_size(kit) {group = group.with_size(size);}
                if let Some(label) = kit.prop("accessibilityLabel") {group = group.aria_label(label.to_owned());}
                for child in node.visible_children() {
                    if child.editable.is_some() {
                        match self.inputs.get(&child.identity.get()) {
                            Some((Field::Line(state), _)) => group = group.input(input_options!(Input::new(state), child, true, self, cx)),
                            Some((Field::Text(state), _)) => group = group.input(input_options!(Textarea::new(state), child, false, self, cx)),
                            None => {}
                        }
                        continue;
                    }
                    let Some(part) = child.kit.as_deref().filter(|part| part.name == "InputGroupAddon") else {continue};
                    let align = match part.prop("align") {Some("inline-end") => InputGroupAddonAlignment::InlineEnd, Some("block-start") => InputGroupAddonAlignment::BlockStart, Some("block-end") => InputGroupAddonAlignment::BlockEnd, _ => InputGroupAddonAlignment::InlineStart};
                    let mut addon = InputGroupAddon::new(ElementId::Integer(child.identity.get())).align(align);
                    for item in child.visible_children() {
                        addon = match item.kit.as_deref().map(|kit| kit.name.as_str()) {
                            Some("InputGroupText") => addon.child(InputGroupText::new().children(self.content(item, cx))),
                            Some("InputGroupButton") => addon.child(self.input_group_button(item, cx)),
                            _ => {let theme = utility_theme(cx); addon.child(self.element(item, &theme, mode(cx), cx))}
                        };
                    }
                    group = group.addon(addon);
                }
                group.into_any_element()
            }
            "Diff" => match self.states.get(&id) {
                Some(KitState::Diff(state, _)) => {
                    let mut diff=Diff::new(state).size_full()
                        .line_number(kit.prop("lineNumbers")!=Some("false"))
                        .syntax_highlight(kit.prop("syntaxHighlight")!=Some("false"))
                        .header_visible(kit.prop("headerVisible")!=Some("false"))
                        .change_background(kit.prop("changeBackground")!=Some("false"))
                        .soft_wrap(kit.flag("softWrap"))
                        .hover_highlight(match kit.prop("hoverHighlight") {Some("line")=>DiffHoverHighlight::Line,Some("line-number")=>DiffHoverHighlight::LineNumber,Some("both")=>DiffHoverHighlight::Both,_=>DiffHoverHighlight::None})
                        .hunk_separator(match kit.prop("hunkSeparator") {Some("line-info")=>DiffHunkSeparator::LineInfo,Some("simple")=>DiffHunkSeparator::Simple,_=>DiffHunkSeparator::Metadata})
                        .change_indicator(match kit.prop("changeIndicator") {Some("bars")=>DiffChangeIndicator::Bars,Some("none")=>DiffChangeIndicator::None,_=>DiffChangeIndicator::Signs});
                    if kit.handles("lineClick") {let root=cx.entity().downgrade();diff=diff.on_line_click(move |position,_,_,cx| {let value=diff_position(position);let _=root.update(cx,|root,cx|root.emit(id,"lineClick",value,cx));});}
                    if kit.handles("lineHover") {let root=cx.entity().downgrade();diff=diff.on_line_hover(move |position,_,cx| {let value=position.map_or_else(||"null".into(),diff_position);let _=root.update(cx,|root,cx|root.emit(id,"lineHover",value,cx));});}
                    if kit.handles("addAnnotation") {let root=cx.entity().downgrade();diff=diff.on_add_annotation(move |range,_,cx| {let value=diff_range(range);let _=root.update(cx,|root,cx|root.emit(id,"addAnnotation",value,cx));});}
                    let position=|item:&Kit|DiffLinePosition::new(item.text("path"),if item.prop("side")==Some("original"){DiffSide::Original}else{DiffSide::Modified},item.index("line").unwrap_or(1));
                    let mut annotations=Vec::new();let mut annotation_nodes=HashMap::new();
                    for (child,item) in items(node,"DiffAnnotation") {
                        let key=ElementId::Integer(child.identity.get());annotation_nodes.insert(key.clone(),child.identity.get());
                        annotations.push(if item.prop("line").is_some(){DiffAnnotation::line(key,position(item))}else{DiffAnnotation::file(key,item.text("path"))});
                    }
                    if !annotations.is_empty() {let root=cx.entity().downgrade();diff=diff.annotations(annotations).render_annotation(move |annotation,_,cx| {
                        root.update(cx,|root,cx|annotation_nodes.get(annotation.id()).map_or_else(||div().into_any_element(),|id|root.slot_of(*id,cx))).unwrap_or_else(|_|div().into_any_element())
                    });}
                    let decorations=items(node,"DiffDecoration").map(|(_,item)| {
                        let side=if item.prop("side")==Some("original"){DiffSide::Original}else{DiffSide::Modified};
                        let end_side=match item.prop("endSide") {Some("original")=>DiffSide::Original,Some("modified")=>DiffSide::Modified,_=>side};
                        let start=item.index("start").unwrap_or(1);
                        let mut decoration=DiffDecoration::new(DiffLineRange::new(item.text("path"),side,start,item.index("end").unwrap_or(start)).with_end_side(end_side));
                        if let Some(color)=theme_color(&item.text("color"),cx) {decoration=decoration.with_background(color);}
                        decoration
                    }).collect::<Vec<_>>();
                    diff=diff.decorations(decorations);
                    let mut markers=Vec::new();let mut marker_nodes=HashMap::new();
                    for (child,item) in items(node,"DiffGutterMarker") {let key=ElementId::Integer(child.identity.get());marker_nodes.insert(key.clone(),child.identity.get());markers.push(DiffGutterMarker::new(key,position(item)));}
                    if !markers.is_empty() {let root=cx.entity().downgrade();diff=diff.gutter_markers(markers).render_gutter_marker(move |marker,_,cx| {
                        root.update(cx,|root,cx|marker_nodes.get(marker.id()).map_or_else(||div().into_any_element(),|id|root.slot_of(*id,cx))).unwrap_or_else(|_|div().into_any_element())
                    });}
                    for part in ["content","prefix","title-suffix","suffix"] {
                        let Some((slot,_))=items(node,"DiffHeader").find(|(_,item)|item.prop("part").unwrap_or("content")==part) else {continue};
                        let target=slot.identity.get();let root=cx.entity().downgrade();
                        let render=move |_:&DiffFile,_:&mut Window,cx:&mut App|root.update(cx,|root,cx|root.slot_of(target,cx)).unwrap_or_else(|_|div().into_any_element());
                        diff=match part {"prefix"=>diff.render_header_prefix(render),"title-suffix"=>diff.render_header_title_suffix(render),"suffix"=>diff.render_header_suffix(render),_=>diff.render_header(render)};
                    }
                    diff.into_any_element()
                },
                _ => div().into_any_element(),
            },
            "Marker" => {
                // MarkerIcon and MarkerContent children, or the icon and text props.
                let variant = match kit.prop("variant") {Some("separator") => MarkerVariant::Separator, Some("border") => MarkerVariant::Border, _ => MarkerVariant::Plain};
                let mut marker = Marker::new().id(element_id).with_variant(variant).loading(kit.flag("loading")).separator_style(self.class_style(&kit.text("separatorClassName"), cx)).refine_style(&style);
                if kit.prop("loadingStyle") == Some("shimmer") {marker = marker.with_loading_style(MarkerLoadingStyle::Shimmer);}
                if let Some(style) = shimmer_style(kit, cx) {marker = marker.with_shimmer_style(style);}
                match kit.prop("alignment") {Some("start") => marker = marker.alignment(MarkerAlignment::Start), Some("center") => marker = marker.alignment(MarkerAlignment::Center), Some("end") => marker = marker.alignment(MarkerAlignment::End), _ => {}}
                if kit.prop("role") == Some("presentation") {marker = marker.role(gpui_kit::base::RoleOverride::Presentational);}
                match first_item(node, "MarkerIcon") {
                    Some(part) => marker = marker.icon(MarkerIcon::new().refine_style(&self.node_style(part)).children(self.content(part, cx))),
                    None => if let Some(icon) = icon_name(&kit.text("icon")) {marker = marker.icon(MarkerIcon::new().child(Icon::new(icon)));},
                }
                match first_item(node, "MarkerContent") {
                    // Plain text keeps Kit's text treatment (shimmer while loading); rich content renders as children.
                    Some(part) if part.visible_children().all(|child| child.kind == Kind::Text && child.kit.is_none()) => marker = marker.content(MarkerContent::new().refine_style(&self.node_style(part)).text(part.content_text())),
                    Some(part) => marker = marker.content(MarkerContent::new().refine_style(&self.node_style(part)).children(self.content(part, cx))),
                    None => if !kit.text("text").is_empty() {marker = marker.content(MarkerContent::new().text(text("text")));},
                }
                marker.children(self.content_except(node, &["MarkerIcon", "MarkerContent"], cx)).into_any_element()
            }
            "Attachment" => self.attachment(node, kit, cx).into_any_element(),
            "AttachmentGroup" => {
                let attachments = items(node, "Attachment").map(|(child, item)| self.attachment(child, item, cx)).collect::<Vec<_>>();
                let mut group = AttachmentGroup::new(element_id).refine_style(&style).children(attachments);
                if let Some(color) = kit.prop("edgeFade").and_then(|color| theme_color(color, cx)) {group = group.with_edge_fade(color);}
                group.into_any_element()
            }
            "DataTable" => match self.states.get(&id) {
                Some(KitState::Table(state, _)) => {
                    let mut table = DataTable::new(state).stripe(kit.flag("stripe")).bordered(kit.prop("bordered") != Some("false")).fixed_cols_border(kit.flag("fixedColumnsBorder"))
                        .scrollbar_visible(kit.prop("scrollbarVisible") != Some("false"), kit.prop("horizontalScrollbarVisible") != Some("false"));
                    if let Some(label) = kit.prop("accessibilityLabel") {table = table.accessibility_label(label.to_owned());}
                    if let Some(bounce) = kit.prop("scrollBounce") {table = table.scroll_bounce(bounce == "true");}
                    if let Some(height) = kit.number("headerRowHeight") {table = table.header_row_height(px(height as f32));}
                    if let Some(size) = kit_size(kit) {table = table.with_size(size);}
                    table.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Settings" => {
                let mut pages = Vec::new();
                for (page_node, page) in items(node, "SettingsPage") {
                    let mut entry = SettingPage::new(page.text("title"));
                    if !page.text("description").is_empty() {entry = entry.description(page.text("description"));}
                    for (group_node, group) in items(page_node, "SettingsGroup") {
                        let mut set = SettingGroup::new().title(group.text("title"));
                        if !group.text("description").is_empty() {set = set.description(group.text("description"));}
                        for (item_node, item) in items(group_node, "SettingItem") {
                            // The control renders later, inside Settings, through the root.
                            let (child, root) = (item_node.identity.get(), root.clone());
                            let field = SettingField::<SharedString>::render(move |_: &RenderOptions, _: &mut Window, cx: &mut App| {
                                root.update(cx, |root, cx| div().flex().items_center().children(root.content_of(child, cx))).unwrap_or_else(|_| div())
                            });
                            let mut row = SettingItem::new(item.text("title"), field);
                            if !item.text("description").is_empty() {row = row.description(item.text("description"));}
                            set = set.item(row);
                        }
                        entry = entry.group(set);
                    }
                    pages.push(entry);
                }
                Settings::new(element_id).pages(pages).into_any_element()
            }
            "SpeechButton" => match self.states.get(&id) {
                Some(KitState::Speech(state)) => {
                    let capturing = state.read(cx).status().is_capturing();
                    let mut row = div().flex().items_center().gap_2();
                    if capturing && kit.flag("waveform") {row = row.child(SpeechWaveform::new(state).w(px(48.)));}
                    row.child(SpeechButton::new(state)).into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Plot" => {
                let marks = node.visible_children().filter_map(|child| child.kit.as_deref()).filter(|mark| matches!(mark.name.as_str(), "PlotLine" | "PlotArea" | "PlotBars")).enumerate().map(|(index, mark)| Mark {
                    label:mark.prop("label").map_or_else(||format!("Series {}",index+1),str::to_owned),
                    kind: match mark.name.as_str() {"PlotArea" => "area", "PlotBars" => "bars", _ => "line"},
                    values: mark.text("values").split('\n').filter_map(|value| value.trim().parse().ok()).collect(),
                    color: plot_color(&mark.text("color"), index, cx),
                    curve: match mark.prop("curve") {Some("linear") => Curve::Linear, Some("step") => Curve::StepAfter, _ => Curve::Natural},
                    dots: mark.flag("dots"),
                    width: mark.number("width").unwrap_or(2.) as f32,
                }).collect();
                let labels=serde_json::from_str::<Vec<String>>(kit.prop("labels").unwrap_or("[]")).unwrap_or_default().into_iter().map(SharedString::from).collect();
                let tooltip=first_item(node,"PlotTooltip").and_then(|tip|tip.kit.as_ref().map(|kit| {
                    let flag=|name:&str,default|kit.prop(name).map_or(default,|value|value=="true");
                    let rows=items(tip,"PlotTooltipRow").map(|(_,row)|PlotTipRow {label:row.text("label"),value:row.text("value"),color:row.prop("color").map(|value|plot_color(value,0,cx)),value_color:row.prop("valueColor").map(|value|plot_color(value,0,cx))}).collect();
                    PlotTip {glide:flag("glide",true),progress:kit.number("progress").map(|value|value as f32),title:kit.prop("title").map(str::to_owned),series:flag("showSeries",true),gap:kit.number("gap").unwrap_or(8.) as f32,cross:kit.text("crossLine"),dots:flag("dots",true),appearance:flag("appearance",true),rows,style:self.node_style(tip)}
                }));
                div().size_full().child(Marks {id,tooltip,labels, marks, axis: kit.flag("axis"), grid: kit.flag("grid"), border: cx.theme().border, muted: cx.theme().muted_foreground}).into_any_element()
            }
            "Questionnaire" => match self.states.get(&id) {
                Some(KitState::Questionnaire(state)) => {
                    let mut survey = Questionnaire::new(state).child(QuestionnaireProgress::new(state));
                    for (child, question) in items(node, "Question") {
                        let name = question.text("id");
                        let choices = items(child, "QuestionChoice").fold(QuestionnaireChoices::new(state, name.clone()), |list, (_, choice)| list.child(QuestionnaireChoice::new(state, name.clone(), choice.text("id"))));
                        let mut item = QuestionnaireItem::new(state, name.clone()).child(QuestionnaireTitle::new(state, name.clone()));
                        if !question.text("description").is_empty() {item = item.child(QuestionnaireDescription::new(state, name.clone()));}
                        survey = survey.child(item
                            .child(choices)
                            .child(QuestionnaireError::new(state, name)));
                    }
                    survey.child(QuestionnaireActions::new(state)
                        .child(QuestionnairePrevious::new(state)).child(QuestionnaireSkip::new(state))
                        .child(QuestionnaireNext::new(state)).child(QuestionnaireSubmit::new(state))).into_any_element()
                }
                _ => div().into_any_element(),
            },
            "Dock" => match self.states.get(&id) {
                Some(KitState::Dock(area)) => div().size_full().child(area.clone()).into_any_element(),
                _ => div().into_any_element(),
            },
            "MessageScroller" => match self.states.get(&id) {
                Some(KitState::Scroller(state, _)) => {
                    let root = root.clone();
                    let mut scroller = MessageScroller::new(element_id, state.clone(), move |index, _, cx| {
                        root.update(cx, |root, cx| root.child_of(id, index, cx)).unwrap_or_else(|_| div().into_any_element())
                    }).size_full();
                    if !kit.text("jumpLabel").is_empty() {scroller = scroller.with_jump_button_label(kit.text("jumpLabel"));}
                    if let Some(visible) = kit.prop("scrollbar") {scroller = scroller.scrollbar(visible == "true");}
                    if let Some(visible) = kit.prop("jumpButton") {scroller = scroller.jump_button(visible == "true");}
                    if let Some(duration) = kit.index("jumpButtonTransition") {scroller = scroller.with_jump_button_transition(std::time::Duration::from_millis(duration as u64));}
                    if let Some(color) = kit.prop("bottomFade").and_then(|color| theme_color(color, cx)) {scroller = scroller.with_bottom_fade(color);}
                    for (key, apply) in [("contentClassName", 0), ("listClassName", 1), ("rowClassName", 2), ("jumpButtonClassName", 3)] {
                        let Some(classes) = kit.prop(key) else {continue};
                        let part = self.class_style(classes, cx);
                        scroller = match apply {0 => scroller.with_content_style(part), 1 => scroller.with_list_style(part), 2 => scroller.with_row_style(part), _ => scroller.with_jump_button_style(part)};
                    }
                    scroller.into_any_element()
                }
                _ => div().into_any_element(),
            },
            "AppMenuBar" => match self.states.get(&id) {
                Some(KitState::AppMenuBar(bar,_)) if !cfg!(target_os="macos") => div().child(bar.clone()).into_any_element(),
                _ => div().into_any_element(),
            },
            "Shortcut" | "Ticker" => div().into_any_element(),
            // Item and slot components render through their container.
            // A trigger nested inside other content toggles its nearest Collapsible.
            "CollapsibleTrigger" => match ancestor_named(&self.node, id, "Collapsible") {
                Some((owner, open)) => div().id(element_id).children(self.content(node, cx)).on_click(cx.listener(move |this, _, _, cx| this.emit(owner, "openChange", (!open).to_string(), cx))).into_any_element(),
                None => div().children(self.content(node, cx)).into_any_element(),
            },
            name => div().child(SharedString::from(format!("{name} must be placed inside its container component"))).into_any_element(),
        }
    }
}

/// TreeItem children as GPUI Kit tree items, recursively.
fn tree_items(node: &Node) -> Vec<TreeItem> {
    items(node, "TreeItem").map(|(child, item)| {
        // A TreeItem's id defaults to its label; expanded/defaultExpanded opens a folder.
        let label = item.text("label");
        let id = item.prop("id").map_or_else(|| label.clone(), str::to_owned);
        TreeItem::new(id, label).expanded(item.prop("expanded").map_or(item.flag("defaultExpanded"), |open| open == "true")).disabled(item.flag("disabled")).children(tree_items(child))
    }).collect()
}

/// Select a tree item by id, expanding its ancestors first so hidden items can be revealed.
fn tree_select(state: &mut TreeState, id: &SharedString, cx: &mut Context<TreeState>) {
    state.reveal_item(id, gpui::ScrollStrategy::Center, cx);
    if let Some(index) = state.index_of(id) {state.set_selected_index(Some(index), cx);}
}

/// Icons per tree item id (TreeItem `icon`), else folder/file by shape.
fn tree_icons(node: &Node, out: &mut HashMap<String, String>) {
    for (child, item) in items(node, "TreeItem") {
        let label = item.text("label");
        if let Some(icon) = item.prop("icon") {out.insert(item.prop("id").map_or(label, str::to_owned), icon.to_owned());}
        tree_icons(child, out);
    }
}

/// A fingerprint of a tree's items, to rebuild the Kit state only when they change.
fn tree_shape(root: &Node, id: u64) -> String {
    fn walk(node: &Node, out: &mut String) {
        for (child, item) in items(node, "TreeItem") {
            for key in ["id", "label", "icon", "expanded", "disabled"] {out.push_str(&item.text(key)); out.push('\u{1}');}
            out.push('['); walk(child, out); out.push(']');
        }
    }
    let mut out = String::new();
    if let Some(node) = find(root, id) {walk(node, &mut out);}
    out
}
