pub mod team;
pub mod team_name;

pub mod solution {
    use serde_derive::{Deserialize, Serialize};

    pub use hashcode_score_calc::ChallengeDate;
    pub use hashcode_score_calc::InputFileName;


    #[derive(Serialize, Deserialize, Debug)]
    pub struct Solution {
        pub challenge: ChallengeDate,
        pub solutions: std::collections::HashMap<InputFileName, String>
    }
}

pub use team::Team;
pub use team_name::TeamName;
