use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TelegramId(i64);

impl TelegramId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for TelegramId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TelegramId> for i64 {
    fn from(value: TelegramId) -> Self {
        value.0
    }
}

impl fmt::Display for TelegramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TgChatId(i64);

impl TgChatId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for TgChatId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TgChatId> for i64 {
    fn from(value: TgChatId) -> Self {
        value.0
    }
}

impl fmt::Display for TgChatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TgMessageId(i32);

impl TgMessageId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for TgMessageId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<TgMessageId> for i32 {
    fn from(value: TgMessageId) -> Self {
        value.0
    }
}

impl fmt::Display for TgMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TtUserId(i32);

impl TtUserId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for TtUserId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<TtUserId> for i32 {
    fn from(value: TtUserId) -> Self {
        value.0
    }
}

impl fmt::Display for TtUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TtChannelId(i32);

impl TtChannelId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for TtChannelId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<TtChannelId> for i32 {
    fn from(value: TtChannelId) -> Self {
        value.0
    }
}

impl fmt::Display for TtChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TtMessageId(i64);

impl TtMessageId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for TtMessageId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TtMessageId> for i64 {
    fn from(value: TtMessageId) -> Self {
        value.0
    }
}

impl fmt::Display for TtMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DbBanId(i64);

impl DbBanId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for DbBanId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<DbBanId> for i64 {
    fn from(value: DbBanId) -> Self {
        value.0
    }
}

impl fmt::Display for DbBanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DbReplyQueueId(i64);

impl DbReplyQueueId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for DbReplyQueueId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<DbReplyQueueId> for i64 {
    fn from(value: DbReplyQueueId) -> Self {
        value.0
    }
}

impl fmt::Display for DbReplyQueueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
