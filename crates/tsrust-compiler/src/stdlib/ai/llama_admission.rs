// Candidate native engine primitive. Prompts, workers and all agent logic stay in Tsrust.
use llama_cpp_2::{context::params::LlamaContextParams, llama_backend::LlamaBackend,
    llama_batch::LlamaBatch, model::{LlamaModel, params::LlamaModelParams}, sampling::LlamaSampler};
use std::num::NonZeroU32;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, RecvTimeoutError, TrySendError}};
use std::thread::JoinHandle;
use tokenizers::Tokenizer;

/// Buffered exploratory generation, not a persistent streaming session.
pub struct Generation {
    pub text: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub stop: String,
}

/// One resident model. Generation borrows it exclusively; fields and cloning are unavailable.
#[tsrust_resource]
pub struct QwenModel {
    // Rust drops fields in declaration order: the model must go before its backend.
    model: LlamaModel,
    tokenizer: Tokenizer,
    backend: LlamaBackend,
}

fn check_cancel(task: &TaskContext) -> Result<(), String> {
    if task.cancellation_requested() { Err("cancelled".into()) } else { Ok(()) }
}

const MAX_OUTPUT_TOKENS: i64 = 4096;
// A 16 KiB UTF-8 prefix fits the 128 KiB JSON frame even with control-byte escaping.
const MAX_OUTPUT_BYTES: usize = 16 * 1024;

#[derive(Default)]
struct OutputDecoder {
    text: String,
    pending: Vec<u8>,
}

impl OutputDecoder {
    fn push(&mut self, bytes: &[u8]) -> Result<bool, String> {
        if self.text.len() + self.pending.len() + bytes.len() > MAX_OUTPUT_BYTES {
            return Err("generated output exceeds 16384 UTF-8 bytes".into());
        }
        self.pending.extend_from_slice(bytes);
        let valid = match std::str::from_utf8(&self.pending) {
            Ok(text) => text.len(),
            Err(error) if error.error_len().is_none() => error.valid_up_to(),
            Err(_) => return Err("generated text is not valid UTF-8".into()),
        };
        self.text.push_str(std::str::from_utf8(&self.pending[..valid]).expect("validated prefix"));
        self.pending.drain(..valid);
        Ok(valid > 0)
    }

    fn finish(self) -> Result<String, String> {
        if !self.pending.is_empty() { return Err("generated text ends with incomplete UTF-8".into()); }
        Ok(self.text)
    }
}

fn validate_budget(max_tokens: i64) -> Result<(), String> {
    if !(1..=MAX_OUTPUT_TOKENS).contains(&max_tokens) {
        return Err("max_tokens must be 1..=4096".into());
    }
    Ok(())
}

fn publish_prefix(sender: &mpsc::SyncSender<String>, bytes: &[u8], check: &dyn Fn() -> Result<(), String>) -> Result<(), String> {
    let mut text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?.to_owned();
    loop {
        check()?;
        match sender.try_send(text) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Full(value)) => { text = value; std::thread::sleep(std::time::Duration::from_millis(1)); },
            Err(TrySendError::Disconnected(_)) => return Err("cancelled".into()),
        }
    }
}

fn request_tokens(tokenizer: &Tokenizer, prompt: &str, max_tokens: i64) -> Result<Vec<u32>, String> {
    validate_budget(max_tokens)?;
    if prompt.len() > 65536 { return Err("admission prompt exceeds 65536 UTF-8 bytes".into()); }
    let expected = tokenizer.encode(prompt, true).map_err(|error| error.to_string())?;
    if expected.get_ids().is_empty() || expected.get_ids().len() > 2048 { return Err("admission context must be 1..=2048 tokens".into()); }
    Ok(expected.get_ids().to_vec())
}

fn load_model(task: &TaskContext, model_path: &str, tokenizer: Tokenizer) -> Result<QwenModel, String> {
    check_cancel(task)?;
    let backend = LlamaBackend::init().map_err(|error| error.to_string())?;
    if !backend.supports_gpu_offload() || !backend.supports_mmap() { return Err("Metal offload and memory mapping are required for this admission".into()); }
    let params = LlamaModelParams::default().with_use_mmap(true).with_n_gpu_layers(1000);
    let model = LlamaModel::load_from_file(&backend, model_path, &params).map_err(|error| error.to_string())?;
    check_cancel(task)?;
    Ok(QwenModel { model, tokenizer, backend })
}

