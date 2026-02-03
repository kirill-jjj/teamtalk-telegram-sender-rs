use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use teamtalk::client::ffi;
use teamtalk::types::{ChannelId, Subscriptions, UserId};
use teamtalk::{Client, Event};

fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_or_i32(name: &str, default: i32) -> i32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default)
}

fn recording_path(channel_id: ChannelId) -> PathBuf {
    let mut dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    dir.push("recordings");
    let _ = fs::create_dir_all(&dir);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    dir.push(format!("channel_{}_{}.wav", channel_id.0, stamp));
    dir
}

fn main() -> teamtalk::Result<()> {
    // Read connection settings.
    let host = env_or("TT_HOST", "127.0.0.1");
    let tcp = env_or_i32("TT_TCP", 10333);
    let udp = env_or_i32("TT_UDP", 10333);
    let nickname = env_or("TT_NICK", "RecorderBot");
    let username = env_or("TT_USER", "guest");
    let password = env_or("TT_PASS", "guest");
    let client_name = env_or("TT_CLIENT", "TeamTalkRust");
    let root_channel = ChannelId(1);

    let client = Client::new()?;
    client.connect(&host, tcp, udp, false)?;

    let mut my_id = UserId(0);
    let mut recording = false;
    let mut active_path: Option<PathBuf> = None;

    // Listen for /start and /stop in the root channel and record audio.
    loop {
        if let Some((event, msg)) = client.poll(100) {
            match event {
                Event::ConnectSuccess => {
                    client.login(&nickname, &username, &password, &client_name);
                }
                Event::MySelfLoggedIn => {
                    my_id = client.my_id();
                    client.join_channel(root_channel, "");
                }
                Event::UserJoined => {
                    if let Some(user) = msg.user()
                        && user.channel_id == root_channel
                    {
                        if user.id != my_id {
                            client.subscribe(user.id, Subscriptions::all());
                        } else {
                            for other in client.get_channel_users(root_channel) {
                                if other.id != my_id {
                                    client.subscribe(other.id, Subscriptions::all());
                                }
                            }
                        }
                    }
                }
                Event::UserLeft => {
                    if let Some(user) = msg.user() {
                        client.unsubscribe_all_from_user(user.id);
                    }
                }
                Event::TextMessage => {
                    if let Some(text) = msg.text()
                        && text.msg_type == ffi::TextMsgType::MSGTYPE_CHANNEL
                        && text.channel_id == root_channel
                    {
                        let command = text.text.trim();
                        if command == "/start" {
                            if !recording {
                                let path = recording_path(root_channel);
                                let ok = client.start_recording_channel(
                                    root_channel.0,
                                    path.to_string_lossy().as_ref(),
                                    ffi::AudioFileFormat::AFF_WAVE_FORMAT,
                                );
                                if ok {
                                    recording = true;
                                    active_path = Some(path.clone());
                                    let notice =
                                        format!("recording started: {}", path.to_string_lossy());
                                    client.send_to_channel(root_channel, &notice);
                                } else {
                                    client.send_to_channel(root_channel, "recording failed");
                                }
                            } else {
                                let path_text = active_path
                                    .as_ref()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "recording".to_string());
                                let notice = format!("recording already active: {}", path_text);
                                client.send_to_channel(root_channel, &notice);
                            }
                        } else if command == "/stop" {
                            if recording {
                                let ok = client.stop_recording_channel(root_channel.0);
                                recording = false;
                                let mut notice = String::from("recording stopped");
                                if ok {
                                    if let Some(path) = active_path.take() {
                                        notice = format!(
                                            "recording stopped: {}",
                                            path.to_string_lossy()
                                        );
                                    }
                                } else {
                                    notice = String::from("recording stop failed");
                                }
                                client.send_to_channel(root_channel, &notice);
                            } else {
                                client.send_to_channel(root_channel, "recording not active");
                            }
                        }
                    }
                }
                Event::ConnectionLost | Event::ConnectFailed | Event::ConnectCryptError => {
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
