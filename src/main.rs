#[macro_use]
extern crate specs_derive;

#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

use rltk::Console;
use rltk::GameState;
use rltk::Rltk;
use rltk::VirtualKeyCode;
use rltk::RGB;
use specs::prelude::*;
use std::cmp::max;
use std::cmp::min;

rltk::add_wasm_support!();

const WIDTH: usize = 80;
const HEIGHT: usize = 50;

struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.run_systems();

        player_input(self, ctx);

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

struct Position {
    x: i32,
    y: i32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

#[derive(PartialEq, Clone, Copy)]
enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * WIDTH) + x as usize
}

pub fn idx_xy(idx: usize) -> (i32, i32) {
    let y = idx / WIDTH;
    let x = idx % WIDTH;
    (x as i32, y as i32)
}

fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; WIDTH * HEIGHT];

    // bordas topo
    for x in 0..WIDTH as i32 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, HEIGHT as i32 - 1)] = TileType::Wall;
    }

    // bordas laterais
    for y in 0..HEIGHT as i32 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(WIDTH as i32 - 1, y)] = TileType::Wall;
    }

    let mut rng = rltk::RandomNumberGenerator::new();

    for _ in 0..400 {
        let x = rng.roll_dice(1, WIDTH as i32 - 1);
        let y = rng.roll_dice(1, HEIGHT as i32 - 1);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (pos, _player) in (&mut positions, &mut players).join() {
        let dest_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[dest_idx] != TileType::Wall {
            pos.x = min(WIDTH as i32 - 1, max(0, pos.x + delta_x));
            pos.y = min(HEIGHT as i32 - 1, max(0, pos.y + delta_y));
        }
    }
}

fn player_input(gs: &mut State, ctx: &Rltk) {
    match ctx.key {
        None => (),
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => (),
        },
    }
}

fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    for (i, tile) in map.iter().enumerate() {
        let (x, y) = idx_xy(i);
        match tile {
            TileType::Floor => ctx.set(
                x,
                y,
                RGB::from_f32(0.5, 0.5, 0.5),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('.'),
            ),
            TileType::Wall => ctx.set(
                x,
                y,
                RGB::from_f32(0., 1., 0.),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('#'),
            ),
        }
    }
}

fn main() {
    let context = Rltk::init_simple8x8(WIDTH as u32, HEIGHT as u32, "Olá mundo!", "resources");
    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    gs.ecs.insert(new_map());

    rltk::main_loop(context, gs);
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck::Arbitrary;
    use quickcheck::Gen;
    use rand::Rng;

    #[derive(Clone, Debug)]
    struct W(i32);

    #[derive(Clone, Debug)]
    struct H(i32);

    impl Arbitrary for W {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            W(g.gen_range(0, WIDTH as i32 - 1))
        }
    }

    impl Arbitrary for H {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            H(g.gen_range(0, HEIGHT as i32 - 1))
        }
    }

    #[quickcheck]
    fn xy_idx_id(x: W, y: H) -> bool {
        let W(x) = x;
        let H(y) = y;
        idx_xy(xy_idx(x, y)) == (x, y)
    }
}
