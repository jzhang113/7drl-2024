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
        _ => true,
    };

    let needs_path = match attack {
        LanceAttack::Sweep => true,
        LanceAttack::Hook => true,
        _ => false,
    };

    let name = match attack {
        LanceAttack::DrawAttack => "Suplex",
        LanceAttack::Hook => "Hook",
        LanceAttack::Charge => "Vault",
        LanceAttack::Sweep => "Exploding Bolt",
    }
    .to_string();

    let stam_cost = match attack {
        LanceAttack::DrawAttack => 3,
        LanceAttack::Hook => 4,
        LanceAttack::Charge => 6,
        LanceAttack::Sweep => 2,
    };

    let attack_type = match attack {
        LanceAttack::Sweep => AttackType::ProjectileArea {
            radius: 4,
            explosion_size: 1,
        },
        LanceAttack::DrawAttack => AttackType::AdvancingFlip { range: 2 },
        LanceAttack::Hook => AttackType::Hook { radius: 4 },
        LanceAttack::Charge => AttackType::Dodge { radius: 3 },
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
