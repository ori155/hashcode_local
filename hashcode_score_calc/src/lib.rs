#[macro_use]
extern crate lazy_static;
pub mod qual2020;
pub mod qual2016;

use thiserror::Error;
use std::fmt::Display;
use core::fmt::Debug;
use serde_derive::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum ScoringError {
    #[error("Missing line")]
    MissingLine,
    #[error("Expected a number")]
    ExpectedANumber,
    #[error("Doesn't have input case with name {0}")]
    UnknownInputCase(InputFileName),
    #[error("Challenge Specific: {0}")]
    ChallengeSpecific(Box<dyn std::error::Error + std::marker::Sync + std::marker::Send>),
    #[error("Error parsing the input file: {0}")]
    InputFileError(Box<dyn std::error::Error + std::marker::Sync + std::marker::Send>)
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct InputFileName(pub(crate) String);

impl Display for InputFileName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "InputFileName: {}", self.0)
    }
}

impl std::convert::From<&str> for InputFileName {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

pub type Score = u64;
pub type Year = u32;

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeDate {
    Qualification(Year),
    Final(Year),
}

pub struct Challenge {
    pub input_file_names: Vec<InputFileName>,
    pub score_function: Box<dyn Fn(&str, &InputFileName) -> Result<Score, ScoringError> + 'static + Send + Sync>,
    pub date: ChallengeDate,
}

pub fn get_challenges() -> Vec<Challenge> {
    vec![
        Challenge{
            input_file_names: vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into(),
                "e".into(),
                "f".into()
            ],
            score_function: Box::new(crate::qual2020::score),
            date: ChallengeDate::Qualification(2020)
        }
    ]
}

