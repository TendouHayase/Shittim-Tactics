use stochastic::distributions::IrwinHall;

use crate::damage::Damage;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DamageCache {
    cached: Option<IrwinHall>,
    last_len: usize,
}

impl DamageCache {
    pub fn get_or_compute(&mut self, history: &[Damage]) -> &IrwinHall {
        if self.cached.is_none() || self.last_len != history.len() {
            let mut acc: Option<IrwinHall> = None;

            for dmg in history {
                let ih = dmg.to_irwin_hall();
                acc = Some(match acc {
                    None => ih,
                    Some(existing) => &existing + &ih,
                })
            }

            self.cached = acc;
            self.last_len = history.len();
        }

        self.cached.as_ref().unwrap()
    }

    pub fn append(&mut self, dmg: &Damage) {
        let ih = dmg.to_irwin_hall();
        match &mut self.cached {
            Some(existing) => *existing += &ih,
            None => self.cached = Some(ih),
        }
    }
}
