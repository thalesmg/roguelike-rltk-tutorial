#[macro_use]
extern crate specs_derive;

#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

mod components;
mod damage_system;
mod game_log;
mod gui;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod visibility_system;
mod spawner;
mod inventory_system;

use rltk::GameState;
use rltk::Point;
use rltk::Rltk;
use rltk::RltkBuilder;
use specs::prelude::*;

use crate::components::*;
use crate::damage_system::DamageSystem;
use crate::game_log::GameLog;
use crate::gui::draw_ui;
use crate::map::*;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAISystem;
use crate::player::*;
use crate::visibility_system::VisibilitySystem;
use crate::inventory_system::ItemCollectionSystem;

rltk::add_wasm_support!();

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
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
        let mut item_collection_system = ItemCollectionSystem {};
        item_collection_system.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        draw_map(&self.ecs, ctx);
        draw_ui(&self.ecs, ctx);

        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let map = self.ecs.fetch::<Map>();

            for (pos, render) in (&positions, &renderables).join() {
                if map.visible_tiles[xy_idx(pos.x, pos.y)] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        let mut newrunstate = *self.ecs.fetch::<RunState>();

        newrunstate = match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::AwaitingInput => player_input(self, ctx),
            RunState::PlayerTurn => {
                self.run_systems();
                RunState::MonsterTurn
            }
            RunState::MonsterTurn => {
                self.run_systems();
                RunState::AwaitingInput
            },
            RunState::ShowInventory => {
                if gui::show_inventory(self, ctx) == gui::ItemMenuResult::Cancel {
                    RunState::AwaitingInput
                } else {
                    RunState::ShowInventory
                }
            }
        };

        *self.ecs.write_resource() = newrunstate;

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

fn main() -> rltk::BError {
    let mut context = RltkBuilder::simple80x50()
        .with_title("Olá mundo!")
        .build()?;
    context.with_post_scanlines(true);
    let mut gs = State { ecs: World::new() };
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

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
    gs.ecs.register::<Item>();
    gs.ecs.register::<HealthPotion>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();

    let map = new_map();

    let (x, y) = map.rooms[0].center();

    let player = spawner::player(&mut gs.ecs, x, y);
    gs.ecs.insert(player);

    for room in map.rooms.iter().skip(1) {
        // spawner::random_monster(&mut gs.ecs, x, y);
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(x, y));
    gs.ecs.insert(GameLog {
        entries: vec!["Bem-vindo, mortal!".to_string()],
    });

    rltk::main_loop(context, gs)?;

    Ok(())
}
