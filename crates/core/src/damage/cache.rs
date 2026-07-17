use std::sync::{
    Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockResult, atomic::AtomicUsize,
};
use stochastic::distributions::IrwinHall;

use crate::damage::Damage;

#[derive(Debug, Default)]
pub struct DamageCache {
    cached: Arc<RwLock<Option<IrwinHall>>>,
    last_len: Arc<AtomicUsize>,
}

impl Clone for DamageCache {
    fn clone(&self) -> Self {
        match &*self.cached.read().unwrap() {
            Some(x) => Self {
                cached: Arc::new(RwLock::new(Some(x.clone()))),
                last_len: self.last_len.clone(),
            },
            None => Self {
                cached: Arc::new(RwLock::new(None)),
                last_len: AtomicUsize::new(0).into(),
            },
        }
    }
}

impl PartialEq for DamageCache {
    fn eq(&self, other: &Self) -> bool {
        self.cached.read().unwrap().as_ref().unwrap()
            == other.cached.read().unwrap().as_ref().unwrap()
    }
}

impl Eq for DamageCache {}

impl DamageCache {
    pub fn get_or_compute(&self, history: &[Damage]) -> RwLockReadGuard<'_, Option<IrwinHall>> {
        let last_len: usize;
        {
            let tmp = self.last_len.clone();
            last_len = tmp.load(std::sync::atomic::Ordering::Relaxed);
        }

        if self.cached.read().unwrap().is_none() || last_len != history.len() {
            let mut acc: IrwinHall = Default::default();

            for dmg in history {
                acc = &acc + &dmg.to_irwin_hall();
            }
            let mut guard = self.cached.write().unwrap();

            *guard = Some(acc);

            self.last_len
                .store(history.len(), std::sync::atomic::Ordering::Relaxed);
        }

        self.cached.read().unwrap()
    }

    pub fn append(&mut self, dmg: &Damage) {
        let ih = dmg.to_irwin_hall();
        let mut cached_mut = self.cached.write().unwrap();
        match &mut *cached_mut {
            Some(existing) => *existing += &ih,
            None => *cached_mut = Some(ih),
        }
    }

    pub fn share(&self) -> Self {
        Self {
            cached: self.cached.clone(),
            last_len: self.last_len.clone(),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Option<IrwinHall>> {
        self.cached.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, Option<IrwinHall>> {
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

    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, Option<IrwinHall>>> {
        self.cached.try_read()
    }

    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, Option<IrwinHall>>> {
        self.cached.try_write()
    }
}
