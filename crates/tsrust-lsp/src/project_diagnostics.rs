//! Project diagnostic conversion uses exact loaded snapshots, never rereads source files.
use crate::{LineIndex, workspace::WorkspaceSnapshot};
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Uri,
};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use tsrust_compiler::project::{ProjectDiagnostic, SourceId, SourceLocation};

/// Local file URI to canonical identity; new unsaved files use normalized absolute paths.
pub fn source_id(uri: &str) -> Result<SourceId, String> {
    let path = source_path_id(uri)?;
    let canonical = std::fs::canonicalize(&path.0).unwrap_or_else(|_| PathBuf::from(&path.0));
    Ok(SourceId(
        canonical
            .to_str()
            .ok_or("Source path must be valid UTF-8.")?
            .into(),
    ))
}
/// Normalized URI path before filesystem canonicalization, retained for rename/watch events.
pub fn source_path_id(uri: &str) -> Result<SourceId, String> {
    let value = uri
        .strip_prefix("file://")
        .ok_or("Project modules require a local file URI.")?;
    let path = if value.starts_with('/') {
        value
    } else if let Some(path) = value.strip_prefix("localhost/") {
        return source_path_id(&format!("file:///{path}"));
    } else {
        return Err("Remote file URI authorities are not supported.".into());
    };
    if path.contains(['?', '#']) {
        return Err("File URIs must not contain a query or fragment.".into());
    }
    let bytes = path.as_bytes();
    let mut decoded = vec![];
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let pair = bytes.get(i + 1..i + 3).ok_or("Incomplete URI escape.")?;
            let hex = std::str::from_utf8(pair).map_err(|_| "Invalid URI escape.")?;
            decoded.push(u8::from_str_radix(hex, 16).map_err(|_| "Invalid URI escape.")?);
            i += 3;
        } else {
            decoded.push(bytes[i]);
            i += 1;
        }
    }
    let path = String::from_utf8(decoded).map_err(|_| "File URI must decode to UTF-8.")?;
    if path.contains('\0') {
        return Err("File URI must not contain a null byte.".into());
    }
    let mut normalized = PathBuf::new();
    for component in Path::new(&path).components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.is_absolute() {
        return Err("File URI must name an absolute path.".into());
    }
    Ok(SourceId(
        normalized
            .to_str()
            .ok_or("Source path must be valid UTF-8.")?
            .into(),
    ))
}
pub fn file_uri(id: &SourceId) -> Result<Uri, String> {
    if tsrust_compiler::project::bundled_source(id).is_some() {
        return format!("tsrust-stdlib:///{}", id.0).parse()
            .map_err(|_| "Cannot encode bundled source URI.".into());
    }
    if !Path::new(&id.0).is_absolute() || id.0.contains('\0') {
        return Err("Source identity must be an absolute local path.".into());
    }
    let mut uri = String::from("file://");
    for byte in id.0.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
            uri.push(char::from(byte));
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri.parse()
        .map_err(|_| "Cannot encode source file URI.".into())
}
/// All loaded files receive a bucket, so repaired project checks can clear old errors.
pub fn convert(
    snapshot: &WorkspaceSnapshot,
    error: Option<&ProjectDiagnostic>,
) -> Result<BTreeMap<SourceId, Vec<Diagnostic>>, String> {
    convert_all(snapshot, error.into_iter())
}

pub fn convert_all<'a>(
    snapshot: &WorkspaceSnapshot,
    errors: impl IntoIterator<Item = &'a ProjectDiagnostic>,
) -> Result<BTreeMap<SourceId, Vec<Diagnostic>>, String> {
    let sources = snapshot.sources();
    let mut output = sources
        .keys()
        .cloned()
        .map(|id| (id, vec![]))
        .collect::<BTreeMap<_, _>>();
    let location = |location: &SourceLocation| -> Result<Location, String> {
        let source = sources
            .get(&location.source).map(|source| source.as_str())
            .or_else(|| tsrust_compiler::project::bundled_source(&location.source))
            .ok_or_else(|| format!("No loaded snapshot for {}", location.source.0))?;
        Ok(Location::new(
            file_uri(&location.source)?,
            LineIndex::new(source).range(source, location.span)?,
        ))
    };
    for error in errors {
        let primary = location(&error.location)?;
        let related = error
            .related
            .iter()
            .map(|(value, message)| {
                Ok(DiagnosticRelatedInformation {
                    location: location(value)?,
                    message: message.clone(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        output
            .entry(error.location.source.clone())
            .or_default()
            .push(Diagnostic {
                range: primary.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String(error.code.into())),
                source: Some("tsrust".into()),
                message: error.message.clone(),
                related_information: (!related.is_empty()).then_some(related),
                ..Diagnostic::default()
            });
    }
    Ok(output)
}
