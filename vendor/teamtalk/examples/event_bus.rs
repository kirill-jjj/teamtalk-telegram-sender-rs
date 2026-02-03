use teamtalk::types::UserId;
use teamtalk::{Client, Event};
use teamtalk_sys as ffi;

fn main() -> teamtalk::Result<()> {
    let client = Client::new()?;
    let _sub_id = client
        .on_event(Event::TextMessage)
        .filter_user(UserId(1))
        .filter_text_type(ffi::TextMsgType::MSGTYPE_USER)
        .group("chat-watchers")
        .subscribe(|ctx| {
            if let Some(text) = ctx.text() {
                println!("message from {}: {}", text.from_username, text.text);
            }
        });

    loop {
        let _ = client.poll(100);
    }
}
