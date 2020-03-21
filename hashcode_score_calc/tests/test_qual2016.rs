use hashcode_score_calc::qual2016;


#[test]
fn example_input() {
    let submission = include_str!("../assets/2016qual/submissions/example_submission.txt");
    let _input = include_str!("../assets/2016qual/inputs/example.in");

    assert_eq!(qual2016::score(submission, &"example".into()).expect("Should succeed"), 194);
}

#[test]
fn should_not_fail() {
    let submission = r#"4
    6 L 0 350 2
    6 L 0 541 1
    6 D 331 350 2
    6 D 331 541 1
    "#;

    assert_eq!(qual2016::score(submission, &"mother_of_all_warehouses".into()).expect("Should succeed"), 0);
}

