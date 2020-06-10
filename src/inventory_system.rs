use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;

pub struct ItemCollectionSystem {}

pub struct PotionUseSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            names,
            mut wants_to_pickup_items,
            mut positions,
            mut in_backpacks,
        ) = data;

        for pickup in (&wants_to_pickup_items).join() {
            positions.remove(pickup.item);
            in_backpacks
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("não consegui colocar na bolsa!");

            if pickup.collected_by == *player_entity {
                let name = &names.get(pickup.item).unwrap().name;
                game_log.entries.push(format!("Você pegou {}", name));
            }
        }

        wants_to_pickup_items.clear();
    }
}

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, HealthPotion>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, WantsToDrinkPotion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut game_log,
            names,
            health_potions,
            mut combat_stats,
            mut wants_to_drink_potions,
        ) = data;

        for (entity, drink, stats) in (&entities, &wants_to_drink_potions, &mut combat_stats).join()
        {
            if let Some(potion) = health_potions.get(drink.potion) {
                stats.hp = i32::min(stats.max_hp as i32, stats.hp + potion.heal_amount as i32);
                if entity == *player_entity {
                    let potion_name = names.get(drink.potion).unwrap();
                    game_log.entries.push(format!(
                        "Você toma uma talagada de {}, e cura {} hp.",
                        potion_name.name, potion.heal_amount
                    ));
                }
                entities
                    .delete(drink.potion)
                    .expect("não consegui reciclar a poção!");
            }
        }

        wants_to_drink_potions.clear();
    }
}
