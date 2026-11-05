use super::*;
fn success<T>(result: Result<T,ByteFailure>) -> T { result.unwrap_or_else(|failure|panic!("{}: {}",failure.code,failure.message)) }
#[test]
fn slice_operations_preserve_bounds_overlap_ownership_and_lexicographic_order() {
    let left=[0,255]; let right=[65,0]; let mut combined=success(concat(&left,&right));
    assert_eq!(combined,[0,255,65,0]); combined[0]=9; assert_eq!(left,[0,255]);
    let mut target=[7;6]; success(copy_into(&mut target,1,&left)); assert_eq!(target,[7,0,255,7,7,7]);
    let before=target; for offset in [-1,5,i64::MAX] {assert_eq!(copy_into(&mut target,offset,&left).err().unwrap().code,"range");assert_eq!(target,before);}
    success(copy_into(&mut target,6,&[]));
    for (target,start,end,expected) in [(1,0,4,[0,0,1,2,3]),(0,1,5,[1,2,3,4,4]),(2,2,2,[0,1,2,3,4]),(0,0,5,[0,1,2,3,4])] {
        let mut bytes=[0,1,2,3,4]; success(copy_within(&mut bytes,target,start,end)); assert_eq!(bytes,expected);
    }
    let mut bytes=[0,1,2]; for (target,start,end) in [(-1,0,1),(3,0,1),(0,-1,1),(0,2,1),(0,0,4),(i64::MAX,0,0)] {assert!(copy_within(&mut bytes,target,start,end).is_err());assert_eq!(bytes,[0,1,2]);}
    assert!(equals(&[],&[])); assert!(!equals(&[0],&[]));
    for (left,right,expected) in [(&[][..],&[0][..],-1),(&[0,255][..],&[1][..],-1),(&[255][..],&[0][..],1),(&[1][..],&[1][..],0)] {assert_eq!(compare(left,right),expected);}
}
#[test]
fn search_handles_empty_needle_start_ranges_and_repetitive_misses_without_encoding() {
    for (bytes,needle,start,expected) in [(&[0,255,0,255][..],&[0,255][..],1,2),(&[0,255][..],&[][..],2,2),(&[0,255][..],&[255,0][..],0,-1),(&[][..],&[][..],0,0)] {assert_eq!(success(index_of(bytes,needle,start)),expected);}
    for start in [-1,3,i64::MAX] {assert_eq!(index_of(&[1,2],&[],start).err().unwrap().code,"range");}
    let bytes=vec![97;65536]; let mut needle=vec![97;256]; needle[255]=98; assert_eq!(success(index_of(&bytes,&needle,0)),-1);
}
#[test]
fn integer_packets_are_exact_unaligned_and_failed_writes_are_atomic() {
    for (value,width,order,expected,unsigned) in [
        (i64::MIN,8,"little",&[0,0,0,0,0,0,0,128][..],false),
        (i64::MAX,8,"big",&[127,255,255,255,255,255,255,255][..],false),
        (-32768,2,"big",&[128,0][..],false),
        (4294967295,4,"little",&[255,255,255,255][..],true),
        (255,1,"little",&[255][..],true),
    ] {
        let mut bytes=[77;10]; if unsigned {success(write_uint(&mut bytes,1,value,width,order));assert_eq!(success(read_uint(&bytes,1,width,order)),value);} else {success(write_int(&mut bytes,1,value,width,order));assert_eq!(success(read_int(&bytes,1,width,order)),value);}
        assert_eq!(&bytes[1..1+width as usize],expected); assert_eq!(bytes[0],77); assert_eq!(bytes[1+width as usize],77);
    }
    for (width,minimum,maximum) in [(1,i64::from(i8::MIN),i64::from(i8::MAX)),(2,i64::from(i16::MIN),i64::from(i16::MAX)),(4,i64::from(i32::MIN),i64::from(i32::MAX)),(8,i64::MIN,i64::MAX)] {
        for value in [minimum,0,maximum] {for order in ["little","big"] {let mut bytes=[0;9];success(write_int(&mut bytes,1,value,width,order));assert_eq!(success(read_int(&bytes,1,width,order)),value);}}
    }
    let mut bytes=[77;10];
    for (offset,value,width,order,code) in [(0,128,1,"little","integer-range"),(0,-129,1,"big","integer-range"),(0,0,3,"little","width"),(-1,0,8,"big","range"),(9,0,2,"little","range"),(0,0,1,"native","byte-order")] {assert_eq!(write_int(&mut bytes,offset,value,width,order).err().unwrap().code,code);assert_eq!(bytes,[77;10]);}
    for (value,width,code) in [(-1,1,"integer-range"),(256,1,"integer-range"),(65536,2,"integer-range"),(4294967296,4,"integer-range"),(0,8,"width")] {assert_eq!(write_uint(&mut bytes,0,value,width,"little").err().unwrap().code,code);assert_eq!(bytes,[77;10]);}
    assert!(read_int(&bytes,i64::MAX,8,"little").is_err()); assert!(read_uint(&bytes,0,8,"little").is_err());
}
#[test]
fn float_packets_round_with_rust_ieee_rules_and_keep_signed_zero() {
    let mut bytes=[77;10];success(write_float32(&mut bytes,1,-0.0,"little"));assert_eq!(&bytes[1..5],&[0,0,0,128]);assert!(success(read_float32(&bytes,1,"little")).is_sign_negative());
    success(write_float32(&mut bytes,1,1.0+2f64.powi(-24),"big"));assert_eq!(&bytes[1..5],&[63,128,0,0]);assert_eq!(success(read_float32(&bytes,1,"big")),1.0);
    for value in [f64::INFINITY,f64::NEG_INFINITY,f64::MAX] {success(write_float32(&mut bytes,1,value,"little"));assert!(success(read_float32(&bytes,1,"little")).is_infinite());}
    success(write_float32(&mut bytes,1,f64::NAN,"big"));assert!(success(read_float32(&bytes,1,"big")).is_nan());
    success(write_float64(&mut bytes,1,1.5,"big"));assert_eq!(&bytes[1..9],&[63,248,0,0,0,0,0,0]);assert_eq!(success(read_float64(&bytes,1,"big")),1.5);
    for value in [-0.0,f64::INFINITY,f64::NEG_INFINITY,f64::MAX] {success(write_float64(&mut bytes,1,value,"little"));assert_eq!(success(read_float64(&bytes,1,"little")).to_bits(),value.to_bits());}
    success(write_float64(&mut bytes,1,f64::NAN,"little"));assert!(success(read_float64(&bytes,1,"little")).is_nan());
    let before=bytes;assert!(write_float64(&mut bytes,3,1.0,"big").is_err());assert_eq!(bytes,before);assert!(write_float32(&mut bytes,i64::MAX,1.0,"little").is_err());assert_eq!(bytes,before);
}
