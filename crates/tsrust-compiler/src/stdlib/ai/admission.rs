// Candidate tensor boundary for the Studio training feasibility proof.
// Not a registered std:ai module or a model-training acceptance claim.
use candle_core::{Device, Tensor, Var};
use candle_nn::{Optimizer, SGD};

/// One scalar optimizer observation. Loss is measured before the update.
pub struct Step {
    pub weight: f64,
    pub loss: f64,
}

/// Minimize (weight - target)^2 with one CPU Candle SGD update.
/// All arguments must be finite and learning_rate must be positive.
/// Returns the updated weight and pre-update loss, or an explanatory error.
/// Inputs are copied; tensor and optimizer owners are disposed before return.
pub fn scalar_step(weight: f64, target: f64, learning_rate: f64) -> Result<Step, String> {
    if !weight.is_finite() || !target.is_finite() || !learning_rate.is_finite() || learning_rate <= 0.0 {
        return Err("finite weight/target and positive finite learning rate required".into());
    }
    let execute = || -> candle_core::Result<Step> {
        let weight = Var::new(weight, &Device::Cpu)?;
        let target = Tensor::new(target, &Device::Cpu)?;
        let loss = weight.as_tensor().sub(&target)?.sqr()?;
        let mut optimizer = SGD::new(vec![weight.clone()], learning_rate)?;
        optimizer.backward_step(&loss)?;
        Ok(Step { weight: weight.as_tensor().to_scalar::<f64>()?, loss: loss.to_scalar::<f64>()? })
    };
    let result = execute().map_err(|error| error.to_string())?;
    if !result.weight.is_finite() || !result.loss.is_finite() {
        return Err("optimizer produced a non-finite result".into());
    }
    Ok(result)
}
