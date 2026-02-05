use crate::app::services::tt_context::TtServiceContext;
use crate::app::state::StateHandle;
use crate::infra::db::Database;

pub async fn preload_lang_cache(db: &Database, state: &StateHandle) -> bool {
    db.load_tt_lang_cache().await.is_ok_and(|cache| {
        state.preload_lang_cache(cache);
        true
    })
}

pub async fn preload_tg_cache(db: &Database, state: &StateHandle) -> bool {
    db.load_tt_tg_cache().await.is_ok_and(|cache| {
        state.preload_tg_cache(cache);
        true
    })
}

pub async fn preload_all(db: &Database, state: &StateHandle) -> bool {
    let lang_ok = preload_lang_cache(db, state).await;
    let tg_ok = preload_tg_cache(db, state).await;
    lang_ok && tg_ok
}

pub async fn preload_all_ctx(ctx: &TtServiceContext) -> bool {
    preload_all(&ctx.db, &ctx.state).await
}
