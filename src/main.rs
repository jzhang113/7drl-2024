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
mod mission_info;
mod monster_part;
mod player;
mod quest;
mod range_type;
mod spawn;
mod sys_ai;
mod sys_attack;
mod sys_death;
mod sys_mapindex;
mod sys_movement;
mod sys_partbreak;
mod sys_particle;
mod sys_partmove;
mod sys_pickup;
mod sys_projectile;
mod sys_push;
mod sys_turn;
mod sys_visibility;
mod weapon;

pub use attack_type::*;
pub use camera::*;
pub use colors::*;
pub use components::*;
pub use direction::Direction;
pub use map::{Map, TileType};
pub use mission_info::MissionInfo;
pub use monster_part::*;
pub use range_type::*;
pub use spawn::info::SpawnInfo;
pub use sys_ai::{Behavior, NextIntent};
pub use sys_particle::{ParticleBuilder, ParticleRequest};

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
    MissionSelect {
        index: usize,
    },
    Shop,
    Blacksmith,
    Dead {
        success: bool,
    },
    Charging {
        dir: crate::Direction,
        speed: u8,
    },
}

pub struct State {
    ecs: World,
    tick: i32,
    tab_targets: Vec<rltk::Point>,
    tab_index: usize,
    attack_modifier: Option<AttackType>,
    quests: quest::log::QuestLog,
    selected_quest: Option<quest::quest::Quest>,
    player_inventory: inventory::Inventory,
    player_charging: (bool, crate::Direction, u8, bool),
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
        self.ecs.register::<Viewable>();
        self.ecs.register::<ViewableIndex>();

        self.ecs.register::<Health>();
        self.ecs.register::<Stamina>();
        self.ecs.register::<AttackIntent>();
        self.ecs.register::<MoveIntent>();
        self.ecs.register::<PartMoveIntent>();
        self.ecs.register::<Moveset>();
        self.ecs.register::<AttackPath>();

        self.ecs.register::<AttackInProgress>();
        self.ecs.register::<BlockAttack>();
        self.ecs.register::<AiState>();
        self.ecs.register::<Heal>();
        self.ecs.register::<Item>();
        self.ecs.register::<Openable>();

        self.ecs.register::<MultiTile>();
        self.ecs.register::<Facing>();

