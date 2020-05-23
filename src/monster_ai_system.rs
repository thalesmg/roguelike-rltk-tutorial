use rltk::console;
use rltk::Point;
use specs::prelude::*;

use crate::components::Monster;
use crate::components::Viewshed;
use crate::components::Name;
use crate::components::Position;
use crate::map::Map;

pub struct MonsterAISystem {}

impl<'a> System<'a> for MonsterAISystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, player_pos, monsters, names, mut viewsheds, mut positions) = data;

        for (mut viewshed, _monster, name, mut pos) in (&mut viewsheds, &monsters, &names, &mut positions).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(format!("{} manda vc para aquele lugar.", name.name));
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x as usize, pos.y as usize),
                    map.xy_idx(player_pos.x as usize, player_pos.y as usize),
                    &*map
                );
                if path.success && path.steps.len() > 1 {
                    let (step_x, step_y) = map.idx_xy(path.steps[1]);
                    pos.x = step_x as i32;
                    pos.y = step_y as i32;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
