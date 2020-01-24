use diesel::sql_types::Text;
use rand::Rng;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(
    Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Debug, Clone, TypeScriptify,
)]
#[sql_type = "Text"]
#[serde(tag = "type")]
pub enum Sampler {
    /// sample randomly, with an average interval of avg_time seconds
    RandomSampler { avg_time: f64 },
    /// where the event duration is known exactly
    Explicit { duration: f64 },
}

impl Sampler {
    pub fn get_sample(&self) -> f64 {
        match &self {
            Sampler::RandomSampler { avg_time } => {
                let distribution = rand::distributions::Uniform::new(0f64, (avg_time) * 2.0);
                let mut rng = rand::thread_rng();
                return rng.sample(distribution);
            }
            Sampler::Explicit { duration } => panic!("cant sample explicit"),
        }
    }
    /// get the (approximate duration value for each entry that used this sampler
    pub fn get_duration(&self) -> f64 {
        match &self {
            Sampler::RandomSampler { avg_time } => *avg_time,
            Sampler::Explicit { duration } => *duration,
        }
    }
}
