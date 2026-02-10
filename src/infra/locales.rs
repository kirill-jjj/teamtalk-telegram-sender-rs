use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{Loader, static_loader};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;
use strum::{EnumIter, IntoStaticStr};
use thiserror::Error;
use unic_langid::LanguageIdentifier;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        customise: |bundle| {
            bundle.set_use_isolating(false);
            if let Err(e) = bundle.add_builtins() {
                tracing::error!(error = %e, "Failed to register Fluent builtins");
            }
        },
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
    AdminErrorSystem,
    AdminErrorUser,
    AdminSubEventSubscribed,
    AdminSubEventUnsubscribed,
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
    BtnAdminSubEvents,
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
    DisplayUnknownUser,
    EventJoin,
    EventJoinMale,
    EventJoinFemale,
    EventJoinNeutral,
    EventLeave,
    EventLeaveMale,
    EventLeaveFemale,
    EventLeaveNeutral,
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
    RespAdminSubEventsUpdated,
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
    SubUserNotifyActor,
    SubUserNotifyActorUsername,
    SubUserNotifyAdminAdded,
    SubUserNotifyAdminRemoved,
    SubUserNotifyBanned,
    SubUserNotifyDeleted,
    SubUserNotifyLang,
    SubUserNotifyLinked,
    SubUserNotifyMuteMode,
    SubUserNotifyNoon,
    SubUserNotifyNotif,
    SubUserNotifyUnlinked,
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
    TtBridgeDisabledUser,
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
    ValLangEn,
    ValLangRu,
    ValYes,
    ValNo,
}

impl LocaleKey {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

pub fn try_get_text(
    lang_code: &str,
    key: LocaleKey,
    args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
) -> Result<String, LocaleError> {
    let lang_id = get_lang_id(lang_code);
    let key = key.as_str();

    let value = args.map_or_else(
        || LOCALES.try_lookup(lang_id, key),
        |args_map| LOCALES.try_lookup_with_args(lang_id, key, args_map),
    );

    if let Some(value) = value {
        return Ok(value);
    }

    let primary_error = args
        .map_or_else(
            || LOCALES.lookup_single_language::<Cow<'static, str>>(lang_id, key, None),
            |args_map| {
                LOCALES.lookup_single_language::<Cow<'static, str>>(lang_id, key, Some(args_map))
            },
        )
        .err()
        .map(|e| e.to_string());

    let fallback = LOCALES.fallback();
    let fallback_error = if lang_id == fallback {
        None
    } else {
        args.map_or_else(
            || LOCALES.lookup_single_language::<Cow<'static, str>>(fallback, key, None),
            |args_map| {
                LOCALES.lookup_single_language::<Cow<'static, str>>(fallback, key, Some(args_map))
            },
        )
        .err()
        .map(|e| e.to_string())
    };

    Err(LocaleError {
        lang: lang_code.to_string(),
        key: key.to_string(),
        primary: primary_error,
        fallback: fallback_error,
    })
}

pub fn get_text(
    lang_code: &str,
    key: LocaleKey,
    args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
) -> String {
    let key_str = key.as_str();
    match try_get_text(lang_code, key, args) {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(
                lang = lang_code,
                key = key_str,
                has_args = args.is_some(),
                error = %error,
                "Localization lookup failed"
            );
            format!("Unknown localization key: {key_str:?}")
        }
    }
}

#[derive(Debug, Error)]
#[error(
    "localization lookup failed for key {key} lang {lang}: primary={primary:?}, fallback={fallback:?}"
)]
pub struct LocaleError {
    lang: String,
    key: String,
    primary: Option<String>,
    fallback: Option<String>,
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
