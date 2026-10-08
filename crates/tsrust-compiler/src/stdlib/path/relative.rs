// Lexical relative paths with injectable working-directory observations.
fn path_separator(byte: u8) -> bool {
    matches!(byte, b'/' | b'\\')
}
#[derive(Default)]
struct Root {
    device: std::string::String,
    tail: usize,
    absolute: bool,
}
fn windows_root(path: &str) -> Root {
    let bytes = path.as_bytes();
    let mut root = Root::default();
    if bytes.first().is_some_and(|byte| path_separator(*byte)) {
        root.absolute = true;
        root.tail = 1;
        if bytes.len() > 2 && path_separator(bytes[1]) && !path_separator(bytes[2]) {
            let mut cursor = 2;
            while cursor < bytes.len() && !path_separator(bytes[cursor]) {
                cursor += 1;
            }
            let host_end = cursor;
            while cursor < bytes.len() && path_separator(bytes[cursor]) {
                cursor += 1;
            }
            let share = cursor;
            while cursor < bytes.len() && !path_separator(bytes[cursor]) {
                cursor += 1;
            }
            if host_end > 2 && share < cursor {
                if matches!(&path[2..host_end], "." | "?") {
                    root.device = format!("\\\\{}", &path[2..host_end]);
                    root.tail = host_end + 1;
                } else {
                    root.device = format!("\\\\{}\\{}", &path[2..host_end], &path[share..cursor]);
                    root.tail = cursor;
                }
            }
        }
    } else if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        root.device = path[..2].into();
        root.tail = 2;
        if bytes.len() > 2 && path_separator(bytes[2]) {
            root.absolute = true;
            root.tail = 3;
        }
    }
    root
}
fn resolved_tail(text: &str, windows: bool) -> std::string::String {
    let mut components = Vec::new();
    for component in text.split(|ch| ch == '/' || windows && ch == '\\') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            _ => components.push(component),
        }
    }
    components.join(if windows { "\\" } else { "/" })
}
fn resolve_path(
    path: &str,
    windows: bool,
    cwd: &mut impl FnMut() -> Result<std::string::String, std::string::String>,
    drive_cwd: &mut impl FnMut(&str) -> Option<std::string::String>,
) -> Result<std::string::String, std::string::String> {
    if !windows {
        let text = if path.starts_with('/') {
            path.to_owned()
        } else {
            format!("{}/{}", cwd()?, path)
        };
        return Ok(format!("/{}", resolved_tail(&text, false)));
    }
    let root = windows_root(path);
    let (device, tail) = if root.absolute && !root.device.is_empty() {
        (root.device, path[root.tail..].to_owned())
    } else {
        let mut base = if root.device.is_empty() {
            cwd()?
        } else {
            drive_cwd(&root.device).map(Ok).unwrap_or_else(&mut *cwd)?
        };
        let mut base_root = windows_root(&base);
        if !root.device.is_empty()
            && !base_root.device.eq_ignore_ascii_case(&root.device)
            && base.as_bytes().get(2) != Some(&b'\\')
        {
            let mut parts = Vec::new();
            for part in path[root.tail..].split(|ch| ch == '/' || ch == '\\') {
                match part {
                    "" | "." => {}
                    ".." if parts.last().is_some_and(|last| *last != "..") => {
                        parts.pop();
                    }
                    _ => parts.push(part),
                }
            }
            return Ok(format!("{}{}", root.device, parts.join("\\")));
        }
        if !root.device.is_empty()
            && (!base_root.absolute || !base_root.device.eq_ignore_ascii_case(&root.device))
        {
            base = format!("{}\\", root.device);
            base_root = windows_root(&base);
        }
        let device = if root.device.is_empty() {
            base_root.device
        } else {
            root.device
        };
        let tail = if root.absolute {
            path[root.tail..].to_owned()
        } else {
            format!("{}\\{}", &base[base_root.tail..], &path[root.tail..])
        };
        (device, tail)
    };
    Ok(format!("{device}\\{}", resolved_tail(&tail, true)))
}
fn posix_components<'a>(path: &'a str, cwd: &'a str) -> Vec<&'a str> {
    let base = if path.starts_with('/') { "" } else { cwd };
    let capacity = base
        .bytes()
        .chain(path.bytes())
        .filter(|byte| *byte == b'/')
        .count()
        + 2;
    let mut parts = Vec::with_capacity(capacity);
    for part in base.split('/').chain(path.split('/')) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts
}
fn posix_relative(from: &str, to: &str, cwd: &str) -> std::string::String {
    let a = posix_components(from, cwd);
    let b = posix_components(to, cwd);
    let common = a.iter().zip(&b).take_while(|(a, b)| a == b).count();
    let parents = a.len() - common;
    let capacity = parents * 3 + b[common..].iter().map(|part| part.len() + 1).sum::<usize>();
    let mut output = std::string::String::with_capacity(capacity);
    for part in std::iter::repeat_n("..", parents).chain(b[common..].iter().copied()) {
        if !output.is_empty() {
            output.push('/');
        }
        output.push_str(part);
    }
    output
}
/// Resolve both inputs against the supplied context, then compare complete components.
/// No canonicalization, directory traversal, or symlink resolution occurs.
pub fn relative_with(
    from: &str,
    to: &str,
    windows: bool,
    mut cwd: impl FnMut() -> Result<std::string::String, std::string::String>,
    mut drive_cwd: impl FnMut(&str) -> Option<std::string::String>,
) -> Result<std::string::String, std::string::String> {
    if from == to {
        return Ok(std::string::String::new());
    }
    if !windows {
        let directory = if from.starts_with('/') && to.starts_with('/') {
            std::string::String::new()
        } else {
            cwd()?
        };
        return Ok(posix_relative(from, to, &directory));
    }
    let mut observed = None;
    let mut context = || {
        if let Some(value) = &observed {
            return Ok(std::string::String::clone(value));
        }
        let value = cwd()?;
        observed = Some(value.clone());
        Ok(value)
    };
    let from = resolve_path(from, windows, &mut context, &mut drive_cwd)?;
    let to = resolve_path(to, windows, &mut context, &mut drive_cwd)?;
    let equal = |left: &str, right: &str| {
        if windows {
            left.to_lowercase() == right.to_lowercase()
        } else {
            left == right
        }
    };
    if equal(&from, &to) {
        return Ok(std::string::String::new());
    }
    let lower_from = from.to_lowercase();
    let lower_to = to.to_lowercase();
    let expands = from.encode_utf16().count() != lower_from.encode_utf16().count()
        || to.encode_utf16().count() != lower_to.encode_utf16().count();
    if !expands {
        // Compare UTF-16 coordinates so destination spelling survives Unicode casing.
        let left = lower_from
            .trim_matches('\\')
            .encode_utf16()
            .collect::<Vec<_>>();
        let right = lower_to
            .trim_matches('\\')
            .encode_utf16()
            .collect::<Vec<_>>();
        let original = to
            .trim_start_matches('\\')
            .encode_utf16()
            .collect::<Vec<_>>();
        let shared = left.iter().zip(&right).take_while(|(a, b)| a == b).count();
        let mut boundary = left[..shared].iter().rposition(|unit| *unit == 92);
        if shared < left.len().min(right.len()) && boundary.is_none() {
            return Ok(to);
        }
        if shared == left.len().min(right.len()) {
            if right.len() > shared {
                if right[shared] == 92 {
                    return Ok(std::string::String::from_utf16(&original[shared + 1..]).unwrap());
                }
                if shared == 2 {
                    return Ok(std::string::String::from_utf16(&original[shared..]).unwrap());
                }
            }
            if left.len() > shared {
                if left[shared] == 92 {
                    boundary = Some(shared);
                } else if shared == 2 {
                    boundary = Some(3);
                }
            }
            boundary = Some(boundary.unwrap_or(0));
        }
        let split = boundary.unwrap_or(0);
        let parents = if split + 1 > left.len() {
            0
        } else {
            left[split + 1..].iter().filter(|unit| **unit == 92).count() + 1
        };
        let suffix =
            std::string::String::from_utf16(&original[split.min(right.len())..right.len()])
                .unwrap();
        let prefix = std::iter::repeat_n("..", parents)
            .collect::<Vec<_>>()
            .join("\\");
        return Ok(if parents == 0 {
            suffix.trim_start_matches('\\').into()
        } else {
            format!("{prefix}{suffix}")
        });
    }
    // Length-changing lowercase mappings require full component comparison.
    let a = from
        .strip_suffix('\\')
        .unwrap_or(&from)
        .split('\\')
        .collect::<Vec<_>>();
    let b = to
        .strip_suffix('\\')
        .unwrap_or(&to)
        .split('\\')
        .collect::<Vec<_>>();
    let common = a.iter().zip(&b).take_while(|(a, b)| equal(a, b)).count();
    if common == 0 {
        return Ok(to);
    }
    Ok(std::iter::repeat_n("..", a.len() - common)
        .chain(b[common..].iter().copied())
        .collect::<Vec<_>>()
        .join("\\"))
}
/// Read host context only when required; report unavailable or non-UTF-8 cwd.
pub fn relative_host(from: &str, to: &str) -> Result<std::string::String, std::string::String> {
    relative_with(
        from,
        to,
        cfg!(windows),
        || {
            let directory = std::env::current_dir()
                .map_err(|error| format!("Cannot read current working directory: {error}"))?;
            directory
                .into_os_string()
                .into_string()
                .map_err(|_| "Current working directory is not valid UTF-8.".into())
        },
        |drive| std::env::var(format!("={drive}")).ok(),
    )
}
