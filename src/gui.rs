use rltk::Point;
use rltk::Rltk;
use rltk::VirtualKeyCode;
use rltk::RGB;
use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::map::Map;
use crate::RunState;
use crate::State;

#[derive(PartialEq)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected((Entity, String)),
    RangeSelected(Point),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MainMenuSelection {
    NewGame,
    Load,
    Quit,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MainMenuResult {
    NoSelection(MainMenuSelection),
    Selected(MainMenuSelection),
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
    if mouse_pos.0 >= map.width as i32 || mouse_pos.1 >= map.height as i32 {
        return;
    }

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
                ctx.print_color(
                    left_x,
                    y + i as i32,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    tooltip,
                );
                let padding = width - tooltip.len() - 1;
                for j in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - j as i32,
                        y + i as i32,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        " ".to_string(),
                    );
                }
                ctx.print_color(
                    arrow_pos.x,
                    arrow_pos.y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    "->".to_string(),
                );
            }
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 4 as i32;
            let y = mouse_pos.1;
            for (i, tooltip) in tooltips.iter().enumerate() {
                ctx.print_color(
                    left_x,
                    y + i as i32,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    tooltip,
                );
                let padding = width - tooltip.len() - 1;
                for j in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + j as i32,
                        y + i as i32,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        " ".to_string(),
                    );
                }
                ctx.print_color(
                    arrow_pos.x,
                    arrow_pos.y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    "<-".to_string(),
                );
            }
        }
    }
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    let entities = gs.ecs.entities();
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let in_backpacks = gs.ecs.read_storage::<InBackpack>();

    let inventory = (&entities, &names, &in_backpacks)
        .join()
        .filter(|(_entity, _name, backpack)| backpack.owner == *player_entity);
    let count = inventory.clone().count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventário",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE para cancelar",
    );

    let mut equipabble: Vec<(Entity, &str)> = Vec::new();
    for (j, (entity, name, _backpack)) in inventory.enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        // letter keys
        let letter_a = 'a' as u16;
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            letter_a + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());

        equipabble.push((entity, &name.name));

        y += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => {
            if key == VirtualKeyCode::Escape {
                ItemMenuResult::Cancel
            } else {
                let selection = rltk::letter_to_option(key);
                if -1 < selection && selection < count as i32 {
                    let (item_entity, name_str) = equipabble[selection as usize];
                    ItemMenuResult::Selected((item_entity, name_str.to_string()))
                } else {
                    ItemMenuResult::NoResponse
                }
            }
        }
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpacks = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&entities, &backpacks, &names)
        .join()
        .filter(|(_entity, backpack, _name)| backpack.owner == *player_entity);
    let count = inventory.clone().count() as i32;
    let y = (25 - (count / 2)) as i32;

    ctx.draw_box(
        15,
        y - 2,
        31,
        count as i32 + 3,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Largar o quê?",
    );
    ctx.print_color(
        18,
        y + count + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE para cancelar",
    );

    let mut equipable = Vec::new();
    let letter_a = 'a' as u16;
    for (j, (entity, _pack, name)) in inventory.enumerate() {
        ctx.set(
            17,
            y + j as i32,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y + j as i32,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            letter_a + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y + j as i32,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y + j as i32, &name.name.to_string());
        equipable.push((entity, &name.name));
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(VirtualKeyCode::Escape) => ItemMenuResult::Cancel,
        Some(key) => {
            let selection = rltk::letter_to_option(key);
            if selection > -1 && selection < count as i32 {
                let (item_entity, name) = &equipable[selection as usize];
                ItemMenuResult::Selected((*item_entity, name.to_string()))
            } else {
                ItemMenuResult::NoResponse
            }
        }
    }
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: u32) -> ItemMenuResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Selecione o alvo",
    );

    // ressaltar células dentro do alcance
    let mut available_cells = Vec::new();
    if let Some(Viewshed { visible_tiles, .. }) = viewsheds.get(*player_entity) {
        for tile in visible_tiles.iter() {
            if rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *tile) <= range as f32 {
                ctx.set_bg(tile.x, tile.y, RGB::named(rltk::BLUE));
                available_cells.push(tile);
            }
        }
    } else {
        return ItemMenuResult::Cancel;
    }

    // desenhar cursor
    let mouse_pos = ctx.mouse_pos();
    let valid_target = available_cells
        .iter()
        .any(|tile| tile.x == mouse_pos.0 && tile.y == mouse_pos.1);
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return ItemMenuResult::RangeSelected(Point {
                x: mouse_pos.0,
                y: mouse_pos.1,
            });
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return ItemMenuResult::Cancel;
        }
    }

    ItemMenuResult::NoResponse
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Tutorial do Rufião",
    );

    match *runstate {
        RunState::MainMenu(selected) => {
            ctx.print_color_centered(
                24,
                color_for(MainMenuSelection::NewGame, selected),
                RGB::named(rltk::BLACK),
                "Novo Jogo",
            );
            ctx.print_color_centered(
                25,
                color_for(MainMenuSelection::Load, selected),
                RGB::named(rltk::BLACK),
                "Carregar",
            );
            ctx.print_color_centered(
                26,
                color_for(MainMenuSelection::Quit, selected),
                RGB::named(rltk::BLACK),
                "Sair",
            );

            match ctx.key {
                None => MainMenuResult::NoSelection(selected),
                Some(VirtualKeyCode::Escape) => MainMenuResult::NoSelection(selected),
                Some(VirtualKeyCode::Up) => MainMenuResult::NoSelection(previous_option(selected)),
                Some(VirtualKeyCode::Down) => MainMenuResult::NoSelection(next_option(selected)),
                Some(VirtualKeyCode::Return) => MainMenuResult::Selected(selected),
                _ => MainMenuResult::NoSelection(selected),
            }
        }
        _ => MainMenuResult::NoSelection(MainMenuSelection::NewGame),
    }
}

fn color_for(item: MainMenuSelection, selected: MainMenuSelection) -> RGB {
    if item == selected {
        RGB::named(rltk::MAGENTA)
    } else {
        RGB::named(rltk::WHITE)
    }
}

fn previous_option(selected: MainMenuSelection) -> MainMenuSelection {
    match selected {
        MainMenuSelection::NewGame => MainMenuSelection::Quit,
        MainMenuSelection::Load => MainMenuSelection::NewGame,
        MainMenuSelection::Quit => MainMenuSelection::Load,
    }
}

fn next_option(selected: MainMenuSelection) -> MainMenuSelection {
    match selected {
        MainMenuSelection::NewGame => MainMenuSelection::Load,
        MainMenuSelection::Load => MainMenuSelection::Quit,
        MainMenuSelection::Quit => MainMenuSelection::NewGame,
    }
}
