//! Generated runtime source mapping: no source contents embedded in the executable.
use crate::project::SourceId;

/// The source table lives in its own module, not inline in `main`: file lengths
/// change with almost every edit, and a separate module keeps that change out of
/// the root module's codegen unit so incremental native builds reuse it.
pub(crate) fn segments_module(segments: &[(SourceId, usize, usize)]) -> String {
    let entries = segments
        .iter()
        .map(|(source, start, len)| format!("({start}usize, {}usize, {:?})", start + len, source.0))
        .collect::<Vec<_>>()
        .join(",");
    format!("\nmod tsrust_source_segments {{\n    pub(crate) static SEGMENTS: &[(usize, usize, &str)] = &[{entries}];\n}}\n")
}

pub(crate) fn setup(_segments: &[(SourceId, usize, usize)]) -> String {
    format!(
        r#"
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |information| {{
        let message = information.payload().downcast_ref::<&str>().copied()
            .or_else(|| information.payload().downcast_ref::<std::string::String>().map(|s| s.as_str()));
        if let Some(message) = message {{
            if let Some((reason, range)) = message.rsplit_once(" at source bytes ") {{
                if let Some((start, end)) = range.split_once("..") {{
                    if let (Ok(start), Ok(end)) = (start.parse::<usize>(), end.parse::<usize>()) {{
                        for &(base, limit, path) in crate::tsrust_source_segments::SEGMENTS {{
                            if start >= base && end >= start && end <= limit {{
                                eprintln!("runtime error: {{reason}} at {{path}} (source bytes {{}}..{{}})", start-base, end-base);
                                return;
                            }}
                        }}
                    }}
                }}
            }}
        }}
        previous_hook(information);
    }}));
"#
    )
}
