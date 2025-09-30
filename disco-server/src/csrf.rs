use axum::extract::{Path, State};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::{Rng, rng};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::time;

use crate::SharedCache;

const EXPIRE_IN_SECS: Duration = Duration::from_secs(1800);

#[derive(Clone)]
pub(crate) struct CsrfCache {
    by_identity: HashMap<String, (String, Instant)>,
    by_state: HashMap<String, String>,
}

impl CsrfCache {
    pub(crate) fn new() -> Self {
        Self {
            by_identity: HashMap::new(),
            by_state: HashMap::new(),
        }
    }
    pub(crate) fn insert(&mut self, id: String, state: String) {
        let instant = Instant::now();
        self.by_identity
            .insert(id.clone(), (state.clone(), instant));
        self.by_state.insert(state, id);
    }

    pub(crate) fn get_by_identity(&self, id: &str) -> Option<(String, Instant)> {
        self.by_identity.get(id).cloned()
    }

    pub(crate) fn take_by_state(&mut self, state: &str) -> Option<String> {
        self.by_state.remove(state).map(|id| {
            self.by_identity.remove(&id);
            id
        })
    }

    pub(crate) fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.by_identity.retain(|_, (state, created)| {
            if now.duration_since(*created) > EXPIRE_IN_SECS {
                self.by_state.remove(state);
                false
            } else {
                true
            }
        });
    }
}

pub(crate) async fn start_cleanup(cache: SharedCache) {
    let mut interval = time::interval(EXPIRE_IN_SECS);
    loop {
        interval.tick().await;
        cache.lock().await.cleanup_expired();
    }
}

pub(crate) async fn get_csrf(
    State(state): State<SharedCache>,
    Path(identity): Path<String>,
) -> String {
    let mut cache = state.lock().await;
    let now = Instant::now();
    if let Some((csrf, timestamp)) = cache.get_by_identity(&identity) {
        if now.duration_since(timestamp) < EXPIRE_IN_SECS {
            return csrf.to_string();
        }
    }
    let csrf = generate_csrf();
    cache.insert(identity, csrf.clone());
    csrf
}

fn generate_csrf() -> String {
    let random_bytes: Vec<u8> = (0..16).map(|_| rng().random::<u8>()).collect();
    BASE64_URL_SAFE_NO_PAD.encode(random_bytes)
}
