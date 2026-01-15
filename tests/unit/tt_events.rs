use super::*;

#[test]
fn parse_gender_handles_values() {
    assert_eq!(parse_gender("male"), UserGender::Male);
    assert_eq!(parse_gender("female"), UserGender::Female);
    assert_eq!(parse_gender("unknown"), UserGender::Neutral);
    assert_eq!(parse_gender(""), UserGender::Neutral);
}
