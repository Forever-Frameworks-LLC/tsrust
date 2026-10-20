use std::collections::HashSet;

/// Preserve substring-based alias selection without scanning the output per record.
pub(super) fn referenced(output: &str, prefix: &str, suffix: &str, count: usize) -> HashSet<usize> {
    let mut referenced = HashSet::new();
    for (start, prefix) in output.match_indices(prefix) {
        let tail = &output[start + prefix.len()..];
        let mut index = 0usize;
        for (position, digit) in tail.bytes().take_while(u8::is_ascii_digit).enumerate() {
            let Some(next) = index.checked_mul(10).and_then(|index| index.checked_add(usize::from(digit - b'0'))) else { break; };
            index = next;
            if index >= count { break; }
            if tail[position + 1..].starts_with(suffix) { referenced.insert(index); }
            if index == 0 { break; }
        }
    }
    referenced
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_scan_preserves_previous_substring_selection() {
        for output in ["", "RecordArray", "RecordArray0 RecordArray12 RecordArray1234<'a>", "xRecordArray17suffix RecordArray001", "RecordArray999999999999999999999999999999999999", "struct RecordArray12( struct RecordArray0( struct RecordArray001( TsrDataSpread1234 TsrDataSpread0"] {
            for count in [0, 1, 10, 128, 1500] {
                for (prefix, suffix) in [("RecordArray", ""), ("struct RecordArray", "("), ("TsrDataSpread", "")] {
                    let actual = referenced(output, prefix, suffix, count);
                    let previous = (0..count).filter(|index| output.contains(&format!("{prefix}{index}{suffix}"))).collect::<HashSet<_>>();
                    assert_eq!(actual, previous, "{output:?}, prefix={prefix}, suffix={suffix}, count={count}");
                }
            }
        }
    }
}
