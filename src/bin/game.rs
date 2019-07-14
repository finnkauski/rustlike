use roguelike::objects::{Object, Tile};
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

const LIMIT_FPS: i32 = 20;

// type synonims
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
fn make_map() -> Map {
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    // TODO: remove, just here for tests
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();
    map
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
pub fn handle_keys(root: &mut Root, player: &mut Object) -> bool {
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

    // initialise the objects
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', colors::WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::YELLOW);
    let mut objects = [player, npc];

    // main loop
    while !root.window_closed() {
        // clear the screen
        con.clear();

        // generate map
        let map = make_map();

        // render the whole thing
        render_all(&mut root, &mut con, &objects, &map);

        // key handler
        let exit = handle_keys(&mut root, &mut objects[0]);
        if exit {
            break;
        }
    }
}
