use rltk::console;
use specs::prelude::*;

use crate::components::Monster;
use crate::components::Position;
use crate::components::Viewshed;

pub struct MonsterAISystem {}

impl<'a> System<'a> for MonsterAISystem {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewsheds, monsters, positions) = data;

        for (viewshed, monster, pos) in (&viewsheds, &monsters, &positions).join() {
            console::log("monstrando");
        }
    }
}
