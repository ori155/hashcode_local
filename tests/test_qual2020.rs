use hashcode_score_calc::qual2020;
use hashcode_score_calc::qual2020::CASE_A;


#[test]
fn example_input() {
    let submission = include_str!("../assets/2020qual/submissions/example_submission.txt");
    let input = include_str!("../assets/2020qual/inputs/a_example.txt");

    assert_eq!(qual2020::score(submission, &*CASE_A), Ok(16));
}

