// Independent hand-written Rust using standard f64 math and checked integer bookkeeping.
fn round_up_ties(value:f64)->f64{if !value.is_finite() || value==0.0{return value;}let lower=value.floor();let result=if value-lower<0.5{lower}else{lower+1.0};if result==0.0{0.0_f64.copysign(value)}else{result}}
fn sign(value:f64)->f64{if value.is_nan() || value==0.0{value}else{value.signum()}}
fn uint32(value:f64)->u32{if !value.is_finite(){0}else{value.trunc().rem_euclid(4294967296.0) as u32}}
fn min_pair(a:f64,b:f64)->f64{if a.is_nan()||b.is_nan(){f64::NAN}else if a==0.0 && b==0.0{if a.is_sign_negative()||b.is_sign_negative(){-0.0}else{0.0}}else{a.min(b)}}
fn max_pair(a:f64,b:f64)->f64{if a.is_nan()||b.is_nan(){f64::NAN}else if a==0.0 && b==0.0{if a.is_sign_positive()||b.is_sign_positive(){0.0}else{-0.0}}else{a.max(b)}}
fn function0(value0: i64, value1: i64) -> f64 {let mut sum=value1 as f64;for i in 0..value0{let value=((i.checked_mul(17).unwrap().checked_add(value1).unwrap()%65).checked_sub(32).unwrap()) as f64*0.125;let result=sign(value);sum+=result;}sum}
fn main(){}
