use rltk::Console;
use rltk::RGB;
use rltk::Rltk;
use std::cmp::max;
use std::cmp::min;

pub const WIDTH: usize = 80;
pub const HEIGHT: usize = 50;
pub const MAX_ROOMS: usize = 30;
pub const MIN_ROOM_SIZE: usize = 6;
pub const MAX_ROOM_SIZE: usize = 10;

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Clone, Debug)]
pub struct Rect {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl Rect {
    fn new(x: usize, y: usize, h: usize, w: usize) -> Self {
        Self{
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    fn intersects(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && other.x1 <= self.x2 &&
            self.y1 <= other.y2 && other.y1 <= self.y2
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * WIDTH) + x as usize
}

pub fn idx_xy(idx: usize) -> (i32, i32) {
    let y = idx / WIDTH;
    let x = idx % WIDTH;
    (x as i32, y as i32)
}

pub fn new_map() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; WIDTH * HEIGHT];

    let mut rooms = Vec::with_capacity(MAX_ROOMS);

    let mut rng = rltk::RandomNumberGenerator::new();

    'room_loop: for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
        let h = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
        let x = rng.roll_dice(1, WIDTH as i32 - w as i32 - 1) - 1;
        let y = rng.roll_dice(1, HEIGHT as i32 - h as i32 - 1) - 1;

        let room = Rect::new(x as usize, y as usize, h, w);

        for other_room in rooms.iter() {
            if room.intersects(other_room) {
                continue 'room_loop;
            }
        }

        apply_room_to_map(&room, &mut map);
        if rooms.len() != 0 {
            let (last_x, last_y) = rooms[rooms.len() - 1].center();
            let (new_x, new_y) = room.center();

            if rng.rand() {
                apply_horizontal_tunnel(&mut map, last_x, new_x, last_y);
                apply_vertical_tunnel(&mut map, last_y, new_y, new_x);
            } else {
                apply_horizontal_tunnel(&mut map, last_x, new_x, new_y);
                apply_vertical_tunnel(&mut map, last_y, new_y, last_x);
            }
        }

        rooms.push(room);
    }

    (rooms, map)
}

pub fn new_map_test() -> Vec<TileType> {
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

fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for x in room.x1 + 1..=room.x2 {
        for y in room.y1 + 1..=room.y2 {
            map[xy_idx(x as i32, y as i32)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: usize, x2: usize, y: usize) {
    for x in min(x1, x2)..=max(x1, x2) {
        map[xy_idx(x as i32, y as i32)] = TileType::Floor;
    }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: usize, y2: usize, x: usize) {
    for y in min(y1, y2)..=max(y1, y2) {
        map[xy_idx(x as i32, y as i32)] = TileType::Floor;
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

    impl Arbitrary for Rect {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut p1 = g.gen::<[usize; 2]>();
            p1.sort();
            let mut p2 = g.gen::<[usize; 2]>();
            p2.sort();

            Rect {
                x1: p1[0],
                y1: p1[1],
                x2: p2[0],
                y2: p2[1],
            }
        }
    }

    #[quickcheck]
    fn xy_idx_id(x: W, y: H) -> bool {
        let W(x) = x;
        let H(y) = y;
        idx_xy(xy_idx(x, y)) == (x, y)
    }

    #[quickcheck]
    fn interesects_reflexive(r1: Rect, r2: Rect) -> bool {
        let r1_i_r2 = r1.intersects(&r2);
        let r2_i_r1 = r2.intersects(&r1);

        r1_i_r2 == r2_i_r1
    }

    #[test]
    fn intersects_partial() {
        let r1 = Rect{x1: 0, y1: 0, x2: 5, y2: 5};
        let r2 = Rect{x1: 2, y1: 2, x2: 7, y2: 7};
        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
    }

    #[test]
    fn intersects_contained() {
        let r1 = Rect{x1: 0, y1: 0, x2: 5, y2: 5};
        let r2 = Rect{x1: 2, y1: 2, x2: 3, y2: 3};
        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
    }

    #[test]
    fn intersects_separated() {
        let r1 = Rect{x1: 0, y1: 0, x2: 5, y2: 5};
        let r2 = Rect{x1: 6, y1: 6, x2: 7, y2: 7};
        assert!(!r1.intersects(&r2));
        assert!(!r2.intersects(&r1));
    }
}
