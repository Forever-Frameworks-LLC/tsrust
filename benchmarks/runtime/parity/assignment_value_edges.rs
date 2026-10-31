// Result-versus-target witnesses for independent Rust update references.
macro_rules! integer {
    ($module:ident,$source:literal,$expected:expr,$overflow:expr)=>{mod $module{
        include!($source);
        #[test]fn checks_observed_target_and_overflow(){
            assert_eq!(update(-1),$expected);
            assert!(std::panic::catch_unwind(||update($overflow)).is_err());
        }
    }};
}
integer!(prefix_inc,"update_int_prefix_increment.rs",(0,0),i64::MAX);
integer!(postfix_inc,"update_int_postfix_increment.rs",(-1,0),i64::MAX);
integer!(prefix_dec,"update_int_prefix_decrement.rs",(-2,-2),i64::MIN);
integer!(postfix_dec,"update_int_postfix_decrement.rs",(-1,-2),i64::MIN);
mod number_increment {
    include!("update_number_postfix_increment.rs");
    #[test]fn preserves_old_zero_sign_and_precision_limit(){
        let(observed,target)=update(-0.0);
        assert_eq!(observed.to_bits(),(-0.0_f64).to_bits());assert_eq!(target,1.0);
        assert_eq!(update(9007199254740992.0),(9007199254740992.0,9007199254740992.0));
    }
}
mod number_decrement {
    include!("update_number_postfix_decrement.rs");
    #[test]fn preserves_observed_negative_zero(){
        let(observed,target)=update(-0.0);
        assert_eq!(observed.to_bits(),(-0.0_f64).to_bits());assert_eq!(target,-1.0);
    }
}
