use super::{ParticleLifetime, Position, Renderable};
use rltk::{FontCharType, Point, Rltk};
use specs::prelude::*;

pub fn cleanup_particles(ecs: &mut World, ctx: &Rltk) {
    let dead_particles = update_lifetimes(ecs, ctx);

    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Failed to delete particle");
    }
}

fn update_lifetimes(ecs: &mut World, ctx: &Rltk) -> Vec<Entity> {
    let mut dead_particles = Vec::new();
    let mut particles = ecs.write_storage::<ParticleLifetime>();
    let entities = ecs.entities();

    for (ent, lifetime) in (&entities, &mut particles).join() {
        lifetime.remaining -= ctx.frame_time_ms;
        if lifetime.remaining < 0.0 {
            dead_particles.push(ent);
        }
    }

    dead_particles
}

#[derive(PartialEq, Copy, Clone)]
pub struct ParticleRequest {
    pub position: Point,
    pub color: rltk::RGB,
    pub symbol: FontCharType,
    pub lifetime: f32,
    pub zindex: u32,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    fn make_particle(&mut self, request: ParticleRequest) {
        self.requests.push(request);
    }

    pub fn make_hit_particle(&mut self, point: Point) {
        self.make_particle(crate::ParticleRequest {
            color: rltk::RGB::named(rltk::RED),
            lifetime: 300.0,
            position: point,
            symbol: rltk::to_cp437('!'),
            zindex: 1,
        });
    }

    pub fn make_bg_particle(&mut self, point: Point) {
        self.make_particle(crate::ParticleRequest {
            color: rltk::RGB::named(rltk::DARKRED),
            lifetime: 300.0,
            position: point,
            symbol: rltk::to_cp437('▒'),
            zindex: 0,
        });
    }
}

pub struct ParticleSpawnSystem;

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut lifetimes, mut builder) = data;

        for request in builder.requests.drain(..) {
            let particle = entities.create();
            positions
                .insert(
                    particle,
                    Position {
                        x: request.position.x,
                        y: request.position.y,
                    },
                )
                .expect("Failed to insert Position for particle");
            renderables
                .insert(
                    particle,
                    Renderable {
                        symbol: request.symbol,
                        fg: request.color,
                        bg: crate::bg_color(),
                        zindex: request.zindex,
                    },
                )
                .expect("Failed to insert Renderable for particle");
            lifetimes
                .insert(
                    particle,
                    ParticleLifetime {
                        base: request.lifetime,
                        remaining: request.lifetime,
                        should_fade: true,
                    },
                )
                .expect("Failed to insert ParticleLifetime for particle");
        }
    }
}
