use crate::*;
use rltk::Point;

pub fn build_arrow_trap(ecs: &mut World, point: Point) -> Entity {
    ecs.create_entity()
        .with(ViewableIndex { list_index: None })
        .with(Viewable {
            name: "Turret".to_string(),
            description: vec!["Fires stunning bolts".to_string()],
            seen: false,
        })
        .with(BlocksTile)
        .with(Schedulable {
            current: 0,
            base: 6,
            delta: 1,
        })
        .with(Position {
            x: point.x,
            y: point.y,
        })
        .with(Renderable {
            symbol: 't' as u16,
            fg: RGB::named(rltk::GREEN),
            bg: bg_color(),
            zindex: 1,
        })
        .with(TrapAiState {
            status: Behavior::Sleep,
        })
        .with(Moveset {
            moves: vec![(AttackType::Line { radius: 6 }, 1.0)],
            bump_attack: AttackType::Push,
        })
        .with(Facing {
            direction: Direction::S,
        })
        .build()
}
