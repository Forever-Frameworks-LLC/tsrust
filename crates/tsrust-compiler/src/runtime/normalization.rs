// Unicode canonical/compatibility decomposition, stable ordering and composition.
// Scalars occupy 21 bits; cache the eight-bit combining class in the same u32.
const TSR_NORMALIZATION_SCALAR_MASK: u32 = (1 << 21) - 1;
fn tsr_normalization_page(code:u32,pages:&[u16])->std::ops::Range<usize>{
    let page=(code>>8) as usize;
    match (pages.get(page),pages.get(page+1)) {
        (Some(&start),Some(&end))=>usize::from(start)..usize::from(end),
        _=>0..0,
    }
}
fn tsr_normalization_class(code: u32) -> u8 {
    if code < TSR_NORMALIZATION_CCC[0].0 { return 0; }
    let rows=&TSR_NORMALIZATION_CCC[tsr_normalization_page(code,TSR_NORMALIZATION_CCC_PAGES)];
    rows.binary_search_by_key(&code, |row| row.0).map_or(0, |index| rows[index].1)
}
fn tsr_normalization_decompose(code:u32,compatibility:bool,output:&mut Vec<u32>){
 if code>=TSR_NORMALIZATION_DECOMPOSE[0].0 {
  if (0xAC00..0xAC00+11172).contains(&code){
   let syllable=code-0xAC00;let tail=syllable%28;
   output.extend_from_slice(&[0x1100+syllable/588,0x1161+syllable%588/28]);
   if tail!=0{output.push(0x11A7+tail);}return;
  }
  let rows=&TSR_NORMALIZATION_DECOMPOSE[tsr_normalization_page(code,TSR_NORMALIZATION_DECOMPOSE_PAGES)];
  if let Ok(index)=rows.binary_search_by_key(&code,|row|row.0){
   let (_,kind,start,length)=rows[index];
   if kind!=1||compatibility {
    let(start,length)=if compatibility&&kind>=2{let(_,start,length)=TSR_NORMALIZATION_COMPATIBILITY[usize::from(kind-2)];(start,length)}else{(start,length)};
    output.extend_from_slice(&TSR_NORMALIZATION_MAPPING[start as usize..start as usize+length as usize]);return;
   }
  }
 }
 output.push(code|((tsr_normalization_class(code) as u32)<<21));
}
fn tsr_normalization_compose(left: u32, right: u32) -> Option<u32> {
    if (0x1100..0x1100 + 19).contains(&left) && (0x1161..0x1161 + 21).contains(&right) {
        return Some(0xAC00 + (left - 0x1100) * 588 + (right - 0x1161) * 28);
    }
    if (0xAC00..0xAC00 + 11172).contains(&left) && (left - 0xAC00) % 28 == 0 && (0x11A8..0x11A7 + 28).contains(&right) {
        return Some(left + right - 0x11A7);
    }
    // Pinned composition pairs have no right scalar below U+0300.
    if right < 0x300 { return None; }
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
        if (codes[index] >> 21) == 0 {
            codes[start..index].sort_by_key(|code|code >> 21);
            start=index+1;
        }
    }
    codes[start..].sort_by_key(|code|code >> 21);
    if compose {
        let mut written = 0;
        let mut starter: Option<usize> = None;
        let mut previous_class = 0;
        for read in 0..codes.len() {
            let packed=codes[read];
            let code=packed & TSR_NORMALIZATION_SCALAR_MASK;
            let class=packed >> 21;
            if let Some(index)=starter {
                if previous_class == 0 || previous_class < class {
                    if let Some(combination)=tsr_normalization_compose(codes[index] & TSR_NORMALIZATION_SCALAR_MASK,code) {
                        codes[index]=combination;continue;
                    }
                }
            }
            if class == 0 { starter=Some(written); }
            codes[written]=packed;written+=1;previous_class=class;
        }
        codes.truncate(written);
    }
    let characters = || codes.iter().map(|code|char::from_u32(code & TSR_NORMALIZATION_SCALAR_MASK).expect("Unicode normalization scalar"));
    let mut output = std::string::String::with_capacity(characters().map(char::len_utf8).sum());
    output.extend(characters());
    String::Owned(output)
}