/// Load one verified Qwen GGUF and tokenizer, using Metal offload and memory mapping.
/// Keep this owner inside its Tsrust worker and borrow it for repeated generation.
/// At most one backend owner may be live in this process. Drop unloads the model.
/// Cancellation is checked around loading; native model loading itself is not interruptible.
pub fn qwen_load(task: &TaskContext, model_path: &str, tokenizer_path: &str) -> Result<QwenModel, String> {
    check_cancel(task)?;
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|error| error.to_string())?;
    load_model(task, model_path, tokenizer)
}

/// Generate from an already loaded model without loading the model or tokenizer again.
/// Each call has a fresh context, strict tokenizer agreement and no conversation state.
/// Exclusive borrowing serializes context creation, which the pinned SDK cannot safely parallelize.
/// Limits: 1..=2048 prompt tokens, 65536 prompt bytes, 1..=4096 output tokens,
/// 16384 output bytes and 180 seconds. Prompt plus budget must fit the trained context.
/// Returns buffered text/counts/stop reason; cancellation returns an error without partial text.
pub fn qwen_generate_loaded(task: &TaskContext, model: &mut QwenModel, prompt: &str, max_tokens: i64) -> Result<Generation, String> {
    check_cancel(task)?;
    let expected = request_tokens(&model.tokenizer, prompt, max_tokens)?;
    generate(&||check_cancel(task), model, prompt, max_tokens, &expected, None)
}

/// Load, generate and unload one model. Prefer qwen_load/qwen_generate_loaded for repeated requests.
/// Validates request limits before loading the model; all owners drop before return.
pub fn qwen_generate(task: &TaskContext, model_path: &str, tokenizer_path: &str, prompt: &str, max_tokens: i64) -> Result<Generation, String> {
    check_cancel(task)?;
    // Reject cheap invalid requests before touching either artifact, preserving the original contract.
    validate_budget(max_tokens)?;
    if prompt.len() > 65536 { return Err("admission prompt exceeds 65536 UTF-8 bytes".into()); }
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|error| error.to_string())?;
    let expected = request_tokens(&tokenizer, prompt, max_tokens)?;
    let mut model = load_model(task, model_path, tokenizer)?;
    generate(&||check_cancel(task), &mut model, prompt, max_tokens, &expected, None)
}

fn generate(check: &dyn Fn() -> Result<(), String>, owner: &mut QwenModel, prompt: &str, max_tokens: i64, expected: &[u32], mut publish: Option<&mut dyn FnMut(&[u8], &dyn Fn() -> Result<(), String>) -> Result<(), String>>) -> Result<Generation, String> {
    check()?;
    let model = &owner.model;
    let backend = &owner.backend;
    let vocab = model.vocab();
    let tokens = vocab.tokenize(prompt.as_bytes(), false, true);
    if tokens.len() != expected.len() || tokens.iter().zip(expected).any(|(token, expected)| token.0 < 0 || token.0 as u32 != *expected) {
        return Err("embedded and external tokenizer IDs differ".into());
    }
    let context_tokens = tokens.len() as u32 + max_tokens as u32;
    if context_tokens > model.n_ctx_train() { return Err("prompt plus response budget exceeds the model context".into()); }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
    let check_outer = check;
    let check = || {
        check_outer()?;
        if std::time::Instant::now() >= deadline { return Err("generation exceeded its 180-second deadline".into()); }
        Ok(())
    };
    let params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(context_tokens))
        .with_n_batch(2048).with_n_ubatch(128).with_n_threads(2).with_n_threads_batch(2);
    let mut context = model.new_context(backend, params).map_err(|error| error.to_string())?;
    let mut batch = LlamaBatch::new(2048, 1);
    for (position, token) in tokens.iter().enumerate() {
        batch.add(*token, position as i32, &[0], position + 1 == tokens.len()).map_err(|error| error.to_string())?;
    }
    check()?;
    context.decode(&mut batch).map_err(|error| error.to_string())?;
    let mut sampler = LlamaSampler::greedy();
    let mut generated = 0usize;
    let mut decoder = OutputDecoder::default();
    let mut piece = Vec::with_capacity(32);
    let mut stop = "length";
    for _ in 0..max_tokens {
        check()?;
        let token = sampler.sample(&context, batch.n_tokens() - 1);
        sampler.accept(token);
        if vocab.is_eog(token) { stop = "eos"; break; }
        generated += 1;
        piece.clear();
        vocab.token_to_piece_into(token, &mut piece, false, None);
        if decoder.push(&piece)? {
            if let Some(publish) = publish.as_mut() { publish(decoder.text.as_bytes(), &check)?; }
        }
        if generated == max_tokens as usize { break; }
        batch.clear();
        batch.add(token, (tokens.len() + generated - 1) as i32, &[0], true).map_err(|error| error.to_string())?;
        context.decode(&mut batch).map_err(|error| error.to_string())?;
    }
    check()?;
    let text = decoder.finish()?;
    Ok(Generation { text, input_tokens: tokens.len() as i64, output_tokens: generated as i64, stop: stop.into() })
}

