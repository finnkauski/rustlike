use rand::Rng;
use roguelike::objects::{Object, Rect, Tile};
use std::cmp;
use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

// constants

// screen settings
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// map settings
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

// colour palette
const COLOR_LIGHT_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };

const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

// room generation params
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 10;
const MAX_ROOMS: i32 = 40;

// fps
const LIMIT_FPS: i32 = 20;

// fov settings
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

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
            create_room(&new_room, &mut map);

            if rooms.is_empty() {
                // first room handled here to get the starting position of the person
                starting_position = (new_x, new_y);
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                if rand::random() {
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            rooms.push(new_room);
        }
    }
    (map, starting_position)
}

// place room on map
fn create_room(room: &Rect, map: &mut Map) {
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
fn render_all(
    root: &mut Root,
    con: &mut Offscreen,
    objects: &[Object],
    map: &mut Map,
    fov_map: &mut FovMap,
    fov_recompute: bool,
) {
    if fov_recompute {
        let player = &objects[0];
        fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }
    // draw the map first
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = fov_map.is_in_fov(x, y);
            let wall = map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut map[x as usize][y as usize].explored;
            if visible {
                // if visible then its explored (we set the value here)
                *explored = true;
            }
            if *explored {
                // if its explored, only then do we show the tile
                con.set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    // draw all objects in a list
    for object in objects {
        if fov_map.is_in_fov(object.x, object.y) {
            object.draw(con);
        }
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
    let (mut map, (player_x, player_y)) = make_map();

    // initialise the objects
    let mut previous_player_position = (-1, -1); // need this to make sure we recompute the fov map if player moves

    // instantiate a player
    let player = Object::new(player_x, player_y, '@', colors::WHITE);
    let mut objects = [player];

    // initialise the fov map once the regular map is generated
    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov_map.set(
                x,
                y,
                !map[x as usize][y as usize].block_sight,
                !map[x as usize][y as usize].blocked,
            )
        }
    }

    // main loop
    while !root.window_closed() {
        // clear the screen
        con.clear();

        // render the whole thing
        let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
        render_all(
            &mut root,
            &mut con,
            &objects,
            &mut map,
            &mut fov_map,
            fov_recompute,
        );

        // declare player
        let player = &mut objects[0];

        // record current position for future check to recompute fov
        previous_player_position = (player.x, player.y);

        // key handler
        let exit = handle_keys(&mut root, player);
        if exit {
            break;
        }
    }
}
