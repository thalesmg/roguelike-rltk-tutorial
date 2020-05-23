use rltk::console;
use rltk::Point;
use specs::prelude::*;

use crate::components::Monster;
use crate::components::Viewshed;
use crate::components::Name;

pub struct MonsterAISystem {}

impl<'a> System<'a> for MonsterAISystem {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewsheds, monsters, names) = data;

        for (viewshed, _monster, Name(name)) in (&viewsheds, &monsters, &names).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(format!("{} manda vc para aquele lugar.", name));
            }
        }
    }
}
