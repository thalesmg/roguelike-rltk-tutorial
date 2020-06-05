use rltk::Point;
use rltk::Rltk;
use rltk::VirtualKeyCode;
use specs::prelude::*;
use std::cmp::max;
use std::cmp::min;

use crate::components::*;
use crate::map::*;
use crate::RunState;
use crate::State;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut players = ecs.write_storage::<Player>();
    let mut ppos = ecs.write_resource::<Point>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (pos, _player, viewshed, entity) in
        (&mut positions, &mut players, &mut viewsheds, &entities).join()
    {
        let dest_idx = map.xy_idx((pos.x + delta_x) as usize, (pos.y + delta_y) as usize);

        for potential_target in map.tile_content[dest_idx].iter() {
            if let Some(_) = combat_stats.get(*potential_target) {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("não consegui inserir a vontade de tretar!");
                return;
            }
        }

        if !map.blocked[dest_idx] {
            pos.x = min(WIDTH as i32 - 1, max(0, pos.x + delta_x));
            pos.y = min(HEIGHT as i32 - 1, max(0, pos.y + delta_y));
            ppos.x = pos.x;
            ppos.y = pos.y;
            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &Rltk) -> RunState {
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right | VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up | VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down | VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            VirtualKeyCode::G => get_item(&mut gs.ecs),

            VirtualKeyCode::I => return RunState::ShowInventory,

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let items = ecs.read_storage::<Item>();
    let mut wants_to_pickup_items = ecs.write_storage::<WantsToPickupItem>();
    let map = ecs.fetch::<Map>();
    let idx = map.xy_idx(player_pos.x as usize, player_pos.y as usize);

    for potential_item in map.tile_content[idx].iter() {
        if let Some(_) = items.get(*potential_item) {
            wants_to_pickup_items
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item: *potential_item,
                    },
                )
                .expect("não consegui inserir a vontade de pegar algo!");
        }
    }
}
