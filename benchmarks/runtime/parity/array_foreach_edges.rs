mod regular {
    include!("array_for_each_int.rs");
    pub fn run(n:i64,seed:i64)->i64{function0(n,seed)}
    #[test]
    fn empty_source_skips_callbacks_and_single_element_keeps_trace() {
        assert_eq!(function0(0,7),7);
        assert_eq!(function0(1,0),0);
        assert_eq!(function0(1,1),134);
    }
}
mod numbers {
    include!("array_for_each_number.rs");
    #[test]
    fn retains_fractional_trace_and_visit_values() {
        assert_eq!(function0(0,7),1.75);
        assert_eq!(function0(1,1),33.5);
    }
}
mod strings {
    include!("array_for_each_string.rs");
    #[test]
    fn combines_overlapping_input_unicode_and_nul_weights() {
        assert_eq!(string_weight("é","é"),15);
        assert_eq!(string_weight("a\0b","a\0b"),9);
        assert_eq!(string_weight("e\u{301}","x"),8);
        assert_eq!(string_weight("😀","😀"),17);
    }
}
mod recursive {
    include!("array_for_each_recursive_record.rs");
    #[test]
    fn recursive_weight_includes_every_owned_descendant() {
        let value=Node{n:1,text:String::Borrowed("x"),children:vec![Node{n:2,text:String::Borrowed("é"),children:vec![]},Node{n:3,text:String::Borrowed("a\0b"),children:vec![]}]};
        assert_eq!(node_weight(&value,"x"),28);
    }
}
mod discard_string {
    include!("array_for_each_discard_string.rs");
    pub fn run(n:i64,seed:i64)->i64{function0(n,seed)}
}
mod discard_record {
    include!("array_for_each_discard_record.rs");
    pub fn run(n:i64,seed:i64)->i64{function0(n,seed)}
}
mod discard_nested {
    include!("array_for_each_discard_nested.rs");
    pub fn run(n:i64,seed:i64)->i64{function0(n,seed)}
}
mod discard_callable {
    include!("array_for_each_discard_callable.rs");
    pub fn run(n:i64,seed:i64)->i64{function0(n,seed)}
    #[test]
    fn callable_owns_the_constructed_text_until_consumed_or_dropped() {
        let callback=make_callable(2);
        assert_eq!(callback(),"allocatedallocatedallocated");
        drop(make_callable(1));
    }
}
#[test]
fn discarding_each_return_type_preserves_all_observed_callback_effects() {
    for (n,seed) in [(0,7),(1,0),(1,1),(17,3),(100,19)] {
        let expected=regular::run(n,seed);
        assert_eq!(discard_string::run(n,seed),expected);
        assert_eq!(discard_record::run(n,seed),expected);
        assert_eq!(discard_nested::run(n,seed),expected);
        assert_eq!(discard_callable::run(n,seed),expected);
    }
}
