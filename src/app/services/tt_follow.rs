use crate::bootstrap::config::{FollowOfflinePolicy, TeamTalkConfig};
use crate::core::types::{TtChannelName, TtChannelPassword};
use crate::infra::db::Database;
use crate::infra::db::app_settings::AppSettingKey;
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct FollowOverride {
    pub enabled: Option<bool>,
    pub offline_policy: Option<FollowOfflinePolicy>,
    pub fallback_channel: ValueOverride<TtChannelName>,
    pub fallback_channel_password: ValueOverride<TtChannelPassword>,
}

#[derive(Debug, Clone, Default)]
pub enum ValueOverride<T> {
    #[default]
    Unchanged,
    Clear,
    Set(T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowSource {
    Config,
    Permanent,
    Session,
}

#[derive(Debug, Clone)]
pub struct EffectiveFollowOwnerConfig {
    pub enabled: bool,
    pub offline_policy: FollowOfflinePolicy,
    pub fallback_channel: Option<TtChannelName>,
    pub fallback_channel_password: Option<TtChannelPassword>,
    pub source: FollowSource,
}

pub async fn load_permanent_override(db: &Database) -> FollowOverride {
    let enabled = match db
        .get_app_setting(AppSettingKey::FollowOwnerEnabledOverride)
        .await
    {
        Ok(val) => parse_bool_opt(val.as_deref()),
        Err(err) => {
            tracing::error!(error = %err, "Failed to load follow owner enabled override");
            None
        }
    };
    let offline_policy = match db
        .get_app_setting(AppSettingKey::FollowOwnerPolicyOverride)
        .await
    {
        Ok(val) => parse_policy_opt(val.as_deref()),
        Err(err) => {
            tracing::error!(error = %err, "Failed to load follow owner policy override");
            None
        }
    };
    let fallback_channel = match db
        .get_app_setting(AppSettingKey::FollowOwnerFallbackChannelOverride)
        .await
    {
        Ok(Some(val)) => parse_opt_channel_value(&val),
        Ok(None) => ValueOverride::Unchanged,
        Err(err) => {
            tracing::error!(error = %err, "Failed to load follow owner fallback channel override");
            ValueOverride::Unchanged
        }
    };
    let fallback_channel_password = match db
        .get_app_setting(AppSettingKey::FollowOwnerFallbackPasswordOverride)
        .await
    {
        Ok(Some(val)) => parse_opt_password_value(&val),
        Ok(None) => ValueOverride::Unchanged,
        Err(err) => {
            tracing::error!(error = %err, "Failed to load follow owner fallback password override");
            ValueOverride::Unchanged
        }
    };

    FollowOverride {
        enabled,
        offline_policy,
        fallback_channel,
        fallback_channel_password,
    }
}

pub async fn save_permanent_override(db: &Database, patch: &FollowOverride) -> Result<()> {
    if let Some(enabled) = patch.enabled {
        db.set_app_setting(
            AppSettingKey::FollowOwnerEnabledOverride,
            if enabled { "1" } else { "0" },
        )
        .await?;
    }
    if let Some(policy) = patch.offline_policy {
        db.set_app_setting(
            AppSettingKey::FollowOwnerPolicyOverride,
            policy_to_str(policy),
        )
        .await?;
    }
    match &patch.fallback_channel {
        ValueOverride::Unchanged => {}
        ValueOverride::Clear => {
            db.set_app_setting(AppSettingKey::FollowOwnerFallbackChannelOverride, "")
                .await?;
        }
        ValueOverride::Set(channel) => {
            db.set_app_setting(
                AppSettingKey::FollowOwnerFallbackChannelOverride,
                channel.as_str(),
            )
            .await?;
        }
    }
    match &patch.fallback_channel_password {
        ValueOverride::Unchanged => {}
        ValueOverride::Clear => {
            db.set_app_setting(AppSettingKey::FollowOwnerFallbackPasswordOverride, "")
                .await?;
        }
        ValueOverride::Set(password) => {
            db.set_app_setting(
                AppSettingKey::FollowOwnerFallbackPasswordOverride,
                password.as_str(),
            )
            .await?;
        }
    }
    Ok(())
}

pub fn resolve_effective_config(
    cfg: &TeamTalkConfig,
    permanent: &FollowOverride,
    session: &FollowOverride,
) -> EffectiveFollowOwnerConfig {
    let has_session = session.enabled.is_some()
        || session.offline_policy.is_some()
        || !matches!(session.fallback_channel, ValueOverride::Unchanged)
        || !matches!(session.fallback_channel_password, ValueOverride::Unchanged);
    let has_permanent = permanent.enabled.is_some()
        || permanent.offline_policy.is_some()
        || !matches!(permanent.fallback_channel, ValueOverride::Unchanged)
        || !matches!(
            permanent.fallback_channel_password,
            ValueOverride::Unchanged
        );

    let source = if has_session {
        FollowSource::Session
    } else if has_permanent {
        FollowSource::Permanent
    } else {
        FollowSource::Config
    };

    let enabled = session
        .enabled
        .or(permanent.enabled)
        .unwrap_or(cfg.follow_owner.enabled);
    let offline_policy = session
        .offline_policy
        .or(permanent.offline_policy)
        .unwrap_or(cfg.follow_owner.offline_policy);
    let fallback_channel = resolve_optional_value(
        cfg.follow_owner.fallback_channel.clone(),
        &permanent.fallback_channel,
        &session.fallback_channel,
    )
    .filter(|v| !v.as_str().trim().is_empty());
    let fallback_channel_password = resolve_optional_value(
        cfg.follow_owner.fallback_channel_password.clone(),
        &permanent.fallback_channel_password,
        &session.fallback_channel_password,
    )
    .filter(|v| !v.as_str().is_empty());

    EffectiveFollowOwnerConfig {
        enabled,
        offline_policy,
        fallback_channel,
        fallback_channel_password,
        source,
    }
}

pub const fn source_to_str(source: FollowSource) -> &'static str {
    match source {
        FollowSource::Config => "config",
        FollowSource::Permanent => "permanent",
        FollowSource::Session => "session",
    }
}

pub const fn policy_to_str(policy: FollowOfflinePolicy) -> &'static str {
    match policy {
        FollowOfflinePolicy::LeaveRoot => "leave_root",
        FollowOfflinePolicy::Stay => "stay",
        FollowOfflinePolicy::FallbackChannel => "fallback_channel",
    }
}

