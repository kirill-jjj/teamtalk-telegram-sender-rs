use crate::adapters::tt::WorkerContext;
use crate::adapters::tt::handlers::commands;
use teamtalk::{Client, Message};

pub(super) fn handle_text_message_event(client: &Client, ctx: &WorkerContext, msg: &Message) {
    if let Some(txt_msg) = msg.text() {
        commands::handle_text_message(client, ctx, &txt_msg);
    }
}
