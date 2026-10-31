#[allow(dead_code)]
mod comments {
    include!("native_jsonc_typed.rs");
    #[test]
    fn quoted_markers_escapes_unicode_and_comment_whitespace() {
        let input = "/* é😊 */{\"name\":\"https://é/\\\"/*keep*/\",// line\n\"values\":[20,17],\"active\":true}";
        let normalized = strip_comments(input).unwrap();
        assert_eq!(normalized.len(), input.len());
        let item: Item = serde_json::from_slice(&normalized).unwrap();
        assert_eq!(item.name, "https://é/\"/*keep*/");
        assert_eq!(item.values, vec![20, 17]);
        assert!(item.active);
        assert_eq!(normalized.iter().filter(|&&b| b == b'\n').count(), 1);
    }
    #[test]
    fn unterminated_comments_and_default_trailing_comma_policy() {
        assert!(strip_comments("/* open").is_err());
        let text =
            strip_comments("{\"name\":\"hello\",\"values\":[20,17,],\"active\":true}").unwrap();
        assert!(serde_json::from_slice::<Item>(&text).is_err());
        assert_eq!(
            function0(
                2,
                7,
                &String::Borrowed(
                    "/* header */{\"name\":\"hello\",\"values\":[20,17],\"active\":true}"
                )
            ),
            91
        );
    }
}
#[allow(dead_code)]
mod owned_reader {
    include!("native_jsonl_typed.rs");
    #[test]
    fn two_records_crlf_final_record_without_newline_and_eof() {
        let mut source=io::Cursor::new(b"{\"name\":\"hello\",\"values\":[20,17],\"active\":true}\r\n{\"name\":\"hello\",\"values\":[20,17],\"active\":true}");
        assert_eq!(consume(&mut source).unwrap(), 84);
        assert!(read_item(&mut source).unwrap().is_none());
    }
    #[test]
    fn invalid_json_and_wrong_shape_are_errors() {
        for input in [
            "{\n",
            "{\"name\":3,\"values\":[20,17],\"active\":true}\n",
            "\n",
        ] {
            assert_eq!(
                read_item(&mut io::Cursor::new(input)).err().unwrap().kind(),
                io::ErrorKind::InvalidData
            );
        }
    }
}
#[allow(dead_code)]
mod borrowed_reader {
    include!("native_jsonl_borrowed.rs");
    #[test]
    fn parent_reader_remains_available_after_borrowed_record() {
        let mut buffer =
            io::Cursor::new(b"{\"name\":\"hello\",\"values\":[20,17],\"active\":true}\ntail\n");
        {
            let reader = &mut buffer;
            let item = read_item(reader).unwrap().unwrap();
            assert_eq!(observe(item), 42);
        }
        let mut tail = std::string::String::new();
        buffer.read_line(&mut tail).unwrap();
        assert_eq!(tail, "tail\n");
    }
}
#[allow(dead_code)]
mod owned_writer {
    include!("native_jsonl_writer.rs");
    #[test]
    fn flush_produces_exact_record_and_newline() {
        let item = Item {
            active: true,
            name: "hello".to_owned(),
            values: vec![20, 17],
        };
        let mut writer = io::BufWriter::with_capacity(8192, Vec::new());
        write_item(&mut writer, &item).unwrap();
        assert!(writer.get_ref().is_empty());
        let output = writer.into_inner().unwrap();
        assert_eq!(
            output,
            b"{\"active\":true,\"name\":\"hello\",\"values\":[20,17]}\n"
        );
    }
    #[test]
    fn output_failure_propagates() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let item = Item {
            active: true,
            name: "hello".to_owned(),
            values: vec![20, 17],
        };
        assert!(write_item(&mut Broken, &item).is_err());
    }
}
#[allow(dead_code)]
mod borrowed_writer {
    include!("native_jsonl_borrowed_writer.rs");
    #[test]
    fn parent_writer_remains_usable_after_borrowed_flush() {
        let mut output = io::BufWriter::with_capacity(8192, Vec::new());
        {
            let writer = &mut output;
            write_item(
                writer,
                &Item {
                    active: true,
                    name: "hello".to_owned(),
                    values: vec![20, 17],
                },
            )
            .unwrap();
            writer.flush().unwrap();
        }
        output.write_all(b"tail\n").unwrap();
        let bytes = output.into_inner().unwrap();
        assert!(bytes.ends_with(b"\ntail\n"));
    }
}
