//! Checked UI factories share one owned renderer representation across targets.
pub(crate) mod emit;
mod components;
pub(crate) use components::retain_state;
pub(crate) const SOURCE: &str = concat!(include_str!("api.tsr"), include_str!("kit.tsr"));
use crate::{
    Mode,
    native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType},
    native_signatures::{NativeTypeId, Signature},
};
const SIGNAL_INT:Signature=Signature::Applied(NativeTypeId::UiSignal,&[Signature::Int]);
const SIGNAL_NUMBER:Signature=Signature::Applied(NativeTypeId::UiSignal,&[Signature::Number]);
const SIGNAL_BOOLEAN:Signature=Signature::Applied(NativeTypeId::UiSignal,&[Signature::Boolean]);
const SIGNAL_STRING:Signature=Signature::Applied(NativeTypeId::UiSignal,&[Signature::String]);
pub(crate) const CHILD_VALUE:Signature=Signature::Optional(&Signature::Union(&[Signature::String,Signature::Int,Signature::Number,Signature::Named(NativeTypeId::UiTextBinding),SIGNAL_INT,SIGNAL_NUMBER,SIGNAL_BOOLEAN,SIGNAL_STRING,Signature::Named(NativeTypeId::UiNode),Signature::Null]));
pub(crate) const CHILDREN:Signature=Signature::Array(&CHILD_VALUE);
pub(crate) const INPUT_DEFAULT:Signature=Signature::Optional(&Signature::Union(&[Signature::String,Signature::Int,Signature::Number]));
const NODE: Signature = Signature::Named(NativeTypeId::UiNode);
const ERROR: Signature = Signature::Named(NativeTypeId::UiError);
const fn parameter(name: &'static str, ty: NativeType, access: Mode) -> NativeParameter {
    NativeParameter {
        name,
        ty,
        access,
        optional: false,
        rest: false,
    }
}
pub(crate) const TEXT: NativeBinding = NativeBinding {
    module: "std:ui-runtime",
    name: "text",
    parameters: &[
        parameter("children", NativeType::Structured(&CHILDREN), Mode::Consume),
        parameter("className", NativeType::String, Mode::Read),
    ],
    result: NativeType::Structured(&Signature::Result(&NODE, &ERROR)),
    returned_borrow: None,
    backend: NativeBackend::Bundled {
        symbol: "tsrust_ui_text",
    },
    features: &["ui"],
};
pub(crate) const VISIBILITY:NativeBinding=NativeBinding{name:"visibility",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_visibility"},..TEXT};
pub(crate) const COPY_BUTTON:NativeBinding=NativeBinding{name:"copyButton",parameters:&[parameter("value",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_copy_button"},..TEXT};
pub(crate) const VIEW: NativeBinding = NativeBinding {
    name: "view",
    parameters:&[parameter("children",NativeType::Structured(&CHILDREN),Mode::Consume),parameter("className",NativeType::String,Mode::Read)],
    backend:NativeBackend::Bundled{symbol:"tsrust_ui_view"},..TEXT
};
pub(crate) const BUTTON: NativeBinding = NativeBinding {
    name:"staticButton",
    parameters:&[parameter("children",NativeType::Structured(&CHILDREN),Mode::Consume),parameter("className",NativeType::String,Mode::Read),parameter("disabled",NativeType::Boolean,Mode::Read)],
    backend:NativeBackend::Bundled{symbol:"tsrust_ui_static_button"},..TEXT
};
pub(crate) const DOCUMENT: NativeBinding = NativeBinding {
    name: "document",
    parameters: &[
        parameter("node", NativeType::Structured(&NODE), Mode::Read),
        parameter("title", NativeType::String, Mode::Read),
    ],
    result: NativeType::String,
    backend: NativeBackend::Bundled {
        symbol: "tsrust_ui_document",
    },
    ..TEXT
};
pub(crate) const THEME:NativeBinding=NativeBinding{name:"theme",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("config",NativeType::String,Mode::Read),parameter("mode",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_theme"},..TEXT};
pub(crate) const THEME_MODE:NativeBinding=NativeBinding{name:"themeMode",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("config",NativeType::String,Mode::Read),parameter("mode",NativeType::Structured(&SIGNAL_STRING),Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_theme_mode"},..TEXT};
pub(crate) const SEMANTICS:NativeBinding=NativeBinding{name:"semantics",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("config",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_semantics"},..TEXT};

pub(crate) const DISABLED_BINDING:NativeBinding=NativeBinding{name:"bindDisabled",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("predicate",NativeType::OwnedBooleanCallback,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_bind_disabled"},..TEXT};

pub(crate) const CLASS_BINDING:NativeBinding=NativeBinding{name:"bindClass",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("evaluate",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_bind_class"},..TEXT};
pub(crate) const SEMANTIC_BINDING:NativeBinding=NativeBinding{name:"bindSemantics",backend:NativeBackend::Bundled{symbol:"tsrust_ui_bind_semantics"},..CLASS_BINDING};
pub(crate) const PATH_BINDING:NativeBinding=NativeBinding{name:"bindPath",backend:NativeBackend::Bundled{symbol:"tsrust_ui_bind_path"},..CLASS_BINDING};
pub(crate) const SVG:NativeBinding=NativeBinding{name:"svg",parameters:&[parameter("children",NativeType::Structured(&CHILDREN),Mode::Consume),parameter("className",NativeType::String,Mode::Read),parameter("options",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_svg"},..TEXT};
pub(crate) const PATH:NativeBinding=NativeBinding{name:"path",parameters:&[parameter("data",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read),parameter("options",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_path"},..TEXT};
pub(crate) const VALIDATE_ROOT:NativeBinding=NativeBinding{name:"validateRoot",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Read)],result:NativeType::Structured(&Signature::Result(&Signature::Void,&ERROR)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_validate_root"},..TEXT};
pub(crate) const VARIANT_BINDING:NativeBinding=NativeBinding{name:"bindVariant",backend:NativeBackend::Bundled{symbol:"tsrust_ui_bind_variant"},..CLASS_BINDING};
pub(crate) const HIDDEN_BINDING:NativeBinding=NativeBinding{name:"bindHidden",backend:NativeBackend::Bundled{symbol:"tsrust_ui_bind_hidden"},..DISABLED_BINDING};

pub(crate) const READ_ONLY_BINDING:NativeBinding=NativeBinding{name:"bindReadOnly",backend:NativeBackend::Bundled{symbol:"tsrust_ui_bind_read_only"},..DISABLED_BINDING};

pub(crate) const RUN: NativeBinding = NativeBinding {
    name: "run",
    parameters: &[
        parameter("node", NativeType::Structured(&NODE), Mode::Consume),
        parameter("title", NativeType::String, Mode::Read),
        parameter("options", NativeType::String, Mode::Read),
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &ERROR)),
    backend: NativeBackend::Bundled {
        symbol: "tsrust_ui_run",
    },
    features: &["ui-native"],
    ..TEXT
};

pub(crate) const USE_ID:NativeBinding=NativeBinding{name:"useId",parameters:&[],result:NativeType::Structured(&Signature::Result(&Signature::String,&ERROR)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_use_id"},..TEXT};
pub(crate) const LOCATION:NativeBinding=NativeBinding{name:"location",parameters:&[],result:NativeType::Structured(&Signature::Result(&SIGNAL_STRING,&ERROR)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_location"},..TEXT};
pub(crate) const NAVIGATE:NativeBinding=NativeBinding{name:"navigate",parameters:&[parameter("path",NativeType::String,Mode::Read)],result:NativeType::Structured(&Signature::Result(&Signature::Void,&ERROR)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_navigate"},..TEXT};
pub(crate) const TRAVERSE:NativeBinding=NativeBinding{name:"traverse",parameters:&[parameter("forward",NativeType::Boolean,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_traverse"},..NAVIGATE};
pub(crate) const LINK:NativeBinding=NativeBinding{name:"link",parameters:&[parameter("children",NativeType::Structured(&CHILDREN),Mode::Consume),parameter("className",NativeType::String,Mode::Read),parameter("href",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_link"},..TEXT};
pub(crate) const SIGNAL_NEW:NativeBinding=NativeBinding{name:"signal",parameters:&[parameter("initial",NativeType::ObservedValue,Mode::Read)],result:NativeType::ObservedValue,backend:NativeBackend::Bundled{symbol:"tsrust_ui_signal"},..TEXT};
pub(crate) const COMPUTED_NEW:NativeBinding=NativeBinding{name:"computed",parameters:&[parameter("source",NativeType::ObservedValue,Mode::Consume)],result:NativeType::ObservedValue,backend:NativeBackend::Bundled{symbol:"tsrust_ui_computed"},..TEXT};
pub(crate) const COMPUTED_GET:NativeBinding=NativeBinding{name:"computedGet",parameters:&[parameter("value",NativeType::ObservedValue,Mode::Read)],result:NativeType::ObservedValue,backend:NativeBackend::Bundled{symbol:"tsrust_ui_computed_get"},..TEXT};
pub(crate) const HANDLE_NEW:NativeBinding=NativeBinding{name:"handle",parameters:&[parameter("signal",NativeType::ObservedValue,Mode::Read)],result:NativeType::ObservedValue,backend:NativeBackend::Bundled{symbol:"tsrust_ui_handle"},..TEXT};
pub(crate) const GET:NativeBinding=NativeBinding{name:"get",parameters:&[parameter("state",NativeType::ObservedValue,Mode::Read)],result:NativeType::ObservedValue,backend:NativeBackend::Bundled{symbol:"tsrust_ui_get"},..TEXT};
pub(crate) const SET:NativeBinding=NativeBinding{name:"set",parameters:&[parameter("state",NativeType::ObservedValue,Mode::Read),parameter("value",NativeType::ObservedValue,Mode::Read)],result:NativeType::ObservedValue,backend:NativeBackend::Bundled{symbol:"tsrust_ui_set"},..TEXT};

pub(crate) const OWN_STATE:NativeBinding=NativeBinding{name:"ownState",result:NativeType::ObservedValue,parameters:&[parameter("node",NativeType::ObservedValue,Mode::Consume),parameter("state",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_own_state"},..TEXT};
pub(crate) const PRESS_BUTTON:NativeBinding=NativeBinding{name:"button",parameters:&[parameter("children",NativeType::Structured(&CHILDREN),Mode::Consume),parameter("className",NativeType::String,Mode::Read),parameter("disabled",NativeType::Boolean,Mode::Read),parameter("onPress",NativeType::OwnedVoidCallback,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_press_button"},..TEXT};

pub(crate) const TEXT_BINDING:NativeBinding=NativeBinding{name:"bindText",parameters:&[parameter("evaluate",NativeType::ObservedValue,Mode::Consume)],result:NativeType::Structured(&Signature::Named(NativeTypeId::UiTextBinding)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_text_binding"},..TEXT};
pub(crate) const ROUTE:NativeBinding=NativeBinding{name:"route",parameters:&[parameter("path",NativeType::String,Mode::Read),parameter("exact",NativeType::Boolean,Mode::Read),parameter("render",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_route"},..TEXT};
pub(crate) const EACH:NativeBinding=NativeBinding{name:"each",parameters:&[parameter("build",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_each"},..TEXT};
pub(crate) const BRANCH:NativeBinding=NativeBinding{name:"branch",parameters:&[parameter("predicate",NativeType::OwnedBooleanCallback,Mode::Consume),parameter("present",NativeType::ObservedValue,Mode::Consume),parameter("absent",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_branch"},..TEXT};

pub(crate) const INPUT:NativeBinding=NativeBinding{name:"input",parameters:&[parameter("value",NativeType::ObservedValue,Mode::Consume),parameter("defaultValue",NativeType::Structured(&INPUT_DEFAULT),Mode::Consume),parameter("options",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_input"},..TEXT};
pub(crate) const INPUT_CHANGE:NativeBinding=NativeBinding{name:"inputChange",parameters:&[parameter("value",NativeType::ObservedValue,Mode::Consume),parameter("defaultValue",NativeType::Structured(&INPUT_DEFAULT),Mode::Consume),parameter("options",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_input_change"},..TEXT};

// GPUI Kit components: one generic node with string props, live props and
// events; generated std:ui wrappers (kit.tsr) give each component typed props.
pub(crate) const KIT:NativeBinding=NativeBinding{name:"kit",parameters:&[parameter("children",NativeType::Structured(&CHILDREN),Mode::Consume),parameter("className",NativeType::String,Mode::Read),parameter("name",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit"},..TEXT};
pub(crate) const KIT_SET:NativeBinding=NativeBinding{name:"kitSet",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("name",NativeType::String,Mode::Read),parameter("value",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_set"},..TEXT};
pub(crate) const KIT_BIND:NativeBinding=NativeBinding{name:"kitBind",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("name",NativeType::String,Mode::Read),parameter("binding",NativeType::Structured(&Signature::Named(NativeTypeId::UiTextBinding)),Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_bind"},..TEXT};
pub(crate) const KIT_ON:NativeBinding=NativeBinding{name:"kitOn",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("event",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_on"},..TEXT};
pub(crate) const KIT_ON_BOOLEAN:NativeBinding=NativeBinding{name:"kitOnBoolean",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("event",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_on_boolean"},..TEXT};
pub(crate) const KIT_ON_INT:NativeBinding=NativeBinding{name:"kitOnInt",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("event",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_on_int"},..TEXT};
pub(crate) const KIT_ON_NUMBER:NativeBinding=NativeBinding{name:"kitOnNumber",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("event",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_on_number"},..TEXT};
pub(crate) const KIT_ON_VOID:NativeBinding=NativeBinding{name:"kitOnVoid",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("event",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_on_void"},..TEXT};

// Application services: queued commands (notifications, windows, clipboard,
// dialogs, URLs, updates), commands with a later text reply, extra windows,
// appearance and the packaged app manifest.
pub(crate) const COMMAND:NativeBinding=NativeBinding{name:"command",parameters:&[parameter("name",NativeType::String,Mode::Read),parameter("payload",NativeType::String,Mode::Read)],result:NativeType::Structured(&Signature::Result(&Signature::Void,&ERROR)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_command"},..TEXT};
pub(crate) const COMMAND_REPLY:NativeBinding=NativeBinding{name:"commandReply",parameters:&[parameter("name",NativeType::String,Mode::Read),parameter("payload",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_command_reply"},..COMMAND};
pub(crate) const OPEN_WINDOW:NativeBinding=NativeBinding{name:"openWindow",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("title",NativeType::String,Mode::Read),parameter("options",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_open_window"},..COMMAND};
pub(crate) const APPEARANCE:NativeBinding=NativeBinding{name:"appearance",parameters:&[],result:NativeType::String,backend:NativeBackend::Bundled{symbol:"tsrust_ui_appearance"},..TEXT};
pub(crate) const APP_INFO:NativeBinding=NativeBinding{name:"appInfo",parameters:&[],result:NativeType::String,backend:NativeBackend::Bundled{symbol:"tsrust_ui_app_info"},..TEXT};

// Translations: catalogs, the current locale, live and snapshot lookups, and one
// value-typed Kit prop setter that accepts text, numbers and live bindings.
pub(crate) const SET_TRANSLATIONS:NativeBinding=NativeBinding{name:"setTranslations",parameters:&[parameter("locale",NativeType::String,Mode::Read),parameter("messages",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_set_translations"},..COMMAND};
pub(crate) const SET_LOCALE:NativeBinding=NativeBinding{name:"setLocale",parameters:&[parameter("locale",NativeType::String,Mode::Read)],result:NativeType::Void,backend:NativeBackend::Bundled{symbol:"tsrust_ui_set_locale"},..TEXT};
pub(crate) const LOCALE:NativeBinding=NativeBinding{name:"locale",parameters:&[],result:NativeType::String,backend:NativeBackend::Bundled{symbol:"tsrust_ui_locale"},..TEXT};
pub(crate) const TRANSLATE:NativeBinding=NativeBinding{name:"translate",parameters:&[parameter("key",NativeType::String,Mode::Read),parameter("values",NativeType::String,Mode::Read)],result:NativeType::String,backend:NativeBackend::Bundled{symbol:"tsrust_ui_translate"},..TEXT};
pub(crate) const TRANSLATION:NativeBinding=NativeBinding{name:"translation",parameters:&[parameter("key",NativeType::String,Mode::Read),parameter("values",NativeType::String,Mode::Read)],result:NativeType::Structured(&Signature::Named(NativeTypeId::UiTextBinding)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_translation"},..TEXT};
pub(crate) const KIT_VALUE:NativeBinding=NativeBinding{name:"kitValue",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("name",NativeType::String,Mode::Read),parameter("value",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_kit_value"},..TEXT};

/// Selects GPUI Kit code highlighting for programs that use CodeEditor or Markdown.
pub(crate) const HIGHLIGHT_MARKER:&str="// tsrust-ui-highlight-selected\n";
/// Selects GPUI Kit speech recognition for programs that use SpeechButton.
pub(crate) const SPEECH_MARKER:&str="// tsrust-ui-speech-selected\n";
/// Selects an HTTP client for remote image sources in programs that use Avatar or Image.
pub(crate) const HTTP_MARKER:&str="// tsrust-ui-http-selected\n";

/// Generated programs embed the Node model from `crates/tsrust-ui` with the GPUI
/// renderer as its `native` module; GPUI Kit comes from the separate locked
/// GPUI dependency set, never from the backend graph.
pub(crate) fn runtime(with_native: bool) -> String {
    macro_rules! source {($name:literal) => {include_str!(concat!("../../../../tsrust-ui/src/", $name))};}
    let style = source!("style.rs")
        .replace("mod layout;",&format!("mod layout {{\n{}\n}}",source!("style/layout.rs")))
        .replace("mod structural;",&format!("mod structural {{\n{}\n}}",source!("style/structural.rs")))
        .replace("mod custom_spacing;",&format!("mod custom_spacing {{\n{}\n}}",source!("style/custom_spacing.rs")))
        .replace("mod text;",&format!("mod text {{\n{}\n}}",source!("style/text.rs")))
        .replace("mod tailwind;",&format!("mod tailwind {{\n{}\n}}",source!("style/tailwind.rs")))
        .replace("mod edges;",&format!("mod edges {{\n{}\n}}",source!("style/edges.rs")))
        .replace("crate::theme", "super::theme");
    let theme=source!("theme.rs")
        .replace("mod paint_color;",&format!("mod paint_color {{\n{}\n}}",source!("theme/paint_color.rs")))
        .replace("include_str!(\"theme.json\")", &format!("{:?}",source!("theme.json")));
    let web = source!("web.rs").replace("use crate::{", "use super::{").replace("crate::style::", "super::style::").replace("crate::theme::", "super::theme::");
    let module = |name: &str, text: &str| format!("mod {name} {{\n{}\n}}", text.replace("use crate::{", "use super::{"));
    let mut body=source!("lib.rs")
        .replace("#[cfg(feature = \"runtime\")]\n", "")
        .replace("#[cfg(test)]\nmod application_tests;", "").replace("#[cfg(test)]\nmod empty_style_tests;", "")
        .replace("#[cfg(test)]\nmod table_semantics_tests;", "").replace("#[cfg(test)]\nmod reactive_semantics_tests;", "")
        .replace("mod visibility;",&module("visibility",source!("visibility.rs")))
        .replace("mod kit;",&module("kit",source!("kit.rs")))
        .replace("mod commands;",&module("commands",source!("commands.rs")))
        .replace("mod i18n;",&module("i18n",source!("i18n.rs")))
        .replace("mod update;",&module("update",source!("update.rs")))
        .replace("mod editable;",&module("editable",source!("editable.rs")))
        .replace("pub mod clipboard;", &format!("pub {}",module("clipboard",&source!("clipboard.rs").replace("crate::Node", "super::Node"))))
        .replace("pub mod navigation;", &format!("pub {}",module("navigation",source!("navigation.rs"))))
        .replace("mod vector;",&module("vector",&source!("vector.rs").replace("crate::theme::", "super::theme::")))
        .replace("mod table;",&module("table",source!("table.rs")))
        .replace("mod identity;",&module("identity",source!("identity.rs")))
        .replace("mod computed;",&module("computed",source!("computed.rs")))
        .replace("mod state;", &module("state",source!("state.rs")))
        .replace("mod window;",&module("window",source!("window.rs")))
        .replace("mod branch;", &module("branch",source!("branch.rs")))
        .replace("pub mod style;",&format!("pub mod style {{\n{style}\n}}"))
        .replace("pub mod theme;",&format!("pub mod theme {{\n{theme}\n}}"))
        .replace("mod web;",&format!("mod web {{\n{web}\n}}"))
        .replace("feature = \"runtime\"", "feature = \"ui\"").replace("feature=\"runtime\"", "feature = \"ui\"");
    if with_native {
        body.push_str(&format!("#[cfg(feature=\"gpui\")] mod native {{\n{}\n{}\n}}\n#[cfg(feature=\"gpui\")] pub use native::{{run,run_configured}};\n",include_str!("gpui.rs"), [include_str!("gpui_kit.rs"), include_str!("gpui_icons.rs")].concat()));
    }
    format!("mod tsrust_native_ui {{\n{body}\n}}\n")
}

/// Unused trusted UI facades must not pull a native event loop into an HTML build.
pub(crate) fn omit_unused(
    program: &mut crate::syntax::Program,
    names: &std::collections::BTreeMap<String, String>,
) -> Result<(), crate::Diagnostic> {
    use crate::syntax::ExprKind;
    use std::collections::{BTreeMap, BTreeSet};
    let facades = names.values().cloned().collect::<BTreeSet<_>>();
    let mut edges = BTreeMap::new();
    let mut pending = Vec::new();
    for function in &mut program.functions {
        let mut references = Vec::new();
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Name(name) = &expression.kind {
                references.push(name.clone());
            }
            Ok(())
        })?;
        if facades.contains(&function.name.text) {
            edges.insert(function.name.text.clone(), references);
        } else {
            pending.extend(references);
        }
    }
    crate::source_walk::body(
        &mut program.module_bindings,
        &mut |_| Ok(()),
        &mut |expression| {
            if let ExprKind::Name(name) = &expression.kind {
                pending.push(name.clone());
            }
            Ok(())
        },
    )?;
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if seen.insert(name.clone())
            && let Some(references) = edges.get(&name)
        {
            pending.extend(references.iter().cloned());
        }
    }
    program.functions.retain(|function| {
        !facades.contains(&function.name.text) || seen.contains(&function.name.text)
    });
    Ok(())
}

pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic> {
    if program.records.iter().any(|record|record.native_type==Some(NativeTypeId::UiNode)) {
        crate::native_binding_types::prepare_signature(&mut program.records,&CHILD_VALUE,&[],crate::Span::default())?;
    }
    if program.records.iter().any(|record|record.native_type==Some(NativeTypeId::UiSignal)) {
        for binding in [&GET,&SET] {crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}
    }
    Ok(())
}
