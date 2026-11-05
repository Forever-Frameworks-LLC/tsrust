use super::*;
fn success<T>(result: Result<T, ByteFailure>) -> T { match result { Ok(value) => value, Err(error) => panic!("{}: {}", error.code, error.message) } }
#[test]
fn published_base64_vectors_are_strict_and_hex_covers_every_byte() {
    for (plain, encoded) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foob", "Zm9vYg=="), ("fooba", "Zm9vYmE="), ("foobar", "Zm9vYmFy")] {
        assert_eq!(success(to_base64(plain.as_bytes(), false)), encoded);
        assert_eq!(success(from_base64(encoded, false)), plain.as_bytes());
        let unpadded = encoded.trim_end_matches('=');
        assert_eq!(success(to_base64(plain.as_bytes(), true)), unpadded);
        assert_eq!(success(from_base64(unpadded, true)), plain.as_bytes());
    }
    assert_eq!(success(to_base64(&[251, 255], false)), "+/8=");
    assert_eq!(success(to_base64(&[251, 255], true)), "-_8");
    assert_eq!(success(from_base64("-_8", true)), [251, 255]);
    for byte in 0..=255u8 {
        let encoded = success(to_hex(&[byte]));
        assert_eq!(encoded, format!("{byte:02x}"));
        assert_eq!(success(from_hex(&encoded.to_uppercase())), [byte]);
        assert_eq!(success(from_base64(&success(to_base64(&[byte], false)), false)), [byte]);
    }
    for (text, url, code) in [("Zh==", false, "base64-trailing-bits"), ("Zg", false, "base64-padding"), ("Zg==", true, "base64-padding"), ("Z g==", false, "base64-byte"), ("-_8=", false, "base64-byte"), ("+/8", true, "base64-byte")] {
        assert_eq!(from_base64(text, url).err().unwrap().code, code, "{text}");
    }
    assert_eq!(from_hex("0").err().unwrap().code, "hex-length");
    let error = from_hex("00g0").err().unwrap(); assert_eq!((error.code, error.offset), ("hex-digit", Some(2)));
    let error = from_base64("Zh==", false).err().unwrap(); assert_eq!(error.offset, Some(1));
}
#[test]
fn utf16_preserves_unicode_nul_bom_and_reports_byte_offsets() {
    for (order, expected) in [("little", &[0x41,0,0x3d,0xd8,0,0xde][..]), ("big", &[0,0x41,0xd8,0x3d,0xde,0][..])] {
        assert_eq!(success(from_utf16("A😀", order)), expected);
        assert_eq!(success(to_utf16(expected, order)), "A😀");
        for text in ["", "é😀", "A\0B", "\u{feff}z", "汉字"] { assert_eq!(success(to_utf16(&success(from_utf16(text, order)), order)), text); }
    }
    for (bytes, offset) in [(&[0][..], 0), (&[0,0xd8][..], 0), (&[0,0xdc][..], 0), (&[65,0,0,0xd8,65,0][..], 2), (&[0x3d,0xd8,0,0xde,0,0xdc][..], 4)] {
        let error = to_utf16(bytes, "little").err().unwrap(); assert_eq!((error.code, error.offset), ("utf16", Some(offset)));
    }
    assert_eq!(from_utf16("", "native").err().unwrap().code, "byte-order");
    assert_eq!(to_utf16(&[], "native").err().unwrap().code, "byte-order");
}
#[test]
fn lossy_utf8_has_standard_replacement_and_independent_output() {
    for (bytes, expected) in [(&[65,195,40][..], "A�("), (&[226,130][..], "�"), (&[255,128][..], "��"), (&[237,160,128][..], "���")] { assert_eq!(success(to_utf8_lossy(bytes)), expected); }
    let mut source = vec![65, 66];
    let output = success(to_utf8_lossy(&source)); source[0] = 67;
    assert_eq!(output, "AB"); assert_ne!(source.as_ptr(), output.as_ptr());
    let encoded = success(from_utf16("hello", "little")); assert_eq!(encoded.capacity(), encoded.len());
    assert!(codec_buffer(usize::MAX).is_err());
}
