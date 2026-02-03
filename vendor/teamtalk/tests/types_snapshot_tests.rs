use insta::assert_debug_snapshot;
use teamtalk::types::UserAccountBuilder;

#[test]
fn user_account_builder_snapshot() {
    let account = UserAccountBuilder::new("alice")
        .password("secret")
        .user_type(2)
        .rights(7)
        .build();
    assert_debug_snapshot!(account);
}
