#[path="adapter.rs"] mod adapter;
#[test]
fn checked_adapters_reject_loss_and_preserve_bounds(){
 for (width,min,max) in [(8,-128,127),(16,-32768,32767),(32,i32::MIN as i64,i32::MAX as i64)]{
  assert_eq!(adapter::signed(min,width),Ok(min));assert_eq!(adapter::signed(max,width),Ok(max));
  assert!(adapter::signed(min-1,width).is_err());assert!(adapter::signed(max+1,width).is_err());
 }
 for (width,max) in [(8,255),(16,65535),(32,4294967295)]{
  assert_eq!(adapter::unsigned(max,width),Ok(max));assert!(adapter::unsigned(max+1,width).is_err());
 }
 assert_eq!(adapter::signed(i64::MIN,64),Ok(i64::MIN));assert_eq!(adapter::unsigned(i64::MAX,64),Ok(i64::MAX));
 for value in [f64::NAN,f64::INFINITY,1e300,1e-300,0.1]{assert!(adapter::float32(value).is_err());}
 assert!(adapter::float32(-0.0).unwrap().is_sign_negative());
 assert_eq!(adapter::incrementWide("18446744073709551615",64),Err("Range".into()));
 assert_eq!(adapter::incrementWide("340282366920938463463374607431768211456",128),Err("Range".into()));
 assert!(adapter::incrementOptional(adapter::OptionalInt::Present(i64::MAX)).is_err());
 assert!(adapter::bytes(vec![-1]).is_err());assert_eq!(adapter::bytes(vec![]),Ok(vec![]));
}
