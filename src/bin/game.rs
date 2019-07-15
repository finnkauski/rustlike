use roguelike::components::Fighter;
use roguelike::objects::Object;
use roguelike::*;
use tcod::colors;
use tcod::console::*;
use tcod::map::Map as FovMap;

fn main() {
    tcod::system::set_fps(LIMIT_FPS);
    // initialise root console

    let mut root = get_root();
    let mut con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    // initialise the objects
    let mut previous_player_position = (-1, -1); // need this to make sure we recompute the fov map if player moves

    // instantiate a player
    let mut player = Object::new(
        SCREEN_WIDTH / 2,
        SCREEN_HEIGHT / 2,
        '@',
        colors::WHITE,
        "player",
        true,
        true,
    );

    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        on_death: DeathCallback::Player,
    });

    let mut objects = vec![player];

    // generate map
    let mut map = make_map(&mut objects);

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
        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
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
        let action = handle_keys(&mut root, &map, &mut objects);
        if action == PlayerAction::Exit {
            break;
        }

        if objects[PLAYER].alive && action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(id, &map, &mut objects, &fov_map)
                }
            }
        }

        if let Some(fighter) = objects[PLAYER].fighter {
            root.print_ex(
                1,
                SCREEN_HEIGHT - 2,
                BackgroundFlag::None,
                TextAlignment::Left,
                format!("HP: {}/{} ", fighter.hp, fighter.max_hp),
            )
        }
    }
}
