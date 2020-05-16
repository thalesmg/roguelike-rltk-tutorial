use crate::components::Player;
use crate::components::Position;
use crate::components::Viewshed;
use crate::map::xy_idx;
use crate::map::Map;

use rltk::field_of_view;
use rltk::Point;
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Player>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (pos, mut viewshed, mut map, player, entity) = data;

        for (position, viewshed, entity) in (&pos, &mut viewshed, &entity).join() {
            if viewshed.dirty {
                viewshed.dirty = false;

                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(
                    Point::new(position.x, position.y),
                    viewshed.range as i32,
                    &*map,
                );
                viewshed.visible_tiles.retain(|p| {
                    p.x >= 0 && p.x <= map.width as i32 && p.y >= 0 && p.y <= map.height as i32
                });

                // If the player, update revealed tiles
                if let Some(_) = player.get(entity) {
                    map.visible_tiles.iter_mut().for_each(|t| *t = false);
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }
    }
}
