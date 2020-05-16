#[macro_use]
extern crate specs_derive;

#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

mod components;
mod map;
mod player;
mod visibility_system;

use rltk::GameState;
use rltk::Rltk;
use rltk::RltkBuilder;
use rltk::RGB;
use specs::prelude::*;

use crate::components::*;
use crate::map::*;
use crate::player::*;
use crate::visibility_system::VisibilitySystem;

rltk::add_wasm_support!();

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility_system = VisibilitySystem {};
        visibility_system.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.run_systems();

        player_input(self, ctx);

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
    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map = new_map();

    let (x, y) = map.rooms[0].center();

    gs.ecs
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
        .build();

    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();
        gs.ecs
            .create_entity()
            .with(Position {
                x: x as i32,
                y: y as i32,
            })
            .with(Renderable {
                glyph: rltk::to_cp437('g'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                range: 8,
                visible_tiles: Vec::new(),
                dirty: true,
            })
            .build();
    }

    gs.ecs.insert(map);

    rltk::main_loop(context, gs)?;

    Ok(())
}
