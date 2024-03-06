use rltk::{Point, RGB};
use specs::prelude::*;
use specs::Component;

#[derive(Component, Copy, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn as_point(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

#[derive(Component)]
pub struct Renderable {
    pub symbol: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub zindex: u32,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Viewshed {
    pub visible: Vec<Point>,
    pub dirty: bool,
    pub range: i32,
}

#[derive(Component)]
pub struct CanActFlag {
    pub is_reaction: bool,
    pub reaction_target: Option<Entity>,
}

#[derive(Component)]
pub struct CanReactFlag;

#[derive(Component)]
pub struct Schedulable {
    pub current: i32,
    pub base: i32,
    pub delta: i32,
}

#[derive(Component)]
pub struct ParticleLifetime {
    pub base: f32,
    pub remaining: f32,
    pub should_fade: bool,
}

#[derive(Component)]
pub struct BlocksTile;

#[derive(Component, Debug)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct Stamina {
    pub current: i32,
    pub max: i32,
    pub recover: bool,
}

#[derive(Component, Copy, Clone)]
pub struct FrameData {
    pub startup: u32,
    pub active: u32,
    pub recovery: u32,
    pub current: u32,
    pub cancelled: bool,
    pub linger_time: i32,
}

#[derive(Component, Copy, Clone)]
pub struct AttackIntent {
    pub main: crate::AttackType,
    pub loc: Point,
}

#[derive(Component, Copy, Clone)]
pub struct MoveIntent {
    pub loc: rltk::Point,
    pub force_facing: Option<crate::Direction>,
}

#[derive(Component)]
pub struct AttackPath {
    pub path: Vec<rltk::Point>,
    pub index: usize,
    pub step_delay: u32,
    pub cur_delay: u32,
    pub on_hit: crate::AttackType,
}

#[derive(Component)]
pub struct Moveset {
    pub moves: Vec<(crate::AttackType, f32)>,
    pub bump_attack: crate::AttackType,
}

#[derive(Component)]
pub struct Viewable {
    pub name: String,
    pub description: Vec<String>,
    pub seen: bool,
}

#[derive(Component)]
pub struct ViewableIndex {
    pub list_index: Option<u32>,
}

#[derive(Component)]
pub struct AttackInProgress;

#[derive(Component)]
pub struct BlockAttack {
    pub block_amount: u32,
}

#[derive(Component)]
pub struct AiState {
    pub status: crate::Behavior,
    pub prev_path: Option<rltk::NavigationPath>,
    pub path_step: usize,
}

#[derive(Component)]
pub struct TrapAiState {
    pub status: crate::Behavior,
}

#[derive(Component)]
pub struct Heal {
    pub amount: u32,
}

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct Openable;

#[derive(Component)]
pub struct MultiTile {
    pub part_list: Vec<crate::MonsterPart>,
    pub bounds: rltk::Rect,
}

#[derive(Component, Copy, Clone)]
pub struct Facing {
    pub direction: crate::Direction,
}

#[derive(Component, Clone)]
pub struct PartMoveIntent {
    pub part_delta: Vec<rltk::Point>,
}

#[derive(Component, Debug)]
pub struct PushForce {
    pub delta: rltk::Point,
}

#[derive(Debug)]
pub enum NpcType {
    Blacksmith,
    Shopkeeper,
    Handler,
}

#[derive(Component, Debug)]
pub struct Npc {
    pub npc_type: NpcType,
}

#[derive(Component)]
pub struct Invulnerable {
    pub duration: u32,
}

#[derive(Component)]
pub struct Stunned {
    pub duration: u32,
}

#[derive(Component)]
pub struct MissionTarget;
