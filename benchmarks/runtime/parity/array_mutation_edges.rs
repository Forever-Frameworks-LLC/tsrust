mod overlap {
    include!("array_copy_within_nested.rs");
    #[test]
    fn copies_owned_overlaps_in_both_directions() {
        let mut values=vec![vec![0],vec![1],vec![2],vec![3],vec![4],vec![5]];
        copy_owned(&mut values,1,0,5);
        assert_eq!(values,vec![vec![0],vec![0],vec![1],vec![2],vec![3],vec![4]]);
        values[1][0]=99;
        assert_eq!(values[0][0],0);
        let mut values=vec![vec![0],vec![1],vec![2],vec![3],vec![4],vec![5]];
        copy_owned(&mut values,0,1,6);
        assert_eq!(values,vec![vec![1],vec![2],vec![3],vec![4],vec![5],vec![5]]);
    }
    #[test]
    fn clips_at_receiver_end_and_handles_empty_ranges() {
        let mut values=vec![vec![0],vec![1],vec![2]];
        copy_owned(&mut values,2,0,3);
        assert_eq!(values,vec![vec![0],vec![1],vec![0]]);
        copy_owned(&mut values,3,0,3);
        copy_owned(&mut values,0,2,1);
        assert_eq!(values,vec![vec![0],vec![1],vec![0]]);
    }
}
mod records {
    include!("array_fill_record.rs");
    #[test]
    fn fill_results_have_independent_owned_strings_and_vectors() {
        let input=String::Owned("é😀".to_owned());
        let mut values=function0(1,0,&input);
        drop(input);
        values[0].field1.to_mut().push('x');
        values[0].field2[0]=999;
        assert_eq!(values[1].field1,"é😀");
        assert_eq!(values[1].field2[0],11);
        assert_eq!(values.len(),6);
    }
}
mod recursive {
    include!("array_copy_within_recursive.rs");
    #[test]
    fn overlapping_recursive_copies_preserve_independent_descendants() {
        let input=String::Borrowed("x");
        let mut values=function0(1,0,&input);
        assert_eq!(values.iter().map(|v|v.field0).collect::<Vec<_>>(),vec![0,0,1,2,3,4]);
        values[1].field2[0].field0=99;
        values[1].field2[1].field2[0].field1.to_mut().push('x');
        assert_eq!(values[0].field2[0].field0,0);
        assert_eq!(values[0].field2[1].field2[0].field1,"leafé");
    }
}
mod spliced {
    include!("array_to_spliced_int.rs");
    #[test]
    fn preserves_omitted_zero_negative_and_oversized_delete_counts() {
        let rows=function0(10,0);
        assert_eq!(rows.iter().map(Vec::len).collect::<Vec<_>>(),vec![6,0,6,6,4,6,6,8,6,2]);
        assert_eq!(rows[7],vec![7,8,9,10,11,18,20,12]);
        assert_eq!(rows[8],vec![19,21,25,11,12,13]);
        assert_eq!(rows[9],vec![20,22]);
    }
}
