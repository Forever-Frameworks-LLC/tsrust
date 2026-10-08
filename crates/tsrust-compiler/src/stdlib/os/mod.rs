pub(crate) mod bindings;

// These predicates are evaluated by rustc for the emitted executable, never by
// the compiler host. Each referenced export has its own unsupported-target check.
const PLATFORM: &[(&str, &str)] = &[
    ("target_os = \"macos\"", "darwin"),
    ("target_os = \"windows\"", "win32"),
    ("target_os = \"linux\"", "linux"),
    ("target_os = \"android\"", "android"),
    ("target_os = \"aix\"", "aix"),
    ("target_os = \"freebsd\"", "freebsd"),
    ("target_os = \"openbsd\"", "openbsd"),
    ("any(target_os = \"solaris\", target_os = \"illumos\")", "sunos"),
];
const ARCH: &[(&str, &str)] = &[
    ("target_arch = \"x86\"", "ia32"),
    ("target_arch = \"x86_64\"", "x64"),
    ("target_arch = \"arm\"", "arm"),
    ("target_arch = \"aarch64\"", "arm64"),
    ("target_arch = \"loongarch64\"", "loong64"),
    ("target_arch = \"powerpc64\"", "ppc64"),
    ("target_arch = \"riscv64\"", "riscv64"),
    ("target_arch = \"s390x\"", "s390x"),
    ("all(target_arch = \"mips\", target_endian = \"big\")", "mips"),
    ("all(target_arch = \"mips\", target_endian = \"little\")", "mipsel"),
];
pub(crate) fn bridge(name: &str) -> std::string::String {
    let mappings = match name { "platform" => PLATFORM, "arch" => ARCH, _ => unreachable!() };
    let predicates = mappings.iter().map(|(predicate, _)| *predicate).collect::<Vec<_>>().join(", ");
    let mut output = format!("#[cfg(not(any({predicates})))]\ncompile_error!(\"std:os.{name}(): unsupported executable compilation target\");\n");
    output.push_str(&format!("fn tsrust_std_os_{name}() -> String {{\n"));
    for (predicate, value) in mappings {
        output.push_str(&format!("#[cfg({predicate})] {{ return String::Borrowed({value:?}); }}\n"));
    }
    output.push_str("}\n");
    output
}

#[cfg(test)]
mod tests {
    use super::bridge;
    use std::process::Command;
    #[test]
    fn executable_target_mappings_follow_rust_target_metadata() {
        // Independent Node names and Rust target triples, not the emission table.
        let cases = [
            ("aarch64-apple-darwin", "darwin", "arm64"),
            ("x86_64-pc-windows-msvc", "win32", "x64"),
            ("i686-unknown-linux-gnu", "linux", "ia32"),
            ("armv7-unknown-linux-gnueabihf", "linux", "arm"),
            ("aarch64-linux-android", "android", "arm64"),
            ("powerpc64-ibm-aix", "aix", "ppc64"),
            ("x86_64-unknown-freebsd", "freebsd", "x64"),
            ("x86_64-unknown-openbsd", "openbsd", "x64"),
            ("x86_64-pc-solaris", "sunos", "x64"),
            ("x86_64-unknown-illumos", "sunos", "x64"),
            ("loongarch64-unknown-linux-gnu", "linux", "loong64"),
            ("riscv64gc-unknown-linux-gnu", "linux", "riscv64"),
            ("s390x-unknown-linux-gnu", "linux", "s390x"),
            ("mips-unknown-linux-gnu", "linux", "mips"),
            ("mipsel-unknown-linux-gnu", "linux", "mipsel"),
            ("powerpc64le-unknown-linux-gnu", "linux", "ppc64"),
        ];
        let directory = std::env::temp_dir().join(format!("tsrust-os-mappings-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let mut source = std::string::String::new();
        let mut compiler = Command::new("rustup");
        compiler.args(["run", crate::RUST_BACKEND_VERSION, "rustc", "--edition=2024", "-Awarnings"]);
        for (index, (target, platform, arch)) in cases.iter().enumerate() {
            let metadata = Command::new("rustup").args(["run", crate::RUST_BACKEND_VERSION, "rustc", "--print", "cfg", "--target", target]).output().unwrap();
            assert!(metadata.status.success(), "{target}: {}", std::string::String::from_utf8_lossy(&metadata.stderr));
            let metadata = std::string::String::from_utf8(metadata.stdout).unwrap();
            let rename = |text: std::string::String| text.replace("target_os", &format!("os_case_{index}")).replace("target_arch", &format!("arch_case_{index}")).replace("target_endian", &format!("endian_case_{index}"));
            for line in metadata.lines().filter(|line| ["target_os=", "target_arch=", "target_endian="].iter().any(|prefix|line.starts_with(prefix))) {
                compiler.arg("--cfg").arg(rename(line.into()));
            }
            source.push_str(&format!("mod case_{index} {{ enum String {{ Borrowed(&'static str) }} {} {} pub fn verify() {{ let String::Borrowed(platform) = tsrust_std_os_platform(); let String::Borrowed(arch) = tsrust_std_os_arch(); assert_eq!(platform,{platform:?}); assert_eq!(arch,{arch:?}); }} }}\n",rename(bridge("platform")),rename(bridge("arch"))));
        }
        source.push_str("fn main() {");
        for index in 0..cases.len() { source.push_str(&format!("case_{index}::verify();")); }
        source.push_str("}");
        let input=directory.join("mapping.rs");let binary=directory.join("mapping");
        std::fs::write(&input,source).unwrap();
        let output=compiler.arg(&input).arg("-o").arg(&binary).output().unwrap();
        assert!(output.status.success(),"{}",std::string::String::from_utf8_lossy(&output.stderr));
        assert!(Command::new(&binary).status().unwrap().success());
        for name in ["platform", "arch"] {
            // No supported metadata selects a branch: native compilation must fail.
            let source=format!("enum String {{ Borrowed(&'static str) }} {} fn main(){{}}",bridge(name).replace("target_os", "unsupported_os").replace("target_arch", "unsupported_arch").replace("target_endian", "unsupported_endian"));
            std::fs::write(&input,source).unwrap();
            let output=Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024","-Awarnings"]).arg(&input).arg("-o").arg(&binary).output().unwrap();
            assert!(!output.status.success());
            assert!(std::string::String::from_utf8_lossy(&output.stderr).contains(&format!("std:os.{name}(): unsupported executable compilation target")));
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}
