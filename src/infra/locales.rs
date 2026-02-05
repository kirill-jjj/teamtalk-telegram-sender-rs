use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{Loader, static_loader};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;
use strum::{EnumIter, IntoStaticStr};
use unic_langid::LanguageIdentifier;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

static LANG_RU: LazyLock<LanguageIdentifier> =
    LazyLock::new(|| "ru".parse().expect("Valid RU langid"));
static LANG_EN: LazyLock<LanguageIdentifier> =
    LazyLock::new(|| "en".parse().expect("Valid EN langid"));

fn get_lang_id(lang_code: &str) -> &LanguageIdentifier {
    match lang_code {
        "ru" => &LANG_RU,
        _ => &LANG_EN,
    }
}

#[derive(IntoStaticStr, EnumIter, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
#[strum(serialize_all = "kebab-case")]
pub enum LocaleKey {
    AdminAlert,
    AdminChannelPm,
    AdminErrorContextCallback,
    AdminErrorContextCommand,
    AdminErrorContextSubscription,
    AdminErrorContextTtCommand,
    AdminErrorContextUpdateListener,
    AdminErrorUser,
    BtnBackManageAcc,
    BtnBackMenu,
    BtnBackMute,
    BtnBackNotif,
    BtnBackSearch,
    BtnBackSettings,
    BtnBackSubs,
    BtnBackUserActions,
    BtnLang,
    BtnLinkNew,
    BtnManageBlacklist,
    BtnManageWhitelist,
    BtnMenuBan,
    BtnMenuHelp,
    BtnMenuKick,
    BtnMenuSettings,
    BtnMenuSubs,
    BtnMenuUnban,
    BtnMenuUnsub,
    BtnMenuWho,
    BtnModeBlacklist,
    BtnModeWhitelist,
    BtnMuteManage,
    BtnMuteServerListBlacklist,
    BtnMuteServerListWhitelist,
    BtnNext,
    BtnNo,
    BtnNoon,
    BtnNotifSettings,
    BtnPrev,
    BtnQueueClear,
    BtnQueueClearAll,
    BtnQueueGlobalToggle,
    BtnQueueSettings,
    BtnQueueUserToggle,
    BtnSubAll,
    BtnSubBan,
    BtnSubDelete,
    BtnSubJoin,
    BtnSubLang,
    BtnSubLeave,
    BtnSubManageTt,
    BtnSubMuteMode,
    BtnSubNone,
    BtnSubNoon,
    BtnSubNotif,
    BtnSubSettings,
    BtnSubViewMute,
    BtnUnlink,
    BtnYes,
    CmdBroadcastEmpty,
    CmdBroadcastSent,
    CmdDescBan,
    CmdDescBroadcast,
    CmdDescExit,
    CmdDescHelp,
    CmdDescKick,
    CmdDescMenu,
    CmdDescMessage,
    CmdDescQueue,
    CmdDescSettings,
    CmdDescSubscribers,
    CmdDescUnban,
    CmdDescUnsub,
    CmdDescWho,
    CmdError,
    CmdFailNoonGuest,
    CmdInvalidDeeplink,
    CmdMessageEmpty,
    CmdMessageSent,
    CmdNoUsers,
    CmdNotSubscribed,
    CmdQueueHelp,
    CmdQueueNoLink,
    CmdShuttingDown,
    CmdSuccessSub,
    CmdSuccessSubGuest,
    CmdSuccessUnsub,
    CmdTtBanned,
    CmdUnauth,
    CmdUserBanned,
    DisplayGuestAccount,
    EventJoin,
    EventLeave,
    HelloStart,
    HelpText,
    IconCheckSimple,
    IconChecked,
    IconUnchecked,
    ItemStatusBlacklistIn,
    ItemStatusBlacklistOut,
    ItemStatusWhitelistIn,
    ItemStatusWhitelistOut,
    ListAllAccsTitle,
    ListBanEmpty,
    ListBanTitle,
    ListKickTitle,
    ListLinkTitle,
    ListMuteEmpty,
    ListMuteTitle,
    ListMuteTitleFor,
    ListSearchEmpty,
    ListSearchHint,
    ListSearchTitle,
    ListSubsEmpty,
    ListSubsTitle,
    ListUnbanTitle,
    MenuTitle,
    ModeBlacklist,
    ModeWhitelist,
    MsgChooseLang,
    MuteGuestNote,
    MuteModeBlacklist,
    MuteModeWhitelist,
    MuteTitle,
    NotifSettingsTitle,
    QueueSettingsTitle,
    RespNoonUpdated,
    RespQueueCleared,
    RespQueueClearedAll,
    RespQueueGlobalAlreadyDisabled,
    RespQueueGlobalAlreadyEnabled,
    RespQueueGlobalDisabled,
    RespQueueGlobalDisabledUser,
    RespQueueGlobalEnabled,
    RespQueueUserAlreadyDisabled,
    RespQueueUserAlreadyEnabled,
    RespQueueUserDisabled,
    RespQueueUserEnabled,
    RespSubUpdated,
    SettingsTitle,
    StatusDisabled,
    StatusEnabled,
    SubDetailsTitle,
    SubLangTitle,
    SubManageTtTitle,
    SubModeTitle,
    SubNotifTitle,
    TgReplyFailed,
    TgReplyOffline,
    TgReplyQueued,
    TgReplySent,
    ToastAccountLinked,
    ToastAccountUnlinked,
    ToastCommandSent,
    ToastLangSet,
    ToastLangUpdated,
    ToastMuteModeSet,
    ToastMuteModeSubSet,
    ToastNoonToggled,
    ToastNotifSet,
    ToastSubscriberDeleted,
    ToastUserBanned,
    ToastUserMuted,
    ToastUserUnbanned,
    TtAdminAddFail,
    TtAdminAdded,
    TtAdminHelpCmds,
    TtAdminHelpHeader,
    TtAdminNoIds,
    TtAdminRemoveFail,
    TtAdminRemoved,
    TtChannelReply,
    TtChannelReplyText,
    TtErrorGeneric,
    TtMsgFailed,
    TtMsgSent,
    TtQueueCleared,
    TtQueueClearedAll,
    TtQueueGlobalAlreadyDisabled,
    TtQueueGlobalAlreadyEnabled,
    TtQueueGlobalDisabled,
    TtQueueGlobalDisabledUser,
    TtQueueGlobalEnabled,
    TtQueueHelp,
    TtQueueNoLink,
    TtQueueUserAlreadyDisabled,
    TtQueueUserAlreadyEnabled,
    TtQueueUserDisabled,
    TtQueueUserEnabled,
    TtReportHeader,
    TtReportRoot,
    TtReportRow,
    TtReportUnauth,
    TtRootChannelName,
    TtSkipSent,
    TtSubLink,
    TtUnsubLink,
    UnsubCancelled,
    UnsubConfirmText,
    ValNone,
    BtnSubAdminAdd,
    BtnSubAdminRemove,
    ConfirmAdminAdd,
    ConfirmAdminRemove,
    ToastAdminAdded,
    ToastAdminRemoved,
    ValYes,
    ValNo,
}

impl LocaleKey {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

pub fn get_text(
    lang_code: &str,
    key: LocaleKey,
    args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
) -> String {
    let lang_id = get_lang_id(lang_code);
    let key = key.as_str();

    args.map_or_else(
        || LOCALES.lookup(lang_id, key),
        |args_map| LOCALES.lookup_with_args(lang_id, key, args_map),
    )
}

#[cfg(test)]
#[path = "../../tests/unit/infra_locales.rs"]
mod tests;

#[macro_export]
/// Helper to build Fluent arguments.
macro_rules! args {
    ( $($k:ident = $v:expr),* ) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert(
                std::borrow::Cow::Borrowed(stringify!($k)),
                fluent_templates::fluent_bundle::FluentValue::from($v)
            );
        )*
        Some(map)
    }};
}


