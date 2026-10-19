//! Native controls delegate shaping, selection and IME editing to Parley.
use parley::{FontContext, FontWeight, LayoutContext, StyleProperty, editing::PlainEditor};
use super::{TextStyle,inline::Brush};

#[derive(Clone)]
pub(super) struct Snapshot {value:String,anchor:usize,focus:usize}
pub(super) struct Editor {
    pub text: PlainEditor<Brush>,
    multiline: bool,
    composition: Option<(String, usize, usize)>,
    pub read_only: bool,
    pub disabled: bool,
    pub origin: (f64,f64),
    pub viewport: (f64,f64),
    pub scroll: (f64,f64),
    pub reveal:bool,
    undo:Vec<Snapshot>,
    redo:Vec<Snapshot>,
}

impl Editor {
    pub fn new(value: &str, multiline: bool) -> Self {
        let mut text = PlainEditor::new(16.);
        text.set_text(&normalize(value, multiline));
        Self { text, multiline, composition: None, read_only:false, disabled:false, origin:(0.,0.), viewport:(0.,0.), scroll:(0.,0.), reveal:true,undo:Vec::new(),redo:Vec::new() }
    }

    pub fn configure(&mut self, style: TextStyle, width: f32) {
        self.text.set_width(self.multiline.then_some(width.max(0.)));
        let styles = self.text.edit_styles();
        styles.insert(StyleProperty::FontSize(style.size));
        styles.insert(StyleProperty::FontWeight(FontWeight::new(style.weight)));
        styles.insert(StyleProperty::LineHeight(style.leading.map(parley::style::LineHeight::FontSizeRelative).unwrap_or(style.line_height)));
        styles.insert(StyleProperty::from(match style.family {
            1 => parley::style::GenericFamily::Serif,
            2 => parley::style::GenericFamily::Monospace,
            _ => parley::style::GenericFamily::SystemUi,
        }));
    }

    pub fn snapshot(&self)->Snapshot {
        if let Some((value,anchor,focus))=&self.composition{return Snapshot{value:value.clone(),anchor:*anchor,focus:*focus};}
        let selection=self.text.raw_selection();Snapshot{value:self.committed(),anchor:selection.anchor().index(),focus:selection.focus().index()}
    }
    pub fn record(&mut self,before:Snapshot) {if before.value!=self.committed(){self.undo.push(before);self.redo.clear();self.reveal=true;}}
    pub fn undo(&mut self,redo:bool,fonts:&mut FontContext,layouts:&mut LayoutContext<Brush>) {
        self.cancel_composition(fonts,layouts);let current=self.snapshot();
        let Some(previous)=(if redo{&mut self.redo}else{&mut self.undo}).pop() else{return;};
        (if redo{&mut self.undo}else{&mut self.redo}).push(current);
        self.text.set_text(&previous.value);self.text.driver(fonts,layouts).select_byte_range(previous.anchor,previous.focus);self.reveal=true;
    }
    pub fn multiline(&self)->bool {self.multiline}
    pub fn ensure_cursor_visible(&mut self) {
        let Some(caret)=self.text.cursor_geometry(1.) else{return;};
        for (scroll,start,end,extent) in [(&mut self.scroll.0,caret.x0,caret.x1,self.viewport.0),(&mut self.scroll.1,caret.y0,caret.y1,self.viewport.1)] {
            if end-*scroll>extent {*scroll=(end-extent).max(0.);}
            if start<*scroll {*scroll=start.max(0.);}
        }
    }
    pub fn pointer(&mut self,point:(f64,f64),extend:bool,fonts:&mut FontContext,layouts:&mut LayoutContext<Brush>) {
        self.cancel_composition(fonts,layouts);self.reveal=true;
        let (x,y)=((point.0-self.origin.0+self.scroll.0) as f32,(point.1-self.origin.1+self.scroll.1) as f32);
        let mut driver=self.text.driver(fonts,layouts);
        if extend{driver.extend_selection_to_point(x,y);}else{driver.move_to_point(x,y);}
    }
    pub fn navigation(&mut self,key:&winit::keyboard::Key,shift:bool,word:bool,command:bool,fonts:&mut FontContext,layouts:&mut LayoutContext<Brush>) ->bool {
        use winit::keyboard::{Key,NamedKey};
        if self.text.is_composing(){return false;}
        let mut driver=self.text.driver(fonts,layouts);
        match key {
            Key::Named(NamedKey::ArrowLeft)=>if command {if shift{driver.select_to_line_start();}else{driver.move_to_line_start();}}else if word {if shift{driver.select_word_left();}else{driver.move_word_left();}}else if shift{driver.select_left();}else{driver.move_left();},
            Key::Named(NamedKey::ArrowRight)=>if command {if shift{driver.select_to_line_end();}else{driver.move_to_line_end();}}else if word {if shift{driver.select_word_right();}else{driver.move_word_right();}}else if shift{driver.select_right();}else{driver.move_right();},
            Key::Named(NamedKey::ArrowUp)=>if command {if shift{driver.select_to_text_start();}else{driver.move_to_text_start();}}else if shift{driver.select_up();}else{driver.move_up();},
            Key::Named(NamedKey::ArrowDown)=>if command {if shift{driver.select_to_text_end();}else{driver.move_to_text_end();}}else if shift{driver.select_down();}else{driver.move_down();},
            Key::Named(NamedKey::Home)=>if command {if shift{driver.select_to_text_start();}else{driver.move_to_text_start();}}else if shift{driver.select_to_line_start();}else{driver.move_to_line_start();},
            Key::Named(NamedKey::End)=>if command {if shift{driver.select_to_text_end();}else{driver.move_to_text_end();}}else if shift{driver.select_to_line_end();}else{driver.move_to_line_end();},
            _=>return false,
        }self.reveal=true;true
    }
    pub fn committed(&self) -> String {
        self.composition.as_ref().map(|snapshot| snapshot.0.clone()).unwrap_or_else(|| self.text.text().to_string())
    }

