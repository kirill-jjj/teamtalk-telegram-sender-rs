use crate::core::types::{LanguageCode, TelegramId, TtUsername};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct CacheStore {
    tt_lang_cache: HashMap<TtUsername, LanguageCode>,
    tt_tg_cache: HashMap<TtUsername, TelegramId>,
    stats: CacheStats,
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            tt_lang_cache: HashMap::new(),
            tt_tg_cache: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    pub fn preload_lang(&mut self, cache: HashMap<TtUsername, LanguageCode>) {
        self.tt_lang_cache = cache;
    }

    pub fn preload_tg(&mut self, cache: HashMap<TtUsername, TelegramId>) {
        self.tt_tg_cache = cache;
    }

    pub fn get_lang(&self, username: &TtUsername) -> Option<LanguageCode> {
        let val = self.tt_lang_cache.get(username).copied();
        if val.is_some() {
            self.stats.lang_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.lang_misses.fetch_add(1, Ordering::Relaxed);
        }
        val
    }

    pub fn get_tg(&self, username: &TtUsername) -> Option<TelegramId> {
        let val = self.tt_tg_cache.get(username).copied();
        if val.is_some() {
            self.stats.tg_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.tg_misses.fetch_add(1, Ordering::Relaxed);
        }
        val
    }

    pub fn set_lang(&mut self, username: TtUsername, lang: LanguageCode) {
        if self.tt_lang_cache.len() > 5000 {
            self.tt_lang_cache.clear();
        }
        self.tt_lang_cache.insert(username, lang);
    }

    pub fn set_tg(&mut self, username: TtUsername, tg_id: TelegramId) {
        if self.tt_tg_cache.len() > 5000 {
            self.tt_tg_cache.clear();
        }
        self.tt_tg_cache.insert(username, tg_id);
    }

    pub fn snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            lang_hits: self.stats.lang_hits.load(Ordering::Relaxed),
            lang_misses: self.stats.lang_misses.load(Ordering::Relaxed),
            tg_hits: self.stats.tg_hits.load(Ordering::Relaxed),
            tg_misses: self.stats.tg_misses.load(Ordering::Relaxed),
        }
    }
}

#[derive(Default)]
struct CacheStats {
    lang_hits: AtomicU64,
    lang_misses: AtomicU64,
    tg_hits: AtomicU64,
    tg_misses: AtomicU64,
}

#[derive(Default, Clone, Copy)]
pub struct CacheStatsSnapshot {
    pub lang_hits: u64,
    pub lang_misses: u64,
    pub tg_hits: u64,
    pub tg_misses: u64,
}
