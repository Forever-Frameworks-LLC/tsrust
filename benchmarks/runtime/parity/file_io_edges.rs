use std::sync::atomic::{AtomicUsize, Ordering};
struct Fixture(std::string::String);
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "tsrust-rust-file-edges-{}-{}.bin",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path.into_os_string().into_string().unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        let _ = std::fs::remove_file(format!("{}.out", self.0));
    }
}
#[allow(dead_code)]
mod explicit_read {
    include!("native_file_read_close.rs");
    #[test]
    fn cursor_limit_eof_and_release() {
        let f = super::Fixture::new(&[1, 2, 3]);
        assert_eq!(sample(&f.0, 2).unwrap().0, vec![2, 3]);
        assert_eq!(sample(&f.0, 10).unwrap().0, vec![3, 6]);
        assert_eq!(sample(&f.0, 0).unwrap().0, vec![0, 0]);
        std::fs::remove_file(&f.0).unwrap();
        assert!(sample(&f.0, 1).is_err());
    }
}
#[allow(dead_code)]
mod scoped_read {
    include!("native_file_scope_drop.rs");
    #[test]
    fn implicit_close_and_short_read() {
        let f = super::Fixture::new(&[4, 5]);
        assert_eq!(sample(&f.0, 3).unwrap().0, vec![2, 9]);
        std::fs::remove_file(&f.0).unwrap();
        assert!(sample(&f.0, 1).is_err());
    }
}
#[allow(dead_code)]
mod single_write {
    include!("native_file_write.rs");
    #[test]
    fn regular_write_truncates_and_reads_actual_bytes() {
        let f = super::Fixture::new(&[]);
        assert_eq!(sample(&f.0, 3).unwrap().0, vec![3, 147]);
        assert_eq!(std::fs::read(format!("{}.out", f.0)).unwrap(), b"012");
        assert_eq!(sample(&f.0, 1).unwrap().0, vec![1, 48]);
        assert_eq!(std::fs::read(format!("{}.out", f.0)).unwrap(), b"0");
    }
}
#[allow(dead_code)]
mod all_write {
    include!("native_file_write_all.rs");
    #[test]
    fn whole_write_observes_empty_and_full_ranges() {
        let f = super::Fixture::new(&[]);
        assert_eq!(sample(&f.0, 64).unwrap().0, vec![64, 4488]);
        assert_eq!(std::fs::read(format!("{}.out", f.0)).unwrap().len(), 64);
        assert_eq!(sample(&f.0, 0).unwrap().0, vec![0, 0]);
        assert!(std::fs::read(format!("{}.out", f.0)).unwrap().is_empty());
    }
}
#[allow(dead_code)]
mod full_file {
    include!("native_file_full_write_read.rs");
    #[test]
    fn ceiling_checks_extra_bytes_and_eof() {
        let f = super::Fixture::new(&[1, 2, 3]);
        assert_eq!(bounded_file_read(&f.0, 3).unwrap(), vec![1, 2, 3]);
        assert_eq!(
            bounded_file_read(&f.0, 2).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            bounded_file_read(&f.0, 0).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(sample(&f.0, 17).unwrap().0, vec![17, 1170]);
    }
}
#[allow(dead_code)]
mod buffered_file {
    include!("native_file_buffered_write_read.rs");
    #[test]
    fn chunks_cross_buffer_boundaries_and_flush_before_reopening() {
        let f = super::Fixture::new(&[]);
        let expected = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        for count in [0, 1, 3, 15, 16, 17, 63, 64] {
            let result = sample(&f.0, count).unwrap().0;
            assert_eq!(
                std::fs::read(format!("{}.out", f.0)).unwrap(),
                &expected[..count]
            );
            assert_eq!(result[0], i64::try_from(count).unwrap());
            assert_eq!(
                result[1],
                expected[..count].iter().map(|&v| i64::from(v)).sum::<i64>()
            );
        }
    }
}
