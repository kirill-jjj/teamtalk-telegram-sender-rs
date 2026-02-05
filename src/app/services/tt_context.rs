use crate::app::state::StateHandle;
use crate::infra::db::Database;

#[derive(Clone)]
pub struct TtServiceContext {
    pub db: Database,
    pub state: StateHandle,
}

impl TtServiceContext {
    pub const fn new(db: Database, state: StateHandle) -> Self {
        Self { db, state }
    }
}
