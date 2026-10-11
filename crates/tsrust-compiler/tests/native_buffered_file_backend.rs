mod support;
#[test]
fn buffered_file_backend_lifecycle_and_partial_io_at_o0_o3() {
 let directory=support::TempDirectory::new();
 let source=format!("mod resource {{ {} }}\nmod fs {{ {} }}\nmod buffered_fs {{ {} }}",include_str!("../src/stdlib/resource.rs"),include_str!("../src/stdlib/fs.rs"),include_str!("../src/stdlib/buffered_fs.rs")).replace("//!","//");
 let path=directory.0.join("buffered.rs");std::fs::write(&path,source).unwrap();
 for level in ["0","3"] {
  let binary=directory.0.join(format!("buffered-{level}"));
  support::success(&support::run(support::rustc().args(["--edition=2024","--test"]).arg(format!("-Copt-level={level}")).arg(&path).arg("-o").arg(&binary),60));
  support::success(&support::run(std::process::Command::new(binary).arg("--test-threads=1"),20));
 }
}
