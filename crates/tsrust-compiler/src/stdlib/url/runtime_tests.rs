use tsrust_native_url::*;

#[test]
fn canonical_url_views_and_checked_setters() {
    let mut value=parse_url("https://user:secret@EXAMPLE.com:443/a/../b?q=1#f",None).unwrap();
    assert_eq!(href(&value),"https://user:secret@example.com/b?q=1#f");
    assert_eq!(protocol(&value),"https:");assert_eq!(username(&value),"user");
    assert_eq!(password(&value),Some("secret"));assert_eq!(hostname(&value),Some("example.com"));
    assert_eq!(port(&value),None);assert_eq!(pathname(&value),"/b");
    assert_eq!(search(&value),Some("?q=1"));assert_eq!(hash(&value),Some("#f"));
    assert_eq!(origin(&value),"https://example.com");
    let address=format_url(&value).as_ptr();
    assert_eq!(href(&value).as_ptr(),address);
    let before=value.clone();
    assert!(set_port(&mut value,Some(65536)).is_err());assert_eq!(value,before);
    assert!(set_protocol(&mut value,"not a scheme").is_err());assert_eq!(value,before);
    assert!(set_href(&mut value,"relative").is_err());assert_eq!(value,before);
    set_search(&mut value,Some("?")).unwrap();assert_eq!(search(&value),Some("?"));
    set_hash(&mut value,Some("#")).unwrap();assert_eq!(hash(&value),Some("#"));
    set_search(&mut value,None).unwrap();assert_eq!(search(&value),None);
    set_hash(&mut value,None).unwrap();assert_eq!(hash(&value),None);
}

#[test]
fn invalid_empty_host_never_enters_upstream_corrupt_state() {
    for input in ["sc://test:12/","tps://example.net:1234","foo://user:secret@host/"] {
        let mut value=parse_url(input,None).unwrap();let before=value.clone();
        let failure=set_hostname(&mut value,Some("")).unwrap_err();
        assert_eq!(failure.code,"host");assert_eq!(value,before);
        let _=password(&value);let _=hostname(&value);let _=pathname(&value);
    }
    let mut allowed=parse_url("foo://host/path",None).unwrap();
    set_hostname(&mut allowed,Some("")).unwrap();assert_eq!(format_url(&allowed),"foo:///path");
}

#[test]
fn queries_are_owned_snapshots_with_ordered_duplicates() {
    let mut value=parse_url("https://example.com/?x=1&x=2&plus=a+b&bad=%FF&literal=%",None).unwrap();
    let mut snapshot=search_params(&value);
    assert_eq!(snapshot.get("x"),Some("1"));assert_eq!(snapshot.get_all("x"),["1","2"]);
    assert_eq!(snapshot.get("plus"),Some("a b"));assert_eq!(snapshot.get("bad"),Some("\u{fffd}"));
    assert_eq!(snapshot.get("literal"),Some("%"));
    let pointer=snapshot.get("x").unwrap().as_ptr();
    assert_eq!(pointer,snapshot.get("x").unwrap().as_ptr());
    snapshot.set("x","3");assert_eq!(snapshot.get_all("x"),["3"]);
    assert!(format_url(&value).contains("x=1&x=2"));
    set_search_params(&mut value,&snapshot);snapshot.delete("x");
    assert!(format_url(&value).contains("x=3"));
    let mut sorted=parse_query("z=last&a=first&a=second");sorted.sort();
    assert_eq!(format_query(&sorted),"a=first&a=second&z=last");
    let question=parse_url("https://example.com/??x=y",None).unwrap();
    assert_eq!(search_params(&question).get("?x"),Some("y"));
}

#[test]
fn components_are_strict_and_distinct_from_form_queries() {
    let text="é /+?%\0";let encoded=encode_uri_component(text);
    assert_eq!(encoded,"%C3%A9%20%2F%2B%3F%25%00");
    assert_eq!(decode_uri_component(&encoded).unwrap(),text);
    assert_eq!(decode_uri_component("a+b").unwrap(),"a+b");
    assert_eq!(encode_uri_component("AZaz09-_.!~*'()"),"AZaz09-_.!~*'()");
    for invalid in ["%","%0","%GG","%FF","%ED%A0%80"] {assert!(decode_uri_component(invalid).is_err());}
}

#[test]
fn file_urls_and_relative_resolution_use_native_backend_rules() {
    let base=parse_url("https://example.com/a/b/",None).unwrap();
    assert_eq!(format_url(&resolve_url(&base,"../c").unwrap()),"https://example.com/a/c");
    assert_eq!(format_url(&parse_url("../c",Some(&base)).unwrap()),"https://example.com/a/c");
    assert!(path_to_file_url("relative").is_err());
    #[cfg(unix)] {
        let value=path_to_file_url("/tmp/é name#?").unwrap();
        assert_eq!(format_url(&value),"file:///tmp/%C3%A9%20name%23%3F");
        assert_eq!(file_url_to_path(&value).unwrap(),"/tmp/é name#?");
        assert!(file_url_to_path(&parse_url("file:///tmp/%FF",None).unwrap()).is_err());
    }
}

#[test]
fn normalization_corpus_covers_idna_ipv6_default_ports_and_opaque_urls() {
    for (input,expected) in [
        ("https://☃.net:443/a/../b", "https://xn--n3h.net/b"),
        ("http://[2001:db8::1]:80/a", "http://[2001:db8::1]/a"),
        ("ftp://EXAMPLE.com:21/a", "ftp://example.com/a"),
        ("mailto:user@example.com?subject=test", "mailto:user@example.com?subject=test"),
        ("https://example.com/?#", "https://example.com/?#"),
    ] {
        let value=parse_url(input,None).unwrap();assert_eq!(format_url(&value),expected);
        assert_eq!(parse_url(format_url(&value),None).unwrap(),value);
    }
    let mut opaque=parse_url("mailto:user@example.com",None).unwrap();
    assert_eq!(origin(&opaque),"null");assert_eq!(hostname(&opaque),None);
    let before=opaque.clone();assert!(set_username(&mut opaque,"other").is_err());assert_eq!(opaque,before);
    assert!(set_port(&mut opaque,Some(12)).is_err());assert_eq!(opaque,before);
    let special=parse_url("http://example.com/",None).unwrap();
    assert!(resolve_url(&special,"///test").is_err());
    assert_eq!(format_url(&resolve_url(&special,"//test").unwrap()),"http://test/");
}
