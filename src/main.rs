#[macro_use]
extern crate lazy_static;

rltk::embedded_resource!(FONT, "../resources/Zilk-16x16.png");
rltk::embedded_resource!(ICONS, "../resources/custom_icons.png");

use crate::map_builder::MapBuilderArgs;
use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod attack_type;
mod camera;
mod colors;
mod components;
mod data;
mod direction;
mod gamelog;
mod gui;
mod inventory;
mod map;
mod map_builder;
mod monster_part;
mod player;
mod range_type;
mod spawn;
mod sys_ai;
mod sys_attack;
mod sys_death;
mod sys_frame_data;
mod sys_mapindex;
mod sys_movement;
mod sys_partbreak;
mod sys_particle;
mod sys_partmove;
mod sys_pickup;
mod sys_projectile;
mod sys_push;
mod sys_spawner;
mod sys_stun;
mod sys_trap_ai;
mod sys_turn;
mod sys_visibility;
mod weapon;

pub mod consts;

pub use attack_type::*;
pub use camera::*;
pub use colors::*;
pub use components::*;
pub use direction::Direction;
pub use map::{Map, TileType};
pub use monster_part::*;
pub use range_type::*;
pub use spawn::info::SpawnInfo;
pub use sys_ai::{Behavior, NextIntent};
pub use sys_particle::{ParticleBuilder, ParticleRequest};
pub use sys_spawner::{SpawnRequest, SpawnType, Spawner};

use gamelog::GameLog;

#[derive(PartialEq, Copy, Clone)]
pub enum TargettingValid {
    All,
    Unblocked,
    Occupied,
    None,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    Targetting {
        attack_type: AttackType,
        cursor_point: rltk::Point,
        validity_mode: TargettingValid,
        show_path: bool,
    },
    ViewEnemy {
        index: u32,
    },
    Running,
    HitPause {
        remaining_time: f32,
    },
    GenerateLevel,
    ChangeMap,
    Shop,
    Blacksmith,
    Dead {
        success: bool,
    },
    Charging {
        dir: crate::Direction,
        speed: u8,
    },
    AbilitySelect {
        index: usize,
    },
    InventorySelect {
        index: usize,
    },
}

pub struct State {
    ecs: World,
    tick: i32,
    tab_targets: Vec<rltk::Point>,
    tab_index: usize,
    attack_modifier: Option<AttackType>,
    player_inventory: inventory::Inventory,
    player_charging: (bool, crate::Direction, u8, bool),
    player_abilities: Vec<AttackData>,
    max_cleared_level: i32,
}

impl State {
    fn register_components(&mut self) {
        self.ecs.register::<Position>();
        self.ecs.register::<Renderable>();
        self.ecs.register::<Player>();
        self.ecs.register::<Viewshed>();
        self.ecs.register::<CanActFlag>();
        self.ecs.register::<CanReactFlag>();
        self.ecs.register::<Schedulable>();
        self.ecs.register::<ParticleLifetime>();
        self.ecs.register::<BlocksTile>();
        self.ecs.register::<BlocksVision>();
        self.ecs.register::<Viewable>();
        self.ecs.register::<ViewableIndex>();

        self.ecs.register::<Health>();
        self.ecs.register::<Stamina>();
        self.ecs.register::<AttackIntent>();
        self.ecs.register::<MoveIntent>();
        self.ecs.register::<PartMoveIntent>();
        self.ecs.register::<Moveset>();
        self.ecs.register::<AttackPath>();
        self.ecs.register::<FrameData>();

        self.ecs.register::<AttackInProgress>();
        self.ecs.register::<BlockAttack>();
        self.ecs.register::<AiState>();
        self.ecs.register::<TrapAiState>();

        self.ecs.register::<Heal>();
        self.ecs.register::<Item>();
        self.ecs.register::<Openable>();
        self.ecs.register::<Fragile>();

        self.ecs.register::<MultiTile>();
        self.ecs.register::<Facing>();

        self.ecs.register::<PushForce>();
        self.ecs.register::<Npc>();
        self.ecs.register::<Invulnerable>();
        self.ecs.register::<Stunned>();
        self.ecs.register::<MissionTarget>();
    }

