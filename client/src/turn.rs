use crate::attack::Attack;

pub struct Turn {
    enemies: Vec<String>,
    attacks: Vec<Attack>,
    current: usize,
}

impl Turn {
    pub fn new(enemies: Vec<String>, attacks: usize) -> Turn {
        Turn {
            enemies, attacks: Vec::with_capacity(attacks), current: 0
        }
    }
}

impl Iterator for Turn {
    type Item = &mut Attack;

    fn next(&mut self) -> Option<Self::Item> {
        let attack = self.attacks.get_mut(self.current);
        self.current += 1;
        attack
    }
}
