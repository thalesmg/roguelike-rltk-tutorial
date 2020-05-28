use specs::prelude::*;
use specs::World;

use crate::components::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut combat_stats, mut suffer_damages) = data;

        for (mut stats, damage) in (&mut combat_stats, &suffer_damages).join() {
            let total_damage: u32 = damage.amount.iter().sum();
            stats.hp -= total_damage as i32;
        }

        suffer_damages.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead = Vec::new();

    {
        let combat_stats = ecs.write_storage::<CombatStats>();
        let entities = ecs.entities();
        (&combat_stats, &entities).join()
            .for_each(|(stats, entity)| {
                if stats.hp <= 0 { dead.push(entity); };
            });
    }

    dead.iter().for_each(|d| ecs.delete_entity(*d).expect("não consegui remover um morto!"));
}
