use rltk::Rltk;
use rltk::RGB;
use specs::prelude::*;

use crate::components::*;
use crate::map::Map;

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
}