    pub fn sync(&mut self, value: &str) {
        let value = normalize(value, self.multiline);
        if self.committed() != value {
            self.composition = None;self.undo.clear();self.redo.clear();self.reveal=true;
            self.text.set_text(&value);
        }
    }

    pub fn insert(&mut self, value: &str, fonts: &mut FontContext, layouts: &mut LayoutContext<Brush>) {
        self.cancel_composition(fonts, layouts);self.reveal=true;
        self.text.driver(fonts, layouts).insert_or_replace_selection(&normalize(value, self.multiline));
    }

    pub fn preedit(&mut self, value: &str, cursor: Option<(usize, usize)>, fonts: &mut FontContext, layouts: &mut LayoutContext<Brush>) -> Result<(), String> {
        self.reveal=true;
        if value.is_empty() {
            self.cancel_composition(fonts, layouts);
            return Ok(());
        }
        if let Some((start, end)) = cursor {
            if !value.is_char_boundary(start) || !value.is_char_boundary(end) {
                return Err("IME cursor must describe valid UTF-8 boundaries within the preedit text".into());
            }
        }
        if self.composition.is_none() {
            let selection = self.text.raw_selection();
            self.composition = Some((self.text.text().to_string(), selection.anchor().index(), selection.focus().index()));
        }
        // Normalize prefixes as well as text so platform byte offsets stay valid.
        let cursor = cursor.map(|(start, end)| (normalize(&value[..start], self.multiline).len(), normalize(&value[..end], self.multiline).len()));
        let value = normalize(value, self.multiline);
        if value.is_empty() { self.cancel_composition(fonts, layouts); }
        else { self.text.driver(fonts, layouts).set_compose(&value, cursor); }
        Ok(())
    }

    pub fn cancel_composition(&mut self, fonts: &mut FontContext, layouts: &mut LayoutContext<Brush>) {
        if let Some((value, anchor, focus)) = self.composition.take() {
            self.text.set_text(&value);
            self.text.driver(fonts, layouts).select_byte_range(anchor, focus);
        }
    }
}

use crate::editable::normalize;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ime_keeps_committed_values_and_restores_replaced_selection_on_cancel() {
        let mut fonts = FontContext::new(); let mut layouts = LayoutContext::new();
        let mut editor = Editor::new("a🙂b", false);
        editor.text.driver(&mut fonts, &mut layouts).select_byte_range(1, 5);
        editor.preedit("日本", Some((3, 6)), &mut fonts, &mut layouts).unwrap();
        assert_eq!(editor.committed(), "a🙂b"); assert_eq!(editor.text.raw_text(), "a日本b");
        editor.cancel_composition(&mut fonts, &mut layouts);
        assert_eq!(editor.text.raw_text(), "a🙂b"); assert_eq!(editor.text.raw_selection().text_range(), 1..5);
        editor.preedit("日本", Some((6, 6)), &mut fonts, &mut layouts).unwrap();
        // Winit sends an empty preedit immediately before the final commit.
        editor.preedit("", None, &mut fonts, &mut layouts).unwrap();
        editor.insert("日本語", &mut fonts, &mut layouts);
        assert_eq!(editor.committed(), "a日本語b");
        assert!(editor.preedit("🙂", Some((1, 4)), &mut fonts, &mut layouts).is_err());
    }
    #[test]
    fn parley_handles_emoji_clusters_and_external_updates_cancel_composition() {
        let mut fonts = FontContext::new(); let mut layouts = LayoutContext::new();
        let mut editor = Editor::new("a👩‍💻", false);
        editor.text.driver(&mut fonts, &mut layouts).move_to_text_end();
        editor.text.driver(&mut fonts, &mut layouts).backdelete();
        assert_eq!(editor.committed(), "a");
        editor.preedit("é", Some((2, 2)), &mut fonts, &mut layouts).unwrap();
        editor.sync("outside"); assert_eq!(editor.committed(), "outside"); assert!(!editor.text.is_composing());
        assert_eq!(normalize("a\r\nb\rc\nd", false), "abcd");
        assert_eq!(normalize("a\r\nb\rc\nd", true), "a\nb\nc\nd");
    }
}

#[cfg(test)]
mod interaction_tests {
 use super::*;
 #[test]
 fn long_unicode_values_scroll_to_selection_and_undo_uses_complete_values() {
  let mut fonts=FontContext::new();let mut layouts=LayoutContext::new();let mut editor=Editor::new(&"é🙂".repeat(40),false);editor.configure(TextStyle::default(),120.);editor.viewport=(120.,32.);editor.text.driver(&mut fonts,&mut layouts).move_to_text_end();editor.text.refresh_layout(&mut fonts,&mut layouts);editor.ensure_cursor_visible();assert!(editor.scroll.0>120.);assert_eq!(editor.scroll.1,0.);
  let before=editor.snapshot();editor.text.driver(&mut fonts,&mut layouts).select_all();editor.insert("changed",&mut fonts,&mut layouts);editor.record(before);editor.undo(false,&mut fonts,&mut layouts);assert_eq!(editor.committed(),"é🙂".repeat(40));editor.undo(true,&mut fonts,&mut layouts);assert_eq!(editor.committed(),"changed");
  editor.sync("external");editor.undo(false,&mut fonts,&mut layouts);assert_eq!(editor.committed(),"external");
 }
}
