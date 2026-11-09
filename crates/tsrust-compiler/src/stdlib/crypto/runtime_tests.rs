#[test] fn service_crypto_known_answers_and_ranges(){
 use tsrust_native_crypto::*;
 assert_eq!(sha256(b"abc"),[0xba,0x78,0x16,0xbf,0x8f,0x01,0xcf,0xea,0x41,0x41,0x40,0xde,0x5d,0xae,0x22,0x23,0xb0,0x03,0x61,0xa3,0x96,0x17,0x7a,0x9c,0xb4,0x10,0xff,0x61,0xf2,0x00,0x15,0xad]);
 assert!(timing_safe_equal(b"abc",b"abc"));assert!(!timing_safe_equal(b"abc",b"abd"));assert!(!timing_safe_equal(b"a",b"aa"));
 assert_eq!(random_bytes(-1).unwrap_err().0,"Range");assert!(random_bytes(0).unwrap().is_empty());assert_eq!(random_bytes(1048577).unwrap().len(),1048577);assert_eq!(random_bytes(32).unwrap().len(),32);
}

#[test]
fn fixed_hash_mac_hkdf_and_streaming_boundaries(){
 use tsrust_native_crypto::*;
 for bytes in [b"".as_slice(),b"abc".as_slice()]{for algorithm in ["sha256","sha384","sha512"]{let expected=match algorithm{"sha256"=>sha256(bytes),"sha384"=>sha384(bytes),_=>sha512(bytes)};for split in 0..=bytes.len(){let mut state=Hasher::new(algorithm);state.update(&bytes[..split]);state.update(&[]);state.update(&bytes[split..]);assert_eq!(state.finalize(),expected);}}}
 let tag=hmac_sha256(&[11;20],b"Hi There");assert!(verify_hmac_sha256(&[11;20],b"Hi There",&tag));assert!(!verify_hmac_sha256(&[11;20],b"Hi There",&tag[..16]));assert!(!verify_hmac_sha256(&[11;20],b"changed",&tag));
 assert!(hkdf_sha256(&[],&[],&[],0).unwrap().is_empty());assert_eq!(hkdf_sha256(&[],&[],&[],8160).unwrap().len(),8160);for length in [-1,8161,i64::MAX]{assert_eq!(hkdf_sha256(&[],&[],&[],length).unwrap_err().0,"Range");}for(min,max)in[(1,1),(2,1)]{assert_eq!(random_int(min,max).unwrap_err().0,"Range");}assert_eq!(random_int(7,8).unwrap(),7);
}
