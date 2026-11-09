//! Application translations: per-locale message catalogs, the current locale,
//! interpolation and simple plural selection. Translated text is a live
//! TextBinding, so changing the locale updates mounted text on the next refresh.
#![cfg_attr(not(feature = "gpui"), allow(dead_code))]
use super::TextBinding;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

thread_local! {
    static CATALOGS: RefCell<HashMap<String, HashMap<String, String>>> = RefCell::new(HashMap::new());
    static LOCALE: RefCell<Option<String>> = const {RefCell::new(None)};
    static CHANGED: Cell<bool> = const {Cell::new(false)};
}

fn flatten(prefix: &str, value: &serde_json::Value, out: &mut HashMap<String, String>) -> Result<(), String> {
    match value {
        serde_json::Value::Object(entries) => {
            for (key, value) in entries {flatten(&if prefix.is_empty() {key.clone()} else {format!("{prefix}.{key}")}, value, out)?;}
            Ok(())
        }
        serde_json::Value::String(text) => {out.insert(prefix.to_owned(), text.clone()); Ok(())}
        _ => Err(format!("Translation {prefix} must be a string or a nested object")),
    }
}

/// Register (or replace) a locale's messages from a JSON object; nested keys join with dots.
pub fn set_translations(locale: &str, json: &str) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|error| format!("Invalid translations for {locale}: {error}"))?;
    if !value.is_object() {return Err(format!("Translations for {locale} must be a JSON object"));}
    let mut messages = HashMap::new();
    flatten("", &value, &mut messages)?;
    CATALOGS.with(|catalogs| catalogs.borrow_mut().insert(normalize(locale), messages));
    CHANGED.with(|changed| changed.set(true));
    Ok(())
}

fn normalize(tag: &str) -> String {tag.trim().replace('_', "-")}

/// The system locale (such as en-US), or en when it cannot be read.
fn system() -> String {sys_locale::get_locale().map(|tag| normalize(&tag)).unwrap_or_else(|| "en".to_owned())}

pub fn locale() -> String {LOCALE.with(|value| value.borrow_mut().get_or_insert_with(system).clone())}

pub fn set_locale(tag: &str) {
    LOCALE.with(|value| *value.borrow_mut() = Some(normalize(tag)));
    CHANGED.with(|changed| changed.set(true));
}

/// Whether the locale or catalogs changed since the renderer last asked.
pub(crate) fn take_changed() -> bool {CHANGED.with(|changed| changed.replace(false))}

/// Look up a message: the exact locale, its language, then en; the key itself if missing.
/// A numeric `count` selects key_zero, key_one or key_other before the plain key.
pub fn translate(key: &str, values: &str) -> String {
    let values: serde_json::Map<String, serde_json::Value> = serde_json::from_str(values).unwrap_or_default();
    let count = values.get("count").and_then(serde_json::Value::as_f64);
    let suffixes: &[&str] = match count {Some(count) if count == 0. => &["_zero", "_other", ""], Some(count) if count == 1. => &["_one", ""], Some(_) => &["_other", ""], None => &[""]};
    let current = locale();
    let language = current.split('-').next().unwrap_or_default().to_owned();
    let template = CATALOGS.with(|catalogs| {
        let catalogs = catalogs.borrow();
        [current.as_str(), language.as_str(), "en"].iter().filter_map(|tag| catalogs.get(*tag))
            .find_map(|messages| suffixes.iter().find_map(|suffix| messages.get(&format!("{key}{suffix}")).cloned()))
    }).unwrap_or_else(|| key.to_owned());
    let mut text = template;
    for (name, value) in &values {
        let value = match value {
            serde_json::Value::String(text) => text.clone(),
            serde_json::Value::Number(number) => number.as_i64().map(|value| value.to_string()).unwrap_or_else(|| number.to_string()),
            other => other.to_string(),
        };
        text = text.replace(&format!("{{{name}}}"), &value);
    }
    text
}

/// Live translated text: re-read on every refresh, so locale changes apply.
pub fn translation(key: String, values: String) -> TextBinding {TextBinding::computed(move || Ok(translate(&key, &values)))}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lookup_falls_back_by_language_then_english_and_selects_plurals() {
        set_translations("en", r#"{"greeting":"Hello, {name}!","items_one":"{count} item","items_other":"{count} items","menu":{"save":"Save"},"only":"English"}"#).unwrap();
        set_translations("fr", r#"{"greeting":"Bonjour, {name} !","menu":{"save":"Enregistrer"}}"#).unwrap();
        set_locale("fr_CA");
        assert_eq!(locale(), "fr-CA");
        assert_eq!(translate("greeting", r#"{"name":"Ada"}"#), "Bonjour, Ada !");
        assert_eq!(translate("menu.save", "{}"), "Enregistrer");
        assert_eq!(translate("only", "{}"), "English");
        assert_eq!(translate("missing.key", "{}"), "missing.key");
        set_locale("en");
        assert_eq!(translate("items", r#"{"count":1}"#), "1 item");
        assert_eq!(translate("items", r#"{"count":3}"#), "3 items");
        let live = translation("menu.save".into(), "{}".into());
        assert_eq!(live.value().unwrap(), "Save");
        set_locale("fr");
        assert_eq!(live.value().unwrap(), "Enregistrer");
        assert!(take_changed());
        assert!(!take_changed());
        assert!(set_translations("de", "[1]").is_err());
    }
}
