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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers) = data;

        map.populate_blocked();

        for (pos, _blocker) in (&positions, &blockers).join() {
            let idx = map.xy_idx(pos.x as usize, pos.y as usize);
            map.blocked[idx] = true;
        }
    }
}
