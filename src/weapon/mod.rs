pub mod lance;

use crate::AttackData;

#[derive(Copy, Clone)]
pub enum WeaponButton {
    Light,
    Heavy,
    Special,
}

pub trait Weapon {
    fn name(&self) -> String;

    fn sheathe(&mut self) -> bool;
    fn reset(&mut self);

    fn invoke_attack(
        &mut self,
        button: WeaponButton,
        from: rltk::Point,
        dir: crate::Direction,
    ) -> Option<crate::AttackIntent>;

    fn get_attack_data(&self, button: WeaponButton) -> Option<AttackData>;
}
