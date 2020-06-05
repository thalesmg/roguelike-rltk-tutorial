use specs::prelude::*;
use rltk::RandomNumberGenerator;
use rltk::RGB;

use crate::components::*;
use crate::map::Rect;

const MAX_MONSTERS: usize = 4;
const MAX_ITEMS: usize = 2;

pub fn player(ecs: &mut World, x: usize, y: usize) -> Entity {
    ecs
        .create_entity()
        .with(Position {
            x: x as i32,
            y: y as i32,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            range: 8,
            visible_tiles: Vec::new(),
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build()
}

pub fn random_monster(ecs: &mut World, x: usize, y: usize) {
    let roll = {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        rng.rand()
    };

    if roll {
        goblin(ecs, x, y);
    } else {
        orc(ecs, x, y);
    }
}

fn goblin(ecs: &mut World, x: usize, y: usize) {
    monster(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

fn orc(ecs: &mut World, x: usize, y: usize) {
    monster(ecs, x, y, rltk::to_cp437('o'), "Orc");
}

fn monster<S : ToString>(ecs: &mut World, x: usize, y: usize, glyph: rltk::FontCharType, name: S) {
    ecs
        .create_entity()
        .with(Position {
            x: x as i32,
            y: y as i32,
        })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            range: 8,
            visible_tiles: Vec::new(),
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points = Vec::new();
    let mut item_spawn_points = Vec::new();

    {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS as i32 + 2) - 3;

        for _ in 0..=num_monsters {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 as i32 - room.x1 as i32)) as usize;
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 as i32 - room.y1 as i32)) as usize;
                if !monster_spawn_points.contains(&(x, y)) {
                    monster_spawn_points.push((x, y));
                    added = true;
                }
            }
        }
    }

    {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        let num_items = rng.roll_dice(1, MAX_ITEMS as i32 + 1) - 1;

        for _ in 0..=num_items {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 as i32 - room.x1 as i32)) as usize;
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 as i32 - room.y1 as i32)) as usize;
                if !item_spawn_points.contains(&(x, y)) {
                    item_spawn_points.push((x, y));
                    added = true;
                }
            }
        }
    }

    for (x, y) in monster_spawn_points.iter() {
        random_monster(ecs, *x, *y);
    }

    for (x, y) in item_spawn_points.iter() {
        health_potion(ecs, *x, *y);
    }
}

pub fn health_potion(ecs: &mut World, x: usize, y: usize) {
    ecs
        .create_entity()
        .with(Position { x: x as i32, y: y as i32 })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Item {})
        .with(Name { name: "Poção de Vida".to_string() })
        .with(HealthPotion { heal_amount: 8 })
        .build();
}
