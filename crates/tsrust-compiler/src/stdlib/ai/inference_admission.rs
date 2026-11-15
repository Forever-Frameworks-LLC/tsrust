// Candidate engine primitive; Studio prompts, tools and scheduling stay in Tsrust.
use candle_core::{Device, Tensor, quantized::gguf_file};
use candle_transformers::models::quantized_qwen2::ModelWeights;
use tokenizers::Tokenizer;

/// Buffered exploratory generation. This does not provide a streaming session.
pub struct Generation {
    pub text: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub stop: String,
}

/// Load a local Qwen2 GGUF and tokenizer on CPU, then greedily generate tokens.
/// prompt is already templated by the caller. max_tokens must be 1..=32 and the
/// encoded prompt must contain 1..=2048 tokens for this bounded admission only.
/// Checks cancellation before loading and between model operations. Returns
/// cancelled errors without partial output; tensor owners drop on every exit.
/// Local files only: this function performs no network or tool operations.
pub fn qwen_generate(task: &TaskContext, model_path: &str, tokenizer_path: &str, prompt: &str, max_tokens: i64) -> Result<Generation, String> {
    let check = || -> Result<(), String> {
        if task.cancellation_requested() { return Err("cancelled".into()); }
        Ok(())
    };
    check()?;
    if !(1..=32).contains(&max_tokens) { return Err("admission max_tokens must be 1..=32".into()); }
    if prompt.len() > 65536 { return Err("admission prompt exceeds 65536 UTF-8 bytes".into()); }
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|error| error.to_string())?;
    let encoded = tokenizer.encode(prompt, true).map_err(|error| error.to_string())?;
    let input = encoded.get_ids();
    if input.is_empty() || input.len() > 2048 { return Err("admission context must be 1..=2048 tokens".into()); }
    let eos = tokenizer.token_to_id("<|im_end|>").ok_or("missing Qwen end-of-message token")?;
    let end = tokenizer.token_to_id("<|endoftext|>").ok_or("missing Qwen end-of-text token")?;
    check()?;
    let execute = || -> candle_core::Result<Generation> {
        let mut file = std::fs::File::open(model_path)?;
        let content = gguf_file::Content::read(&mut file)?;
        let mut model = ModelWeights::from_gguf(content, &mut file, &Device::Cpu)?;
        let mut generated = Vec::new();
        let mut next = input.to_vec();
        let mut position = 0;
        let mut stop = "length";
        for _ in 0..max_tokens {
            if task.cancellation_requested() { return Err(candle_core::Error::Msg("cancelled".into())); }
            let tensor = Tensor::new(next.as_slice(), &Device::Cpu)?.unsqueeze(0)?;
            let logits = model.forward(&tensor, position)?;
            let token = logits.squeeze(0)?.argmax(0)?.to_scalar::<u32>()?;
            if token == eos || token == end { stop = "eos"; break; }
            generated.push(token);
            position += next.len();
            next = vec![token];
        }
        let text = tokenizer.decode(&generated, false).map_err(|error| candle_core::Error::Msg(error.to_string()))?;
        Ok(Generation { text, input_tokens: input.len() as i64, output_tokens: generated.len() as i64, stop: stop.into() })
    };
    let result = execute().map_err(|error| error.to_string())?;
    check()?;
    Ok(result)
}
