//! Native terminal view; VT parsing and PTY ownership live in terminal.rs.
use super::*;
use super::terminal::{Session, Size as TerminalSize};
use alacritty_terminal::{
    event::WindowSize, grid::{Dimensions, Scroll}, index::{Column, Line, Side},
    selection::{Selection, SelectionType}, term::{cell::Flags, TermMode, viewport_to_point},
    vte::ansi::{Color, CursorShape},
};
use std::ops::Range;

#[derive(Default)]
struct TerminalCleanup {pending: Vec<async_channel::Receiver<()>>}
impl Global for TerminalCleanup {}

fn register_cleanup(completion: async_channel::Receiver<()>, cx: &mut App) {
    if !cx.has_global::<TerminalCleanup>() {
        cx.set_global(TerminalCleanup::default());
        cx.on_app_quit(|cx| {
            let pending = std::mem::take(&mut cx.global_mut::<TerminalCleanup>().pending);
            async move {for completion in pending {let _ = completion.recv().await;}}
        }).detach();
    }
    let pending = &mut cx.global_mut::<TerminalCleanup>().pending;
    pending.retain(|completion| !completion.is_closed());
    pending.push(completion);
}

pub(super) enum TerminalEvent {Exit, OpenLink(String)}
impl EventEmitter<TerminalEvent> for TerminalView {}

pub(super) struct TerminalView {
    session: Option<Session>,
    generation: u64,
    directory: String,
    program: String,
    args: Vec<String>,
    focus: FocusHandle,
    font_size: f32,
    bounds: Bounds<Pixels>,
    cell: Size<Pixels>,
    selecting: bool,
    composition: String,
    composition_selection: Range<usize>,
    error: String,
    _events: Task<()>,
    _quit: Subscription,
    _focus: Vec<Subscription>,
    scroll_remainder: f32,
    exit_notified: bool,
}

