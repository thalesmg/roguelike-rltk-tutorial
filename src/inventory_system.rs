use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;

pub struct ItemCollectionSystem {}

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

pub struct PotionUseSystem {}

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

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToDropItem>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player_entity, mut game_log, names, mut wants_to_drop_items, mut positions, mut in_backpacks) = data;

        for (entity, to_drop) in (&entities, &mut wants_to_drop_items).join() {
            let mut dropper_pos = Position{x: 0, y: 0};
            {
                let pos = positions.get(entity).unwrap();
                dropper_pos.x = pos.x;
                dropper_pos.y = pos.y;
            }
            positions.insert(to_drop.item, dropper_pos).expect("o item não voltou pro mapa!");
            in_backpacks.remove(to_drop.item);

            if entity == *player_entity {
                let item_name = names.get(to_drop.item).unwrap();
                game_log.entries.push(format!("Você larga {} no chão.", item_name.name));
            }
        }

        wants_to_drop_items.clear();
    }
}
