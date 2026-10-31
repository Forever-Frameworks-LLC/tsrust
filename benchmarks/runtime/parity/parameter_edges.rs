mod nullable {
    include!("default_parameter_boolean_value.rs");
    #[test]
    fn explicit_null_and_false_do_not_evaluate_defaults() {
        let mut trace=Trace{calls:0,defaults:0};
        assert_eq!(apply(&mut trace,0,Some(None)),None);
        assert_eq!(trace.defaults,0);
        assert_eq!(apply(&mut trace,0,None),Some(true));
        assert_eq!(apply(&mut trace,0,Some(Some(false))),Some(false));
        assert_eq!((trace.calls,trace.defaults),(3,1));
    }
}
mod arrays {
    include!("default_parameter_array_value.rs");
    #[test]
    fn supplied_empty_array_remains_empty_without_fallback() {
        let mut trace=Trace{calls:0,defaults:0};
        assert!(apply(&mut trace,3,Some(vec![])).is_empty());
        assert_eq!(trace.defaults,0);
        assert_eq!(apply(&mut trace,3,None),vec![3,4]);
        assert_eq!((trace.calls,trace.defaults),(2,1));
    }
}
mod dependent {
    include!("default_parameter_dependent_positions.rs");
    #[test]
    fn dependent_default_uses_the_resolved_first_argument() {
        let mut trace=Trace{calls:0,defaults:0};
        assert_eq!(apply(&mut trace,2,None,3,None),vec![12,3,16,28]);
        assert_eq!(trace.defaults,2);
        assert_eq!(apply(&mut trace,2,Some(0),3,None),vec![0,3,4,4]);
        assert_eq!(trace.defaults,3);
    }
}
mod reusable {
    include!("default_parameter_owned_reusable_callback.rs");
    #[test]
    fn reusable_default_capture_changes_between_both_invocations() {
        let input=String::Borrowed("");
        let defaulted=function0(1,0,&input);
        assert_eq!(defaulted.0[0].0,vec![101,103]);
        assert_eq!(defaulted.0[1].0,vec![2,1]);
        let supplied=function0(1,2,&input);
        assert_eq!(supplied.0[0].0,vec![5,7]);
        assert_eq!(supplied.0[1].0,vec![2,0]);
    }
}
mod once {
    include!("default_parameter_owned_once_callback.rs");
    #[test]
    fn once_defaults_move_owned_text_into_retained_results() {
        let input=String::Owned("x".to_owned());
        let rows=function0(1,0,&input);
        drop(input);
        assert_eq!(rows[0].0[0],"x:default:0");
        assert_eq!(rows[1].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["1","1"]);
    }
}
mod stored {
    include!("default_parameter_stored_family.rs");
    #[test]
    fn stored_function_identity_selects_the_correct_default() {
        let rows=function0(2,0,&String::Borrowed(""));
        assert_eq!(rows.0[0].0,vec![11]);
        assert_eq!(rows.0[1].0,vec![1,1]);
        assert_eq!(rows.0[2].0,vec![102]);
        assert_eq!(rows.0[3].0,vec![1,1]);
    }
}
mod prefix {
    include!("rest_parameter_default_prefix.rs");
    #[test]
    fn default_prefix_keeps_tail_argument_counts_and_order() {
        let rows=function0(3,0,&String::Borrowed(""));
        assert_eq!(rows.0[0].0,vec![10]);
        assert_eq!(rows.0[1].0,vec![0,1]);
        assert_eq!(rows.0[2].0,vec![10,13]);
        assert_eq!(rows.0[3].0,vec![1,1]);
        assert_eq!(rows.0[4].0,vec![26,26,27,28,29,30]);
        assert_eq!(rows.0[5].0,vec![5,0]);
    }
}
mod read {
    include!("rest_parameter_read_callback.rs");
    #[test]
    fn borrowed_read_callback_keeps_its_capture_unchanged() {
        let rows=function0(2,2,&String::Borrowed(""));
        assert_eq!(rows.0[0].0,vec![2,4,5,6,7,8]);
        assert_eq!(rows.0[2].0,vec![2]);
    }
}
mod mutable {
    include!("rest_parameter_mutate_callback.rs");
    #[test]
    fn borrowed_mutable_callback_updates_capture_across_calls() {
        let rows=function0(2,2,&String::Borrowed(""));
        assert_eq!(rows.0[0].0,vec![3,5,6,7,8,9]);
        assert_eq!(rows.0[2].0,vec![4]);
    }
}
mod owned_rest {
    include!("rest_parameter_owned_once_callback.rs");
    #[test]
    fn owned_rest_callback_transfers_capture_and_tail_results() {
        let input=String::Owned("x".to_owned());
        let rows=function0(1,1,&input);
        drop(input);
        assert_eq!(rows[0].0.iter().map(|s|s.as_ref()).collect::<Vec<_>>(),vec!["xx:1","1"]);
        assert_eq!(rows[1].0[0],"1");
    }
}
