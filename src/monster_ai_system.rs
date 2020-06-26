use rltk::Point;
use specs::prelude::*;

use crate::components::*;
use crate::map::Map;
use crate::RunState;

pub struct MonsterAISystem {}

impl<'a> System<'a> for MonsterAISystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            player_pos,
            player_entity,
            runstate,
            monsters,
            mut confusions,
            mut viewsheds,
            mut positions,
            mut wants_to_melees,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        };

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewsheds, &monsters, &mut positions).join()
        {
            if let Some(confusion) = confusions.get_mut(entity) {
                confusion.turns -= 1;
                if confusion.turns <= 0 {
                    confusions
                        .remove(entity)
                        .expect("não consegui desconfundir!");
                };
            } else {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melees
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("não consegui criar a vontade de matar!");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x as usize, pos.y as usize),
                        map.xy_idx(player_pos.x as usize, player_pos.y as usize),
                        &*map,
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
}
