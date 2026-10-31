#[allow(dead_code)]
mod views {
    include!("native_byte_views.rs");
    #[test]
    fn slices_alias_owner_mutations_and_keep_relative_error_offsets() {
        let mut owner = vec![0, 7, 8, 255, 9];
        let part = view(&owner, 1, 4).unwrap();
        assert_eq!(part.as_ptr(), owner[1..].as_ptr());
        assert_eq!(view(part, 1, 2).unwrap(), &[8]);
        let copied = view(part, 0, 2).unwrap().to_vec();
        {
            let window = view_mut(&mut owner, 1, 3).unwrap();
            set(window, 1, 10).unwrap();
            let e = set(window, 2, 256).unwrap_err();
            assert_eq!((e.code, e.offset), ("index-range", 2));
        }
        assert_eq!(owner[2], 10);
        assert_eq!(copied, vec![7, 8]);
        assert_eq!(view(&owner, 3, 1).unwrap_err().offset, 1);
        assert_eq!(view(&owner, 4, 9).unwrap_err().offset, 9);
    }
}
#[allow(dead_code)]
mod recursive {
    include!("native_byte_views_recursive.rs");
    #[test]
    fn recursive_record_moves_retain_one_borrowed_owner() {
        let owner = vec![7, 255];
        let child = Chunk {
            data: window(&owner),
            children: vec![],
        };
        let mut root = Chunk {
            data: window(&owner),
            children: vec![child],
        };
        let moved = root.children.pop().unwrap();
        assert_eq!(moved.data.as_ptr(), owner.as_ptr());
        assert_eq!(function0(1, 7, &"bytes".into()).0[0].0, vec![17, 119, 2]);
    }
}
#[allow(dead_code)]
mod linked {
    include!("native_byte_views_linked.rs");
    #[test]
    fn linked_record_moves_keep_borrowed_view_valid() {
        assert_eq!(function0(1, 7, &"bytes".into()).0[0].0, vec![17, 119, 2]);
    }
}
#[allow(dead_code)]
mod callbacks {
    include!("native_byte_views_callbacks.rs");
    #[test]
    fn once_picker_returns_other_owner_and_checks_captured_view() {
        let captured_owner = vec![7, 8, 9];
        let other = vec![1, 2, 3];
        let picker = make(view(&captured_owner, 0, 1).unwrap(), 7);
        let selected = picker(view(&other, 1, 3).unwrap());
        assert_eq!(selected.as_ptr(), other[1..].as_ptr());
        let selected = make_owner(view(&captured_owner, 0, 1).unwrap(), 7)(&other);
        assert_eq!(selected, &[2, 3]);
        assert_eq!(function0(1, 7, &"bytes".into()).0[0].0, vec![2, 8, 2, 8]);
    }
}
#[allow(dead_code)]
mod stored {
    include!("native_byte_views_stored_callbacks.rs");
    #[test]
    fn extracting_once_callback_keeps_borrowed_capture_after_record_move() {
        let owner = vec![7, 255];
        let tail = take(node(&owner));
        let selected = tail();
        assert_eq!(selected.as_ptr(), owner.as_ptr());
        assert_eq!(function0(1, 7, &"bytes".into()).0[0].0, vec![17, 119, 2]);
    }
}
#[allow(dead_code)]
mod captured_only {
    include!("native_byte_views_captured_only.rs");
    #[test]
    fn capture_only_record_exposes_view_after_extraction() {
        let owner = vec![7, 255];
        let tail = take(alternate(&owner));
        assert_eq!(tail(), &[7, 255]);
        assert_eq!(function0(1, 7, &"bytes".into()).0[0].0, vec![17, 119, 2]);
    }
}
#[allow(dead_code)]
mod nullable {
    include!("native_byte_views_nullable_callbacks.rs");
    #[test]
    fn terminal_nullable_callback_stays_absent() {
        assert!(take(empty()).is_none());
        assert_eq!(function0(1, 7, &"bytes".into()).0[0].0, vec![17, 112, 1]);
    }
}
#[allow(dead_code)]
mod replacement {
    include!("native_byte_views_parameter_stores.rs");
    #[test]
    fn replacement_capture_uses_new_parameter_view() {
        let owner = vec![7, 8];
        let mut chunk = node(&owner);
        replace(&mut chunk, view(&owner, 1, 2).unwrap());
        let action = take(chunk).unwrap();
        let selected = action();
        assert_eq!(selected.as_ptr(), owner[1..].as_ptr());
        assert_eq!(selected, &[8]);
        assert_eq!(function0(1, 7, &"bytes".into()).0[0].0, vec![17, 128, 1]);
    }
}
#[allow(dead_code)]
mod maps {
    include!("native_byte_views_maps.rs");
    #[test]
    fn returned_map_view_borrows_owner_after_map_drop() {
        let owner = vec![7, 255];
        let selected = exposed(&owner);
        assert_eq!(selected.as_ptr(), owner.as_ptr());
        assert_eq!(
            function0(1, 7, &"bytes".into()).0[0].0,
            vec![8, 28, 56, 7, 2]
        );
    }
}
#[allow(dead_code)]
mod deque {
    include!("native_byte_views_deque.rs");
    #[test]
    fn deque_moves_views_without_copying_owner() {
        let owner = vec![7, 255];
        let mut values = VecDeque::new();
        values.push_back(window(&owner));
        assert_eq!(values.pop_front().unwrap().as_ptr(), owner.as_ptr());
        assert_eq!(
            function0(1, 7, &"bytes".into()).0[0].0,
            vec![9, 7, 63, 0, 63]
        );
    }
}
#[allow(dead_code)]
mod deque_callbacks {
    include!("native_byte_views_deque_callbacks.rs");
    #[test]
    fn deque_reader_borrows_reusable_capture() {
        let owner = vec![7, 255];
        let callback = reader(window(&owner));
        assert_eq!(((callback.read)(), (callback.read)()), (7, 7));
        assert_eq!(
            function0(1, 7, &"bytes".into()).0[0].0,
            vec![9, 7, 63, 0, 63]
        );
    }
}
