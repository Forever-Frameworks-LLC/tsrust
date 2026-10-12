// Unicode canonical/compatibility decomposition, stable ordering and composition.
fn tsr_normalization_class(code: u32) -> u8 {
    TSR_NORMALIZATION_CCC.binary_search_by_key(&code, |row| row.0)
        .map_or(0, |index| TSR_NORMALIZATION_CCC[index].1)
}
fn tsr_normalization_decompose(code: u32, compatibility: bool, output: &mut Vec<u32>) {
    let mut pending = vec![code];
    while let Some(code) = pending.pop() {
        if (0xAC00..0xAC00 + 11172).contains(&code) {
            let syllable = code - 0xAC00;
            let tail = syllable % 28;
            if tail != 0 { pending.push(0x11A7 + tail); }
            pending.push(0x1161 + syllable % 588 / 28);
            pending.push(0x1100 + syllable / 588);
        } else if let Ok(index) = TSR_NORMALIZATION_DECOMPOSE.binary_search_by_key(&code, |row| row.0) {
            let (_, compatible, start, length) = TSR_NORMALIZATION_DECOMPOSE[index];
            if compatible && !compatibility { output.push(code); continue; }
            pending.extend(TSR_NORMALIZATION_MAPPING[start as usize..start as usize + length as usize].iter().rev().copied());
        } else { output.push(code); }
    }
}
fn tsr_normalization_compose(left: u32, right: u32) -> Option<u32> {
    if (0x1100..0x1100 + 19).contains(&left) && (0x1161..0x1161 + 21).contains(&right) {
        return Some(0xAC00 + (left - 0x1100) * 588 + (right - 0x1161) * 28);
    }
    if (0xAC00..0xAC00 + 11172).contains(&left) && (left - 0xAC00) % 28 == 0 && (0x11A8..0x11A7 + 28).contains(&right) {
        return Some(left + right - 0x11A7);
    }
    let key = ((left as u64) << 21) | right as u64;
    TSR_NORMALIZATION_COMPOSE.binary_search_by_key(&key, |row| row.0)
        .ok().map(|index| TSR_NORMALIZATION_COMPOSE[index].1)
}
fn tsr_string_normalize(text: &str, form: Option<&str>, failure: &'static str) -> String {
    let (compatibility, compose) = match form.unwrap_or("NFC") {
        "NFC" => (false,true), "NFD" => (false,false),
        "NFKC" => (true,true), "NFKD" => (true,false),
        _ => panic!("{failure}"),
    };
    if text.is_ascii() { return String::Owned(text.to_owned()); }
    let mut codes = Vec::with_capacity(text.chars().count());
    for character in text.chars() { tsr_normalization_decompose(character as u32,compatibility,&mut codes); }
    let mut start = 0;
    for index in 0..codes.len() {
        if tsr_normalization_class(codes[index]) == 0 {
            codes[start..index].sort_by_key(|code|tsr_normalization_class(*code));
            start=index+1;
        }
    }
    codes[start..].sort_by_key(|code|tsr_normalization_class(*code));
    if compose {
        let mut composed: Vec<u32> = Vec::with_capacity(codes.len());
        let mut starter: Option<usize> = None;
        let mut previous_class = 0;
        for code in codes {
            let class=tsr_normalization_class(code);
            if let Some(index)=starter {
                if previous_class == 0 || previous_class < class {
                    if let Some(combination)=tsr_normalization_compose(composed[index],code) {
                        composed[index]=combination;continue;
                    }
                }
            }
            if class == 0 { starter=Some(composed.len()); }
            composed.push(code);previous_class=class;
        }
        codes=composed;
    }
    String::Owned(codes.into_iter().map(|code|char::from_u32(code).expect("Unicode normalization scalar")).collect())
}
