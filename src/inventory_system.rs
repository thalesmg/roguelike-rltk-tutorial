use specs::prelude::*;

use crate::components::*;
use crate::game_log::GameLog;
use crate::map::Map;

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

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, WantsToUseItem>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            map,
            mut game_log,
            names,
            healing_providers,
            consumables,
            inflicts_damages,
            areas_of_effects,
            mut confusions,
            mut combat_stats,
            mut wants_to_use_items,
            mut suffer_damages,
        ) = data;

        for (entity, item_user) in (&entities, &wants_to_use_items).join() {
            let item_name = names.get(item_user.item).unwrap();

            let mut targets = Vec::new();
            if let Some(target_point) = item_user.target {
                let idx = map.xy_idx(target_point.x as usize, target_point.y as usize);
                if let Some(aoe) = areas_of_effects.get(item_user.item) {
                    let mut blast_tiles =
                        rltk::field_of_view(target_point, aoe.radius as i32, &*map);
                    blast_tiles.retain(|p| {
                        p.x > 0
                            && p.x < map.width as i32 - 1
                            && p.y > 0
                            && p.y < map.height as i32 - 1
                    });
                    for blast_tile in blast_tiles.iter() {
                        let blast_tile_idx =
                            map.xy_idx(blast_tile.x as usize, blast_tile.y as usize);
                        for mob in map.tile_content[blast_tile_idx].iter() {
                            targets.push(*mob);
                        }
                    }
                } else {
                    for mob in map.tile_content[idx].iter() {
                        targets.push(*mob);
                    }
                }
            } else {
                targets.push(*player_entity);
            }

            if let Some(healer) = healing_providers.get(item_user.item) {
                for target in targets.iter() {
                    if let Some(stats) = combat_stats.get_mut(*target) {
                        stats.hp =
                            i32::min(stats.max_hp as i32, stats.hp + healer.heal_amount as i32);
                        if entity == *player_entity {
                            game_log.entries.push(format!(
                                "Você toma uma talagada de {}, e cura {} hp.",
                                item_name.name, healer.heal_amount
                            ));
                        }
                    }
                }
            }

            if let Some(InflictsDamage { damage }) = inflicts_damages.get(item_user.item) {
                for target in targets.iter() {
                    SufferDamage::new_damage(&mut suffer_damages, *target, *damage);
                    if entity == *player_entity {
                        let target_name = names.get(*target).unwrap();
                        game_log.entries.push(format!(
                            "Você usa {} em {}, causando {} de dano.",
                            item_name.name, target_name.name, damage
                        ));
                    }
                }
            }

            let mut to_confuse = Vec::new();
            if let Some(confusion) = confusions.get(item_user.item) {
                for target in targets.iter() {
                    to_confuse.push((target, confusion.turns));
                    if entity == *player_entity {
                        let target_name = names.get(*target).unwrap();
                        game_log.entries.push(format!(
                            "Você usa {} em {}, causando a maior confusão que até Deus duvida",
                            item_name.name, target_name.name
                        ));
                    }
                }
            }
            to_confuse.drain(..).for_each(|(target, turns)| {
                confusions
                    .insert(*target, Confusion { turns })
                    .expect("não consegui confundir!");
            });

            if let Some(_) = consumables.get(item_user.item) {
                entities
                    .delete(item_user.item)
                    .expect("não consegui consumir!");
            }
        }

        wants_to_use_items.clear();
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
        let (
            entities,
            player_entity,
            mut game_log,
            names,
            mut wants_to_drop_items,
            mut positions,
            mut in_backpacks,
        ) = data;

        for (entity, to_drop) in (&entities, &mut wants_to_drop_items).join() {
            let mut dropper_pos = Position { x: 0, y: 0 };
            {
                let pos = positions.get(entity).unwrap();
                dropper_pos.x = pos.x;
                dropper_pos.y = pos.y;
            }
            positions
                .insert(to_drop.item, dropper_pos)
                .expect("o item não voltou pro mapa!");
            in_backpacks.remove(to_drop.item);

            if entity == *player_entity {
                let item_name = names.get(to_drop.item).unwrap();
                game_log
                    .entries
                    .push(format!("Você larga {} no chão.", item_name.name));
            }
        }

        wants_to_drop_items.clear();
    }
}
