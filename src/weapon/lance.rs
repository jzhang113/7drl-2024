use super::{Weapon, WeaponButton};
use crate::{AttackData, AttackIntent, AttackTrait, AttackType};

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

fn get_attack_data(attack: LanceAttack) -> AttackData {
    let needs_target = match attack {
        LanceAttack::Sweep => true,
        _ => false,
    };

    let needs_path = match attack {
        LanceAttack::Sweep => true,
        _ => false,
    };

    let name = match attack {
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
    .to_string();

    let stam_cost = match attack {
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
    };

    let attack_type = match attack {
        LanceAttack::Sweep => AttackType::Bolt { radius: 6 },
        LanceAttack::DrawAttack => AttackType::LanceDraw,
        LanceAttack::Thrust { level } => AttackType::Melee,
        LanceAttack::Charge => AttackType::Melee,
    };

    let traits = match attack {
        LanceAttack::Sweep => vec![AttackTrait::FollowsPath],
        LanceAttack::DrawAttack => vec![AttackTrait::Damage { amount: 1 }],
        LanceAttack::Thrust { level } => vec![AttackTrait::Damage {
            amount: level as i32,
        }],
        LanceAttack::Charge => vec![AttackTrait::Damage { amount: 1 }],
    };

    let frame_data = crate::FrameData {
        startup: 3,
        active: 1,
        recovery: 6,
    };

    AttackData {
        needs_target,
        needs_path,
        name,
        stam_cost,
        attack_type,
        traits,
        frame_data,
    }
}

fn get_attack_intent(
    attack: LanceAttack,
    from_point: rltk::Point,
    dir: crate::Direction,
) -> AttackIntent {
    let source_point = crate::direction::Direction::point_in_direction(from_point, dir);
    let frame_data = crate::FrameData {
        startup: 15,
        active: 15,
        recovery: 15,
    };

    match attack {
        LanceAttack::DrawAttack => AttackIntent {
            main: AttackType::LanceDraw,
            modifier: None,
            loc: source_point,
            frame_data,
        },
        LanceAttack::Thrust { level } => AttackIntent {
            main: AttackType::LanceThrust {
                level,
                dest: crate::direction::Direction::point_in_direction(source_point, dir),
            },
            modifier: None,
            loc: from_point,
            frame_data,
        },
        LanceAttack::Charge => AttackIntent {
            main: AttackType::LanceCharge { dir },
            modifier: None,
            loc: from_point,
            frame_data,
        },
        LanceAttack::Sweep => AttackIntent {
            main: AttackType::Bolt { radius: 4 },
            modifier: None,
            loc: from_point,
            frame_data,
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
        true
    }

    fn reset(&mut self) {
        if self.state != LanceState::Sheathed {
            self.state = LanceState::Ready;
        }
    }

    fn invoke_attack(
        &mut self,
        button: WeaponButton,
        from: rltk::Point,
        dir: crate::Direction,
    ) -> Option<AttackIntent> {
        if let Some((attack, next_state)) = self.next_state(button) {
            self.state = next_state;
            Some(get_attack_intent(attack, from, dir))
        } else {
            None
        }
    }

    fn get_attack_data(&self, button: WeaponButton) -> Option<AttackData> {
        self.next_state(button)
            .map(|(attack, _)| get_attack_data(attack))
    }
}
