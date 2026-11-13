//! Native decoder and transport controls run without loading model weights.
mod support;

#[test]
fn incremental_model_output_and_backpressure() {
    let source = include_str!("../src/stdlib/ai/llama_admission.rs");
    let start = source.find("const MAX_OUTPUT_TOKENS:").unwrap();
    let end = source.find("fn request_tokens(").unwrap();
    let directory = support::TempDirectory::new();
    let input = directory.0.join("output.rs");
    let executable = directory.0.join(format!("output{}", std::env::consts::EXE_SUFFIX));
    let implementation = format!("use std::sync::mpsc::{{self, TrySendError}};\n{}\n{}", &source[start..end], TESTS);
    std::fs::write(&input, implementation).unwrap();
    let mut build = std::process::Command::new("rustup");
    build.args(["run", tsrust_compiler::RUST_BACKEND_VERSION, "rustc", "--edition=2024", "--test"])
        .arg(&input).arg("-o").arg(&executable);
    support::success(&support::run(&mut build, 60));
    support::success(&support::run(&mut std::process::Command::new(executable), 30));
}

const TESTS: &str = r#"
#[cfg(test)]
mod output_tests {
    use super::*;

    #[test]
    fn byte_split_unicode_preserves_every_valid_prefix() {
        let expected = "Hello 日本語 🦀\n";
        let mut decoder = OutputDecoder::default();
        for byte in expected.as_bytes() {
            decoder.push(&[*byte]).unwrap();
            assert!(expected.starts_with(&decoder.text));
            assert!(decoder.pending.len() <= 3);
        }
        assert_eq!(decoder.finish().unwrap(), expected);
    }

    #[test]
    fn malformed_truncated_and_oversized_output_fail() {
        for bytes in [&[0xff][..], &[0xc0, 0x80][..], &[0xed, 0xa0, 0x80][..]] {
            assert!(OutputDecoder::default().push(bytes).is_err());
        }
        let mut decoder = OutputDecoder::default();
        decoder.push(&[0xf0, 0x9f]).unwrap();
        assert!(decoder.finish().is_err());
        let mut decoder = OutputDecoder::default();
        decoder.push(&vec![b'a'; MAX_OUTPUT_BYTES]).unwrap();
        assert!(decoder.push(b"b").is_err());
        assert_eq!(decoder.finish().unwrap().len(), MAX_OUTPUT_BYTES);
        assert_eq!(OutputDecoder::default().finish().unwrap(), "");
    }

    #[test]
    fn full_output_queue_observes_cancellation_and_deadline() {
        let (sender, receiver) = mpsc::sync_channel(1);
        publish_prefix(&sender, b"first", &|| Ok(())).unwrap();
        for reason in ["cancelled", "deadline"] {
            let polls = std::cell::Cell::new(0);
            let check = || {
                polls.set(polls.get() + 1);
                if polls.get() == 3 { Err(reason.into()) } else { Ok(()) }
            };
            assert_eq!(publish_prefix(&sender, b"blocked", &check).unwrap_err(), reason);
        }
        assert_eq!(receiver.recv().unwrap(), "first");
        drop(receiver);
        assert_eq!(publish_prefix(&sender, b"gone", &|| Ok(())).unwrap_err(), "cancelled");
    }

    #[test]
    fn response_budget_bounds() {
        for budget in [1, 32, 1024, 4096] { validate_budget(budget).unwrap(); }
        for budget in [-1, 0, 4097, i64::MAX] { assert!(validate_budget(budget).is_err()); }
    }
}
"#;
