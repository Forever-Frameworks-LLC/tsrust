//! Checked creation-time window options, shared with browser diagnostics.
#[derive(Debug, Default, PartialEq, serde::Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Titlebar { #[default] Native, Custom }

#[derive(Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all="camelCase")]
pub(crate) struct Options {
    pub width:f64, pub height:f64, pub min_width:f64, pub min_height:f64,
    pub titlebar:Titlebar,
}
impl Default for Options {
    fn default()->Self {Self{width:640.,height:480.,min_width:240.,min_height:160.,titlebar:Titlebar::Native}}
}
impl Options {
    pub fn parse(json:&str)->Result<Self,String> {
        let options:Self=serde_json::from_str(json).map_err(|error|format!("Invalid UI window options: {error}"))?;
        for (name,value,minimum) in [("width",options.width,240.),("height",options.height,160.),("minWidth",options.min_width,240.),("minHeight",options.min_height,160.)] {
            if !value.is_finite()||value<minimum||value>16384. {return Err(format!("Window {name} must be finite and between {minimum} and 16384 logical pixels"));}
        }
        if options.min_width>options.width||options.min_height>options.height{return Err("Window minimum dimensions cannot exceed initial dimensions".into());}
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn application_window_dimensions_and_unknown_options_are_checked_before_opening() {
        let options=Options::parse(r#"{"width":1040,"height":760,"minWidth":480,"titlebar":"custom"}"#).unwrap();
        assert_eq!(options.titlebar,Titlebar::Custom);assert_eq!(options.min_height,160.);
        for json in [r#"{"width":0}"#,r#"{"height":1e20}"#,r#"{"width":300,"minWidth":400}"#,r#"{"titlebar":"frameless"}"#,r#"{"transparent":true}"#,r#"{"height":null}"#] {assert!(Options::parse(json).is_err(),"{json}");}
    }
}
