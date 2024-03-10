#[allow(dead_code)]
mod dimensions;
#[allow(dead_code)]
mod draw;
#[allow(dead_code)]
mod game;
#[allow(dead_code)]
mod map;
#[allow(dead_code)]
mod sprite_sheet;

extern crate rand;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::Sdl;

use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::dimensions::{ToWorld, WindowPoint, WorldCoord, WorldPoint};
use crate::draw::{draw_frame, CachingTextRenderer};
use crate::sprite_sheet::SpriteSheet;

const SPRITE_SHEET_PATH: &str = "media/sprite-sheet.sps";

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

const TARGET_FRAME_PER_SEC: u32 = 120;
const TARGET_FRAME_DUR: Duration = Duration::new(0, 1_000_000_000u32 / TARGET_FRAME_PER_SEC);

const PERF_DEBUG: bool = false; // Enable this to get some perf debug info.
macro_rules! perf {
  ($( $x:expr ),* ) => {
    if PERF_DEBUG { println!( $($x),* ); }
  };
}
const MEAN_FRAME_DEBUG: bool = false;
const LONG_FRAME_DEBUG: bool = false;

#[allow(dead_code)]
struct State<'canvas, 'b> {
  // "Immutable" stuff.
  sprite_sheet: SpriteSheet<'canvas>,
  font: Font<'b, 'static>,
  text_renderer: CachingTextRenderer<'canvas>,

  // State of the game.
  running: bool,
  game: game::State,

  // Interaction state.
  key_state: KeyState,
  camera_pos: WorldPoint,
  mouse_pos: WindowPoint,
}

impl<'canvas, 'b> State<'canvas, 'b> {
  pub fn new<'s, 'f>(
    sprite_sheet: SpriteSheet<'s>,
    font: Font<'f, 'static>,
    text_renderer: CachingTextRenderer<'s>,
  ) -> State<'s, 'f> {
    State {
      sprite_sheet,
      font,
      text_renderer,

      running: true,
      game: game::State::level1(),

      key_state: KeyState::new(),
      camera_pos: WorldPoint::new(WorldCoord(0.), WorldCoord(0.)),
      // This is wrong, but will be set on the next MouseMotion event.
      mouse_pos: WindowPoint::new(0, 0),
    }
  }

  // Returns a world point corresponding to the top-left corner of the
  // renderable window.
  pub fn camera_pos(&self) -> WorldPoint {
    self.camera_pos
  }
}

#[allow(dead_code)]
struct DisplayBounds {
  top_left_x: i32,
  top_left_y: i32,
  width: u32,
  height: u32,
}

struct KeyState {
  left_ctrl_down: bool,
  right_ctrl_down: bool,
  left_shift_down: bool,
  right_shift_down: bool,
  left_alt_down: bool,
  right_alt_down: bool,
}

#[allow(dead_code)]
impl KeyState {
  pub fn new() -> KeyState {
    KeyState {
      left_ctrl_down: false,
      right_ctrl_down: false,
      left_shift_down: false,
      right_shift_down: false,
      left_alt_down: false,
      right_alt_down: false,
    }
  }

  pub fn ctrl(&self) -> bool {
    self.left_ctrl_down || self.right_ctrl_down
  }
  pub fn shift(&self) -> bool {
    self.left_shift_down || self.right_shift_down
  }
  pub fn alt(&self) -> bool {
    self.left_alt_down || self.right_alt_down
  }

  pub fn update_shift_alt_ctrl(&mut self, keycode: Option<Keycode>, is_down: bool) {
    match keycode {
      Some(Keycode::LCtrl) => {
        self.left_ctrl_down = is_down;
      }
      Some(Keycode::RCtrl) => {
        self.right_ctrl_down = is_down;
      }
      Some(Keycode::LShift) => {
        self.left_shift_down = is_down;
      }
      Some(Keycode::RShift) => {
        self.right_shift_down = is_down;
      }
      Some(Keycode::LAlt) => {
        self.left_alt_down = is_down;
      }
      Some(Keycode::RAlt) => {
        self.right_alt_down = is_down;
      }
      _ => {}
    }
  }
}

fn main() {
  let sdl_context = sdl2::init().unwrap();
  let _sdl_image_context = image::init(image::InitFlag::PNG).unwrap();

  let sdl_ttf_context = sdl2::ttf::init().unwrap();
  let font = sdl_ttf_context
    .load_font("media/Serif.ttf", 24)
    .expect("couldn't load font");

  let video = sdl_context.video().unwrap();

  let window = video
    .window("rts!", WINDOW_WIDTH, WINDOW_HEIGHT)
    .position_centered()
    .build()
    .unwrap();

  let canvas = window.into_canvas().software().build().unwrap();
  let canvas_txc = canvas.texture_creator();

  let sprite_sheet = SpriteSheet::from_file(SPRITE_SHEET_PATH, &canvas_txc).unwrap_or_else(|e| {
    println!(
      "error loading sprite sheet \"{}\": {}",
      SPRITE_SHEET_PATH, e
    );
    exit(1);
  });

  let text_renderer = CachingTextRenderer::new(&canvas_txc);

  let state = { State::new(sprite_sheet, font, text_renderer) };
  main_loop(state, canvas, sdl_context);
}

