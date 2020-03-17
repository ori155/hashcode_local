use hashcode_score_calc::qual2016;


#[test]
fn example_input() {
    let submission = include_str!("../assets/2016qual/submissions/example_submission.txt");
    let _input = include_str!("../assets/2016qual/inputs/example.in");

    assert_eq!(qual2016::score(submission, &"example".into()).expect("Should succeed"), 194);
}

