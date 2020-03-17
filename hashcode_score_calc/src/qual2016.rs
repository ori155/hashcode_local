use thiserror::Error;
use crate::{ScoringError, InputFileName, Score};


#[derive(Error, Debug, PartialEq, Eq)]
pub enum Qual2016ScoringError {

}


impl From<Qual2016ScoringError> for ScoringError {
    fn from(e: Qual2016ScoringError) -> Self {
        ScoringError::ChallengeSpecific(Box::new(e))
    }
}



pub fn score(submission: &str, case: &InputFileName) -> Result<Score, ScoringError> {
   Ok(0)
}
