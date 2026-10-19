//! Shared control values and callbacks; platform editors never own application state.
pub(crate) type ValueBinding = Box<dyn FnMut() -> Result<String, String>>;
pub(crate) type ChangeHandler = Box<dyn FnMut(&str,&str,&str) -> Result<(), String>>;

#[derive(Default,serde::Deserialize)]
#[serde(default,deny_unknown_fields,rename_all="camelCase")]
pub(crate) struct Options {
    pub placeholder:String,
    pub disabled:bool,
    pub read_only:bool,
    pub multiline:bool,
    pub required:bool,
    pub name:String,
    pub auto_complete:String,
    #[serde(rename="type")]
    pub input_type:String,
}

pub(crate) struct Editable {
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub read_only: bool,
    pub multiline: bool,
    pub required: bool,
    pub name:String,
    pub auto_complete:String,
    pub binding: Option<ValueBinding>,
    #[cfg_attr(not(any(test,feature="native",all(target_arch="wasm32",feature="ui-web"))),allow(dead_code))]
    pub changed: Option<ChangeHandler>,
}

impl Editable {
    pub fn refresh(&mut self) -> Result<bool, String> {
        let Some(binding) = &mut self.binding else { return Ok(false); };
        let value = normalize(&binding()?, self.multiline);
        if value == self.value { return Ok(false); }
        self.value = value;
        Ok(true)
    }

    #[cfg_attr(not(any(test,feature="native",all(target_arch="wasm32",feature="ui-web"))),allow(dead_code))]
    pub fn input(&mut self, value: &str,id:&str) -> Result<bool, String> {
        if self.disabled || self.read_only { return Ok(false); }
        let value = normalize(value, self.multiline);
        if value == self.value { return Ok(false); }
        self.value = value;
        if let Some(handler) = &mut self.changed { handler(&self.value,&self.name,id)?; }
        self.refresh()?;
        Ok(true)
    }
}

// Match HTML value normalization without changing strings stored in application state.
pub(crate) fn normalize(value: &str, multiline: bool) -> String {
    if multiline { value.replace("\r\n", "\n").replace('\r', "\n") }
    else { value.chars().filter(|ch| !matches!(ch, '\r' | '\n')).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    fn field() -> Editable {
        Editable { value: "first".into(), placeholder: "Name".into(), disabled: false, read_only: false, multiline: false, required:false, name:String::new(), auto_complete:String::new(), binding: None, changed: None }
    }
    #[test]
    fn uncontrolled_values_keep_edits_and_inactive_fields_do_not_dispatch() {
        let count = Rc::new(RefCell::new(0)); let observed = count.clone();
        let mut field = field(); field.changed = Some(Box::new(move |_,_,_| { *count.borrow_mut() += 1; Ok(()) }));
        assert!(field.input("é🙂\r\n","").unwrap()); assert_eq!(field.value, "é🙂"); assert_eq!(*observed.borrow(), 1);
        assert!(!field.input("é🙂","").unwrap());
        field.read_only = true; assert!(!field.input("ignored","").unwrap());
        field.read_only = false; field.disabled = true; assert!(!field.input("ignored","").unwrap());
        assert_eq!(field.value, "é🙂"); assert_eq!(*observed.borrow(), 1);
    }
    #[test]
    fn controlled_values_follow_application_updates_or_restore_the_current_value() {
        let state = Rc::new(RefCell::new(String::from("first"))); let read = Rc::downgrade(&state); let write = read.clone();
        let mut field = field(); field.binding = Some(Box::new(move || read.upgrade().map(|value|value.borrow().clone()).ok_or_else(||"disposed".into())));
        assert!(field.input("ignored","").unwrap()); assert_eq!(field.value, "first");
        field.changed = Some(Box::new(move |value,_,_| { *write.upgrade().ok_or("disposed")?.borrow_mut() = value.to_owned(); Ok(()) }));
        assert!(field.input("second","").unwrap()); assert_eq!(field.value, "second"); assert_eq!(*state.borrow(), "second");
        drop(state); assert!(field.refresh().is_err());
    }
}
