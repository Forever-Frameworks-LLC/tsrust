#[allow(dead_code)]
mod recursive {
    include!("recursive_result_inference.rs");
    #[test]
    fn actual_recursion_keeps_base_cases_and_captured_push_order() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(5), 120);
        assert!(parity(0));
        assert!(!parity(5));
        assert_eq!(label(-1), "");
        assert_eq!(label(3), "xxx");
        let mut steps = Vec::new();
        assert_eq!(depth(3, &mut steps), 3);
        assert_eq!(steps, vec![3, 2, 1, 0]);
        assert_eq!(
            function0(1, 4, &String::Borrowed("recursive")).0[0].0,
            vec![120, 0, 5, 5, 6, 5]
        );
        assert_eq!(
            function0(5, 4, &String::Borrowed("recursive")).0[4].0,
            vec![362880, 0, 9, 9, 10, 34]
        );
    }
}
#[allow(dead_code)]
mod environment {
    include!("std_env_get.rs");
    #[test]
    fn real_environment_preserves_present_empty_and_missing() {
        assert_eq!(
            read("TSRUST_BENCH_ENV_TEXT").unwrap().as_deref(),
            Some("héllo 🦀")
        );
        assert_eq!(read("TSRUST_BENCH_ENV_EMPTY").unwrap().as_deref(), Some(""));
        assert!(read("TSRUST_BENCH_ENV_MISSING").unwrap().is_none());
        let values = function0(3, 0);
        assert_eq!(
            values.0.iter().map(|v| v.as_ref()).collect::<Vec<_>>(),
            vec!["héllo 🦀", "", "<missing>"]
        );
    }
}
#[allow(dead_code)]
mod clock {
    include!("std_time_monotonic_now.rs");
    #[test]
    fn clock_reads_remain_finite_and_monotonic_with_one_epoch() {
        let mut previous = now();
        for _ in 0..1000 {
            let current = now();
            assert!(current.is_finite() && current >= previous && current >= 0.0);
            previous = current;
        }
        assert!(function0(0, 7).0.is_empty());
        assert!(function0(1000, 7).0.into_iter().all(|value| value));
    }
}
#[allow(dead_code)]
mod imported {
    include!("native_json_import.rs");
    #[test]
    fn compiled_settings_match_the_pinned_input_and_are_actually_scanned() {
        assert_eq!(
            include_str!("../workloads/typed-data/settings.json"),
            "{\"base\":5,\"values\":[3,11,23]}\n"
        );
        assert_eq!(SETTINGS.base, 5);
        assert_eq!(SETTINGS.values, [3, 11, 23]);
        assert_eq!(function0(0, 7), 7);
        assert_eq!(function0(2, 7), 91);
    }
}
#[allow(dead_code)]
mod state {
    include!("native_module_state.rs");
    #[test]
    fn exclusive_state_retains_all_calls_without_reinitializing() {
        let mut completed = 0;
        assert_eq!(function0(2, 7, &mut completed), 13);
        assert_eq!(completed, 2);
        assert_eq!(function0(3, 1, &mut completed), 10);
        assert_eq!(completed, 5);
        assert_eq!(function0(0, 9, &mut completed), 9);
        assert_eq!(completed, 5);
        let mut maximum = i64::MAX;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| increment(&mut maximum)))
                .is_err()
        );
        assert_eq!(maximum, i64::MAX);
    }
}
