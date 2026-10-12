//! Generated runtime source mapping: no source contents embedded in the executable.
use crate::project::SourceId;

pub(crate) fn setup(segments: &[(SourceId, usize, usize)]) -> String {
    let entries = segments
        .iter()
        .map(|(source, start, len)| format!("({start}usize, {}usize, {:?})", start + len, source.0))
        .collect::<Vec<_>>()
        .join(",");
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
                        for &(base, limit, path) in &[{entries}] {{
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
