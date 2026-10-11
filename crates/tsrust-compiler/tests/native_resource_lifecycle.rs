mod support;

#[test]
fn private_resource_backend_releases_once_at_o0_and_o3() {
    let directory = support::TempDirectory::new();
    let source = directory.0.join("resource.rs");
    std::fs::write(&source, include_str!("../src/stdlib/resource.rs")).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("resource-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024", "--test"])
            .arg(format!("-Copt-level={level}")).arg(&source).arg("-o").arg(&binary), 60));
        support::success(&support::run(&mut std::process::Command::new(binary), 20));
    }
}

#[test]
fn private_resource_backend_cannot_be_reused_cloned_compared_or_hashed() {
    let directory = support::TempDirectory::new();
    let runtime = include_str!("../src/stdlib/resource.rs").replace("//!", "//");
    let backend = "struct Handle; impl Backend for Handle { type Error = (); fn close(self)->Result<(),()>{Ok(())} }";
    for (body, error) in [
        ("let owner=Resource::new(Handle); owner.close().unwrap(); owner.close().unwrap();", "use of moved value"),
        ("let owner=Resource::new(Handle); owner.close().unwrap(); owner.get();", "borrow of moved value"),
        ("let owner=Resource::new(Handle); let _copy=owner.clone();", "no method named `clone`"),
        ("let left=Resource::new(Handle);let right=Resource::new(Handle);let _equal=left==right;", "cannot be applied"),
        ("fn hash<T:std::hash::Hash>(_:&T){} let owner=Resource::new(Handle);hash(&owner);", "Hash"),
    ] {
        let source = directory.0.join("reject.rs");
        std::fs::write(&source, format!("{runtime}\n{backend}\nfn main(){{{body}}}")).unwrap();
        let output = support::run(support::rustc().args(["--edition=2024", "--emit=metadata"])
            .arg(&source).arg("-o").arg(directory.0.join("reject.rmeta")), 60);
        assert!(!output.status.success(), "unexpected acceptance: {body}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(error), "{}", String::from_utf8_lossy(&output.stderr));
    }
}
