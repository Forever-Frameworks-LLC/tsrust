//! Checked UI factories share one owned renderer representation across targets.
pub(crate) mod emit;
pub(crate) const SOURCE: &str = include_str!("api.tsr");
use crate::{
    Mode,
    native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType},
    native_signatures::{NativeTypeId, Signature},
};
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
        parameter("text", NativeType::String, Mode::Read),
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

pub(crate) fn runtime(with_native: bool) -> String {
    let native =
        include_str!("../../../../tsrust-ui/src/native.rs").replace("use crate::{", "use super::{");
    let style = include_str!("../../../../tsrust-ui/src/style.rs");
    let web = include_str!("../../../../tsrust-ui/src/web.rs")
        .replace("use crate::{", "use super::{")
        .replace("crate::style::", "super::style::");
    let body=include_str!("../../../../tsrust-ui/src/lib.rs")
        .replace("#[cfg(feature = \"native\")]\nmod native;",if with_native {"__UI_NATIVE__"} else {""})
        .replace("__UI_NATIVE__",&format!("mod native {{\n{native}\n}}"))
        .replace("#[cfg(feature = \"native\")]\npub use native::run;",if with_native {"#[cfg(not(feature = \"ui-acceptance\"))] pub use native::run;\n#[cfg(feature = \"ui-acceptance\")] pub use native::verify_window as run;"}else{""})
        .replace("#[cfg(feature = \"acceptance\")]\n#[doc(hidden)]\npub use native::verify_window;",if with_native {"#[cfg(feature = \"ui-acceptance\")] pub use native::verify_window;"}else{""})
        .replace("mod style;",&format!("mod style {{\n{style}\n}}"))
        .replace("mod web;",&format!("mod web {{\n{web}\n}}"))

        .replace("feature = \"acceptance\"","feature = \"ui-acceptance\"");
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