impl TerminalView {
    pub fn new(directory: String, program: String, args: Vec<String>, font_size: f32, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let command = (!program.is_empty()).then(|| alacritty_terminal::tty::Shell::new(program.clone(), args.clone()));
        let session = Session::start(std::path::Path::new(&directory), command, TerminalSize(WindowSize {num_cols: 80, num_lines: 24, cell_width: 8, cell_height: 16}), 10000);
        let (session, error) = match session {Ok(session) => (Some(session), String::new()), Err(error) => (None, error.to_string())};
        let wake = session.as_ref().map(|session| session.wake.clone());
        let events = cx.spawn(async move |this, cx| {
            if wake.is_none() {let _ = this.update(cx, |this, cx| {this.exit_notified = true; cx.emit(TerminalEvent::Exit); cx.notify();});}
            if let Some(wake) = wake {while wake.recv().await.is_ok() {cx.background_executor().timer(std::time::Duration::from_millis(16)).await; if this.update(cx, |this, cx| {if this.session.as_ref().is_some_and(|session| session.status().exited) && !this.exit_notified {this.exit_notified = true; cx.emit(TerminalEvent::Exit);} cx.notify();}).is_err() {break;}}}
        });
        if let Some(session) = &session {register_cleanup(session.completed.clone(), cx);}
        let quit = cx.on_app_quit(|this, _| {
            let completion = this.session.as_mut().map(|session| {session.close(); session.completed.clone()});
            async move {if let Some(completion) = completion {let _ = completion.recv().await;}}
        });
        let focus = cx.focus_handle();
        let focused = cx.on_focus(&focus, window, |this, _, cx| {if this.mode().contains(TermMode::FOCUS_IN_OUT) {if let Some(session) = &this.session {let _ = session.write(b"\x1b[I".to_vec());}} cx.notify();});
        let blurred = cx.on_blur(&focus, window, |this, _, cx| {if this.mode().contains(TermMode::FOCUS_IN_OUT) {if let Some(session) = &this.session {let _ = session.write(b"\x1b[O".to_vec());}} cx.notify();});
        static NEXT_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {session, generation: NEXT_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed), directory, program, args, focus, font_size, bounds: Bounds::default(), cell: size(px(8.), px(16.)), selecting: false, composition: String::new(), composition_selection: 0..0, error, _events: events, _quit: quit, _focus: vec![focused, blurred], scroll_remainder: 0., exit_notified: false}
    }

    pub fn configure(&mut self, font_size: f32, cx: &mut Context<Self>) {if self.font_size != font_size {self.font_size = font_size; cx.notify();}}
    fn mode(&self) -> TermMode {self.session.as_ref().map(|session| *session.terminal.lock().mode()).unwrap_or_default()}
    fn write(&mut self, bytes: Vec<u8>, cx: &mut Context<Self>) -> Result<(), String> {
        if bytes.len() > 1024 * 1024 {return Err("Terminal input exceeds 1 MiB per request".into());}
        let session = self.session.as_ref().ok_or("Terminal session is unavailable")?;
        session.write(bytes)?;
        {let mut term = session.terminal.lock(); term.selection = None; term.scroll_display(Scroll::Bottom);}
        cx.notify();
        Ok(())
    }
    fn input(&mut self, bytes: Vec<u8>, cx: &mut Context<Self>) {if let Err(error) = self.write(bytes, cx) {self.error = error; cx.notify();}}
    fn paste_text(&mut self, text: &str, cx: &mut Context<Self>) -> Result<(), String> {
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        let bytes = if self.mode().contains(TermMode::BRACKETED_PASTE) {format!("\x1b[200~{}\x1b[201~", text.replace('\x1b', "").replace('\u{009b}', "")).into_bytes()} else {text.replace('\n', "\r").into_bytes()};
        self.write(bytes, cx)
    }
    fn point(&self, position: Point<Pixels>) -> alacritty_terminal::index::Point {
        let Some(session) = &self.session else {return alacritty_terminal::index::Point::new(Line(0), Column(0));};
        let term = session.terminal.lock();
        let column = (((position.x - self.bounds.left()).as_f32() / self.cell.width.as_f32()).floor().max(0.) as usize).min(term.columns() - 1);
        let line = (((position.y - self.bounds.top()).as_f32() / self.cell.height.as_f32()).floor().max(0.) as usize).min(term.screen_lines() - 1);
        viewport_to_point(term.grid().display_offset(), alacritty_terminal::index::Point::new(line, Column(column)))
    }
    fn link_at(&self, position: Point<Pixels>) -> Option<String> {
        let point = self.point(position);
        let term = self.session.as_ref()?.terminal.lock();
        let grid = term.grid();
        let mut first = point.line;
        while first > grid.topmost_line() && grid[first - 1usize][Column(grid.columns() - 1)].flags.contains(Flags::WRAPLINE) {first -= 1;}
        let mut text = String::new();
        let mut hovered = None;
        let mut row = first;
        loop {
            for column in 0..grid.columns() {
                let cell = &grid[row][Column(column)];
                if row == point.line && column == point.column.0 {hovered = Some(text.len());}
                if cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {continue;}
                text.push(cell.c);
                if let Some(chars) = cell.zerowidth() {text.extend(chars);}
                if text.len() > 8192 {return None;}
            }
            if row >= grid.bottommost_line() || !grid[row][Column(grid.columns() - 1)].flags.contains(Flags::WRAPLINE) {break;}
            row += 1;
        }
        let link = super::super::terminal_links::file_link(&text, hovered?)?;
        Some(serde_json::json!({"path": link.path, "line": link.line, "column": link.column}).to_string())
    }
    fn mouse(&mut self, button: u8, position: Point<Pixels>, modifiers: Modifiers, released: bool, motion: bool, cx: &mut Context<Self>) -> bool {
        let mode = self.mode();
        if modifiers.shift || !mode.intersects(TermMode::MOUSE_MODE) {return false;}
        if motion && !mode.intersects(TermMode::MOUSE_MOTION | TermMode::MOUSE_DRAG) {return true;}
        let column = ((position.x-self.bounds.left()).as_f32()/self.cell.width.as_f32()).floor().max(0.) as u32 + 1;
        let row = ((position.y-self.bounds.top()).as_f32()/self.cell.height.as_f32()).floor().max(0.) as u32 + 1;
        let (columns, rows) = self.session.as_ref().map(|session| {let term = session.terminal.lock(); (term.columns() as u32, term.screen_lines() as u32)}).unwrap_or((1,1));
        let column = column.min(columns); let row = row.min(rows);
        let code = button + if motion {32} else {0} + if modifiers.alt {8} else {0} + if modifiers.control {16} else {0};
        let bytes = if mode.contains(TermMode::SGR_MOUSE) {format!("\x1b[<{code};{column};{row}{}", if released {'m'} else {'M'}).into_bytes()}
            else if mode.contains(TermMode::UTF8_MOUSE) {format!("\x1b[M{}{}{}", char::from(if released {35} else {code+32}), char::from_u32((column+32).min(2047)).unwrap(), char::from_u32((row+32).min(2047)).unwrap()).into_bytes()}
            else {vec![27,b'[',b'M',if released {35} else {code+32}, (column+32).min(255) as u8, (row+32).min(255) as u8]};
        self.input(bytes, cx); true
    }
    fn copy(&self, cx: &mut Context<Self>) {
        if let Some(text) = self.session.as_ref().and_then(|session| session.terminal.lock().selection_to_string()).filter(|text| !text.is_empty()) {cx.write_to_clipboard(ClipboardItem::new_string(text));}
    }
    fn key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        if !self.composition.is_empty() && !key.modifiers.control && !key.modifiers.platform {return;}
        if key.modifiers.platform || (cfg!(windows) && key.modifiers.control && key.modifiers.shift) {
            if key.key == "c" {self.copy(cx); cx.stop_propagation();}
            if key.key == "v" {if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {if let Err(error) = self.paste_text(&text, cx) {self.error = error; cx.notify();}} cx.stop_propagation();}
            return;
        }
        let modifier = 1 + usize::from(key.modifiers.shift) + 2 * usize::from(key.modifiers.alt) + 4 * usize::from(key.modifiers.control);
        let cursor = match key.key.as_str() {"up" => Some('A'), "down" => Some('B'), "right" => Some('C'), "left" => Some('D'), "home" => Some('H'), "end" => Some('F'), _ => None};
        let bytes = if let Some(last) = cursor {
            Some(if modifier != 1 {format!("\x1b[1;{modifier}{last}")} else if self.mode().contains(TermMode::APP_CURSOR) {format!("\x1bO{last}")} else {format!("\x1b[{last}")}.into_bytes())
        } else {
            match key.key.as_str() {
                "enter" => Some(vec![b'\r']), "backspace" => Some(vec![if key.modifiers.control {8} else {127}]),
                "escape" => Some(vec![27]), "tab" => Some(if key.modifiers.shift {b"\x1b[Z".to_vec()} else {vec![b'\t']}),
                "insert" | "delete" | "pageup" | "pagedown" => {let code = match key.key.as_str() {"insert" => 2, "delete" => 3, "pageup" => 5, _ => 6}; Some(if modifier == 1 {format!("\x1b[{code}~")} else {format!("\x1b[{code};{modifier}~")}.into_bytes())},
                _ if key.key.starts_with('f') && key.key[1..].parse::<usize>().is_ok_and(|number| (1..=12).contains(&number)) => {
                    let number = key.key[1..].parse::<usize>().unwrap();
                    Some(if number <= 4 {let last = (b'P' + number as u8 - 1) as char; if modifier == 1 {format!("\x1bO{last}")} else {format!("\x1b[1;{modifier}{last}")}} else {let code = [15, 17, 18, 19, 20, 21, 23, 24][number-5]; if modifier == 1 {format!("\x1b[{code}~")} else {format!("\x1b[{code};{modifier}~")}}.into_bytes())
                },
                _ if key.modifiers.control => {let byte = if key.key == "space" {Some(0)} else {key.key.as_bytes().first().filter(|_| key.key.len() == 1).and_then(|byte| {let byte = byte.to_ascii_uppercase(); (b'@'..=b'_').contains(&byte).then_some(byte & 31)})}; byte.map(|byte| vec![byte])},
                _ => None,
            }
        };
        if let Some(bytes) = bytes {self.input(bytes, cx); cx.stop_propagation();}
    }

    pub fn command(&mut self, value: &serde_json::Value, window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        let expected = value.get("expectedGeneration").map(|value| value.as_u64().ok_or("Invalid terminal generation")).transpose()?.unwrap_or(0);
        if expected != 0 && expected != self.generation {return Err("Terminal session changed".into());}
        let action = value["action"].as_str().ok_or("Missing terminal action")?;
        match action {
            "restart" => {
                if self.session.as_ref().is_some_and(|session| !session.status().exited) {return Err("Exit the running shell before restarting it".into());}
                let focused = self.focus.is_focused(window);
                *self = Self::new(self.directory.clone(), self.program.clone(), self.args.clone(), self.font_size, window, cx);
                if focused {self.focus.focus(window, cx);}
            },
            "focus" => self.focus.focus(window, cx),
            "write" => self.write(value["text"].as_str().ok_or("Missing terminal input")?.as_bytes().to_vec(), cx)?,
            "paste" => self.paste_text(value["text"].as_str().ok_or("Missing terminal paste text")?, cx)?,
            "interrupt" => self.write(vec![3], cx)?,
            "copy" => self.copy(cx),
            "select" => {
                let range = value["selection"].as_array().filter(|range| range.len()==4).ok_or("Terminal selection requires four viewport coordinates")?;
                let coordinates = range.iter().map(|value| value.as_u64().and_then(|value| usize::try_from(value).ok()).ok_or("Invalid terminal selection coordinate")).collect::<Result<Vec<_>,_>>()?;
                let session = self.session.as_ref().ok_or("Terminal session is unavailable")?; let mut term = session.terminal.lock();
                if coordinates[0]>=term.screen_lines() || coordinates[2]>=term.screen_lines() || coordinates[1]>=term.columns() || coordinates[3]>=term.columns() {return Err("Terminal selection is outside the visible grid".into());}
                let start = viewport_to_point(term.grid().display_offset(), alacritty_terminal::index::Point::new(coordinates[0], Column(coordinates[1])));
                let end = viewport_to_point(term.grid().display_offset(), alacritty_terminal::index::Point::new(coordinates[2], Column(coordinates[3])));
                let mut selection = Selection::new(SelectionType::Simple, start, Side::Left); selection.update(end, Side::Right); term.selection = Some(selection);
            },
            "scroll" => {let lines = value["lines"].as_i64().and_then(|lines| i32::try_from(lines).ok()).ok_or("Terminal scroll requires a signed 32-bit line count")?; if let Some(session) = &self.session {session.terminal.lock().scroll_display(Scroll::Delta(lines));}},
            "close" => {if let Some(session) = &mut self.session {session.close();}},
            _ => return Err("Unknown terminal action".into()),
        }
        cx.notify();
        Ok(())
    }
    pub fn snapshot(&self, window: &Window) -> serde_json::Value {
        let Some(session) = &self.session else {return serde_json::json!({"found": true, "generation": self.generation, "exited": true, "error": self.error, "directory": self.directory});};
        let status = session.status(); let term = session.terminal.lock();
        let text = term.bounds_to_string(alacritty_terminal::index::Point::new(Line(-(term.grid().display_offset() as i32)), Column(0)), alacritty_terminal::index::Point::new(Line(term.screen_lines() as i32 - 1 - term.grid().display_offset() as i32), Column(term.columns()-1)));
        serde_json::json!({"found": true, "generation": self.generation, "directory": self.directory, "title": status.title, "exited": status.exited, "exitCode": status.code, "error": self.error, "text": text, "selectedText": term.selection_to_string().unwrap_or_default(), "columns": term.columns(), "rows": term.screen_lines(), "scrollOffset": term.grid().display_offset(), "focused": self.focus.is_focused(window)})
    }
}

