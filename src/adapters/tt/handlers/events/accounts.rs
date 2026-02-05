use crate::adapters::tt::WorkerContext;
use teamtalk::{Client, Message};

pub(super) fn handle_user_account(ctx: &WorkerContext, msg: &Message) {
    if let Some(account) = msg.account() {
        ctx.state.notify_upsert_user_account(account);
    }
}

pub(super) fn handle_user_account_change(client: &Client, ctx: &WorkerContext) {
    ctx.state.notify_clear_user_accounts();
    client.list_user_accounts(0, 1000);
}
