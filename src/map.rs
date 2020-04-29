use rltk::Console;
use rltk::RGB;
use rltk::Rltk;

pub const WIDTH: usize = 80;
pub const HEIGHT: usize = 50;

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
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

pub fn new_map() -> Vec<TileType> {
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

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
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
