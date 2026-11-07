mod tsrust_native_crypto_password {
 // Explicit bounded CPU jobs; backend owns Argon2/PHC and secret primitives.
 use super::tsrust_native_crypto::Error;
 use crate::tsrust_native_threads as pool;
 use argon2::{Algorithm,Argon2,Params,Version,password_hash::{PasswordHasher,PasswordVerifier,phc::PasswordHash}};
 use zeroize::Zeroizing;
 use std::convert::Infallible;
 pub enum Failure{Crypto(Error),Admission(pool::SpawnError),Completion(pool::JoinError<Infallible>)}
 pub struct HashOptions{pub memory:Option<i64>,pub iterations:Option<i64>,pub parallelism:Option<i64>,pub output:Option<i64>}
 pub struct VerifyOptions{pub memory:i64,pub iterations:i64,pub parallelism:i64}
 fn positive(value:i64)->Result<u32,Error>{u32::try_from(value).ok().filter(|value|*value>0).ok_or(("Options","Password costs must be positive backend-representable integers"))}
 fn params(options:HashOptions)->Result<Params,Error>{
  let defaults=Params::default();let memory=options.memory.map(positive).transpose()?.unwrap_or(defaults.m_cost());let iterations=options.iterations.map(positive).transpose()?.unwrap_or(defaults.t_cost());let parallelism=options.parallelism.map(positive).transpose()?.unwrap_or(defaults.p_cost());
  let output=options.output.map(|value|usize::try_from(value).map_err(|_|("Options","Password output length must be nonnegative and fit the target"))).transpose()?;
  // The PHC backend's encoded-output bounds differ from raw Argon2 output.
  if output.is_some_and(|value|value<argon2::password_hash::phc::Output::MIN_LENGTH || value>argon2::password_hash::phc::Output::MAX_LENGTH){return Err(("Options","Password output length is outside the PHC backend range"));}
  Params::new(memory,iterations,parallelism,output).map_err(|_|("Options","Invalid Argon2 cost or output parameters"))
 }
 fn preflight(encoded:&str,policy:Option<VerifyOptions>)->Result<(),Error>{
  let parsed=PasswordHash::new(encoded).map_err(|_|("Password","Malformed encoded password hash"))?;
  Algorithm::try_from(parsed.algorithm.as_str()).map_err(|_|("Password","Unsupported encoded password algorithm"))?;
  if let Some(version)=parsed.version{Version::try_from(version).map_err(|_|("Password","Unsupported encoded password version"))?;}
  if parsed.salt.is_none()||parsed.hash.is_none(){return Err(("Password","Encoded password hash needs salt and output"));}
  let costs=Params::try_from(&parsed).map_err(|_|("Password","Invalid encoded password parameters"))?;
  if let Some(policy)=policy{let memory=positive(policy.memory)?;let iterations=positive(policy.iterations)?;let parallelism=positive(policy.parallelism)?;if costs.m_cost()>memory||costs.t_cost()>iterations||costs.p_cost()>parallelism{return Err(("Policy","Encoded password costs exceed explicit verification limits"));}}
  Ok(())
 }
 fn finish<T>(result:Result<Result<T,Error>,pool::AsyncTaskError<Infallible>>)->Result<T,Failure>{match result{Ok(result)=>result.map_err(Failure::Crypto),Err(pool::AsyncTaskError::Admission(error))=>Err(Failure::Admission(error)),Err(pool::AsyncTaskError::Completion(error))=>Err(Failure::Completion(error))}}
 pub async fn hash(capability:pool::PoolSubmission,password:&[u8],options:HashOptions)->Result<String,Failure>{
  let params=params(options).map_err(Failure::Crypto)?;
  let password=Zeroizing::new(password.to_vec());
  finish(pool::task_async(capability,move||{let mut salt=[0;argon2::RECOMMENDED_SALT_LEN];let result=getrandom::fill(&mut salt).map_err(|_|("Random","Operating system randomness unavailable")).and_then(|()|Argon2::new(Algorithm::Argon2id,Version::V0x13,params).hash_password_with_salt(&password,&salt).map(|hash|hash.to_string()).map_err(|_|("Password","Password hashing failed")));Ok::<_,Infallible>(result)}).await)
 }
 pub async fn verify(capability:pool::PoolSubmission,password:&[u8],encoded:&str,policy:Option<VerifyOptions>)->Result<bool,Failure>{
  preflight(encoded,policy).map_err(Failure::Crypto)?;
  let password=Zeroizing::new(password.to_vec());let encoded=encoded.to_owned();
  finish(pool::task_async(capability,move||Ok::<_,Infallible>(match Argon2::default().verify_password(&password,&encoded){Ok(())=>Ok(true),Err(argon2::password_hash::Error::PasswordInvalid)=>Ok(false),Err(_)=>Err(("Password","Password verification failed"))})).await)
 }
}
