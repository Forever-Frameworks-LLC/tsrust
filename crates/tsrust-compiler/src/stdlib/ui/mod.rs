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
const SIGNAL:Signature=Signature::Named(NativeTypeId::UiSignal);
const HANDLE:Signature=Signature::Named(NativeTypeId::UiSignal);
pub(crate) const TEXT_CHILDREN:Signature=Signature::Array(&Signature::Union(&[Signature::String,HANDLE]));
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
        parameter("children", NativeType::Structured(&TEXT_CHILDREN), Mode::Consume),
        parameter("className", NativeType::String, Mode::Read),
    ],
    result: NativeType::Structured(&Signature::Result(&NODE, &ERROR)),
    returned_borrow: None,
    backend: NativeBackend::Bundled {
        symbol: "tsrust_ui_text",
    },
    features: &["ui"],
};
pub(crate) const VIEW: NativeBinding = NativeBinding {
    name: "view",
    parameters: &[
        parameter(
            "children",
            NativeType::Structured(&Signature::Array(&NODE)),
            Mode::Consume,
        ),
        parameter("className", NativeType::String, Mode::Read),
    ],
    backend: NativeBackend::Bundled {
        symbol: "tsrust_ui_view",
    },
    ..TEXT
};
pub(crate) const BUTTON: NativeBinding = NativeBinding {
    name: "disabledButton",
    parameters:&[parameter("text",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read)],
    backend: NativeBackend::Bundled {
        symbol: "tsrust_ui_button",
    },
    ..TEXT
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
pub(crate) const RUN: NativeBinding = NativeBinding {
    name: "run",
    parameters: &[
        parameter("node", NativeType::Structured(&NODE), Mode::Consume),
        parameter("title", NativeType::String, Mode::Read),
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &ERROR)),
    backend: NativeBackend::Bundled {
        symbol: "tsrust_ui_run",
    },
    features: &["ui-native"],
    ..TEXT
};

pub(crate) const SIGNAL_NEW:NativeBinding=NativeBinding{name:"signal",parameters:&[parameter("initial",NativeType::Int,Mode::Read)],result:NativeType::Structured(&SIGNAL),backend:NativeBackend::Bundled{symbol:"tsrust_ui_signal"},..TEXT};
pub(crate) const HANDLE_NEW:NativeBinding=NativeBinding{name:"handle",parameters:&[parameter("signal",NativeType::Structured(&SIGNAL),Mode::Read)],result:NativeType::Structured(&HANDLE),backend:NativeBackend::Bundled{symbol:"tsrust_ui_handle"},..TEXT};
pub(crate) const GET:NativeBinding=NativeBinding{name:"get",parameters:&[parameter("state",NativeType::Structured(&SIGNAL),Mode::Read)],result:NativeType::Structured(&Signature::Result(&Signature::Int,&ERROR)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_get"},..TEXT};
pub(crate) const SET:NativeBinding=NativeBinding{name:"set",parameters:&[parameter("state",NativeType::Structured(&SIGNAL),Mode::Read),parameter("value",NativeType::Int,Mode::Read)],result:NativeType::Structured(&Signature::Result(&Signature::Void,&ERROR)),backend:NativeBackend::Bundled{symbol:"tsrust_ui_set"},..TEXT};

pub(crate) const OWN_STATE:NativeBinding=NativeBinding{name:"ownState",parameters:&[parameter("node",NativeType::Structured(&NODE),Mode::Consume),parameter("state",NativeType::Structured(&SIGNAL),Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_own_state"},..TEXT};
pub(crate) const PRESS_BUTTON:NativeBinding=NativeBinding{name:"button",parameters:&[parameter("text",NativeType::String,Mode::Read),parameter("className",NativeType::String,Mode::Read),parameter("disabled",NativeType::Boolean,Mode::Read),parameter("onPress",NativeType::OwnedVoidCallback,Mode::Consume)],backend:NativeBackend::Bundled{symbol:"tsrust_ui_press_button"},..TEXT};

pub(crate) fn runtime(with_native: bool) -> String {
    let native =
        include_str!("../../../../tsrust-ui/src/native.rs").replace("use crate::{", "use super::{");
    let style = include_str!("../../../../tsrust-ui/src/style.rs");
    let web = include_str!("../../../../tsrust-ui/src/web.rs")
        .replace("use crate::{", "use super::{")
        .replace("crate::style::", "super::style::");
    let body=include_str!("../../../../tsrust-ui/src/lib.rs")
        .replace("#[cfg(feature = \"native\")]\nmod native;",if with_native {"__UI_NATIVE__"} else {""})
        .replace("__UI_NATIVE__",&format!("#[cfg(not(target_arch=\"wasm32\"))] mod native {{\n{native}\n}}"))
        .replace("#[cfg(feature = \"native\")]\npub use native::run;",if with_native {"#[cfg(all(not(target_arch=\"wasm32\"),not(feature = \"ui-acceptance\")))] pub use native::run;\n#[cfg(feature = \"ui-acceptance\")] pub use native::verify_window as run;"}else{""})
        .replace("#[cfg(feature = \"acceptance\")]\n#[doc(hidden)]\npub use native::verify_window;",if with_native {"#[cfg(feature = \"ui-acceptance\")] pub use native::verify_window;"}else{""})
        .replace("pub mod browser;",&format!("pub mod browser {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/browser.rs").replace("use crate::{", "use super::{")))
        .replace("mod state;", &format!("mod state {{\n{}\n}}",include_str!("../../../../tsrust-ui/src/state.rs")))
        .replace("mod style;",&format!("mod style {{\n{style}\n}}"))
        .replace("mod web;",&format!("mod web {{\n{web}\n}}"))

        .replace("feature = \"acceptance\"","feature = \"ui-acceptance\"");
    format!("mod tsrust_native_ui {{\n{body}\n#[cfg(all(target_arch=\"wasm32\",feature=\"ui-web\"))] pub use browser::run;\n}}\n")
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
    if program.records.iter().any(|record|record.native_type==Some(NativeTypeId::UiSignal)) {
        for binding in [&GET,&SET] {crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}
    }
    Ok(())
}
