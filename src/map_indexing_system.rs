use specs::prelude::*;

use crate::map::Map;
use crate::components::Position;
use crate::components::BlocksTile;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();

        for (pos, entity) in (&positions, &entities).join() {
            let idx = map.xy_idx(pos.x as usize, pos.y as usize);

            if let Some(_) = blockers.get(entity) {
                map.blocked[idx] = true;
            }

            map.tile_content[idx].push(entity);
        }
    }
}
