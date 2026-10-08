//! Bundled UTF-8 path operations; no filesystem access.
/// Last path component, with platform separators and optional case-sensitive suffix removal.
/// Explicit platform selection permits portable boundary testing of both separator conventions.
pub fn basename(path: &str, suffix: Option<&str>, windows: bool) -> std::string::String {
    let separator = |ch| ch == '/' || (windows && ch == '\\');
    let bytes = path.as_bytes();
    let drive = windows && bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let mut start = if drive { 2 } else { 0 };
    let suffix = suffix.filter(|suffix| {
        !suffix.is_empty() && suffix.encode_utf16().count() <= path.encode_utf16().count()
    });
    if suffix == Some(path) {
        return std::string::String::new();
    }
    let mut expected = suffix.map(|suffix| suffix.chars().rev());
    let mut next = expected.as_mut().and_then(Iterator::next);
    let mut end = None;
    let mut component_end = None;
    for (offset, ch) in path
        .char_indices()
        .rev()
        .take_while(|(offset, _)| *offset >= start)
    {
        if separator(ch) {
            if component_end.is_some() {
                start = offset + ch.len_utf8();
                break;
            }
            continue;
        }
        let first_end = *component_end.get_or_insert(offset + ch.len_utf8());
        if suffix.is_none() {
            end.get_or_insert(first_end);
            continue;
        }
        if let Some(wanted) = next {
            if wanted == ch {
                next = expected.as_mut().and_then(Iterator::next);
                if next.is_none() {
                    end = Some(offset);
                }
            } else {
                next = None;
                end = Some(first_end);
            }
        }
    }
    let end = if suffix.is_some() {
        if end == Some(start) {
            component_end.unwrap_or(path.len())
        } else {
            end.unwrap_or(path.len())
        }
    } else {
        end.unwrap_or(start)
    };
    path[start..end].to_owned()
}