impl EntityInputHandler for TerminalView {
    fn text_for_range(&mut self, range: Range<usize>, adjusted: &mut Option<Range<usize>>, _: &mut Window, _: &mut Context<Self>) -> Option<String> {let units = self.composition.encode_utf16().collect::<Vec<_>>(); let range = range.start.min(units.len())..range.end.min(units.len()); *adjusted = Some(range.clone()); String::from_utf16(units.get(range)?).ok()}
    fn selected_text_range(&mut self, _: bool, _: &mut Window, _: &mut Context<Self>) -> Option<UTF16Selection> {Some(UTF16Selection {range: self.composition_selection.clone(), reversed: false})}
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {(!self.composition.is_empty()).then(|| 0..self.composition.encode_utf16().count())}
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {self.composition.clear(); self.composition_selection = 0..0; cx.notify();}
    fn replace_text_in_range(&mut self, _: Option<Range<usize>>, text: &str, _: &mut Window, cx: &mut Context<Self>) {self.composition.clear(); self.composition_selection = 0..0; self.input(text.as_bytes().to_vec(), cx);}
    fn replace_and_mark_text_in_range(&mut self, _: Option<Range<usize>>, text: &str, selected: Option<Range<usize>>, _: &mut Window, cx: &mut Context<Self>) {self.composition = text.to_owned(); let length = text.encode_utf16().count(); self.composition_selection = selected.map(|range| range.start.min(length)..range.end.min(length)).unwrap_or(length..length); cx.notify();}
    fn bounds_for_range(&mut self, _: Range<usize>, _: Bounds<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<Bounds<Pixels>> {let session = self.session.as_ref()?; let term = session.terminal.lock(); let cursor = term.renderable_content().cursor.point; Some(Bounds::new(point(self.bounds.left() + self.cell.width * cursor.column.0 as f32, self.bounds.top() + self.cell.height * (cursor.line.0 + term.grid().display_offset() as i32) as f32), self.cell))}
    fn character_index_for_point(&mut self, _: Point<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {Some(self.composition_selection.end)}
    fn paste(&mut self, item: ClipboardItem, _: &mut Window, cx: &mut Context<Self>) {if let Some(text) = item.text() {if let Err(error) = self.paste_text(&text, cx) {self.error = error; cx.notify();}}}
}

impl Render for TerminalView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity(); let painting = entity.clone();
        let background = cx.theme().background;
        let foreground = cx.theme().foreground;
        let selection = cx.theme().primary.opacity(0.3);
        let face = font(native_font_family(2)); let font_size = px(self.font_size);
        let output = self.session.as_ref().map(|session| {let term = session.terminal.lock(); let offset = term.grid().display_offset() as i32; term.bounds_to_string(alacritty_terminal::index::Point::new(Line(-offset), Column(0)), alacritty_terminal::index::Point::new(Line(term.screen_lines() as i32-1-offset), Column(term.columns()-1)))}).unwrap_or_default();
        div().id("terminal-view").size_full().min_h_0().overflow_hidden().bg(background).text_size(font_size).role(Role::Terminal).aria_value(output).track_focus(&self.focus).key_context("Terminal")
            .on_key_down(cx.listener(Self::key))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, window, cx| {
                if event.modifiers.platform || (!cfg!(target_os="macos") && event.modifiers.control) {
                    if let Some(link) = this.link_at(event.position) {this.selecting = false; cx.emit(TerminalEvent::OpenLink(link)); cx.stop_propagation(); return;}
                }
                this.focus.focus(window, cx); if this.mouse(0, event.position, event.modifiers, false, false, cx) {return;} let position = this.point(event.position); this.selecting = true;
                if let Some(session) = &this.session {session.terminal.lock().selection = Some(Selection::new(if event.click_count > 1 {SelectionType::Semantic} else {SelectionType::Simple}, position, Side::Left));}
                cx.notify();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {if event.pressed_button==Some(MouseButton::Left) && this.mouse(0, event.position, event.modifiers, false, true, cx) {return;} if this.selecting {let point = this.point(event.position); if let Some(session) = &this.session {if let Some(selection) = &mut session.terminal.lock().selection {selection.update(point, Side::Right);}} cx.notify();}}))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, event: &MouseUpEvent, _, cx| {this.mouse(0, event.position, event.modifiers, true, false, cx); this.selecting = false;}))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|this, event: &MouseUpEvent, _, cx| {this.mouse(0, event.position, event.modifiers, true, false, cx); this.selecting = false;}))
            .on_scroll(cx.listener(|this, event: &ScrollEvent, _, cx| {let delta = match event.delta {ScrollDelta::Lines(delta) => delta.y, ScrollDelta::Pixels(delta) => delta.y.as_f32() / this.cell.height.as_f32()}; this.scroll_remainder += delta; let lines = this.scroll_remainder.trunc() as i32; this.scroll_remainder -= lines as f32; if lines != 0 {
                let reported = this.mouse(if lines>0 {64} else {65}, event.position, event.modifiers, false, false, cx);
                if reported {for _ in 1..lines.unsigned_abs() {this.mouse(if lines>0 {64} else {65}, event.position, event.modifiers, false, false, cx);}}
                else if this.mode().contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) && !event.modifiers.shift {for _ in 0..lines.unsigned_abs() {this.input(if lines>0 {b"\x1bOA".to_vec()} else {b"\x1bOB".to_vec()}, cx);}}
                else if let Some(session) = &this.session {session.terminal.lock().scroll_display(Scroll::Delta(lines));}
            } cx.stop_propagation(); cx.notify();}))
            .child(canvas(move |bounds, window, cx| {
                entity.update(cx, |this, _| {
                    let font_id = window.text_system().resolve_font(&face);
                    let width = window.text_system().em_advance(font_id, font_size).unwrap_or_else(|_| window.text_system().layout_width(font_id, font_size, 'm')).max(px(1.));
                    let height = px((window.text_system().ascent(font_id, font_size).as_f32().abs() + window.text_system().descent(font_id, font_size).as_f32().abs()) * 1.15).max(px(1.));
                    this.bounds = bounds; this.cell = size(width, height);
                    if let Some(session) = &this.session {session.set_colors(packed(foreground), packed(background)); if !session.status().exited {if let Err(error) = session.resize(TerminalSize(WindowSize {num_cols: (bounds.size.width.as_f32() / width.as_f32()).floor().clamp(2., u16::MAX as f32) as u16, num_lines: (bounds.size.height.as_f32() / height.as_f32()).floor().clamp(1., u16::MAX as f32) as u16, cell_width: width.as_f32().round().clamp(1., u16::MAX as f32) as u16, cell_height: height.as_f32().round().clamp(1., u16::MAX as f32) as u16})) {this.error = error;}}}
                });
            }, move |bounds, _, window, cx| {
                let (focus, cell_size, composition, content) = {
                    let this = painting.read(cx);
                    let content = this.session.as_ref().map(|session| {
                        let defaults = session.colors(); let term = session.terminal.lock(); let content = term.renderable_content();
                        (defaults, content.display_iter.map(|cell| (cell.point, cell.cell.clone())).collect::<Vec<_>>(), content.cursor, content.display_offset, *content.colors, content.selection, term.screen_lines())
                    });
                    (this.focus.clone(), this.cell, this.composition.clone(), content)
                };
                window.handle_input(&focus, ElementInputHandler::new(bounds, painting.clone()), cx);
                let Some((defaults, cells, cursor, offset, colors, selected, rows)) = content else {return;};
                let color = |value: Color| -> Hsla {let value = match value {Color::Spec(value) => value, Color::Named(name) => colors[name as usize].unwrap_or(defaults[name as usize]), Color::Indexed(index) => colors[index as usize].unwrap_or(defaults[index as usize])}; rgb((u32::from(value.r) << 16) | (u32::from(value.g) << 8) | u32::from(value.b)).into()};
                let font = font(native_font_family(2));
                // Paint every background before glyphs, so a wide-character spacer cannot erase half its glyph.
                for (position, cell) in &cells {
                    if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {continue;}
                    let origin = point(bounds.left() + cell_size.width * position.column.0 as f32, bounds.top() + cell_size.height * (position.line.0 + offset as i32) as f32);
                    let bg = color(if cell.flags.contains(Flags::INVERSE) {cell.fg} else {cell.bg});
                    let width = cell_size.width * if cell.flags.contains(Flags::WIDE_CHAR) {2.} else {1.};
                    window.paint_quad(fill(Bounds::new(origin, size(width, cell_size.height)), bg));
                    if selected.as_ref().is_some_and(|range| range.contains(*position)) {window.paint_quad(fill(Bounds::new(origin, size(width, cell_size.height)), selection));}
                }
                for (position, cell) in cells {
                    let origin = point(bounds.left() + cell_size.width * position.column.0 as f32, bounds.top() + cell_size.height * (position.line.0 + offset as i32) as f32);
                    let mut fg = color(if cell.flags.contains(Flags::INVERSE) {cell.bg} else {cell.fg});
                    if cell.flags.contains(Flags::DIM) {fg = fg.opacity(0.55);}
                    if cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER | Flags::HIDDEN) {continue;}
                    if cell.flags.intersects(Flags::ALL_UNDERLINES | Flags::STRIKEOUT) || cell.c != ' ' || cell.zerowidth().is_some_and(|chars| !chars.is_empty()) {
                        let mut text = cell.c.to_string(); if let Some(chars) = cell.zerowidth() {text.extend(chars);}
                        let mut face = font.clone(); if cell.flags.contains(Flags::BOLD) {face.weight = FontWeight::BOLD;} if cell.flags.contains(Flags::ITALIC) {face.style = FontStyle::Italic;}
                        let run = TextRun {len: text.len(), font: face, color: fg, background_color: None, underline: cell.flags.intersects(Flags::ALL_UNDERLINES).then_some(UnderlineStyle {color: Some(fg), thickness: px(1.), wavy: cell.flags.contains(Flags::UNDERCURL)}), strikethrough: cell.flags.contains(Flags::STRIKEOUT).then_some(StrikethroughStyle {color: Some(fg), thickness: px(1.)})};
                        let line = window.text_system().shape_line(text.into(), font_size, &[run], Some(cell_size.width)); let _ = line.paint(origin, cell_size.height, TextAlign::Left, None, window, cx);
                    }
                }
                let cursor_line = cursor.point.line.0 + offset as i32;
                if cursor.shape != CursorShape::Hidden && cursor_line >= 0 && cursor_line < rows as i32 {
                    let origin = point(bounds.left() + cell_size.width * cursor.point.column.0 as f32, bounds.top() + cell_size.height * cursor_line as f32);
                    let cursor_size = match cursor.shape {CursorShape::Beam => size(px(2.), cell_size.height), CursorShape::Underline => size(cell_size.width, px(2.)), _ => cell_size};
                    window.paint_quad(fill(Bounds::new(if cursor.shape == CursorShape::Underline {origin + point(px(0.), cell_size.height - px(2.))} else {origin}, cursor_size), foreground.opacity(if focus.is_focused(window) {0.55} else {0.2})));
                    if !composition.is_empty() {let run = TextRun {len: composition.len(), font, color: foreground, background_color: Some(background), underline: Some(UnderlineStyle {color: Some(foreground), thickness: px(1.), wavy: false}), strikethrough: None}; let line = window.text_system().shape_line(composition.clone().into(), font_size, &[run], None); let _ = line.paint(origin, cell_size.height, TextAlign::Left, None, window, cx);}
                }
            }).size_full())
            .children((!self.error.is_empty()).then(|| div().absolute().bottom_0().left_0().right_0().bg(background).text_color(cx.theme().danger).child(super::super::i18n::translate(&self.error,"{}"))))
    }
}

fn packed(color: Hsla) -> u32 {let color = color.to_rgb(); let channel = |value: f32| (value.clamp(0., 1.) * 255.).round() as u32; (channel(color.r) << 16) | (channel(color.g) << 8) | channel(color.b)}