        self.ecs.register::<PushForce>();
        self.ecs.register::<Npc>();
        self.ecs.register::<Invulnerable>();
        self.ecs.register::<MissionTarget>();
    }

    fn new_game(&mut self) {
        self.register_components();

        self.ecs.insert(RunState::Running);
        self.ecs.insert(sys_particle::ParticleBuilder::new());

        let mut rng = rltk::RandomNumberGenerator::new();

        // Add a dummy map and player to the ecs
        let map = Map::new(1, 1, &"Dummy".to_string(), &"#FFFFFF".to_string(), &mut rng);
        self.ecs.insert(map);

        let player = spawn::spawner::build_player(&mut self.ecs, rltk::Point::new(0, 0));
        self.ecs.insert(player);

        for _ in 0..3 {
            self.quests.add_quest(&mut rng, 1);
        }
        self.ecs.insert(rng);

        let log = gamelog::GameLog {
            entries: Vec::new(),
            dirty: false,
        };
        self.ecs.insert(log);

        // TODO: temp in-mission info handling
        let mission_info = MissionInfo::new();
        self.ecs.insert(mission_info);

        self.load_overworld();
    }

    fn run_systems(&mut self) -> RunState {
        self.tick += 1;

        sys_ai::AiSystem.run_now(&self.ecs);
        sys_turn::TurnSystem.run_now(&self.ecs);

        sys_movement::MovementSystem.run_now(&self.ecs);
        sys_attack::AttackSystem.run_now(&self.ecs);
        sys_projectile::ProjectileSystem.run_now(&self.ecs);

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
        self.new_level(
            &MapBuilderArgs {
                builder_type: 4,
                width: 20,
                height: 20,
                name: "Base".to_string(),
                map_color: "#D4BF8E".to_string(),
            },
            &SpawnInfo {
                major_monsters: vec![],
                minor_monsters: vec![],
                resources: vec![],
                difficulty: 0,
            },
        )
    }

    fn new_level(&mut self, map_builder_args: &MapBuilderArgs, spawn_info: &SpawnInfo) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_need_cleanup();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        let mut map_builder = map_builder::with_builder(map_builder_args);
        let new_map = {
            let mut rng = self.ecs.fetch_mut::<rltk::RandomNumberGenerator>();
            map_builder.build_map(&mut rng)
        };

        {
            // update player position
            let player = self.ecs.fetch::<Entity>();
            let mut positions = self.ecs.write_storage::<Position>();
            let player_pos = positions
                .get_mut(*player)
                .expect("player didn't have a position");

            let new_player_pos = map_builder.get_starting_position();
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

        // fill the map
        map_builder.spawn_entities(&mut self.ecs, spawn_info);
    }

    fn reset_player(&mut self) {
        let player = self.ecs.fetch::<Entity>();
        let mut healths = self.ecs.write_storage::<Health>();
        let player_healths = healths
            .get_mut(*player)
            .expect("player didn't have a health");

        player_healths.current = player_healths.max;
    }

    fn advance_day(&mut self) {
        let mut rng = self.ecs.fetch_mut::<rltk::RandomNumberGenerator>();

        self.quests.advance_day();
        for _ in 0..3 {
            self.quests.add_quest(&mut rng, self.max_cleared_level + 1);
        }
    }

    fn apply_rewards(&mut self) {
        if let Some(quest) = &self.selected_quest {
            self.player_inventory.money += quest.reward;

            let log_quest = self.quests.entries.iter_mut().find(|q| q == &quest);
            if let Some(log_quest) = log_quest {
                log_quest.completed = true;
            }
        }
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

        let is_weapon_sheathed = {
            &self
                .player_inventory
                .weapon
                .get_attack_data(weapon::WeaponButton::Light)
                .map_or(false, |data| data.name == "Draw Atk")
        };

        // draw map + gui
        gui::map::draw_all(&self.ecs, ctx, *is_weapon_sheathed);

        // non-map elements
        gui::sidebar::draw_sidebar(&self, ctx);
        gui::log::update_log_text(&self.ecs, ctx);

        match next_status {
            RunState::AwaitingInput => {
                next_status = player::player_input(self, ctx, *is_weapon_sheathed);

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
                            let intent = crate::attack_type::get_attack_intent(
                                attack_type,
                                target,
                                self.attack_modifier,
                            );
                            let player = self.ecs.fetch::<Entity>();
                            let mut attacks = self.ecs.write_storage::<AttackIntent>();

                            attacks
                                .insert(*player, intent)
                                .expect("Failed to insert attack from Player");

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
                std::thread::sleep(std::time::Duration::from_millis(50));
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
            RunState::GenerateLevel => match self.selected_quest.take() {
                None => {
                    let mut log = self.ecs.fetch_mut::<GameLog>();
                    log.add("You need to select a quest first");
                    next_status = RunState::AwaitingInput;
                }
                Some(mut quest) => {
                    self.new_level(&quest.map_builder_args, &quest.spawn_info);
                    sys_visibility::VisibilitySystem.run_now(&self.ecs);

                    // todo, merge with MissionInfo?
                    quest.started = true;
                    self.selected_quest = Some(quest);
                    next_status = RunState::AwaitingInput;
                }
            },
            RunState::ChangeMap => {
                // TODO: support multi-level maps
                unreachable!();
                // update visibility immediately so the screen isn't dark for a cycle
                // sys_visibility::VisibilitySystem.run_now(&self.ecs);
                // next_status = RunState::AwaitingInput;
            }
            RunState::Dead { success } => {
                match ctx.key {
                    None => {}
                    Some(key) => {
                        if key == rltk::VirtualKeyCode::R {
                            self.load_overworld();
                            self.reset_player();

                            if success {
                                self.apply_rewards();
                                self.max_cleared_level = std::cmp::max(
                                    self.selected_quest
                                        .as_ref()
                                        .map(|v| v.spawn_info.difficulty)
                                        .unwrap_or(0),
                                    self.max_cleared_level,
                                );
                            }

                            // clear out temp mission info
                            {
                                let mut m_info = self.ecs.fetch_mut::<MissionInfo>();
                                m_info.reset();
                            }
                            self.selected_quest = None;
                            self.advance_day();

                            next_status = RunState::Running;
                        }
                    }
                }
            }
            RunState::MissionSelect { index } => {
                gui::overworld::draw_missions(ctx, &self.quests, &self.selected_quest, index);

                next_status = player::mission_select_input(self, ctx, index);
            }
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
        quests: quest::log::QuestLog::new(),
        selected_quest: None,
        player_inventory: inventory::Inventory::new(),
        player_charging: (false, crate::Direction::N, 0, false),
        max_cleared_level: 0,
    };

    gs.new_game();

    rltk::main_loop(context, gs)
}
