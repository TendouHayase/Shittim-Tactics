use std::sync::{
    Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockResult, atomic::AtomicUsize,
};
use stochastic::dist::HitBag;

use crate::damage::Damage;

#[derive(Debug, Default)]
pub struct DamageCache {
    cached: Arc<RwLock<HitBag>>,
    last_len: Arc<AtomicUsize>,
}

impl Clone for DamageCache {
    fn clone(&self) -> Self {
        Self {
            cached: Arc::new(RwLock::new(self.cached.read().unwrap().clone())),
            last_len: self.last_len.clone(),
        }
    }
}

impl PartialEq for DamageCache {
    fn eq(&self, other: &Self) -> bool {
        *self.cached.read().unwrap() == *other.cached.read().unwrap()
    }
}

impl Eq for DamageCache {}

impl DamageCache {
    pub fn get_or_compute(&self, history: &[Damage]) -> RwLockReadGuard<'_, HitBag> {
        let last_len = self.last_len.load(std::sync::atomic::Ordering::Relaxed);

        if last_len != history.len() {
            let mut acc = HitBag::default();
            for dmg in history {
                acc.push(dmg.to_hit());
            }

            *self.cached.write().unwrap() = acc;
            self.last_len
                .store(history.len(), std::sync::atomic::Ordering::Relaxed);
        }

        self.cached.read().unwrap()
    }

    pub fn append(&mut self, dmg: &Damage) {
        self.cached.write().unwrap().push(dmg.to_hit());
        self.last_len
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn share(&self) -> Self {
        Self {
            cached: self.cached.clone(),
            last_len: self.last_len.clone(),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, HitBag> {
        self.cached.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, HitBag> {
        self.cached.write().unwrap()
    }

    pub fn try_clone(&self) -> Result<Self, error::Error> {
        self.cached
            .read()
            .map(|guard| DamageCache {
                cached: Arc::new(RwLock::new(guard.clone())),
                last_len: self.last_len.clone(),
            })
            .map_err(|err| err.into())
    }

    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, HitBag>> {
        self.cached.try_read()
    }

    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, HitBag>> {
        self.cached.try_write()
    }
}
