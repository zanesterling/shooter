use crate::dimensions::{WorldCoord as Coord, WorldPoint as Point, WorldRect as Rect};
use crate::map::{Map, TilePoint, ToTilePoint};
use crate::sprite_sheet::SpriteKey;

const TICKS_PER_SEC: u32 = 120; // TODO: Drop to 24 when fps and tps differ.

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
  pub units: Vec<Unit>,
  pub map: Map,
  pub next_uid: UID,
}

impl State {
  pub fn blank() -> State {
    State {
      units: vec![],

      map: Map::from_file("media/test-map.txt").expect("couldn't load the map"),

      next_uid: 0,
    }
  }

  pub fn level1() -> State {
    let mut state = State::blank();

    let uid = state.next_uid();
    state.units.push(Unit {
      uid,
      pos: Point::new(Coord(100.), Coord(100.)),
      rad: Coord(10.0),
      base_speed: Coord(1.0),

      sprite_key: "newt_gingrich".to_string(),
    });

    state
  }

  pub fn tick(&mut self) {
    // TODO: Implement this
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

pub struct Unit {
  pub uid: UID,
  pub pos: Point,
  pub rad: Coord,
  pub base_speed: Coord,
  pub sprite_key: SpriteKey,
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
}
