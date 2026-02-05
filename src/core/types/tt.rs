use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::TtUserId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TtUsername(String);

impl TtUsername {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TtUsername {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TtUsername {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtUsername {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtUsername {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl FromStr for TtUsername {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TtChannelName(String);

impl TtChannelName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TtChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TtChannelName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for TtChannelName {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err("channel name is empty");
        }
        Ok(Self(trimmed.to_string()))
    }
}

impl From<String> for TtChannelName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TtChannelPassword(String);

impl TtChannelPassword {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TtChannelPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtChannelPassword {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtChannelPassword {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for TtChannelPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TtNickname(String);

impl TtNickname {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TtNickname {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtNickname {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtNickname {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for TtNickname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TtHostName(String);

impl TtHostName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TtHostName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtHostName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtHostName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for TtHostName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TtLoginName(String);

impl TtLoginName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TtLoginName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtLoginName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtLoginName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for TtLoginName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TtPassword(String);

impl TtPassword {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TtPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtPassword {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtPassword {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TtClientName(String);

impl TtClientName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TtClientName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtClientName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtClientName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for TtClientName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TtServerName(String);

impl TtServerName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TtServerName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtServerName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtServerName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for TtServerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteUser {
    pub id: TtUserId,
    pub nickname: TtNickname,
    pub username: TtUsername,
    pub channel_name: TtChannelName,
}
