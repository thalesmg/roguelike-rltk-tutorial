use rltk::Point;
use rltk::Rltk;
use rltk::RGB;
use rltk::VirtualKeyCode;
use specs::prelude::*;

use crate::State;
use crate::components::*;
use crate::game_log::GameLog;
use crate::map::Map;

#[derive(PartialEq)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let map_width = map.width;
    let map_height = map.height;
    let start_x = 0;
    let start_y = map_height;
    let height = 6;
    let width = map_width - 1;
    let fg = RGB::named(rltk::WHITE);
    let bg = RGB::named(rltk::BLACK);

    ctx.draw_box(start_x, start_y, width, height, fg, bg);

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!("HP: {} / {}", stats.hp, stats.max_hp);
        // FIXME: relative coords
        ctx.print_color(
            12,
            43,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            health,
        );
        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    let game_log = ecs.fetch::<GameLog>();
    for (i, entry) in game_log.entries.iter().rev().enumerate() {
        let y = 44 + i;
        if y < 49 {
            ctx.print(2, y, entry);
        }
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width as i32 || mouse_pos.1 >= map.height as i32 { return; }

    let mut tooltips = Vec::new();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x as usize, position.y as usize);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltips.push(name.name.to_string());
        }
    }

    if !tooltips.is_empty() {
        let width = 3 + tooltips.iter().max_by_key(|s| s.len()).unwrap().len();
        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width as i32;
            let y = mouse_pos.1;
            for (i, tooltip) in tooltips.iter().enumerate() {
                ctx.print_color(left_x, y + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), tooltip);
                let padding = width - tooltip.len() - 1;
                for j in 0..padding {
                    ctx.print_color(arrow_pos.x - j as i32, y + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), " ".to_string());
                }
                ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), "->".to_string());
            }
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 4 as i32;
            let y = mouse_pos.1;
            for (i, tooltip) in tooltips.iter().enumerate() {
                ctx.print_color(left_x, y + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), tooltip);
                let padding = width - tooltip.len() - 1;
                for j in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + j as i32, y + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), " ".to_string());
                }
                ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), "<-".to_string());
            }
        }
    }
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let in_backpacks = gs.ecs.read_storage::<InBackpack>();

    let inventory = (&names, &in_backpacks).join()
        .filter(|(name, backpack)| backpack.owner == *player_entity);
    let count = inventory.clone().count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventário");
    ctx.print_color(18, y + count as i32 + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE para cancelar");

    for (j, (name, _backpack)) in inventory.enumerate() {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97 + j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        y += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => ItemMenuResult::Cancel,
                _ => ItemMenuResult::NoResponse,
            }
        }
    }
}
