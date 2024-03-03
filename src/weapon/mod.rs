pub mod lance;

pub enum WeaponButton {
    Light,
    Heavy,
    Special,
}

pub trait Weapon {
    fn name(&self) -> String;

    fn sheathe(&mut self) -> bool;
    fn reset(&mut self);

    fn light_attack(
        &mut self,
        from: rltk::Point,
        dir: crate::Direction,
    ) -> Option<crate::AttackIntent>;
    fn heavy_attack(
        &mut self,
        from: rltk::Point,
        dir: crate::Direction,
    ) -> Option<crate::AttackIntent>;
    fn special_attack(
        &mut self,
        from: rltk::Point,
        dir: crate::Direction,
    ) -> Option<crate::AttackIntent>;

    fn can_activate_cost(&self, button: WeaponButton) -> Option<i32>;
    fn attack_name(&self, button: WeaponButton) -> Option<String>;
}