fn main_loop(mut state: State, mut canvas: Canvas<Window>, sdl_context: Sdl) {
  let mut event_pump = sdl_context.event_pump().unwrap();
  let mut mean_frame_dur = Duration::from_nanos(0);
  while state.running {
    let frame_start = Instant::now();

    // Handle input.
    for event in event_pump.poll_iter() {
      handle_event(&mut state, &mut canvas, event);
    }
    let events_done = Instant::now();

    // Update world.
    // TODO: Make game ticks operate on a different clock than render ticks.
    state.game.tick();
    let tick_done = Instant::now();

    // Render.
    draw_frame(
      &mut canvas,
      &mut state, // this ref is mut to allow mutation of the text_renderer
    );
    let render_done = Instant::now();
    // TODO: If the user is dragging the screen around, this call might block.
    // Consider using a non-blocking variant.
    canvas.present();
    let present_done = Instant::now();

    let frame_dur = frame_start.elapsed();
    mean_frame_dur = mean_frame_dur * 9 / 10 + frame_dur * 1 / 10;
    if MEAN_FRAME_DEBUG {
      perf!("mean_frame_dur: {:?}", mean_frame_dur);
    }
    if frame_dur < TARGET_FRAME_DUR {
      sleep(TARGET_FRAME_DUR - frame_dur);
    } else if LONG_FRAME_DEBUG {
      perf!(
        "err: long frame took {:?} > {:?}",
        frame_dur,
        TARGET_FRAME_DUR
      );
    }
    perf!(
      "frame:   {:?}\n\
       -------------\n\
       events:  {:?}\n\
       tick:    {:?}\n\
       render:  {:?}\n\
       present: {:?}\n",
      frame_dur,
      events_done - frame_start,
      tick_done - events_done,
      render_done - events_done,
      present_done - render_done
    );
  }
}

fn handle_event(state: &mut State, _canvas: &mut Canvas<Window>, event: Event) {
  match event {
    // Quit.
    Event::Quit { .. }
    | Event::KeyDown {
      keycode: Some(Keycode::Escape),
      ..
    } => {
      state.running = false;
    }

    // Left mouse down / up: box select.
    Event::MouseButtonDown {
      x,
      y,
      mouse_btn: MouseButton::Left,
      ..
    } => {
      for player in state.game.players.iter_mut() {
        // TODO: When using gamepads, separate out who's who.
        player.unit.shooting = true;
      }
    }
    Event::MouseButtonUp {
      x,
      y,
      mouse_btn: MouseButton::Left,
      ..
    } => {
      for player in state.game.players.iter_mut() {
        // TODO: When using gamepads, separate out who's who.
        player.unit.shooting = false;
      }
    }

    Event::MouseMotion {
      x, y, xrel, yrel, ..
    } => {
      let mouse_pos_world = WindowPoint::new(x, y).to_world(state.camera_pos);
      for player in state.game.players.iter_mut() {
        // TODO: When using gamepads, separate out who's who.
        player.unit.aim_at(mouse_pos_world);
      }
    }

    Event::KeyDown {
      repeat: false,
      keycode,
      ..
    } => {
      state.key_state.update_shift_alt_ctrl(keycode, true);
      if let Some(keycode) = keycode {
        for player in state.game.players.iter_mut() {
          if keycode == player.keys.up {
            player.unit.move_dir.y.0 -= 1.0;
          }
          if keycode == player.keys.down {
            player.unit.move_dir.y.0 += 1.0;
          }
          if keycode == player.keys.left {
            player.unit.move_dir.x.0 -= 1.0;
          }
          if keycode == player.keys.right {
            player.unit.move_dir.x.0 += 1.0;
          }
        }
      }
    }
    Event::KeyUp { keycode, .. } => {
      state.key_state.update_shift_alt_ctrl(keycode, false);
      if let Some(keycode) = keycode {
        for player in state.game.players.iter_mut() {
          if keycode == player.keys.up {
            player.unit.move_dir.y.0 += 1.0;
          }
          if keycode == player.keys.down {
            player.unit.move_dir.y.0 -= 1.0;
          }
          if keycode == player.keys.left {
            player.unit.move_dir.x.0 += 1.0;
          }
          if keycode == player.keys.right {
            player.unit.move_dir.x.0 -= 1.0;
          }
        }
      }
    }

    _ => {}
  }
}
