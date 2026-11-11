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

/// Load the verified Qwen GGUF with memory mapping and Metal layer offload.
/// Requires Metal support, a pretemplated prompt of 1..=2048 tokens and
/// max_tokens in 1..=32. Embedded and external tokenizer IDs must agree exactly.
/// Checks cancellation between operations and returns errors without partial
/// output. All model/context owners drop before return. No network or tools run.
pub fn qwen_generate(task: &TaskContext, model_path: &str, tokenizer_path: &str, prompt: &str, max_tokens: i64) -> Result<Generation, String> {
    let check = || -> Result<(), String> {
        if task.cancellation_requested() { return Err("cancelled".into()); }
        Ok(())
    };
    check()?;
    if !(1..=32).contains(&max_tokens) { return Err("admission max_tokens must be 1..=32".into()); }
    if prompt.len() > 65536 { return Err("admission prompt exceeds 65536 UTF-8 bytes".into()); }
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|error| error.to_string())?;
    let expected = tokenizer.encode(prompt, true).map_err(|error| error.to_string())?;
    if expected.get_ids().is_empty() || expected.get_ids().len() > 2048 { return Err("admission context must be 1..=2048 tokens".into()); }
    check()?;
    let backend = LlamaBackend::init().map_err(|error| error.to_string())?;
    if !backend.supports_gpu_offload() || !backend.supports_mmap() { return Err("Metal offload and memory mapping are required for this admission".into()); }
    let params = LlamaModelParams::default().with_use_mmap(true).with_n_gpu_layers(1000);
    let model = LlamaModel::load_from_file(&backend, model_path, &params).map_err(|error| error.to_string())?;
    check()?;
    let vocab = model.vocab();
    let tokens = vocab.tokenize(prompt.as_bytes(), false, true);
    if tokens.len() != expected.get_ids().len() || tokens.iter().zip(expected.get_ids()).any(|(token, expected)| token.0 < 0 || token.0 as u32 != *expected) {
        return Err("embedded and external tokenizer IDs differ".into());
    }
    let params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(2080))
        .with_n_batch(2048).with_n_ubatch(128).with_n_threads(2).with_n_threads_batch(2);
    let mut context = model.new_context(&backend, params).map_err(|error| error.to_string())?;
    let mut batch = LlamaBatch::new(2048, 1);
    for (position, token) in tokens.iter().enumerate() {
        batch.add(*token, position as i32, &[0], position + 1 == tokens.len()).map_err(|error| error.to_string())?;
    }
    check()?;
    context.decode(&mut batch).map_err(|error| error.to_string())?;
    let mut sampler = LlamaSampler::greedy();
    let mut generated = Vec::new();
    let mut stop = "length";
    for _ in 0..max_tokens {
        check()?;
        let token = sampler.sample(&context, batch.n_tokens() - 1);
        sampler.accept(token);
        if vocab.is_eog(token) { stop = "eos"; break; }
        generated.push(token);
        if generated.len() == max_tokens as usize { break; }
        batch.clear();
        batch.add(token, (tokens.len() + generated.len() - 1) as i32, &[0], true).map_err(|error| error.to_string())?;
        context.decode(&mut batch).map_err(|error| error.to_string())?;
    }
    check()?;
    let text = String::from_utf8(vocab.detokenize(&generated, false, true)).map_err(|error| error.to_string())?;
    Ok(Generation { text, input_tokens: tokens.len() as i64, output_tokens: generated.len() as i64, stop: stop.into() })
}
