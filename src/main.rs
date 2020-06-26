#[macro_use]
extern crate specs_derive;

#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

mod components;
mod damage_system;
mod game_log;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod spawner;
mod visibility_system;

use rltk::GameState;
use rltk::Point;
use rltk::Rltk;
use rltk::RltkBuilder;
use specs::prelude::*;

use crate::components::*;
use crate::damage_system::DamageSystem;
use crate::game_log::GameLog;
use crate::gui::draw_ui;
use crate::inventory_system::ItemCollectionSystem;
use crate::inventory_system::ItemDropSystem;
use crate::inventory_system::ItemUseSystem;
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
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: u32, item: Entity },
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
        let mut item_use_system = ItemUseSystem {};
        item_use_system.run_now(&self.ecs);
        let mut item_drop_system = ItemDropSystem {};
        item_drop_system.run_now(&self.ecs);
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

            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|(_p1, r1), (_p2, r2)| r2.render_order.cmp(&r1.render_order));
            for (pos, render) in data.iter() {
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
            }
            RunState::ShowInventory => match gui::show_inventory(self, ctx) {
                gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                gui::ItemMenuResult::NoResponse => RunState::ShowInventory,
                gui::ItemMenuResult::Selected((item_entity, _item_name)) => {
                    // TODO check when other items exist
                    let mut wants_to_use_items = self.ecs.write_storage::<WantsToUseItem>();
                    let player_entity = self.ecs.fetch::<Entity>();
                    let ranged_items = self.ecs.read_storage::<Ranged>();
                    if let Some(ranged) = ranged_items.get(item_entity) {
                        RunState::ShowTargeting {
                            range: ranged.range,
                            item: item_entity,
                        }
                    } else {
                        wants_to_use_items
                            .insert(
                                *player_entity,
                                WantsToUseItem {
                                    item: item_entity,
                                    target: None,
                                },
                            )
                            .expect("nao consegui criar a vontade de usar!");
                        RunState::PlayerTurn
                    }
                }
                gui::ItemMenuResult::RangeSelected(_) => RunState::PlayerTurn,
            },
            RunState::ShowDropItem => match gui::drop_item_menu(self, ctx) {
                gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                gui::ItemMenuResult::NoResponse => RunState::ShowDropItem,
                gui::ItemMenuResult::Selected((item_entity, _item_name)) => {
                    let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                    let player_entity = self.ecs.fetch::<Entity>();
                    intent
                        .insert(*player_entity, WantsToDropItem { item: item_entity })
                        .expect("não teve vontade de largar nada...");
                    RunState::PlayerTurn
                }
                gui::ItemMenuResult::RangeSelected(_) => RunState::PlayerTurn,
            },
            RunState::ShowTargeting { range, item } => match gui::ranged_target(self, ctx, range) {
                gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                gui::ItemMenuResult::NoResponse => RunState::ShowTargeting { range, item },
                gui::ItemMenuResult::Selected(_) => RunState::AwaitingInput,
                gui::ItemMenuResult::RangeSelected(target) => {
                    let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                    let player_entity = self.ecs.fetch::<Entity>();
                    intent
                        .insert(
                            *player_entity,
                            WantsToUseItem {
                                item,
                                target: Some(target),
                            },
                        )
                        .expect("não inseri a vontade de usar um item!");
                    RunState::PlayerTurn
                }
            },
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
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();

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
