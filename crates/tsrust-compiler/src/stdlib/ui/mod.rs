//! Checked UI factories share one owned renderer representation across targets.
pub(crate) mod emit;
mod components;
pub(crate) use components::retain_state;
pub(crate) const SOURCE: &str = include_str!("api.tsr");
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
pub(crate) const BRANCH:NativeBinding=NativeBinding{name:"branch",parameters:&[parameter("predicate",NativeType::OwnedBooleanCallback,Mode::Consume),parameter("present",NativeType::ObservedValue,Mode::Consume),parameter("absent",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_branch"},..TEXT};

pub(crate) const INPUT:NativeBinding=NativeBinding{name:"input",parameters:&[parameter("value",NativeType::ObservedValue,Mode::Consume),parameter("defaultValue",NativeType::Structured(&INPUT_DEFAULT),Mode::Consume),parameter("options",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_input"},..TEXT};
pub(crate) const INPUT_CHANGE:NativeBinding=NativeBinding{name:"inputChange",parameters:&[parameter("value",NativeType::ObservedValue,Mode::Consume),parameter("defaultValue",NativeType::Structured(&INPUT_DEFAULT),Mode::Consume),parameter("options",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read),parameter("handler",NativeType::ObservedValue,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_input_change"},..TEXT};

pub(crate) fn runtime(with_native: bool) -> String {
    let native =
        include_str!("../../../../tsrust-ui/src/native.rs").replace("use crate::{", "use super::{")
        .replace("#[path = \"native_geometry.rs\"]\nmod geometry;",&format!("mod geometry {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/native_geometry.rs")))
        .replace("#[path = \"native_viewport.rs\"]\nmod viewport;",&format!("mod viewport {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/native_viewport.rs")))
        .replace("#[path = \"native_scrolling.rs\"]\nmod scrolling;",&format!("mod scrolling {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/native_scrolling.rs")))
        .replace("#[path = \"native_vector.rs\"]\nmod vector;",&format!("mod vector {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/native_vector.rs").replace("crate::", "super::super::")))
        .replace("#[path = \"native_table.rs\"]\nmod table;",&format!("mod table {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/native_table.rs").replace("crate::", "super::super::")))
        .replace("#[path = \"native_inline.rs\"]\nmod inline;",&format!("mod inline {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/native_inline.rs").replace("crate::theme::", "super::super::theme::").replace("crate::Semantics", "super::super::Semantics")))
        .replace("#[path = \"native_editing.rs\"]\nmod editing;",&format!("mod editing {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/native_editing.rs").replace("use crate::editable::normalize;", "use super::super::editable::normalize;")));
    let style = include_str!("../../../../tsrust-ui/src/style.rs")
        .replace("mod layout;",&format!("mod layout {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/style/layout.rs")))
        .replace("mod structural;",&format!("mod structural {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/style/structural.rs")))
        .replace("mod custom_spacing;",&format!("mod custom_spacing {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/style/custom_spacing.rs")))
        .replace("mod text;",&format!("mod text {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/style/text.rs")))
        .replace("mod edges;",&format!("mod edges {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/style/edges.rs")))
        .replace("crate::theme", "super::theme");
    let theme=include_str!("../../../../tsrust-ui/src/theme.rs")
        .replace("mod paint_color;",&format!("mod paint_color {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/theme/paint_color.rs")))
        .replace("include_str!(\"theme.json\")", &format!("{:?}",include_str!("../../../../tsrust-ui/src/theme.json")));
    let web = include_str!("../../../../tsrust-ui/src/web.rs")
        .replace("use crate::{", "use super::{")
        .replace("crate::style::", "super::style::").replace("crate::theme::", "super::theme::");
    let body=include_str!("../../../../tsrust-ui/src/lib.rs")
        .replace("#[cfg(feature = \"runtime\")]\n", "")
        .replace("#[cfg(feature = \"native\")]\nmod native;",if with_native {"__UI_NATIVE__"} else {""})
        .replace("__UI_NATIVE__",&format!("#[cfg(not(target_arch=\"wasm32\"))] mod native {{\n{native}\n}}"))
        .replace("#[cfg(feature = \"native\")]\npub use native::run;",if with_native {"#[cfg(all(not(target_arch=\"wasm32\"),not(feature = \"ui-acceptance\")))] pub use native::run;\n#[cfg(feature = \"ui-acceptance\")] pub use native::verify_window as run;"}else{""})
        .replace("#[cfg(feature = \"acceptance\")]\n#[doc(hidden)]\npub use native::verify_window;",if with_native {"#[cfg(feature = \"ui-acceptance\")] pub use native::verify_window;"}else{""})
        .replace("pub mod browser;",&format!("pub mod browser {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/browser.rs").replace("use crate::{", "use super::{")))
        .replace("mod visibility;",&format!("mod visibility {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/visibility.rs").replace("use crate::{", "use super::{")))
        .replace("mod editable;",&format!("mod editable {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/editable.rs")))
        .replace("pub mod clipboard;", &format!("pub mod clipboard {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/clipboard.rs").replace("crate::Node", "super::Node")))
        .replace("pub mod navigation;", &format!("pub mod navigation {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/navigation.rs")))
        .replace("mod vector;",&format!("mod vector {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/vector.rs").replace("use crate::{", "use super::{").replace("crate::theme::", "super::theme::")))
        .replace("mod table;",&format!("mod table {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/table.rs")))
        .replace("mod identity;",&format!("mod identity {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/identity.rs")))
        .replace("mod computed;",&format!("mod computed {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/computed.rs")))
        .replace("mod state;", &format!("mod state {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/state.rs")))
        .replace("#[cfg(test)]\nmod application_tests;", "")
        .replace("mod window;",&format!("mod window {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/window.rs")))
        .replace("#[cfg(feature = \"native\")]\npub use native::run_configured;",if with_native {"#[cfg(all(not(target_arch=\"wasm32\"),not(feature=\"ui-acceptance\")))] pub use native::run_configured;\n#[cfg(feature=\"ui-acceptance\")] pub use native::verify_window_configured as run_configured;"}else{""})
        .replace("mod branch;", &format!("mod branch {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/branch.rs")))
        .replace("pub mod style;",&format!("pub mod style {{\n{style}\n}}"))
        .replace("pub mod theme;",&format!("pub mod theme {{\n{theme}\n}}"))
        .replace("mod web;",&format!("mod web {{\n{web}\n}}"))

        .replace("feature=\"native\"","not(target_arch=\"wasm32\")")
        .replace("feature = \"runtime\"", "feature = \"ui\"")
        .replace("feature = \"acceptance\"","feature = \"ui-acceptance\"");
    format!("mod tsrust_native_ui {{\n{body}\n#[cfg(all(target_arch=\"wasm32\",feature=\"ui-web\"))] pub use browser::{{run,run_configured}};\n}}\n")
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
