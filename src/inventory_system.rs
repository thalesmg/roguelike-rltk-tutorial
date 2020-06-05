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
        let (player_entity, mut game_log, names, mut wants_to_pickup_items, mut positions, mut in_backpacks) = data;

        for pickup in (&wants_to_pickup_items).join() {
            positions.remove(pickup.item);
            in_backpacks.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("não consegui colocar na bolsa!");

            if pickup.collected_by == *player_entity {
                let name = &names.get(pickup.item).unwrap().name;
                game_log.entries.push(format!("Você pegou {}", name));
            }
        }

        wants_to_pickup_items.clear();
    }
}
