#[allow(dead_code)]
mod read {
    include!("optional_parameter_read_callback.rs");
    #[test]
    fn omitted_and_undefined_callbacks_leave_trace_untouched() {
        let result = function0(4, 0, &"x".into());
        assert!(result.0[0].0.is_empty());
        assert!(result.0[2].0.is_empty());
        assert_eq!(result.0[1].0, vec![0, 0]);
        assert_eq!(result.0[3].0, vec![0, 0]);
        assert_eq!(result.0[4].0, vec![29]);
        assert_eq!(result.0[6].0, vec![42]);
    }
}
#[allow(dead_code)]
mod mutable {
    include!("optional_parameter_mutate_callback.rs");
    #[test]
    fn mutable_callback_state_advances_only_on_present_calls() {
        let result = function0(8, 0, &"x".into());
        assert_eq!(result.0[4].0, vec![27]);
        assert_eq!(result.0[6].0, vec![41]);
        assert!(result.0[8].0.is_empty());
        assert!(result.0[10].0.is_empty());
        assert_eq!(result.0[12].0, vec![81]);
        assert_eq!(result.0[14].0, vec![95]);
    }
}
#[allow(dead_code)]
mod once {
    include!("optional_parameter_once_callback.rs");
    #[test]
    fn once_callback_moves_owned_text_into_retained_result() {
        let input: String = "é🦀".into();
        let result = function0(3, 0, &input);
        drop(input);
        assert!(result[0].0.is_empty());
        assert!(result[2].0.is_empty());
        assert_eq!(result[4].0[0], "é🦀é🦀é🦀:present:26");
        assert_eq!(result[5].0, vec!["1", "0"]);
        let mut trace = Trace::default();
        let text = "owned".to_owned();
        let action = move |trace: &mut Trace| {
            trace.calls = add(trace.calls, 1);
            text
        };
        assert_eq!(apply(&mut trace, Some(action)), vec!["owned"]);
        assert_eq!(trace.calls, 1);
    }
}
#[allow(dead_code)]
mod expressions {
    include!("function_expressions.rs");
    #[test]
    fn defaults_rest_and_sorted_owned_results_survive_input_drop() {
        assert_eq!(join(None, vec![]), "a0");
        assert_eq!(join(Some("".to_owned()), vec!["x".to_owned()]), "1");
        let input: String = "é🦀".into();
        let result = function0(1, 0, &input);
        drop(input);
        assert_eq!(result[0].0, vec!["0", "5", "0", "a0", "b2"]);
    }
}
#[allow(dead_code)]
mod callable {
    include!("callable_expressions.rs");
    #[test]
    fn nested_calls_and_factory_argument_evaluation_preserve_order() {
        assert_eq!(chain(7)(2)(3), 12);
        let mut trace = vec![];
        let action = factory(&mut trace, 7);
        let value = mark(&mut trace, 5);
        assert_eq!(action(value), 12);
        assert_eq!(trace, vec![1, 2]);
        let result = function0(1, 0, &"abc".into());
        assert_eq!(
            result[0].0,
            vec!["3", "5", "0", "4", "2", "7", "5", "1,2", "abc0"]
        );
    }
}
#[allow(dead_code)]
mod named {
    include!("named_function_expressions.rs");
    #[test]
    fn recursive_captures_and_generic_owned_selection_preserve_results() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(5), 120);
        let mut total = 0;
        down(3, &[2, 3], &mut total);
        assert_eq!(total, 12);
        assert_eq!(make(7)(0), 7);
        assert_eq!(make(7)(3), 10);
        assert_eq!(
            choose("left".to_owned(), "right".to_owned(), false),
            "right"
        );
        assert_eq!(
            function0(1, 0, &"x".into()).0[0].0,
            vec![1, 9, 3, 4, 3, 1, 1]
        );
    }
}