    fn new_game(&mut self) {
        self.register_components();

        self.ecs.insert(RunState::Running);
        self.ecs.insert(sys_particle::ParticleBuilder::new());
        self.ecs.insert(sys_spawner::Spawner::new());

        let mut rng = rltk::RandomNumberGenerator::new();

        // Add a dummy map and player to the ecs
        let map = Map::new(1, 1, &"Dummy".to_string(), &"#FFFFFF".to_string(), &mut rng);
        self.ecs.insert(map);

        let player = spawn::spawner::build_player(&mut self.ecs, rltk::Point::new(0, 0));
        self.ecs.insert(player);
        self.ecs.insert(rng);

        let log = gamelog::GameLog {
            entries: Vec::new(),
            dirty: false,
        };
        self.ecs.insert(log);

        self.load_overworld();
    }

    fn run_systems(&mut self) -> RunState {
        self.tick += 1;

        sys_trap_ai::TrapAiSystem.run_now(&self.ecs);
        sys_ai::AiSystem.run_now(&self.ecs);
        sys_turn::TurnSystem.run_now(&self.ecs);

        sys_frame_data::FrameDataSystem.run_now(&self.ecs);
        sys_attack::AttackSystem.run_now(&self.ecs);
        sys_movement::MovementSystem.run_now(&self.ecs);
        sys_projectile::ProjectileSystem.run_now(&self.ecs);
        sys_stun::StunSystem.run_now(&self.ecs);

        // ensure indexes are correct before handling part movements
        sys_mapindex::MapIndexSystem.run_now(&self.ecs);

        sys_partmove::PartMoveSystem.run_now(&self.ecs);
        sys_partbreak::PartBreakSystem.run_now(&self.ecs);

        // re-index because part movements may have changed blocked tiles
        sys_mapindex::MapIndexSystem.run_now(&self.ecs);
        sys_push::PushSystem.run_now(&self.ecs);

        // pickups happen after movement
        sys_pickup::PickupSystem.run_now(&self.ecs);

        // death needs to run after attacks so bodies are cleaned up
        sys_death::DeathSystem.run_now(&self.ecs);

        sys_spawner::SpawnSystem.run_now(&self.ecs);
        sys_visibility::VisibilitySystem.run_now(&self.ecs);
        sys_particle::ParticleSpawnSystem.run_now(&self.ecs);

        self.ecs.maintain();

        RunState::Running
    }

    fn entities_need_cleanup(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();

        let mut to_delete = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn load_overworld(&mut self) {
        self.new_level(Some(MapBuilderArgs {
            builder_type: 4,
            width: 20,
            height: 20,
            name: "Base".to_string(),
            map_color: "#D4BF8E".to_string(),
        }))
    }

    fn new_level(&mut self, map_builder_args: Option<MapBuilderArgs>) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_need_cleanup();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        let is_overworld = map_builder_args.is_some();
        let mut map_builder = if let Some(args) = map_builder_args {
            map_builder::with_builder(&args)
        } else {
            map_builder::random_builder(80, 50, "-".to_string())
        };

        let new_map = {
            let mut rng = self.ecs.fetch_mut::<rltk::RandomNumberGenerator>();
            map_builder.build_map(&mut rng);
            map_builder.build_data.map.clone()
        };

        {
            // update player position
            let player = self.ecs.fetch::<Entity>();
            let mut positions = self.ecs.write_storage::<Position>();
            let player_pos = positions
                .get_mut(*player)
                .expect("player didn't have a position");

            let new_player_pos = map_builder.build_data.starting_position;
            player_pos.x = new_player_pos.x;
            player_pos.y = new_player_pos.y;

            // Mark the player's visibility as dirty
            let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
            let vs = viewshed_components.get_mut(*player);
            if let Some(vs) = vs {
                vs.dirty = true;
            }

            // replace map
            let mut map_writer = self.ecs.write_resource::<Map>();
            *map_writer = new_map;
        }

        // TODO: handle spawning as a meta map builder
        // fill the map
        if is_overworld {
            map_builder.spawn_overworld(&mut self.ecs);
        } else {
            map_builder.spawn_entities(&mut self.ecs);
        }
    }

