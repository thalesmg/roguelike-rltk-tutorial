use rltk::Rltk;
use rltk::VirtualKeyCode;
use specs::prelude::*;
use std::cmp::max;
use std::cmp::min;

use crate::components::*;
use crate::map::*;
use crate::State;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (pos, _player) in (&mut positions, &mut players).join() {
        let dest_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[dest_idx] != TileType::Wall {
            pos.x = min(WIDTH as i32 - 1, max(0, pos.x + delta_x));
            pos.y = min(HEIGHT as i32 - 1, max(0, pos.y + delta_y));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &Rltk) {
    match ctx.key {
        None => (),
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),
            _ => (),
        },
    }
}
