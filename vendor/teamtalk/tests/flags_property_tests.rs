use proptest::prelude::*;
use teamtalk::types::{ClientFlags, Subscriptions, UserState};

proptest! {
    #[test]
    fn client_flags_raw_roundtrip(raw in any::<u32>()) {
        let flags = ClientFlags::from_raw(raw);
        prop_assert_eq!(flags.raw(), raw);
    }

    #[test]
    fn subscriptions_has_self_consistent(raw in any::<u32>()) {
        let subs = Subscriptions::from_raw(raw);
        let flag = raw | 1;
        prop_assert!(subs.has(flag) || !subs.has(flag));
    }

    #[test]
    fn user_state_has_self_consistent(raw in any::<u32>()) {
        let state = UserState::from_raw(raw);
        prop_assert_eq!(state.raw(), raw);
    }
}
