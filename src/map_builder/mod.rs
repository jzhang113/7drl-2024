pub mod bsp;
mod common;
pub mod drunk_walk;
pub mod overworld;

const SHOW_MAPGEN_VISUALIZER: bool = false;

pub trait MapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) -> super::Map;
    fn spawn_entities(&mut self, ecs: &mut super::World, spawn_info: &super::SpawnInfo);
    fn get_map(&self) -> super::Map;
    fn get_starting_position(&self) -> super::Position;
    fn get_snapshot_history(&self) -> Vec<super::Map>;
    fn take_snapshot(&mut self);
}

#[derive(Clone, PartialEq)]
pub struct MapBuilderArgs {
    pub width: i32,
    pub height: i32,
    pub builder_type: usize,
    pub name: String,
    pub map_color: String,
}

pub fn random_builder(width: i32, height: i32, name: String) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder_type = rng.roll_dice(1, 3);
    println!("Building map type {}", builder_type);
    get_builder(
        &MapBuilderArgs {
            builder_type: builder_type as usize,
            width,
            height,
            name,
            map_color: "#FFFFFF".to_string(),
        },
        &mut rng,
    )
}

pub fn with_builder(args: &MapBuilderArgs) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    get_builder(args, &mut rng)
}

fn get_builder(
    args: &MapBuilderArgs,
    rng: &mut rltk::RandomNumberGenerator,
) -> Box<dyn MapBuilder> {
    match args.builder_type {
        //1 => Box::new(BspDungeonBuilder::new(new_depth)),
        // 2 => Box::new(BspInteriorBuilder::new(new_depth)),
        // 3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        1 => Box::new(drunk_walk::DrunkardsWalkBuilder::open_area(args, rng)),
        2 => Box::new(drunk_walk::DrunkardsWalkBuilder::open_halls(args, rng)),
        4 => Box::new(overworld::OverworldBuilder::new(args, rng)),
        _ => Box::new(drunk_walk::DrunkardsWalkBuilder::winding_passages(
            args, rng,
        )),
        //_ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
}
