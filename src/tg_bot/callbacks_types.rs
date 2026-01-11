use crate::types::{LanguageCode, MuteListMode, NotificationSetting};
use anyhow::{Result, anyhow};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum CallbackAction {
    Menu(MenuAction),
    Admin(AdminAction),
    Settings(SettingsAction),
    Subscriber(SubAction),
    Mute(MuteAction),
    Unsub(UnsubAction),
    NoOp,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MenuAction {
    Who,
    Settings,
    Help,
    Unsub,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AdminAction {
    KickList { page: usize },
    KickPerform { user_id: i32 },
    BanList { page: usize },
    BanPerform { user_id: i32 },
    UnbanList { page: usize },
    UnbanPerform { ban_db_id: i64, page: usize },
    SubsList { page: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub enum SettingsAction {
    Main,
    LangSelect,
    LangSet { lang: LanguageCode },
    SubSelect,
    SubSet { setting: NotificationSetting },
    NotifSelect,
    NoonToggle,
    MuteManage,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubAction {
    Details {
        sub_id: i64,
        page: usize,
    },
    Delete {
        sub_id: i64,
        page: usize,
    },
    Ban {
        sub_id: i64,
        page: usize,
    },
    ManageTt {
        sub_id: i64,
        page: usize,
    },
    Unlink {
        sub_id: i64,
        page: usize,
    },
    LinkList {
        sub_id: i64,
        page: usize,
        list_page: usize,
    },
    LinkPerform {
        sub_id: i64,
        page: usize,
        username: String,
    },
    LangMenu {
        sub_id: i64,
        page: usize,
    },
    LangSet {
        sub_id: i64,
        page: usize,
        lang: LanguageCode,
    },
    NotifMenu {
        sub_id: i64,
        page: usize,
    },
    NotifSet {
        sub_id: i64,
        page: usize,
        val: NotificationSetting,
    },
    NoonToggle {
        sub_id: i64,
        page: usize,
    },
    ModeMenu {
        sub_id: i64,
        page: usize,
    },
    ModeSet {
        sub_id: i64,
        page: usize,
        mode: MuteListMode,
    },
    MuteView {
        sub_id: i64,
        page: usize,
        view_page: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum MuteAction {
    ModeSet { mode: MuteListMode },
    Menu { mode: MuteListMode },
    List { page: usize },
    Toggle { username: String, page: usize },
    ServerList { page: usize },
    ServerToggle { username: String, page: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnsubAction {
    Confirm,
    Cancel,
}

impl fmt::Display for CallbackAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallbackAction::Menu(a) => write!(f, "m:{}", a),
            CallbackAction::Admin(a) => write!(f, "a:{}", a),
            CallbackAction::Settings(a) => write!(f, "s:{}", a),
            CallbackAction::Subscriber(a) => write!(f, "u:{}", a),
            CallbackAction::Mute(a) => write!(f, "mt:{}", a),
            CallbackAction::Unsub(a) => write!(f, "x:{}", a),
            CallbackAction::NoOp => write!(f, "noop"),
        }
    }
}

impl fmt::Display for MenuAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuAction::Who => write!(f, "who"),
            MenuAction::Settings => write!(f, "set"),
            MenuAction::Help => write!(f, "hlp"),
            MenuAction::Unsub => write!(f, "uns"),
        }
    }
}

impl fmt::Display for AdminAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdminAction::KickList { page } => write!(f, "kl:{}", page),
            AdminAction::KickPerform { user_id } => write!(f, "kp:{}", user_id),
            AdminAction::BanList { page } => write!(f, "bl:{}", page),
            AdminAction::BanPerform { user_id } => write!(f, "bp:{}", user_id),
            AdminAction::UnbanList { page } => write!(f, "ul:{}", page),
            AdminAction::UnbanPerform { ban_db_id, page } => write!(f, "up:{}:{}", ban_db_id, page),
            AdminAction::SubsList { page } => write!(f, "sl:{}", page),
        }
    }
}

impl fmt::Display for SettingsAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsAction::Main => write!(f, "main"),
            SettingsAction::LangSelect => write!(f, "lsel"),
            SettingsAction::LangSet { lang } => write!(f, "lset:{}", lang.as_str()),
            SettingsAction::SubSelect => write!(f, "ssel"),
            SettingsAction::SubSet { setting } => write!(f, "sset:{}", setting),
            SettingsAction::NotifSelect => write!(f, "nsel"),
            SettingsAction::NoonToggle => write!(f, "noon"),
            SettingsAction::MuteManage => write!(f, "mm"),
        }
    }
}

