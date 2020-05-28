#[macro_use]
extern crate specs_derive;

#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

mod components;
mod damage_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod visibility_system;

use rltk::GameState;
use rltk::Point;
use rltk::Rltk;
use rltk::RltkBuilder;
use rltk::RGB;
use specs::prelude::*;

use crate::components::*;
use crate::damage_system::DamageSystem;
use crate::map::*;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAISystem;
use crate::player::*;
use crate::visibility_system::VisibilitySystem;

rltk::add_wasm_support!();

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility_system = VisibilitySystem {};
        visibility_system.run_now(&self.ecs);
        let mut monster_ai_system = MonsterAISystem {};
        monster_ai_system.run_now(&self.ecs);
        let mut map_indexing_system = MapIndexingSystem {};
        map_indexing_system.run_now(&self.ecs);
        let mut melee_combat_system = MeleeCombatSystem {};
        melee_combat_system.run_now(&self.ecs);
        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut newrunstate = *self.ecs.fetch::<RunState>();

        newrunstate = match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            },
            RunState::AwaitingInput => {
                player_input(self, ctx)
            },
            RunState::PlayerTurn => {
                self.run_systems();
                RunState::MonsterTurn
            },
            RunState::MonsterTurn => {
                self.run_systems();
                RunState::AwaitingInput
            },
        };

        *self.ecs.write_resource() = newrunstate;

        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            if map.visible_tiles[xy_idx(pos.x, pos.y)] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("Olá mundo!")
        .build()?;
    let mut gs = State {
        ecs: World::new(),
    };
    gs.ecs.insert(RunState::PreRun);

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let map = new_map();

    let (x, y) = map.rooms[0].center();

    let player = gs
        .ecs
        .create_entity()
        .with(Position {
            x: x as i32,
            y: y as i32,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            range: 8,
            visible_tiles: Vec::new(),
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build();
    gs.ecs.insert(player);

    let mut rng = rltk::RandomNumberGenerator::new();

    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        let (glyph, name) = if rng.rand() {
            (rltk::to_cp437('g'), "Goblin".to_string())
        } else {
            (rltk::to_cp437('o'), "Orc".to_string())
        };

        gs.ecs
            .create_entity()
            .with(Position {
                x: x as i32,
                y: y as i32,
            })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                range: 8,
                visible_tiles: Vec::new(),
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} {}", name, i),
            })
            .with(BlocksTile {})
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            })
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(x, y));

    rltk::main_loop(context, gs)?;

    Ok(())
}
