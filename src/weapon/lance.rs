use crate::{AttackData, AttackType};

#[derive(Copy, Clone)]
pub enum LanceAttack {
    DrawAttack,
    Charge,
    Sweep,
    Hook,
}

pub fn get_attack_data(attack: LanceAttack) -> AttackData {
    let needs_target = match attack {
        LanceAttack::Sweep => true,
        LanceAttack::DrawAttack => true,
        LanceAttack::Hook => true,
        _ => false,
    };

    let needs_path = match attack {
        LanceAttack::Sweep => true,
        LanceAttack::Hook => true,
        _ => false,
    };

    let name = match attack {
        LanceAttack::DrawAttack => "Suplex",
        LanceAttack::Hook => "Hook",
        LanceAttack::Charge => "Walls",
        LanceAttack::Sweep => "Bomb",
    }
    .to_string();

    let stam_cost = match attack {
        LanceAttack::DrawAttack => 1,
        LanceAttack::Hook => 1,
        LanceAttack::Charge => 2,
        LanceAttack::Sweep => 3,
    };

    let attack_type = match attack {
        LanceAttack::Sweep => AttackType::RangedArea {
            radius: 6,
            explosion_size: 2,
        },
        LanceAttack::DrawAttack => AttackType::AdvancingFlip { range: 2 },
        LanceAttack::Hook => AttackType::Hook { radius: 6 },
        LanceAttack::Charge => AttackType::Barrier,
    };

    // TODO: Commenting this out so we don't use the wrong framedata
    // let frame_data = crate::FrameData {
    //     startup: 15,
    //     active: 1,
    //     recovery: 15,
    //     current: 0,
    //     cancelled: false,
    //     linger_time: 10,
    // };

    AttackData {
        needs_target,
        needs_path,
        name,
        stam_cost,
        attack_type,
        // frame_data,
    }
}
