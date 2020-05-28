use std::cmp::max;

use rltk::console;
use specs::prelude::*;

use crate::components::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_to_melees, names, mut suffer_damages, combat_stats) = data;

        for (entity, mut wants_to_melee, name, stats) in (&entities, &mut wants_to_melees, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_to_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_to_melee.target).unwrap();
                    let damage = max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        console::log(format!("{} não faz nem um arranhão em {}...", name.name, target_name.name));
                    } else {
                        SufferDamage::new_damage(&mut suffer_damages, wants_to_melee.target, damage);
                        console::log(format!("{} sabuga {} causando {} de dano!", name.name, target_name.name, damage));
                    }
                }
            }
        }

        wants_to_melees.clear();
    }
}
