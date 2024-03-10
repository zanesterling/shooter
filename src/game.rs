use sdl2::keyboard::Keycode;

use crate::dimensions::{WorldCoord as Coord, WorldPoint as Point, WorldRect as Rect};
use crate::map::Map;
use crate::sprite_sheet::SpriteKey;

const TICKS_PER_SEC: u32 = 120; // TODO: Drop to 24 when fps and tps differ.
const TICK_TIME: f32 = 1.0 / (TICKS_PER_SEC as f32);

#[derive(Clone, Copy, PartialEq)]
pub struct GameDur {
  pub ticks: u32,
}
impl GameDur {
  pub fn from_secs(secs: u32) -> GameDur {
    GameDur {
      ticks: secs * TICKS_PER_SEC,
    }
  }
}

// UIDs are used to refer uniquely to buildings or units.
pub type UID = u32;

pub struct State {
  pub players: Vec<Player>,
  pub units: Vec<Unit>,
  pub bullets: Vec<Bullet>,

  pub map: Map,

  pub next_uid: UID,
}

impl State {
  pub fn blank() -> State {
    State {
      players: vec![],
      units: vec![],
      bullets: vec![],

      map: Map::from_file("media/test-map.txt").expect("couldn't load the map"),

      next_uid: 0,
    }
  }

  pub fn level1() -> State {
    let mut state = State::blank();

    let uid = state.next_uid();
    state.players.push(Player {
      keys: PlayerKeys {
        up: Keycode::W,
        down: Keycode::S,
        left: Keycode::A,
        right: Keycode::D,
      },
      unit: Unit {
        uid,
        sprite_key: "newt_gingrich".to_string(),

        pos: Point::new(Coord(100.0), Coord(100.0)),
        heading: Point::new(Coord(1.0), Coord(0.0)),
        move_dir: Point::new(Coord(0.0), Coord(0.0)),
        rad: Coord(10.0),
        base_speed: Coord(300.0),

        shooting: false,
        ticks_per_shot: TICKS_PER_SEC / 2,
        ticks_to_shot: 0,
      },
    });

    state
  }

  pub fn tick(&mut self) {
    for player in self.players.iter_mut() {
      let vel = player.unit.move_dir.normalized() * Coord(TICK_TIME) * player.unit.base_speed;
      let new_pos = player.unit.pos + vel;
      let new_bounds = player.unit.bounding_box_at(new_pos);
      if !self.map.rect_intersects_wall(new_bounds) {
        player.unit.pos = player.unit.pos + vel;
      }

      if player.unit.shooting && player.unit.ticks_to_shot == 0 {
        let heading = player.unit.heading;
        self.bullets.push(Bullet {
          pos: player.unit.pos + heading * player.unit.rad * Coord(1.1),
          heading,
          rad: Coord(2.0),
          speed: Coord(500.0),
          will_die_at_end_of_tick: false,
        });
        player.unit.ticks_to_shot = player.unit.ticks_per_shot;
      }

      if player.unit.ticks_to_shot > 0 {
        player.unit.ticks_to_shot -= 1;
      }
    }

    for bullet in self.bullets.iter_mut() {
      let vel = bullet.heading * bullet.speed * Coord(TICK_TIME);
      bullet.pos = bullet.pos + vel;
      if self.map.rect_intersects_wall(bullet.bounding_box()) {
        bullet.will_die_at_end_of_tick = true;
      }

      // TODO: Check for intersection with player and update health.
    }
    self.bullets.retain(|b| !b.will_die_at_end_of_tick);
  }

  fn next_uid(&mut self) -> UID {
    let uid = self.next_uid;
    if self.next_uid == UID::MAX {
      println!("error: ran out of UIDs!");
    }
    self.next_uid += 1;
    uid
  }
}

pub struct Player {
  pub keys: PlayerKeys,
  pub unit: Unit,
}

#[derive(Debug)]
pub struct PlayerKeys {
  pub up: Keycode,
  pub down: Keycode,
  pub left: Keycode,
  pub right: Keycode,
}

pub struct Unit {
  pub uid: UID,
  pub sprite_key: SpriteKey,

  pub pos: Point,
  pub heading: Point,
  pub move_dir: Point,
  pub rad: Coord,
  pub base_speed: Coord,

  pub shooting: bool,
  pub ticks_per_shot: u32,
  pub ticks_to_shot: u32,
}

impl Unit {
  fn speed(&self) -> Coord {
    self.base_speed
  }

  fn rad(&self) -> Coord {
    self.rad
  }

  pub fn bounding_box(&self) -> Rect {
    self.bounding_box_at(self.pos)
  }
  fn bounding_box_at(&self, p: Point) -> Rect {
    let top_left = p - Point::new(self.rad(), self.rad());
    Rect {
      top_left,
      width: self.rad() * Coord(2.),
      height: self.rad() * Coord(2.),
    }
  }

  pub fn window_rad(&self) -> u32 {
    self.rad().0 as u32
  }

  pub fn aim_at(&mut self, target: Point) {
    let heading_raw = target - self.pos;
    if heading_raw.x == Coord(0.0) && heading_raw.y == Coord(0.0) {
      // Then avoid a zero heading by keeping the old heading.
    } else {
      self.heading = heading_raw.normalized();
    }
  }
}

pub struct Bullet {
  pub pos: Point,
  pub heading: Point,
  pub speed: Coord,
  pub rad: Coord,

  pub will_die_at_end_of_tick: bool,
}

impl Bullet {
  pub fn bounding_box(&self) -> Rect {
    Rect {
      top_left: Point::new(self.pos.x - self.rad, self.pos.y - self.rad),
      width: self.rad * Coord(2.),
      height: self.rad * Coord(2.),
    }
  }
}
