use std::collections::HashMap;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::dimensions::WindowPoint;
use crate::game;
use crate::map::{GridTile, TILE_WIDTH};
use crate::State;

const EMPTY_TILE_COLOR: Color = Color::RGB(40, 42, 54);
const OBSTACLE_COLOR: Color = Color::RGB(255, 184, 108);
const UNIT_COLOR: Color = Color::RGB(255, 121, 198);

const COLOR_WHITE: Color = Color::RGB(248, 248, 242);

pub fn draw_frame(canvas: &mut Canvas<Window>, state: &mut State) {
  canvas.set_draw_color(Color::BLACK);
  canvas.clear();

  for tile in state.game.map.tiles() {
    canvas.set_draw_color(match tile.tile {
      GridTile::Empty => EMPTY_TILE_COLOR,
      GridTile::Obstacle => OBSTACLE_COLOR,
    });
    let window_pos = tile.pos.to_world_point().to_window(state.camera_pos());
    let _ = canvas.fill_rect(Rect::new(
      window_pos.x(),
      window_pos.y(),
      TILE_WIDTH,
      TILE_WIDTH,
    ));
  }

  for player in state.game.players.iter() {
    draw_unit(canvas, state, &player.unit);
  }
  for unit in state.game.units.iter() {
    draw_unit(canvas, state, unit);
  }
}

fn draw_unit(canvas: &mut Canvas<Window>, state: &State, unit: &game::Unit) {
  canvas.set_draw_color(UNIT_COLOR);
  let bounds = unit.bounding_box();
  let _ = canvas.fill_rect(bounds.to_window_rect(state.camera_pos));
}

fn rect_from_points(p1: WindowPoint, p2: WindowPoint) -> Rect {
  let xmin = i32::min(p1.x(), p2.x());
  let xmax = i32::max(p1.x(), p2.x());
  let ymin = i32::min(p1.y(), p2.y());
  let ymax = i32::max(p1.y(), p2.y());
  Rect::new(xmin, ymin, (xmax - xmin) as u32, (ymax - ymin) as u32)
}

fn rect_from_center_rad(p: WindowPoint, rad: u32) -> Rect {
  Rect::from_center(p, rad * 2, rad * 2)
}

// A text renderer which caches all the created textures.
//
// This is significantly faster than re-rendering if you often render the same
// text, because you don't have to request and close a new texture each time --
// an expensive operation!
pub struct CachingTextRenderer<'canvas> {
  texture_creator: &'canvas TextureCreator<WindowContext>,
  texture_map: HashMap<String, Texture<'canvas>>,
}

impl<'canvas> CachingTextRenderer<'canvas> {
  pub fn new(
    texture_creator: &'canvas TextureCreator<WindowContext>,
  ) -> CachingTextRenderer<'canvas> {
    CachingTextRenderer {
      texture_creator,
      texture_map: HashMap::new(),
    }
  }

  fn render_text(&mut self, font: &Font, text: &str) -> Result<&Texture<'canvas>, String> {
    if self.texture_map.contains_key(text) {
      return Ok(self.texture_map.get(text).unwrap());
    }

    let surface = font
      .render(text)
      .solid(COLOR_WHITE)
      .map_err(|e| format!("couldn't render text: {}", e))?;
    let texture = self
      .texture_creator
      .create_texture_from_surface(&surface)
      .map_err(|e| format!("couldn't create texture: {}", e))?;
    self.texture_map.insert(text.to_string(), texture);
    Ok(self.texture_map.get(text).unwrap())
  }

  // TODO: Improve resolution of drawn text.
  fn draw_to_canvas(
    &mut self,
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
    p: WindowPoint,
  ) -> Result<(), String> {
    let texture = self.render_text(font, text)?;

    let bounds = texture.query();
    let target_rect = Rect::new(p.x, p.y, bounds.width, bounds.height);
    canvas
      .copy(&texture, None, target_rect)
      .map_err(|e| format!("couldn't copy texture to canvas: {}", e))
  }
}
