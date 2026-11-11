// Independent std floor; no compiler runtime/ABI inclusion.
type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let mut bytes=vec![0u8;64];let little=value1%2==0;let mut checksum=0i64;
 for _ in 0..value0 {
  bytes[1..2].copy_from_slice(&if little{(65 as i8).to_le_bytes()}else{(65 as i8).to_be_bytes()});
  checksum+=(if little{i8::from_le_bytes(bytes[1..2].try_into().unwrap())}else{i8::from_be_bytes(bytes[1..2].try_into().unwrap())}) as i64*4;
  bytes[3..5].copy_from_slice(&if little{(-12345 as i16).to_le_bytes()}else{(-12345 as i16).to_be_bytes()});
  checksum+=(if little{i16::from_le_bytes(bytes[3..5].try_into().unwrap())}else{i16::from_be_bytes(bytes[3..5].try_into().unwrap())}) as i64*4;
  bytes[5..9].copy_from_slice(&if little{(-1234567 as i32).to_le_bytes()}else{(-1234567 as i32).to_be_bytes()});
  checksum+=(if little{i32::from_le_bytes(bytes[5..9].try_into().unwrap())}else{i32::from_be_bytes(bytes[5..9].try_into().unwrap())}) as i64*4;
  bytes[9..17].copy_from_slice(&if little{(123456789 as i64).to_le_bytes()}else{(123456789 as i64).to_be_bytes()});
  checksum+=(if little{i64::from_le_bytes(bytes[9..17].try_into().unwrap())}else{i64::from_be_bytes(bytes[9..17].try_into().unwrap())}) as i64*4;
  bytes[17..18].copy_from_slice(&if little{(255 as u8).to_le_bytes()}else{(255 as u8).to_be_bytes()});
  checksum+=(if little{u8::from_le_bytes(bytes[17..18].try_into().unwrap())}else{u8::from_be_bytes(bytes[17..18].try_into().unwrap())}) as i64*4;
  bytes[19..21].copy_from_slice(&if little{(65535 as u16).to_le_bytes()}else{(65535 as u16).to_be_bytes()});
  checksum+=(if little{u16::from_le_bytes(bytes[19..21].try_into().unwrap())}else{u16::from_be_bytes(bytes[19..21].try_into().unwrap())}) as i64*4;
  bytes[21..25].copy_from_slice(&if little{(4294967295 as u32).to_le_bytes()}else{(4294967295 as u32).to_be_bytes()});
  checksum+=(if little{u32::from_le_bytes(bytes[21..25].try_into().unwrap())}else{u32::from_be_bytes(bytes[21..25].try_into().unwrap())}) as i64*4;
  bytes[25..29].copy_from_slice(&if little{(1.5 as f32).to_le_bytes()}else{(1.5 as f32).to_be_bytes()});
  checksum+=if (if little{f32::from_le_bytes(bytes[25..29].try_into().unwrap())}else{f32::from_be_bytes(bytes[25..29].try_into().unwrap())})==1.5{6}else{0};
  bytes[33..41].copy_from_slice(&if little{(-2.25 as f64).to_le_bytes()}else{(-2.25 as f64).to_be_bytes()});
  checksum+=if (if little{f64::from_le_bytes(bytes[33..41].try_into().unwrap())}else{f64::from_be_bytes(bytes[33..41].try_into().unwrap())})==-2.25{-9}else{0};
 }checksum
}
fn main(){}
