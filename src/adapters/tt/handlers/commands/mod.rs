use crate::adapters::tt::WorkerContext;
use teamtalk::Client;
use teamtalk::types::TextMessage;

mod admin;
mod bridge;
mod channel;
mod help;
mod plugins;
mod queue;
mod skip;
mod sub;
mod user;

pub(super) fn handle_text_message(client: &Client, ctx: &WorkerContext, msg: &TextMessage) {
    if msg.from_id == client.my_id() {
        return;
    }

    if msg.msg_type == teamtalk::client::ffi::TextMsgType::MSGTYPE_CHANNEL {
        channel::handle_channel_message(client, ctx, msg);
        return;
    }

    user::handle_user_message(client, ctx, msg);
}