    fn reset_player(&mut self) {
        let player = self.ecs.fetch::<Entity>();
        let mut healths = self.ecs.write_storage::<Health>();
        let player_healths = healths
            .get_mut(*player)
            .expect("player didn't have a health");

        player_healths.current = player_healths.max;
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // cleanup
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        sys_particle::cleanup_particles(&mut self.ecs, ctx);

        let mut next_status;
        let player_point;

        // wrapping to limit borrowed lifetimes
        {
            let player = self.ecs.fetch::<Entity>();
            let positions = self.ecs.read_storage::<Position>();
            let player_pos = positions
                .get(*player)
                .expect("player didn't have a position");
            player_point = rltk::Point::new(player_pos.x, player_pos.y);

            // get the current RunState
            next_status = *self.ecs.fetch::<RunState>();
        }

        // draw map + gui
        gui::map::draw_all(&self.ecs, ctx);

        // non-map elements
        gui::sidebar::draw_sidebar(&self, ctx);
        gui::log::update_log_text(&self.ecs, ctx);

        match next_status {
            RunState::AwaitingInput => {
                next_status = player::player_input(self, ctx);

                if next_status == RunState::Running {
                    player::end_turn_cleanup(&mut self.ecs);
                }
            }
            RunState::Charging { dir, speed } => {
                self.player_charging = (true, dir, speed, false);
                next_status = RunState::Running
            }
            RunState::Targetting {
                attack_type,
                cursor_point,
                validity_mode,
                show_path,
            } => {
                let range_type = crate::attack_type::get_attack_range(attack_type);
                let tiles_in_range = crate::range_type::resolve_range_at(&range_type, player_point);

                let result = player::ranged_target(
                    self,
                    ctx,
                    cursor_point,
                    tiles_in_range,
                    validity_mode,
                    show_path,
                );
                match result.0 {
                    player::SelectionResult::Canceled => {
                        next_status = RunState::AwaitingInput;
                    }
                    player::SelectionResult::NoResponse => {
                        if let Some(new_cursor) = result.1 {
                            next_status = RunState::Targetting {
                                attack_type,
                                cursor_point: new_cursor,
                                validity_mode,
                                show_path,
                            }
                        }
                    }
                    player::SelectionResult::Selected => {
                        {
                            // we should generally have a target at this point
                            // if we don't have a point, assume its because we won't need one later
                            let target = result.1.unwrap_or(rltk::Point::zero());

                            let mut attacks = self.ecs.write_storage::<AttackIntent>();
                            let mut frames = self.ecs.write_storage::<FrameData>();
                            let player = self.ecs.fetch::<Entity>();

                            attacks
                                .insert(*player, get_attack_intent(attack_type, target, None))
                                .ok();

                            frames.insert(*player, get_frame_data(attack_type)).ok();

                            // TODO: remove attack_modifier
                            self.attack_modifier = None;
                        }

                        next_status = RunState::Running;
                        player::end_turn_cleanup(&mut self.ecs);
                    }
                }
            }
            RunState::ViewEnemy { index } => {
                next_status = player::view_input(self, ctx, index);
            }
            RunState::Running => {
                self.run_systems();
                std::thread::sleep(std::time::Duration::from_millis(1));
                next_status = *self.ecs.fetch::<RunState>();
            }
            RunState::HitPause { remaining_time } => {
                sys_particle::ParticleSpawnSystem.run_now(&self.ecs);

                let new_time = remaining_time - ctx.frame_time_ms;
                if new_time < 0.0 {
                    next_status = RunState::Running;
                } else {
                    next_status = RunState::HitPause {
                        remaining_time: new_time,
                    }
                }
            }
            RunState::GenerateLevel => {
                self.new_level(None);
                sys_visibility::VisibilitySystem.run_now(&self.ecs);

                next_status = RunState::AwaitingInput;
            }
            RunState::ChangeMap => {
                // TODO: support multi-level maps
                unreachable!();
                // update visibility immediately so the screen isn't dark for a cycle
                // sys_visibility::VisibilitySystem.run_now(&self.ecs);
                // next_status = RunState::AwaitingInput;
            }
            RunState::Dead { success } => match ctx.key {
                None => {}
                Some(key) => {
                    if key == rltk::VirtualKeyCode::R {
                        self.load_overworld();
                        self.reset_player();

                        next_status = RunState::Running;
                    }
                }
            },
            RunState::Shop => {
                gui::overworld::draw_shop(ctx);
                match ctx.key {
                    None => {}
                    Some(key) => {
                        if key == rltk::VirtualKeyCode::Escape {
                            next_status = RunState::Running;
                        }
                    }
                }
            }
            RunState::Blacksmith => {
                gui::overworld::draw_upgrades(ctx);
                match ctx.key {
                    None => {}
                    Some(key) => {
                        if key == rltk::VirtualKeyCode::Escape {
                            next_status = RunState::Running;
                        }
                    }
                }
            }
            RunState::AbilitySelect { index } => {
                if self.player_abilities.is_empty() {
                    let mut log = self.ecs.fetch_mut::<GameLog>();
                    log.add("You know no abilities");
                    next_status = RunState::Running;
                } else {
                    gui::ability_select::draw_abilities(self, ctx, index);
                    next_status = player::ability_select_input(self, ctx, index);

                    if next_status == RunState::Running {
                        player::end_turn_cleanup(&mut self.ecs);
                    }
                }
            }
            RunState::InventorySelect { index } => {
                if self.player_inventory.consumables.is_empty() {
                    let mut log = self.ecs.fetch_mut::<GameLog>();
                    log.add("You have no items");
                    next_status = RunState::Running;
                } else {
                    gui::inventory::draw_inventory(self, ctx, index);
                    next_status = player::inventory_select_input(self, ctx, index);

                    if next_status == RunState::Running {
                        player::end_turn_cleanup(&mut self.ecs);
                    }
                }
            }
        }

        let mut status_writer = self.ecs.write_resource::<RunState>();
        *status_writer = next_status;
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    rltk::link_resource!(FONT, "resources/Zilk-16x16.png");
    rltk::link_resource!(ICONS, "resources/custom_icons.png");

    let context = RltkBuilder::simple(gui::consts::CONSOLE_WIDTH, gui::consts::CONSOLE_HEIGHT)?
        .with_title("arenarl")
        .with_font("Zilk-16x16.png", 16, 16)
        .with_font("custom_icons.png", 16, 16)
        .with_tile_dimensions(16, 16)
        .with_simple_console_no_bg(
            gui::consts::CONSOLE_WIDTH,
            gui::consts::CONSOLE_HEIGHT,
            "Zilk-16x16.png",
        ) // main layer
        .with_sparse_console_no_bg(
            gui::consts::CONSOLE_WIDTH,
            gui::consts::CONSOLE_HEIGHT,
            "custom_icons.png",
        ) // custom icons
        .with_sparse_console_no_bg(
            gui::consts::CONSOLE_WIDTH,
            gui::consts::CONSOLE_HEIGHT,
            "Zilk-16x16.png",
        ) // control line
        .build()
        .expect("Failed to build console");

    let mut gs = State {
        ecs: World::new(),
        tick: 0,
        tab_targets: Vec::new(),
        tab_index: 0,
        attack_modifier: None,
        player_inventory: inventory::Inventory::new(),
        player_charging: (false, crate::Direction::N, 0, false),
        player_abilities: pabb(),
        max_cleared_level: 0,
    };

    gs.new_game();

    rltk::main_loop(context, gs)
}

use crate::weapon::lance::LanceAttack;
fn pabb() -> Vec<AttackData> {
    let mut attacks = Vec::new();
    attacks.push(crate::weapon::lance::get_attack_data(
        LanceAttack::DrawAttack,
    ));
    attacks.push(crate::weapon::lance::get_attack_data(LanceAttack::Sweep));
    attacks.push(crate::weapon::lance::get_attack_data(LanceAttack::Charge));
    attacks.push(crate::weapon::lance::get_attack_data(LanceAttack::Hook));

    attacks
}
