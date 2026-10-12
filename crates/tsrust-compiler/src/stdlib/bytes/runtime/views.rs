// Native byte views keep real slice lifetimes and never own the source allocation.

pub fn view(bytes: &[u8], start: i64, end: i64) -> Result<&[u8], ByteFailure> {
    Ok(&bytes[checked_range(bytes.len(), start, end)?])
}

pub fn view_mut(bytes: &mut [u8], start: i64, end: i64) -> Result<&mut [u8], ByteFailure> {
    let range = checked_range(bytes.len(), start, end)?;
    Ok(&mut bytes[range])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readonly_views_keep_storage_identity_and_allow_empty_end_ranges() {
        let bytes = [0, 65, 66, 255];
        let part = view(&bytes, 1, 3).unwrap_or_else(|_| panic!("valid view"));
        assert_eq!(part, [65, 66]);
        assert_eq!(part.as_ptr(), bytes[1..].as_ptr());
        let nested = view(part, 1, 2).unwrap_or_else(|_| panic!("valid nested view"));
        assert_eq!(nested.as_ptr(), bytes[2..].as_ptr());
        assert!(view(&bytes, 4, 4).is_ok_and(|part| part.is_empty()));
        assert!(view(&[], 0, 0).is_ok_and(|part| part.is_empty()));
    }

    #[test]
    fn mutable_views_write_through_and_reborrow_without_copying() {
        let mut bytes = [0, 65, 66, 255];
        let expected = bytes[1..].as_ptr();
        let part = view_mut(&mut bytes, 1, 3).unwrap_or_else(|_| panic!("valid view"));
        assert_eq!(part.as_ptr(), expected);
        assert_eq!(at(part, 1).ok(), Some(66));
        {
            let nested = view_mut(part, 1, 2).unwrap_or_else(|_| panic!("valid nested view"));
            assert!(set(nested, 0, 7).is_ok());
        }
        assert!(set(part, 0, 8).is_ok());
        assert_eq!(bytes, [0, 8, 7, 255]);
        assert!(view_mut(&mut bytes, 4, 4).is_ok_and(|part| part.is_empty()));
        assert!(view_mut(&mut [], 0, 0).is_ok_and(|part| part.is_empty()));
    }

    #[test]
    fn view_bounds_match_copy_errors_and_leave_source_unchanged() {
        let mut bytes = [0, 65, 255];
        for (start, end, offset) in [
            (-1, 99, -1),
            (4, 0, 4),
            (0, 4, 4),
            (2, 1, 1),
            (0, -1, -1),
            (0, i64::MAX, i64::MAX),
        ] {
            let read = view(&bytes, start, end)
                .err()
                .expect("invalid readonly range");
            let write = view_mut(&mut bytes, start, end)
                .err()
                .expect("invalid mutable range");
            let owned = copy(&bytes, start, end).err().expect("invalid copy range");
            assert_eq!(
                (read.code, read.message, read.offset),
                (owned.code, owned.message, Some(offset))
            );
            assert_eq!(
                (write.code, write.message, write.offset),
                (owned.code, owned.message, Some(offset))
            );
            assert_eq!(bytes, [0, 65, 255]);
        }
    }
}
