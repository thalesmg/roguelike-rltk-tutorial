use rltk::RGB;
use specs::prelude::*;

pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: u16,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
    pub range: u8,
    pub visible_tiles: Vec<rltk::Point>,
    pub dirty: bool,
}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component)]
pub struct BlocksTile {}

#[derive(Component)]
pub struct CombatStats {
    pub max_hp: u32,
    pub hp: i32,
    pub defense: u32,
    pub power: u32,
}

#[derive(Component)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component)]
pub struct SufferDamage {
    pub amount: Vec<u32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: u32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = Self {
                amount: vec![amount],
            };
            store
                .insert(victim, dmg)
                .expect("não deu para inserir dano!");
        }
    }
}

#[derive(Component)]
pub struct Item {}

#[derive(Component)]
pub struct HealthPotion {
    pub heal_amount: usize,
}

#[derive(Component)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}
