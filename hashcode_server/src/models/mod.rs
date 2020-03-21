pub mod team;
pub mod team_name;

pub mod solution {
    use serde_derive::{Deserialize, Serialize};

    pub use hashcode_score_calc::ChallengeDate;
    pub use hashcode_score_calc::InputFileName;
    use crate::models::TeamName;
    use hex_string::HexString;


    #[derive(Serialize, Deserialize, Debug)]
    pub struct Solution {
        pub challenge: ChallengeDate,
        pub solutions: std::collections::HashMap<InputFileName, String>
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SolutionSubmitRequest {
        pub solution: Solution,
        pub team_name: TeamName,
        #[serde(with="token_from_string")]
        pub token: HexString
    }

    pub mod token_from_string {
        use serde::{Deserialize, Deserializer, Serializer};
        use serde::de;
        use hex_string::HexString;
        use std::str::FromStr;

        pub fn serialize<S>(token: &HexString, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
            s.serialize_str(token.as_str())
        }
        pub fn deserialize<'de, D>(d: D) -> Result<HexString, D::Error> where D: Deserializer<'de> {
            let s = String::deserialize(d)?;
            HexString::from_str(&s).map_err(|e| de::Error::custom(e))
        }
    }
}

pub use team::Team;
pub use team_name::TeamName;
