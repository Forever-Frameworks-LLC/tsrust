#[allow(dead_code)]
mod construct {
    include!("native_bytes.rs");
    #[test]
    fn byte_construction_preserves_boundaries_and_invalid_offset() {
        assert_eq!(from_array(&[0, 255]).unwrap(), vec![0, 255]);
        assert!(from_array(&[]).unwrap().is_empty());
        for bad in [-1, 256] {
            let e = from_array(&[7, bad, 0]).unwrap_err();
            assert_eq!((e.code, e.offset), ("byte-range", Some(1)));
        }
    }
}
#[allow(dead_code)]
mod access {
    include!("native_byte_access.rs");
    #[test]
    fn invalid_index_precedes_byte_validation_and_preserves_receiver() {
        let mut bytes = vec![0, 7, 255];
        let e = write(&mut bytes, -1, 256).unwrap_err();
        assert_eq!((e.code, e.offset), ("index-range", Some(-1)));
        let e = write(&mut bytes, 1, 256).unwrap_err();
        assert_eq!((e.code, e.offset), ("byte-range", Some(1)));
        assert_eq!(bytes, vec![0, 7, 255]);
        assert_eq!(at(&bytes, 3).unwrap_err().offset, Some(3));
    }
}
#[allow(dead_code)]
mod growth {
    include!("native_byte_growth.rs");
    #[test]
    fn resize_validates_before_mutation_even_when_shrinking() {
        let mut bytes = alloc(2, 7).unwrap();
        resize(&mut bytes, 4, 0).unwrap();
        assert_eq!(bytes, vec![7, 7, 0, 0]);
        assert_eq!(resize(&mut bytes, 0, 256).unwrap_err().code, "byte-range");
        assert_eq!(bytes, vec![7, 7, 0, 0]);
        assert_eq!(
            resize(&mut bytes, -1, 256).unwrap_err().code,
            "length-range"
        );
        resize(&mut bytes, 1, 0).unwrap();
        resize(&mut bytes, 3, 255).unwrap();
        assert_eq!(bytes, vec![7, 255, 255]);
    }
}
#[allow(dead_code)]
mod copy_append {
    include!("native_byte_copy_append.rs");
    #[test]
    fn failed_append_is_atomic_and_range_copy_has_independent_owner() {
        let mut bytes = vec![7, 65];
        append(&mut bytes, &[255, 7]).unwrap();
        let e = append(&mut bytes, &[8, 256]).unwrap_err();
        assert_eq!(e.offset, Some(1));
        assert_eq!(bytes, vec![7, 65, 255, 7]);
        let mut part = copy(&bytes, 0, 2).unwrap();
        write(&mut bytes, 0, 8).unwrap();
        write(&mut part, 1, 9).unwrap();
        assert_eq!(part, vec![7, 9]);
        assert_eq!(bytes, vec![8, 65, 255, 7]);
        assert!(copy(&bytes, 4, 4).unwrap().is_empty());
        assert_eq!(copy(&bytes, -1, 5).unwrap_err().offset, Some(-1));
        assert_eq!(copy(&bytes, 0, 5).unwrap_err().offset, Some(5));
    }
}
#[allow(dead_code)]
mod clone {
    include!("native_byte_clone.rs");
    #[test]
    fn cloned_nested_byte_owner_stays_independent() {
        let mut rows = vec![Holder {
            bytes: vec![7, 7, 7],
        }];
        let saved = rows.clone();
        rows[0].bytes[0] = 8;
        assert_eq!(saved[0].bytes, vec![7, 7, 7]);
        assert_eq!(
            function0(1, 7, &"bytes".into()).0[0].0,
            vec![7, 7, 7, 7, 8, 3, 3]
        );
    }
}
#[allow(dead_code)]
mod utf8 {
    include!("native_byte_utf8.rs");
    #[test]
    fn utf8_keeps_nul_bom_and_noncharacters_and_reports_first_invalid_byte() {
        let text = "\u{feff}é🦀\0\u{fdd0}\u{10ffff}";
        let bytes = from_utf8(text);
        assert_eq!(to_utf8(&bytes).unwrap(), text);
        for bad in [vec![65, 195, 40], vec![65, 226, 130], vec![65, 255]] {
            let e = to_utf8(&bad).unwrap_err();
            assert_eq!((e.code, e.offset), ("utf8", Some(1)));
        }
        assert_eq!(to_utf8(&[0xed, 0xa0, 0x80]).unwrap_err().offset, Some(0));
    }
}
#[allow(dead_code)]
mod deque {
    include!("native_deque_clone.rs");
    #[test]
    fn deque_clone_recursively_copies_item_arrays() {
        assert_eq!(
            function0(1, 7, &"bytes".into()).0[0].0,
            vec![2, 7, 3, 8, 0, 0, 7]
        );
        let mut original = VecDeque::from([Item { values: vec![7] }]);
        let mut copied = original.clone();
        copied.front_mut().unwrap().values.push(8);
        assert_eq!(original.pop_front().unwrap().values, vec![7]);
        assert_eq!(copied.pop_front().unwrap().values, vec![7, 8]);
    }
}
#[allow(dead_code)]
mod heap {
    include!("native_heap.rs");
    #[test]
    fn heap_defaults_to_max_and_copy_preserves_original() {
        let mut values = BinaryHeap::from([2, 9, 1]);
        let mut copied = values.clone();
        assert_eq!(
            (copied.pop(), copied.pop(), copied.pop()),
            (Some(9), Some(2), Some(1))
        );
        assert_eq!(values.pop(), Some(9));
        assert_eq!(
            function0(1, 0, &"bytes".into()).0[0].0,
            vec![9, 8, 36, 0, 36, 36, 9]
        );
    }
}
#[allow(dead_code)]
mod ordered_set {
    include!("native_ordered_set.rs");
    #[test]
    fn ordered_set_sorts_wrapping_keys_and_deduplicates() {
        let result = function0(1, 252, &"bytes".into());
        assert_eq!(
            (result.0[0].0[0], result.0[0].0[1], result.0[0].0[2]),
            (9, 0, 255)
        );
        assert_eq!(&result.0[0].0[4..], &[1, 1, 8, 9]);
    }
}
#[allow(dead_code)]
mod ordered_map {
    include!("native_ordered_map.rs");
    #[test]
    fn ordered_map_updates_duplicate_key_and_preserves_copy_size() {
        let result = function0(1, 252, &"bytes".into());
        assert_eq!(
            (result.0[0].0[0], result.0[0].0[1], result.0[0].0[2]),
            (9, 0, 255)
        );
        assert_eq!(&result.0[0].0[5..], &[1, 1, 8, 9]);
        let mut map = BTreeMap::new();
        map.insert(2, 3);
        map.insert(1, 4);
        map.insert(2, 9);
        assert_eq!(map.into_iter().collect::<Vec<_>>(), vec![(1, 4), (2, 9)]);
    }
}