fn separator(byte: u8, windows: bool) -> bool {
    byte == b'/' || (windows && byte == b'\\')
}
fn drive_prefix(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
/// Lexical parent, preserving original separators and root spelling.
pub fn dirname(path: &str, windows: bool) -> std::string::String {
    let bytes = path.as_bytes();
    if bytes.is_empty() { return ".".into(); }
    let mut root = 0;
    if separator(bytes[0], windows) {
        root = 1;
        if windows && bytes.len() > 1 && separator(bytes[1], true) {
            let mut cursor = 2;
            let host = cursor;
            while cursor < bytes.len() && !separator(bytes[cursor], true) { cursor += 1; }
            if cursor > host && cursor < bytes.len() {
                while cursor < bytes.len() && separator(bytes[cursor], true) { cursor += 1; }
                let share = cursor;
                while cursor < bytes.len() && !separator(bytes[cursor], true) { cursor += 1; }
                if cursor > share {
                    if cursor == bytes.len() { return path.into(); }
                    root = cursor + 1;
                }
            }
        }
    } else if windows && drive_prefix(bytes) {
        root = if bytes.len() > 2 && separator(bytes[2], true) { 3 } else { 2 };
    }
    let mut found_component = false;
    let offset = if windows { root } else { 1 };
    for index in (offset..bytes.len()).rev() {
        if separator(bytes[index], windows) {
            if found_component {
                let end = if !windows && root == 1 && index == 1 { 2 } else { index };
                return path[..end].into();
            }
        } else { found_component = true; }
    }
    if root == 0 { ".".into() } else { path[..root].into() }
}
/// Last dot suffix of the final component; leading-dot names and dot-dot have no suffix.
pub fn extname(path: &str, windows: bool) -> std::string::String {
    let component = basename(path, None, windows);
    if component == ".." { return std::string::String::new(); }
    match component.rfind('.') {
        Some(index) if index > 0 => component[index..].into(),
        _ => std::string::String::new(),
    }
}


/// Test platform root syntax without resolving the path.
pub fn is_absolute(path: &str, windows: bool) -> bool {
    let bytes = path.as_bytes();
    !bytes.is_empty() && (separator(bytes[0],windows) ||
        (windows && drive_prefix(bytes) && bytes.len() > 2 && separator(bytes[2],true)))
}
fn reserved_device(text: &str) -> bool {
    let upper=text.to_uppercase();
    matches!(upper.as_str(),"CON"|"PRN"|"AUX"|"NUL"|"COM¹"|"COM²"|"COM³"|"LPT¹"|"LPT²"|"LPT³") ||
        (upper.len()==4 && (upper.starts_with("COM") || upper.starts_with("LPT")) && matches!(upper.as_bytes()[3],b'1'..=b'9'))
}
/// Collapse separators and dot segments lexically; never consult the filesystem.
pub fn normalize(path: &str, windows: bool) -> std::string::String {
    let bytes=path.as_bytes();
    if bytes.is_empty() {return ".".into();}
    if windows && bytes.len()==1 {return if bytes[0]==b'/' {"\\".into()} else {path.into()};}
    let absolute=is_absolute(path,windows);
    let mut offset=0;
    let mut device=std::string::String::new();
    if separator(bytes[0],windows) {
        offset=1;
        if windows && bytes.len()>1 && separator(bytes[1],true) {
            // A UNC root contains two nonempty components after exactly two separators.
            let mut cursor=2;
            let host=cursor;
            while cursor<bytes.len() && !separator(bytes[cursor],true) {cursor+=1;}
            let host_end=cursor;
            if cursor>host && cursor<bytes.len() {
                while cursor<bytes.len() && separator(bytes[cursor],true) {cursor+=1;}
                let share=cursor;
                while cursor<bytes.len() && !separator(bytes[cursor],true) {cursor+=1;}
                if cursor>share {
                    let server=&path[host..host_end];
                    if matches!(server,"."|"?") {
                        device=format!("\\\\{server}");offset=4;
                        if let Some(colon)=path.find(':') {
                            if colon>=4 && reserved_device(&path[4..colon]) {
                                device=format!("\\\\?\\{}", &path[4..=colon]);offset=colon+1;
                            }
                        }
                    } else {
                        device=format!("\\\\{server}\\{}",&path[share..cursor]);offset=cursor;
                        if cursor==bytes.len() {return format!("{device}\\");}
                    }
                }
            }
        }
    } else if windows {
        if drive_prefix(bytes) {
            device=path[..2].into();offset=if absolute {3} else {2};
        } else if let Some(colon)=path.find(':') {
            if reserved_device(&path[..colon]) {device=path[..=colon].into();offset=colon+1;}
        }
    }
    let mut components:Vec<&str>=Vec::new();
    for component in path[offset..].split(|ch| ch=='/' || (windows && ch=='\\')) {
        match component {
            ""|"."=>{},
            ".."=>{
                if components.last().is_some_and(|last| *last!="..") {components.pop();}
                else if !absolute {components.push(component);}
            },
            _=>components.push(component),
        }
    }
    let sep=if windows {"\\"} else {"/"};
    let mut tail=components.join(sep);
    if tail.is_empty() && !absolute {tail.push('.');}
    if !tail.is_empty() && separator(*bytes.last().unwrap(),windows) {tail.push_str(sep);}
    if windows && !absolute && device.is_empty() && path.contains(':') {
        let tail_bytes=tail.as_bytes();
        let suspicious=drive_prefix(tail_bytes) || bytes.iter().enumerate().any(|(index,byte)| *byte==b':' && (index+1==bytes.len() || separator(bytes[index+1],true)));
        if suspicious {return format!(".\\{tail}");}
    }
    if windows && path.find(':').is_some_and(|colon| reserved_device(&path[..colon])) {return format!(".\\{device}{tail}");}
    if absolute {format!("{device}{sep}{tail}")} else {format!("{device}{tail}")}
}