impl fmt::Display for SubAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubAction::Details { sub_id, page } => write!(f, "det:{}:{}", sub_id, page),
            SubAction::Delete { sub_id, page } => write!(f, "del:{}:{}", sub_id, page),
            SubAction::Ban { sub_id, page } => write!(f, "ban:{}:{}", sub_id, page),
            SubAction::ManageTt { sub_id, page } => write!(f, "mtt:{}:{}", sub_id, page),
            SubAction::Unlink { sub_id, page } => write!(f, "unl:{}:{}", sub_id, page),
            SubAction::LinkList {
                sub_id,
                page,
                list_page,
            } => write!(f, "llst:{}:{}:{}", sub_id, page, list_page),
            SubAction::LinkPerform {
                sub_id,
                page,
                username,
            } => write!(f, "lprf:{}:{}:{}", sub_id, page, username),
            SubAction::LangMenu { sub_id, page } => write!(f, "lmn:{}:{}", sub_id, page),
            SubAction::LangSet { sub_id, page, lang } => {
                write!(f, "lset:{}:{}:{}", sub_id, page, lang.as_str())
            }
            SubAction::NotifMenu { sub_id, page } => write!(f, "nmn:{}:{}", sub_id, page),
            SubAction::NotifSet { sub_id, page, val } => {
                write!(f, "nset:{}:{}:{}", sub_id, page, val)
            }
            SubAction::NoonToggle { sub_id, page } => write!(f, "noon:{}:{}", sub_id, page),
            SubAction::ModeMenu { sub_id, page } => write!(f, "mmn:{}:{}", sub_id, page),
            SubAction::ModeSet { sub_id, page, mode } => {
                write!(f, "mset:{}:{}:{}", sub_id, page, mode)
            }
            SubAction::MuteView {
                sub_id,
                page,
                view_page,
            } => write!(f, "mvw:{}:{}:{}", sub_id, page, view_page),
        }
    }
}

impl fmt::Display for MuteAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MuteAction::ModeSet { mode } => write!(f, "mset:{}", mode),
            MuteAction::Menu { mode } => write!(f, "menu:{}", mode),
            MuteAction::List { page } => write!(f, "lst:{}", page),
            MuteAction::Toggle { username, page } => write!(f, "tgl:{}:{}", page, username),
            MuteAction::ServerList { page } => write!(f, "slst:{}", page),
            MuteAction::ServerToggle { username, page } => write!(f, "stgl:{}:{}", page, username),
        }
    }
}

impl fmt::Display for UnsubAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnsubAction::Confirm => write!(f, "yes"),
            UnsubAction::Cancel => write!(f, "no"),
        }
    }
}

impl FromStr for CallbackAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "noop" {
            return Ok(CallbackAction::NoOp);
        }

        let mut parts = s.splitn(2, ':');
        let prefix = parts.next().ok_or_else(|| anyhow!("Empty callback"))?;
        let rest = parts.next().unwrap_or("");

        match prefix {
            "m" => Ok(CallbackAction::Menu(MenuAction::from_str(rest)?)),
            "a" => Ok(CallbackAction::Admin(AdminAction::from_str(rest)?)),
            "s" => Ok(CallbackAction::Settings(SettingsAction::from_str(rest)?)),
            "u" => Ok(CallbackAction::Subscriber(SubAction::from_str(rest)?)),
            "mt" => Ok(CallbackAction::Mute(MuteAction::from_str(rest)?)),
            "x" => Ok(CallbackAction::Unsub(UnsubAction::from_str(rest)?)),
            _ => Err(anyhow!("Unknown category: {}", prefix)),
        }
    }
}

impl FromStr for MenuAction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "who" => Ok(MenuAction::Who),
            "set" => Ok(MenuAction::Settings),
            "hlp" => Ok(MenuAction::Help),
            "uns" => Ok(MenuAction::Unsub),
            _ => Err(anyhow!("Unknown menu action")),
        }
    }
}

impl FromStr for AdminAction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let cmd = parts.next().ok_or_else(|| anyhow!("No admin cmd"))?;

        match cmd {
            "kl" => Ok(AdminAction::KickList {
                page: parts.next().unwrap_or("0").parse()?,
            }),
            "kp" => Ok(AdminAction::KickPerform {
                user_id: parts.next().ok_or(anyhow!("No ID"))?.parse()?,
            }),
            "bl" => Ok(AdminAction::BanList {
                page: parts.next().unwrap_or("0").parse()?,
            }),
            "bp" => Ok(AdminAction::BanPerform {
                user_id: parts.next().ok_or(anyhow!("No ID"))?.parse()?,
            }),
            "ul" => Ok(AdminAction::UnbanList {
                page: parts.next().unwrap_or("0").parse()?,
            }),
            "up" => {
                let id = parts.next().ok_or(anyhow!("No ID"))?.parse()?;
                let page = parts.next().unwrap_or("0").parse()?;
                Ok(AdminAction::UnbanPerform {
                    ban_db_id: id,
                    page,
                })
            }
            "sl" => Ok(AdminAction::SubsList {
                page: parts.next().unwrap_or("0").parse()?,
            }),
            _ => Err(anyhow!("Unknown admin cmd")),
        }
    }
}

