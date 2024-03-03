use super::{Weapon, WeaponButton};
use crate::{AttackIntent, AttackType};

pub struct Lance {
    state: LanceState,
    level: u8,
}

#[derive(PartialEq)]
enum LanceState {
    Sheathed,
    Ready,
    Attack { prev_count: u8 },
    Running,
    Counter,
    Guard,
    Wait,
}

enum LanceAttack {
    DrawAttack,
    Thrust { level: u8 },
    Charge,
    Sweep,
}

fn get_attack_name(attack: LanceAttack) -> String {
    match attack {
        LanceAttack::DrawAttack => "Draw Atk",
        LanceAttack::Thrust { level } => match level {
            1 => "Stab I",
            2 => "Stab II",
            3 => "Stab III",
            4 => "Charge Stab",
            _ => unreachable!(),
        },
        LanceAttack::Charge => "Charge",
        LanceAttack::Sweep => "Sweep",
    }
    .to_string()
}

fn get_attack_stamina_req(attack: LanceAttack) -> i32 {
    match attack {
        LanceAttack::DrawAttack => 1,
        LanceAttack::Thrust { level } => match level {
            1 => 2,
            2 => 3,
            3 => 3,
            4 => 0,
            _ => unreachable!(),
        },
        LanceAttack::Charge => 2,
        LanceAttack::Sweep => 3,
    }
}

fn get_attack_intent(
    attack: LanceAttack,
    from_point: rltk::Point,
    dir: crate::Direction,
) -> AttackIntent {
    let source_point = crate::direction::Direction::point_in_direction(from_point, dir);

    match attack {
        LanceAttack::DrawAttack => AttackIntent {
            main: AttackType::LanceDraw,
            modifier: None,
            loc: source_point,
            delay: 0,
        },
        LanceAttack::Thrust { level } => AttackIntent {
            main: AttackType::LanceThrust {
                level,
                dest: crate::direction::Direction::point_in_direction(source_point, dir),
            },
            modifier: None,
            loc: from_point,
            delay: 0,
        },
        LanceAttack::Charge => AttackIntent {
            main: AttackType::LanceCharge { dir },
            modifier: None,
            loc: from_point,
            delay: 0,
        },
        LanceAttack::Sweep => AttackIntent {
            main: AttackType::LanceSweep,
            modifier: None,
            loc: from_point,
            delay: 1,
        },
    }
}

impl Lance {
    pub fn new() -> Self {
        Self {
            state: LanceState::Sheathed,
            level: 0,
        }
    }

    fn next_state(&self, button: WeaponButton) -> Option<(LanceAttack, LanceState)> {
        match button {
            WeaponButton::Light => self.next_light_state(),
            WeaponButton::Heavy => self.next_heavy_state(),
            WeaponButton::Special => self.next_special_state(),
        }
    }

    fn next_light_state(&self) -> Option<(LanceAttack, LanceState)> {
        match self.state {
            // draw attack
            LanceState::Sheathed => Some((
                LanceAttack::DrawAttack,
                LanceState::Attack { prev_count: 1 },
            )),
            // mid thrust 1
            LanceState::Ready => Some((
                LanceAttack::Thrust { level: 1 },
                LanceState::Attack { prev_count: 1 },
            )),
            // mid thrust 2 / 3
            LanceState::Attack { prev_count } => {
                if prev_count < 3 {
                    Some((
                        LanceAttack::Thrust {
                            level: prev_count + 1,
                        },
                        LanceState::Attack {
                            prev_count: prev_count + 1,
                        },
                    ))
                } else {
                    None
                }
            }
            // final thrust
            LanceState::Running => Some((
                LanceAttack::Thrust { level: 4 },
                LanceState::Attack { prev_count: 1 },
            )),
            LanceState::Counter => None,
            // guard thrust
            LanceState::Guard => Some((
                LanceAttack::Thrust { level: 1 },
                LanceState::Attack { prev_count: 1 },
            )),
            _ => None,
        }
    }

    fn next_heavy_state(&self) -> Option<(LanceAttack, LanceState)> {
        match self.state {
            // sweep
            LanceState::Ready => Some((LanceAttack::Sweep, LanceState::Wait)),
            LanceState::Attack { prev_count } => {
                if prev_count < 3 {
                    Some((LanceAttack::Sweep, LanceState::Wait))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn next_special_state(&self) -> Option<(LanceAttack, LanceState)> {
        match self.state {
            LanceState::Ready => Some((LanceAttack::Charge, LanceState::Running)),
            _ => None,
        }
    }
}

impl Weapon for Lance {
    fn name(&self) -> String {
        format!("Lance+{}", self.level)
    }

    fn sheathe(&mut self) -> bool {
        if self.state == LanceState::Sheathed {
            return false;
        }

        self.state = LanceState::Sheathed;
        return true;
    }

    fn reset(&mut self) {
        if self.state != LanceState::Sheathed {
            self.state = LanceState::Ready;
        }
    }

    fn light_attack(&mut self, from: rltk::Point, dir: crate::Direction) -> Option<AttackIntent> {
        if let Some((attack, next_state)) = self.next_state(WeaponButton::Light) {
            self.state = next_state;
            Some(get_attack_intent(attack, from, dir))
        } else {
            None
        }
    }

    fn heavy_attack(&mut self, from: rltk::Point, dir: crate::Direction) -> Option<AttackIntent> {
        if let Some((attack, next_state)) = self.next_state(WeaponButton::Heavy) {
            self.state = next_state;
            Some(get_attack_intent(attack, from, dir))
        } else {
            None
        }
    }

    fn special_attack(&mut self, from: rltk::Point, dir: crate::Direction) -> Option<AttackIntent> {
        if let Some((attack, next_state)) = self.next_state(WeaponButton::Special) {
            self.state = next_state;
            Some(get_attack_intent(attack, from, dir))
        } else {
            None
        }
    }

    fn can_activate_cost(&self, button: WeaponButton) -> Option<i32> {
        self.next_state(button)
            .map(|(attack, _)| get_attack_stamina_req(attack))
    }

    fn attack_name(&self, button: WeaponButton) -> Option<String> {
        self.next_state(button)
            .map(|(attack, _)| get_attack_name(attack))
    }
}