/// In-flight native token evaluation. Owns the model until qwen_stream_finish returns it.
/// Dropping the stream cancels, disconnects output and joins the engine thread.
#[tsrust_resource]
pub struct QwenStream {
    output: Option<Receiver<String>>,
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<(QwenModel, Result<Generation, String>)>>,
}

impl Drop for QwenStream {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.output.take();
        if let Some(worker) = self.worker.take() { let _ = worker.join(); }
    }
}

/// One output read. Changed text is the complete UTF-8 prefix, not a fragment.
/// finished means output is drained; call qwen_stream_finish to recover the model/result.
pub struct StreamPoll {
    pub changed: bool,
    pub finished: bool,
    pub text: String,
}

/// The model is returned even on cancellation or an invalid generation request.
/// generation is Some only on success; error is empty only on success.
pub struct StreamFinished {
    pub model: QwenModel,
    pub generation: Option<Generation>,
    pub error: String,
}

/// Move a model into one native evaluation thread. Only one UTF-8 prefix is queued.
/// Tsrust owns request scheduling, polling and cancellation; no tools or agent code run here.
/// Invalid requests report through finish and return the same model. Thread-spawn
/// failure drops the model and returns an error. No context/owner lifetime is extended unsafely.
pub fn qwen_stream_start(model: QwenModel, prompt: String, max_tokens: i64) -> Result<QwenStream, String> {
    let (sender, output) = mpsc::sync_channel(1);
    let cancel = Arc::new(AtomicBool::new(false));
    let cancelled = Arc::clone(&cancel);
    let worker = std::thread::Builder::new().name("tsrust-model-eval".into()).spawn(move || {
        let mut model = model;
        let check = || if cancelled.load(Ordering::Acquire) { Err("cancelled".into()) } else { Ok(()) };
        let result = (|| {
            check()?;
            let expected = request_tokens(&model.tokenizer, &prompt, max_tokens)?;
            let mut publish = |bytes: &[u8], check: &dyn Fn() -> Result<(), String>| -> Result<(), String> {
                publish_prefix(&sender, bytes, check)
            };
            generate(&check, &mut model, &prompt, max_tokens, &expected, Some(&mut publish))
        })();
        (model, result)
    }).map_err(|error|error.to_string())?;
    Ok(QwenStream { output: Some(output), cancel, worker: Some(worker) })
}

/// Wait up to 10 milliseconds on a background worker. Empty differs from finished.
/// The bounded wait yields CPU and lets Tsrust check requests/cancellation between reads.
pub fn qwen_stream_next(stream: &mut QwenStream) -> StreamPoll {
    match stream.output.as_ref().expect("live stream").recv_timeout(std::time::Duration::from_millis(10)) {
        Ok(text) => StreamPoll { changed: true, finished: false, text },
        Err(RecvTimeoutError::Timeout) => StreamPoll { changed: false, finished: false, text: String::new() },
        Err(RecvTimeoutError::Disconnected) => StreamPoll { changed: false, finished: true, text: String::new() },
    }
}

/// Request cancellation, including while the single-prefix queue is full.
/// Native decode is cooperative: cancellation is observed between model operations.
pub fn qwen_stream_cancel(stream: &QwenStream) {
    stream.cancel.store(true, Ordering::Release);
}

/// Join and recover the model plus success/error. Call after poll.finished for normal completion.
/// Finishing an unfinished stream cancels it. A native panic returns an error and drops the model.
pub fn qwen_stream_finish(mut stream: QwenStream) -> Result<StreamFinished, String> {
    if !stream.worker.as_ref().expect("live worker").is_finished() {
        stream.cancel.store(true, Ordering::Release);
    }
    stream.output.take();
    let (model, result) = stream.worker.take().expect("live worker").join().map_err(|_|"native generation worker panicked".to_owned())?;
    Ok(match result {
        Ok(generation) => StreamFinished { model, generation: Some(generation), error: String::new() },
        Err(error) => StreamFinished { model, generation: None, error },
    })
}