impl FromStr for SettingsAction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let cmd = parts.next().ok_or_else(|| anyhow!("No setting cmd"))?;

        match cmd {
            "main" => Ok(SettingsAction::Main),
            "lsel" => Ok(SettingsAction::LangSelect),
            "lset" => Ok(SettingsAction::LangSet {
                lang: LanguageCode::try_from(parts.next().unwrap_or("en"))
                    .map_err(|e: &'static str| anyhow!(e))?,
            }),
            "ssel" => Ok(SettingsAction::SubSelect),
            "sset" => Ok(SettingsAction::SubSet {
                setting: NotificationSetting::try_from(parts.next().unwrap_or("all"))
                    .map_err(|e: &'static str| anyhow!(e))?,
            }),
            "nsel" => Ok(SettingsAction::NotifSelect),
            "noon" => Ok(SettingsAction::NoonToggle),
            "mm" => Ok(SettingsAction::MuteManage),
            _ => Err(anyhow!("Unknown setting cmd")),
        }
    }
}

impl FromStr for SubAction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let cmd = parts.next().ok_or_else(|| anyhow!("No sub cmd"))?;
        let sub_id: i64 = parts.next().ok_or(anyhow!("No sub_id"))?.parse()?;
        let page: usize = parts.next().unwrap_or("0").parse()?;

        let get_rest =
            |iter: std::str::Split<'_, char>| -> String { iter.collect::<Vec<&str>>().join(":") };

        match cmd {
            "det" => Ok(SubAction::Details { sub_id, page }),
            "del" => Ok(SubAction::Delete { sub_id, page }),
            "ban" => Ok(SubAction::Ban { sub_id, page }),
            "mtt" => Ok(SubAction::ManageTt { sub_id, page }),
            "unl" => Ok(SubAction::Unlink { sub_id, page }),
            "llst" => {
                let list_page = parts.next().unwrap_or("0").parse()?;
                Ok(SubAction::LinkList {
                    sub_id,
                    page,
                    list_page,
                })
            }
            "lprf" => {
                let username = get_rest(parts);
                Ok(SubAction::LinkPerform {
                    sub_id,
                    page,
                    username,
                })
            }
            "lmn" => Ok(SubAction::LangMenu { sub_id, page }),
            "lset" => Ok(SubAction::LangSet {
                sub_id,
                page,
                lang: LanguageCode::try_from(parts.next().unwrap_or("en"))
                    .map_err(|e: &'static str| anyhow!(e))?,
            }),
            "nmn" => Ok(SubAction::NotifMenu { sub_id, page }),
            "nset" => Ok(SubAction::NotifSet {
                sub_id,
                page,
                val: NotificationSetting::try_from(parts.next().unwrap_or("all"))
                    .map_err(|e: &'static str| anyhow!(e))?,
            }),
            "noon" => Ok(SubAction::NoonToggle { sub_id, page }),
            "mmn" => Ok(SubAction::ModeMenu { sub_id, page }),
            "mset" => Ok(SubAction::ModeSet {
                sub_id,
                page,
                mode: MuteListMode::try_from(parts.next().unwrap_or("blacklist"))
                    .map_err(|e: &'static str| anyhow!(e))?,
            }),
            "mvw" => {
                let view_page = parts.next().unwrap_or("0").parse()?;
                Ok(SubAction::MuteView {
                    sub_id,
                    page,
                    view_page,
                })
            }
            _ => Err(anyhow!("Unknown sub cmd")),
        }
    }
}

impl FromStr for MuteAction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let cmd = parts.next().ok_or_else(|| anyhow!("No mute cmd"))?;

        let get_rest =
            |iter: std::str::Split<'_, char>| -> String { iter.collect::<Vec<&str>>().join(":") };

        match cmd {
            "mset" => Ok(MuteAction::ModeSet {
                mode: MuteListMode::try_from(parts.next().unwrap_or("blacklist"))
                    .map_err(|e: &'static str| anyhow!(e))?,
            }),
            "menu" => Ok(MuteAction::Menu {
                mode: MuteListMode::try_from(parts.next().unwrap_or("blacklist"))
                    .map_err(|e: &'static str| anyhow!(e))?,
            }),
            "lst" => Ok(MuteAction::List {
                page: parts.next().unwrap_or("0").parse()?,
            }),
            "tgl" => {
                let page = parts.next().unwrap_or("0").parse()?;
                let username = get_rest(parts);
                Ok(MuteAction::Toggle { username, page })
            }
            "slst" => Ok(MuteAction::ServerList {
                page: parts.next().unwrap_or("0").parse()?,
            }),
            "stgl" => {
                let page = parts.next().unwrap_or("0").parse()?;
                let username = get_rest(parts);
                Ok(MuteAction::ServerToggle { username, page })
            }
            _ => Err(anyhow!("Unknown mute cmd")),
        }
    }
}

impl FromStr for UnsubAction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yes" => Ok(UnsubAction::Confirm),
            "no" => Ok(UnsubAction::Cancel),
            _ => Err(anyhow!("Unknown unsub cmd")),
        }
    }
}
