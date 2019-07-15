// ingest other files as modules
pub mod components;
pub mod objects;

// namespacing
use components::{Ai, Fighter};
use objects::{Object, Rect, Tile};
use rand::Rng;
use std::cmp;
use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

// constants

// player index in the object list
pub const PLAYER: usize = 0;

// screen settings
pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;

// map settings
pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

// colour palette
pub const COLOR_LIGHT_WALL: Color = Color { r: 0, g: 0, b: 100 };
pub const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };

pub const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
pub const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

// room generation params
pub const ROOM_MAX_SIZE: i32 = 10;
pub const ROOM_MIN_SIZE: i32 = 10;
pub const MAX_ROOMS: i32 = 40;

// monster params
pub const MAX_ROOM_MONSTERS: i32 = 3;
// fps
pub const LIMIT_FPS: i32 = 20;

// fov settings
pub const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
pub const FOV_LIGHT_WALLS: bool = true;
pub const TORCH_RADIUS: i32 = 10;

// type synonyms
type Map = Vec<Vec<Tile>>;

// enums
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

// initialise the root window
pub fn get_root() -> Root {
    Root::initializer()
        .font("resources/arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Game")
        .init()
}

// make map
pub fn make_map(objects: &mut Vec<Object>) -> Map {
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
            place_objects(&new_room, objects);

            if rooms.is_empty() {
                // first room handled here to get the starting position of the person
                objects[PLAYER].set_pos(new_x, new_y);
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
    map
}

// place room on map
pub fn create_room(room: &Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

pub fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

pub fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

// movement functions
pub fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    // move the object by a given amount
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

pub fn player_move_or_attack(dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    // figure out where the player is going to move to
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;
    // see if anything is in these coordinates
    let target_id = objects
        .iter()
        .position(|object| object.fighter.is_some() && object.pos() == (x, y));
    // attack if target found, otherwise move
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(PLAYER, target_id, objects);
            player.attack(target);
        }
        None => move_by(PLAYER, dx, dy, map, objects),
    }
}

pub fn move_towards(id: usize, target_x: i32, target_y: i32, map: &Map, objects: &mut [Object]) {
    // vector from this object to the target and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, map, objects);
}

// place objects on map
pub fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    if map[x as usize][y as usize].blocked {
        return true;
    }

    objects.iter().any(|obj| obj.blocks && obj.pos() == (x, y))
}

pub fn place_objects(room: &Rect, objects: &mut Vec<Object>) {
    // choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        // choose location
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        let mut monster = if rand::random::<f32>() < 0.8 {
            // chance of getting an orc
            let mut orc = Object::new(x, y, 'o', colors::DESATURATED_GREEN, "orc", true, true);
            orc.fighter = Some(Fighter {
                max_hp: 10,
                hp: 10,
                defense: 0,
                power: 3,
                on_death: DeathCallback::Monster,
            });
            orc.ai = Some(Ai);
            orc
        } else {
            let mut troll = Object::new(x, y, 'T', colors::DARKER_GREEN, "troll", true, true);
            troll.fighter = Some(Fighter {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
                on_death: DeathCallback::Monster,
            });
            troll.ai = Some(Ai);
            troll
        };
        monster.alive = true;
        objects.push(monster);
    }
}

// rendering system
pub fn render_all(
    root: &mut Root,
    con: &mut Offscreen,
    objects: &[Object],
    map: &mut Map,
    fov_map: &mut FovMap,
    fov_recompute: bool,
) {
    if fov_recompute {
        let player = &objects[PLAYER];
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

    // create a new vector to sort so we don't mess up the indices in the main one
    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| fov_map.is_in_fov(o.x, o.y))
        .collect();
    // sort so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw all objects in a list
    for object in &to_draw {
        if fov_map.is_in_fov(object.x, object.y) {
            object.draw(con);
        }
    }

    // blit the console onto the root
    blit(con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), root, (0, 0), 1.0, 1.0);
    root.flush();
}

// handle the keypresses
pub fn handle_keys(root: &mut Root, map: &Map, objects: &mut [Object]) -> PlayerAction {
    use tcod::input::{Key, KeyCode::*};
    use PlayerAction::*;

    let key = root.wait_for_keypress(true);
    let player_alive = objects[PLAYER].alive;

    match (key, player_alive) {
        (Key { code: Up, .. }, true) => {
            player_move_or_attack(0, -1, map, objects);
            TookTurn
        }
        (Key { code: Down, .. }, true) => {
            player_move_or_attack(0, 1, map, objects);
            TookTurn
        }
        (Key { code: Left, .. }, true) => {
            player_move_or_attack(-1, 0, map, objects);
            TookTurn
        }
        (Key { code: Right, .. }, true) => {
            player_move_or_attack(1, 0, map, objects);
            TookTurn
        }

        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
        ) => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }

        (Key { code: Escape, .. }, _) => Exit,

        _ => DidntTakeTurn,
    }
}

// handle turns
// deals with borrowing from same list
pub fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}
// take turn
pub fn ai_take_turn(monster_id: usize, map: &Map, objects: &mut [Object], fov_map: &FovMap) {
    // a basic monster takes its turn. If you can see it, it can see you
    let (monster_x, monster_y) = objects[monster_id].pos();
    if fov_map.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[PLAYER]) >= 2.0 {
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(monster_id, player_x, player_y, map, objects);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            let (monster, player) = mut_two(monster_id, PLAYER, objects);
            monster.attack(player);
        }
    }
}

// handling death callbacks
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    pub fn callback(self, object: &mut Object) {
        use DeathCallback::*;
        let callback: fn(&mut Object) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object);
    }
}

pub fn player_death(player: &mut Object) {
    // the game ends
    println!("You died!");

    // for added effect transform the player into a corspe!
    player.char = '%';
    player.color = colors::DARK_RED;
}

pub fn monster_death(monster: &mut Object) {
    //transform to corpse, can't be attacked and doesn't move
    monster.char = '%';
    monster.color = colors::DARK_RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("corpse of {}", monster.name);
}
