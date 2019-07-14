use rand::Rng;
use roguelike::objects::{Object, Rect, Tile};
use std::cmp;
use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;

// constants
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 10;
const MAX_ROOMS: i32 = 30;

const LIMIT_FPS: i32 = 20;

// type synonyms
type Map = Vec<Vec<Tile>>;

// initialise the root window
fn get_root() -> Root {
    Root::initializer()
        .font("resources/arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Game")
        .init()
}

// make map
fn make_map() -> (Map, (i32, i32)) {
    let mut starting_position = (0, 0);
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // store center values
            let (new_x, new_y) = new_room.center();

            // if there are no intersections this room is valid and we can 'carve it'
            create_room(new_room, &mut map);
            if rooms.is_empty() {
                // first room handled here to get the starting position of the person
                starting_position = (new_x, new_y);
            }
        }
    }
    (map, starting_position)
}

// place room on map
fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

// rendering system
fn render_all(root: &mut Root, con: &mut Offscreen, objects: &[Object], map: &Map) {
    // draw the map first
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = map[x as usize][y as usize].block_sight;
            if wall {
                con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }
    // draw all objects in a list
    for object in objects {
        object.draw(con);
    }

    // blit the console onto the root
    blit(con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), root, (0, 0), 1.0, 1.0);
    root.flush();
}

// handle the keypresses
fn handle_keys(root: &mut Root, player: &mut Object) -> bool {
    use tcod::input::{Key, KeyCode::*};

    let key = root.wait_for_keypress(true);
    match key {
        Key { code: Up, .. } => player.move_by(0, -1),
        Key { code: Down, .. } => player.move_by(0, 1),
        Key { code: Left, .. } => player.move_by(-1, 0),
        Key { code: Right, .. } => player.move_by(1, 0),

        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }

        Key { code: Escape, .. } => return true,

        _ => {}
    }

    false
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);
    // initialise root console

    let mut root = get_root();
    let mut con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    // generate map
    let (map, (player_x, player_y)) = make_map();

    // initialise the objects
    let player = Object::new(player_x, player_y, '@', colors::WHITE);
    let mut objects = [player];

    // main loop
    while !root.window_closed() {
        // clear the screen
        con.clear();

        // render the whole thing
        render_all(&mut root, &mut con, &objects, &map);

        // key handler
        let exit = handle_keys(&mut root, &mut objects[0]);
        if exit {
            break;
        }
    }
}
