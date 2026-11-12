// Candidate native engine primitive. Prompts, workers and all agent logic stay in Tsrust.
use llama_cpp_2::{context::params::LlamaContextParams, llama_backend::LlamaBackend,
    llama_batch::LlamaBatch, model::{LlamaModel, params::LlamaModelParams}, sampling::LlamaSampler};
use std::num::NonZeroU32;
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

fn request_tokens(tokenizer: &Tokenizer, prompt: &str, max_tokens: i64) -> Result<Vec<u32>, String> {
    if !(1..=32).contains(&max_tokens) { return Err("admission max_tokens must be 1..=32".into()); }
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
/// Limits remain 1..=2048 prompt tokens, 65536 UTF-8 bytes and 1..=32 output tokens.
/// Returns buffered text/counts/stop reason; cancellation returns an error without partial text.
pub fn qwen_generate_loaded(task: &TaskContext, model: &mut QwenModel, prompt: &str, max_tokens: i64) -> Result<Generation, String> {
    check_cancel(task)?;
    let expected = request_tokens(&model.tokenizer, prompt, max_tokens)?;
    generate(task, model, prompt, max_tokens, &expected)
}

/// Load, generate and unload one model. Prefer qwen_load/qwen_generate_loaded for repeated requests.
/// Validates request limits before loading the model; all owners drop before return.
pub fn qwen_generate(task: &TaskContext, model_path: &str, tokenizer_path: &str, prompt: &str, max_tokens: i64) -> Result<Generation, String> {
    check_cancel(task)?;
    // Reject cheap invalid requests before touching either artifact, preserving the original contract.
    if !(1..=32).contains(&max_tokens) { return Err("admission max_tokens must be 1..=32".into()); }
    if prompt.len() > 65536 { return Err("admission prompt exceeds 65536 UTF-8 bytes".into()); }
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|error| error.to_string())?;
    let expected = request_tokens(&tokenizer, prompt, max_tokens)?;
    let mut model = load_model(task, model_path, tokenizer)?;
    generate(task, &mut model, prompt, max_tokens, &expected)
}

fn generate(task: &TaskContext, owner: &mut QwenModel, prompt: &str, max_tokens: i64, expected: &[u32]) -> Result<Generation, String> {
    check_cancel(task)?;
    let model = &owner.model;
    let backend = &owner.backend;
    let vocab = model.vocab();
    let tokens = vocab.tokenize(prompt.as_bytes(), false, true);
    if tokens.len() != expected.len() || tokens.iter().zip(expected).any(|(token, expected)| token.0 < 0 || token.0 as u32 != *expected) {
        return Err("embedded and external tokenizer IDs differ".into());
    }
    let params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(2080))
        .with_n_batch(2048).with_n_ubatch(128).with_n_threads(2).with_n_threads_batch(2);
    let mut context = model.new_context(backend, params).map_err(|error| error.to_string())?;
    let mut batch = LlamaBatch::new(2048, 1);
    for (position, token) in tokens.iter().enumerate() {
        batch.add(*token, position as i32, &[0], position + 1 == tokens.len()).map_err(|error| error.to_string())?;
    }
    check_cancel(task)?;
    context.decode(&mut batch).map_err(|error| error.to_string())?;
    let mut sampler = LlamaSampler::greedy();
    let mut generated = Vec::new();
    let mut stop = "length";
    for _ in 0..max_tokens {
        check_cancel(task)?;
        let token = sampler.sample(&context, batch.n_tokens() - 1);
        sampler.accept(token);
        if vocab.is_eog(token) { stop = "eos"; break; }
        generated.push(token);
        if generated.len() == max_tokens as usize { break; }
        batch.clear();
        batch.add(token, (tokens.len() + generated.len() - 1) as i32, &[0], true).map_err(|error| error.to_string())?;
        context.decode(&mut batch).map_err(|error| error.to_string())?;
    }
    check_cancel(task)?;
    let text = String::from_utf8(vocab.detokenize(&generated, false, true)).map_err(|error| error.to_string())?;
    Ok(Generation { text, input_tokens: tokens.len() as i64, output_tokens: generated.len() as i64, stop: stop.into() })
}
