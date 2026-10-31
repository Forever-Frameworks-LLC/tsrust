#[allow(dead_code)]
mod basename {
    include!("std_path_basename_ascii.rs");
    #[test]
    fn suffix_and_trailing_separators() {
        assert_eq!(basename("/a/report.txt/", ".txt"), "report");
        assert_eq!(basename("/a/report.txt", ".gz"), "report.txt");
        assert_eq!(basename("/", ".txt"), "");
        assert_eq!(basename("文/é😊.txt", ".txt"), "é😊");
    }
}
#[allow(dead_code)]
mod dirname {
    include!("std_path_dirname_ascii.rs");
    #[test]
    fn lexical_parent_preserves_interior_separators() {
        assert_eq!(dirname("/a/b/"), "/a");
        assert_eq!(dirname("/a//b"), "/a/");
        assert_eq!(dirname("//a"), "//");
        assert_eq!(dirname("/"), "/");
        assert_eq!(dirname("report.txt"), ".");
    }
}
#[allow(dead_code)]
mod extension {
    include!("std_path_extname_ascii.rs");
    #[test]
    fn hidden_names_and_last_extension() {
        assert_eq!(extname("/a/archive.tar.gz/"), ".gz");
        assert_eq!(extname(".config"), "");
        assert_eq!(extname(".."), "");
        assert_eq!(extname("..config"), ".config");
        assert_eq!(extname("file."), ".");
    }
}
#[allow(dead_code)]
mod normalize {
    include!("std_path_normalize_ascii.rs");
    #[test]
    fn parent_segments_root_and_trailing_separator() {
        for (input, expected) in [
            ("", "."),
            ("/../../a", "/a"),
            ("a/../../b", "../b"),
            ("./", "./"),
            ("/tmp//reports/../report.txt", "/tmp/report.txt"),
            ("reports/../archive/", "archive/"),
            ("a/../", "./"),
        ] {
            assert_eq!(normalize(input), expected);
        }
    }
}
#[allow(dead_code)]
mod join {
    include!("std_path_join_ascii.rs");
    #[test]
    fn empty_parts_unicode_and_parent_components() {
        assert_eq!(join(&["/", "", "child", "..", ""]), "/");
        assert_eq!(join(&["", "", "child", "..", ""]), ".");
        assert_eq!(join(&["é😊", "文", "..", "résumé/"]), "é😊/résumé/");
        assert_eq!(
            join(&["/tmp", "/reports", "archive.txt"]),
            "/tmp/reports/archive.txt"
        );
    }
}
#[allow(dead_code)]
mod relative {
    include!("std_path_relative.rs");
    #[test]
    fn common_prefix_and_unicode_components() {
        assert_eq!(relative("reports/é😊", "reports/é😊/2026"), "2026");
        assert_eq!(
            relative("reports/é😊", "reports/archive.txt"),
            std::path::Path::new("..")
                .join("archive.txt")
                .to_str()
                .unwrap()
        );
        assert_eq!(relative("reports/é😊", "reports/é😊"), "");
    }
}
#[allow(dead_code)]
mod platform {
    include!("std_os_platform.rs");
    #[test]
    fn target_os_alias() {
        let expected = if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(windows) {
            "win32"
        } else {
            std::env::consts::OS
        };
        assert_eq!(identity(), expected);
    }
}
#[allow(dead_code)]
mod arch {
    include!("std_os_arch.rs");
    #[test]
    fn target_arch_alias() {
        let expected = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "x86") {
            "ia32"
        } else {
            std::env::consts::ARCH
        };
        assert_eq!(identity(), expected);
    }
}
#[allow(dead_code)]
mod endianness {
    include!("std_os_endianness.rs");
    #[test]
    fn byte_order_matches_safe_native_bytes() {
        assert_eq!(
            identity(),
            if 1_u16.to_ne_bytes()[0] == 1 {
                "LE"
            } else {
                "BE"
            }
        );
    }
}
