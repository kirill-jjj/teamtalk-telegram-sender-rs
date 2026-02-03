use teamtalk::types::BannedUser;
use teamtalk::utils::strings::to_string;

#[test]
fn banned_user_to_ffi_copies_fields() {
    let user = BannedUser {
        ip: "1.2.3.4".to_string(),
        channel_path: "/root".to_string(),
        nickname: "nick".to_string(),
        username: "user".to_string(),
        ban_time: "now".to_string(),
        ban_types: 7,
        owner: "admin".to_string(),
    };
    let raw = user.to_ffi();
    assert_eq!(to_string(&raw.szIPAddress), "1.2.3.4");
    assert_eq!(to_string(&raw.szChannelPath), "/root");
    assert_eq!(to_string(&raw.szNickname), "nick");
    assert_eq!(to_string(&raw.szUsername), "user");
    assert_eq!(to_string(&raw.szOwner), "admin");
    assert_eq!(raw.uBanTypes, 7);
}
