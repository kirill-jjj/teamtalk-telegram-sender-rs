use super::{
    JoinGender, LanguageCode, NotificationType, TgChatId, TgMessageId, TtChannelId, TtChannelName,
    TtNickname, TtServerName, TtUserId, TtUsername,
};
use std::path::PathBuf;

#[derive(Debug)]
pub enum BridgeEvent {
    Broadcast {
        event_type: NotificationType,
        nickname: TtNickname,
        server_name: TtServerName,
        related_tt_username: TtUsername,
        gender: JoinGender,
    },
    ToAdmin {
        user_id: TtUserId,
        nick: TtNickname,
        tt_username: TtUsername,
        msg_content: String,
        server_name: TtServerName,
    },
    ToAdminChannel {
        channel_id: TtChannelId,
        channel_name: TtChannelName,
        server_name: TtServerName,
        msg_content: String,
    },
    WhoReport {
        chat_id: TgChatId,
        text: String,
        reply_to: Option<TgMessageId>,
    },
}

#[derive(Debug)]
pub enum TtCommand {
    Shutdown,
    Broadcast {
        text: String,
    },
    ReplyToUser {
        user_id: TtUserId,
        text: String,
    },
    SendToChannel {
        channel_id: TtChannelId,
        text: String,
    },
    EnqueueStream {
        channel_id: TtChannelId,
        file_path: PathBuf,
        duration_ms: u32,
        announce_text: Option<String>,
    },
    StopStreamingIf {
        stream_id: u64,
    },
    SkipStream,
    SetStreamingStatus {
        streaming: bool,
    },
    KickUser {
        user_id: TtUserId,
    },
    BanUser {
        user_id: TtUserId,
    },
    Who {
        chat_id: TgChatId,
        lang: LanguageCode,
        reply_to: Option<TgMessageId>,
    },
    LoadAccounts,
}
