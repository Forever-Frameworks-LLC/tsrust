// Independent safe Rust binary16 rounding and retained result rows.
type String=std::borrow::Cow<'static,str>;
// Decode the binary64 exponent to select the binary16 spacing; round on the
// exact power-of-two grid with the standard ties-to-even operation.
fn half_round(value:f64)->f64{if !value.is_finite()||value==0.0{return value;}let magnitude=value.abs();if magnitude>=65520.0{return f64::INFINITY.copysign(value);}let exponent=((magnitude.to_bits()>>52)&0x7ff) as i32-1023;let spacing_exponent=(exponent-10).max(-24);let step=f64::from_bits(((spacing_exponent+1023) as u64)<<52);((magnitude/step).round_ties_even()*step).copysign(value)}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<f64>>{let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let scaled=base as f64;output.push(vec![half_round(scaled/7.0+0.1),half_round(2049.0+scaled*2.0),half_round(-(scaled+1.0)/67108864.0),half_round(65500.0+scaled),half_round(i as f64*0.001-1.0)]);}output}
fn main(){}
