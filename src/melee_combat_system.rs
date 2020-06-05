use std::cmp::max;

use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut game_log, mut wants_to_melees, names, mut suffer_damages, combat_stats) =
            data;

        for (_entity, wants_to_melee, name, stats) in
            (&entities, &wants_to_melees, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_to_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_to_melee.target).unwrap();
                    let damage = max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        game_log.entries.push(format!(
                            "{} não faz nem um arranhão em {}...",
                            name.name, target_name.name
                        ));
                    } else {
                        SufferDamage::new_damage(
                            &mut suffer_damages,
                            wants_to_melee.target,
                            damage,
                        );
                        game_log.entries.push(format!(
                            "{} sabuga {} causando {} de dano!",
                            name.name, target_name.name, damage
                        ));
                    }
                }
            }
        }

        wants_to_melees.clear();
    }
}
