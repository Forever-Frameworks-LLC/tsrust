// Append the unchanged slice-observation.rs (pruned only by existing adapters).
// Rename the generated main, then use this main; no workload body rewriting.
fn main() {
    let flat = function1();
    assert_eq!(slice_exact(&flat, false), "t:[b0,b1]");
    assert_eq!(slice_checksum(&flat), 580007604);
    let nested = function0(0, 0);
    assert_eq!(slice_exact(&nested, false), "t:[[b0,b1],[],[b1,b0]]");
    assert_eq!(slice_checksum(&nested), 945527745);
    println!("old-Boolean-flat:new-Boolean-rank2:accepted");
}
