//! Checked creation-time window options, shared with browser diagnostics.
#[derive(Debug, Default, PartialEq, serde::Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Titlebar { #[default] Native, Custom }

#[derive(Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all="camelCase")]
pub(crate) struct Options {
    pub width:f64, pub height:f64, pub min_width:f64, pub min_height:f64,
    pub titlebar:Titlebar,
    pub opens_folders:bool,
    /// Font files (.ttf/.otf) loaded before the window opens.
    pub fonts:Vec<String>,
    /// Default UI font family; empty keeps the platform font.
    pub font:String,
}
impl Default for Options {
    fn default()->Self {Self{width:640.,height:480.,min_width:240.,min_height:160.,titlebar:Titlebar::Native,opens_folders:false,fonts:Vec::new(),font:String::new()}}
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

/// Browser-like zoom levels for application text zoom.
pub(crate) const ZOOM_STEPS: [f32; 13] = [0.5, 0.67, 0.75, 0.8, 0.9, 1., 1.1, 1.25, 1.5, 1.75, 2., 2.5, 3.];

/// The zoom after a `text-zoom` command: an exact `level`, or a `step` of 1 (larger),
/// -1 (smaller) or 0 (reset). Results clamp to 0.5–3; non-finite levels keep `current`.
#[cfg_attr(not(any(test, feature = "gpui")), expect(dead_code, reason = "Only the native renderer applies zoom"))]
pub(crate) fn next_zoom(current: f32, step: Option<i64>, level: Option<f64>) -> f32 {
    let next = match (level, step.unwrap_or(0)) {
        (Some(level), _) => level as f32,
        (None, 0) => 1.,
        (None, step) if step > 0 => ZOOM_STEPS.iter().copied().find(|level| *level > current + 0.001).unwrap_or(3.),
        (None, _) => ZOOM_STEPS.iter().rev().copied().find(|level| *level < current - 0.001).unwrap_or(0.5),
    };
    if next.is_finite() {next.clamp(0.5, 3.)} else {current}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn application_window_dimensions_and_unknown_options_are_checked_before_opening() {
        let options=Options::parse(r#"{"width":1040,"height":760,"minWidth":480,"titlebar":"custom"}"#).unwrap();
        assert_eq!(options.titlebar,Titlebar::Custom);assert_eq!(options.min_height,160.);
        for json in [r#"{"width":0}"#,r#"{"height":1e20}"#,r#"{"width":300,"minWidth":400}"#,r#"{"titlebar":"frameless"}"#,r#"{"transparent":true}"#,r#"{"height":null}"#] {assert!(Options::parse(json).is_err(),"{json}");}
    }
    #[test]
    fn zoom_steps_follow_browser_levels_and_clamp() {
        use super::next_zoom;
        assert_eq!(next_zoom(1., Some(1), None), 1.1);
        assert_eq!(next_zoom(1.1, Some(1), None), 1.25);
        assert_eq!(next_zoom(1.25, Some(-1), None), 1.1);
        assert_eq!(next_zoom(1.17, Some(-1), None), 1.1, "off-step levels move to the neighbouring step");
        assert_eq!(next_zoom(2.5, Some(0), None), 1.);
        assert_eq!(next_zoom(1., None, Some(9.)), 3.);
        assert_eq!(next_zoom(3., Some(1), None), 3.);
        assert_eq!(next_zoom(1., None, Some(0.1)), 0.5);
        assert_eq!(next_zoom(0.5, Some(-1), None), 0.5);
        assert_eq!(next_zoom(1.25, None, Some(f64::NAN)), 1.25);
    }
}