fn parse_bool_opt(raw: Option<&str>) -> Option<bool> {
    let raw = raw?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Some(true),
        "0" | "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

fn parse_policy_opt(raw: Option<&str>) -> Option<FollowOfflinePolicy> {
    let raw = raw?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "leave_root" => Some(FollowOfflinePolicy::LeaveRoot),
        "stay" => Some(FollowOfflinePolicy::Stay),
        "fallback_channel" => Some(FollowOfflinePolicy::FallbackChannel),
        _ => None,
    }
}

fn parse_opt_channel_value(raw: &str) -> ValueOverride<TtChannelName> {
    if raw.trim().is_empty() {
        ValueOverride::Clear
    } else {
        ValueOverride::Set(TtChannelName::from(raw.to_string()))
    }
}

fn parse_opt_password_value(raw: &str) -> ValueOverride<TtChannelPassword> {
    if raw.is_empty() {
        ValueOverride::Clear
    } else {
        ValueOverride::Set(TtChannelPassword::from(raw.to_string()))
    }
}

fn resolve_optional_value<T: Clone>(
    base: Option<T>,
    permanent: &ValueOverride<T>,
    session: &ValueOverride<T>,
) -> Option<T> {
    let from_permanent = match permanent {
        ValueOverride::Unchanged => base,
        ValueOverride::Clear => None,
        ValueOverride::Set(val) => Some(val.clone()),
    };
    match session {
        ValueOverride::Unchanged => from_permanent,
        ValueOverride::Clear => None,
        ValueOverride::Set(val) => Some(val.clone()),
    }
}
