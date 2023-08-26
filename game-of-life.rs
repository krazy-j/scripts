extern crate clap;
extern crate rand;
extern crate miniquad;
extern crate rgb;
extern crate clipboard;

use std::ops::Range;
use std::time::Instant;
use clap::Parser;
use rand::{SeedableRng, RngCore};

use miniquad::{
	GraphicsContext, Pipeline, Bindings,
	Buffer, BufferType, VertexAttribute, VertexFormat,
	KeyCode, KeyMods, MouseButton,
};
use rgb::{ComponentBytes, RGB8};


trait DefaultColors {
	const BLACK: Self;
	const GRAY: Self;
	const WHITE: Self;
	const RED: Self;
	const YELLOW: Self;
	const GREEN: Self;
	const AQUA: Self;
	const BLUE: Self;
}

impl DefaultColors for RGB8 {
	const BLACK: Self = Self {r: 0, g: 0, b: 0};
	const GRAY: Self = Self {r: 128, g: 128, b: 128};
	const WHITE: Self = Self {r: 255, g: 255, b: 255};
	const RED: Self = Self {r: 255, g: 0, b: 0};
	const YELLOW: Self = Self {r: 255, g: 255, b: 0};
	const GREEN: Self = Self {r: 0, g: 255, b: 0};
	const AQUA: Self = Self {r: 0, g: 255, b: 255};
	const BLUE: Self = Self {r: 0, g: 0, b: 255};
}

#[repr(C)]
struct Vec2 {
	x: f32,
	y: f32,
}

#[repr(C)]
struct Vertex {
	pos: Vec2,
	uv: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellState {
	Dead = 0,
	Alive,
	Block,
}

const SHADER_VERT: &str = r#"#version 100
attribute vec2 pos;
attribute vec2 uv;

varying highp vec2 texcoord;

void main() {
	gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);
	texcoord = uv;
}"#;

const SHADER_FRAG: &str = r#"#version 100
varying highp vec2 texcoord;

uniform sampler2D tex;

void main() {
	gl_FragColor = texture2D(tex, texcoord);
}"#;

const BASE64: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const GLIDER: [u8; 7] = [
	1,0,3,
	    0,
	0,0,0,
];
const GLIDER_GUN: [u8; 61] = [
	// 36x9
	24,                                 0,11+
	22,                             0,1,0,11+
	12,         0,0,6,          0,0,12,         0,0,
	11,       0,3,    0,4,      0,0,12,         0,0,
	0,0,8,  0,5,        0,3,    0,0,14,
	0,0,8,  0,3,    0,1,0,0,4,      0,1,0,11+
	10,     0,5,        0,7,            0,11+
	11,       0,3,    0,20+
	12,         0,0,
];
const SPACESHIP_S: [u8; 12] = [
	// 5x4
	3,    0,1+
	4,      0,
	0,3,    0,
	1,0,0,0,0,
];
const SPACESHIP_M: [u8; 13] = [
	// 6x4
	4,      0,1+
	5,        0,
	0,4,      0,
	1,0,0,0,0,0,
];
const SPACESHIP_L: [u8; 14]  = [
	// 7x4
	5,        0,1+
	6,          0,
	0,5,        0,
	1,0,0,0,0,0,0,
];
const ACORN: [u8; 11] = [
	// 7x3
	1,0,5+
	3,    0,3,
	0,0,2,  0,0,0,
];

const DIGIT_BITMAP: [[u8; 4]; 49] = [
	[
		0b0111110,
		0b1000001,
		0b1000001,
		0b0111110,
	],
	[
		0,
		0b1000010,
		0b1111111,
		0b1000000,
	],
	[
		0b1100010,
		0b1010001,
		0b1001001,
		0b1000110,
	],
	[
		0b0100010,
		0b1000001,
		0b1001001,
		0b0110110,
	],
	[
		0b0011000,
		0b0010100,
		0b0010010,
		0b1111111,
	],
	[
		0b0101111,
		0b1001001,
		0b1001001,
		0b0110001,
	],
	[
		0b0111110,
		0b1001001,
		0b1001001,
		0b0110000,
	],
	[
		0b0000001,
		0b1100001,
		0b0011001,
		0b0000111,
	],
	[
		0b0110110,
		0b1001001,
		0b1001001,
		0b0110110,
	],
	[
		0b0000110,
		0b1001001,
		0b1001001,
		0b0111110,
	], // 10
	[
		0b1111110,
		0b0001001,
		0b0001001,
		0b1111110,
	],
	[
		0b1111111,
		0b1001001,
		0b1001001,
		0b0110110,
	],
	[
		0b0111110,
		0b1000001,
		0b1000001,
		0b0100010,
	],
	[
		0b1111111,
		0b1000001,
		0b1000001,
		0b0111110,
	],
	[
		0b1111111,
		0b1001001,
		0b1001001,
		0b1000001,
	],
	[
		0b1111111,
		0b0001001,
		0b0001001,
		0b0000001,
	], // 16
	[
		0b0111110,
		0b1000001,
		0b1001001,
		0b0111000,
	],
	[
		0b1111111,
		0b0001000,
		0b0001000,
		0b1111111,
	],
	[
		0,
		0b1000001,
		0b1111111,
		0b1000001,
	],
	[
		0b0100001,
		0b1000001,
		0b0111111,
		0b0000001,
	],
	[
		0b1111111,
		0b0001000,
		0b0010100,
		0b1100011,
	],
	[
		0b1111111,
		0b1000000,
		0b1000000,
		0b1000000,
	],
	[
		0,
		0b1011111,
		0,
		0b1011111,
	], // 23
	
	[
		0b0110110,
		0b1000001,
		0b1000001,
		0b0110110,
	],
	[
		0,
		0,
		0,
		0b0110110,
	],
	[
		0b0110000,
		0b1001001,
		0b1001001,
		0b0000110,
	],
	[
		0,
		0b1001001,
		0b1001001,
		0b0110110,
	],
	[
		0b0000110,
		0b0001000,
		0b0001000,
		0b0110110,
	],
	[
		0b0000110,
		0b1001001,
		0b1001001,
		0b0110000,
	],
	[
		0b0110110,
		0b1001001,
		0b1001001,
		0b0110000,
	],
	[
		0,
		0b0000001,
		0b0000001,
		0b0110110,
	],
	[
		0b0110110,
		0b1001001,
		0b1001001,
		0b0110110,
	],
	[
		0b0000110,
		0b1001001,
		0b1001001,
		0b0110110,
	], // 33
	[
		0b1110110,
		0b0001001,
		0b0001001,
		0b1110110,
	],
	[
		0b0110110,
		0b1001000,
		0b1001000,
		0b0110000,
	],
	[
		0b0110110,
		0b1000001,
		0b1000001,
		0,
	],
	[
		0b0110000,
		0b1001000,
		0b1001000,
		0b0110110,
	],
	[
		0b0110110,
		0b1001001,
		0b1001001,
		0,
	],
	[
		0b0110110,
		0b0001001,
		0b0001001,
		0,
	], // 39
	
	[
		0b0111110,
		0b1000001,
		0b1000001,
		0b0111110,
	],
	[
		0,
		0,
		0,
		0b1111111,
	],
	[
		0b1110001,
		0b1001001,
		0b1001001,
		0b1000110,
	],
	[
		0b1000001,
		0b1001001,
		0b1001001,
		0b0110110,
	],
	[
		0b0001111,
		0b0001000,
		0b0001000,
		0b1111111,
	],
	[
		0b1000111,
		0b1001001,
		0b1001001,
		0b0110001,
	],
	[
		0b0111110,
		0b1001001,
		0b1001001,
		0b0110000,
	],
	[
		0b0000001,
		0b0000001,
		0b0000001,
		0b1111111,
	],
	[
		0b0110110,
		0b1001001,
		0b1001001,
		0b0110110,
	],
	[
		0b0000110,
		0b1001001,
		0b1001001,
		0b0111110,
	], // 49
];

const TEXT_BITMAP_GENERATION: [u8; 50] = [
	0b0111110, // G
	0b1000001,
	0b1000001,
	0b1001001,
	0b0111010,
	0,
	0b0111000, // e
	0b1010100,
	0b1010100,
	0b1010100,
	0b0011000,
	0,
	0b1111100, // n
	0b0000100,
	0b0000100,
	0b1111000,
	0,
	0b0111000, // e
	0b1010100,
	0b1010100,
	0b1010100,
	0b0011000,
	0,
	0b1111100, // r
	0b0001000,
	0b0000100,
	0b0000100,
	0b0001000,
	0,
	0b0111000, // a
	0b1000100,
	0b1000100,
	0b0100100,
	0b1111100,
	0,
	0b0000010, // t
	0b1111111,
	0b0000010,
	0,
	0b1111010, // i
	0,
	0b0111000, // o
	0b1000100,
	0b1000100,
	0b0111000,
	0,
	0b1111100, // n
	0b0000100,
	0b0000100,
	0b1111000,
];

const TEXT_BITMAP_SPEED: [u8; 39] = [
	0b0111110, // G
	0b1000001,
	0b1000001,
	0b1001001,
	0b0111010,
	0,
	0b0111000, // e
	0b1010100,
	0b1010100,
	0b1010100,
	0b0011000,
	0,
	0b1111100, // n
	0b0000100,
	0b0000100,
	0b1111000,
	0,
	0b1000000,
	0,
	0b1100000, // /
	0b0011100,
	0b0000111,
	0b1001000, // s
	0b1010100,
	0b1010100,
	0b0100100,
	0,
	0b0111000, // e
	0b1010100,
	0b1010100,
	0b1010100,
	0b0011000,
	0,
	0b0111000, // c
	0b1000100,
	0b1000100,
	0b0101000,
	0,
	0b1000000,
];

const TEXT_BITMAP_POPULATION: [u8; 42] = [
	0b1111111, // P
	0b0001001,
	0b0001001,
	0b0000110,
	0b0111000, // o
	0b1000100,
	0b1000100,
	0b0111000,
	0,
	0b11111100, // p
	0b01000100,
	0b01000100,
	0b00111000,
	0,
	0b0111100, // u
	0b1000000,
	0b1000000,
	0b1111100,
	0,
	0b1111111, // l
	0,
	0b0111000, // a
	0b1000100,
	0b1000100,
	0b0100100,
	0b1111100,
	0,
	0b0000010, // t
	0b1111111,
	0b0000010,
	0,
	0b1111010, // i
	0,
	0b0111000, // o
	0b1000100,
	0b1000100,
	0b0111000,
	0,
	0b1111100, // n
	0b0000100,
	0b0000100,
	0b1111000,
];

const TEXT_BITMAP_MACRO: [u8; 27] = [
	0b1111111, // M
	0b0000010,
	0b0001100,
	0b0000010,
	0b1111111,
	0,
	0b0111000, // a
	0b1000100,
	0b1000100,
	0b0100100,
	0b1111100,
	0,
	0b0111000, // c
	0b1000100,
	0b1000100,
	0b0101000,
	0,
	0b1111100, // r
	0b0001000,
	0b0000100,
	0b0000100,
	0b0001000,
	0,
	0b0111000, // o
	0b1000100,
	0b1000100,
	0b0111000,
];

const BITMAP_SLASH: [u8; 3] = [
	0b1100000,
	0b0011100,
	0b0000011,
];
const BITMAP_TIMES: [u8; 5] = [
	0b0100010,
	0b0010100,
	0b0001000,
	0b0010100,
	0b0100010,
];

const BITMAP_ICON_FLIP_X: [u8; 7] = [
	0b0001000,
	0b0010100,
	0b0000000,
	0b1111111,
	0b0000000,
	0b0010100,
	0b0001000,
];
const BITMAP_ICON_FLIP_Y: [u8; 7] = [
	0b0001000,
	0b0001000,
	0b0101010,
	0b1001001,
	0b0101010,
	0b0001000,
	0b0001000,
];
const BITMAP_ICON_FLIP_XY: [u8; 5] = [
	0b0110010,
	0b0100100,
	0b0001000,
	0b0010010,
	0b0100110,
];

const DEFAULT_SPEED: i32 = 16;
const MAX_SPEED: i32 = 8000;
const INFO_HEIGHT: i32 = 22;

#[derive(Debug)]
struct Game {
	width: i32,
	height: i32,
	window_width: i32,
	window_height: i32,
	
	pipeline: Pipeline,
	bindings: Bindings,
	buffer: Vec<RGB8>,
	digit_bitmap_range: Range<usize>,
	
	cells: Vec<CellState>,
	neighbors: Vec<[usize; 8]>,
	alive_cells: Vec<usize>,
	//population: usize,
	generation: u64,
	cursor_pos: (i32, i32),
	selection: Option<(i32, i32)>,
	cursor_hidden: bool,
	
	paused: bool,
	speed: i32,
	instant: Instant,
	gen_queue: f32,
	
	macro_flip_x: bool,
	macro_flip_y: bool,
	macro_swap_xy: bool,
}

impl Game {
	const CURSOR_PIXEL_OFFSETS: [i32; 6] = [-4, -3, -2, 2, 3, 4];
	const CLEAR_COLOR: RGB8 = RGB8::BLACK;
	const BORDER_COLOR: RGB8 = RGB8::AQUA;
	
	fn init(ctx: &mut GraphicsContext, width: i32, height: i32, seed: u64, digit_bitmap_range: Range<usize>) -> Self {
		const VERTICES: [Vertex; 4] = [
			Vertex { pos: Vec2 { x: -1., y: -1. }, uv: Vec2 { x: 0., y: 1. } },
			Vertex { pos: Vec2 { x:  1., y: -1. }, uv: Vec2 { x: 1., y: 1. } },
			Vertex { pos: Vec2 { x:  1., y:  1. }, uv: Vec2 { x: 1., y: 0. } },
			Vertex { pos: Vec2 { x: -1., y:  1. }, uv: Vec2 { x: 0., y: 0. } },
		];
		let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &VERTICES);
		
		const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
		let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &INDICES);
		
		let texture = miniquad::Texture::new_render_texture(ctx, miniquad::TextureParams {
			format: miniquad::TextureFormat::RGB8,
			wrap: miniquad::TextureWrap::Clamp,
			filter: miniquad::FilterMode::Nearest,
			width: width as u32,
			height: height as u32,
		});
		
		let bindings = Bindings {
			vertex_buffers: vec![vertex_buffer],
			index_buffer,
			images: vec![texture],
		};
		
		let shader_meta = miniquad::ShaderMeta {
			images: vec!["tex".to_string()],
			uniforms: miniquad::UniformBlockLayout {
				uniforms: vec![],
			},
		};
		
		let shader = miniquad::Shader::new(ctx, SHADER_VERT, SHADER_FRAG, shader_meta).unwrap();
		let pipeline = Pipeline::new(
			ctx,
			&[miniquad::BufferLayout::default()],
			&[
				VertexAttribute::new("pos", VertexFormat::Float2),
				VertexAttribute::new("uv", VertexFormat::Float2),
			],
			shader,
		);
		
		let game_height = height - INFO_HEIGHT;
		let cell_count = width as usize * game_height as usize;
		let mut game = Self {
			width,
			height: game_height,
			window_width: width,
			window_height: height,
			
			pipeline,
			bindings,
			buffer: vec![Self::CLEAR_COLOR; width as usize * height as usize],
			digit_bitmap_range,
			
			cells: vec![CellState::Dead; cell_count],
			neighbors:  Vec::with_capacity(cell_count),
			alive_cells: Vec::with_capacity(cell_count),
			//population: 0,
			generation: 0,
			cursor_pos: (0, 0),
			selection: None,
			cursor_hidden: false,
			
			paused: true,
			speed: DEFAULT_SPEED,
			instant: Instant::now(),
			gen_queue: 0.0,
			
			macro_flip_x: false,
			macro_flip_y: false,
			macro_swap_xy: false,
		};
		game.generate_neighbors();
		const GEN_BOX_END: i32 = TEXT_BITMAP_GENERATION.len() as i32 + 45;
		const POP_X_POS: i32 = GEN_BOX_END + 2 + TEXT_BITMAP_POPULATION.len() as i32;
		let cells_digits = cell_count.ilog10() as i32 + 1;
		let pop_box_end = POP_X_POS + 47 + cells_digits * 5;
		// horizontal lines
		for x in 0..width {
			game.set_pixel_nowrap(x, game_height, Self::BORDER_COLOR);
			game.set_pixel_nowrap(x, game_height + 11, Self::BORDER_COLOR);
		}
		// vertical lines
		for y in game_height..height {
			game.set_pixel_nowrap(0, y, Self::BORDER_COLOR);
			game.set_pixel_nowrap(GEN_BOX_END, y, Self::BORDER_COLOR);
		}
		for y in game_height..game_height+11 {
			game.set_pixel_nowrap(pop_box_end, y, Self::BORDER_COLOR);
		}
		// labels
		game.draw_bitmap(&TEXT_BITMAP_GENERATION, 3, game_height + 2, RGB8::GRAY);
		game.draw_bitmap(&TEXT_BITMAP_SPEED, 3, game_height + 13, RGB8::GRAY);
		
		game.draw_bitmap(&TEXT_BITMAP_POPULATION, GEN_BOX_END + 3, game_height + 2, RGB8::GRAY);
		game.set_pixel_nowrap(GEN_BOX_END + 12, game_height + 10, RGB8::GRAY);
		game.draw_bitmap(&BITMAP_SLASH, POP_X_POS + 42, game_height + 2, RGB8::GRAY);
		game.draw_int(cell_count as u64, cells_digits, POP_X_POS + 46, game_height + 2, RGB8::WHITE, false);
		
		game.draw_bitmap(&TEXT_BITMAP_MACRO, GEN_BOX_END + 3, game_height + 13, RGB8::GRAY);
		
		let width_digits = width.ilog10() as i32 + 1;
		game.draw_int(width as u64, width_digits, pop_box_end + 3, game_height + 2, RGB8::WHITE, false);
		game.draw_bitmap(&BITMAP_TIMES, pop_box_end + 3 + width_digits * 5, game_height + 2, RGB8::GRAY);
		game.draw_int(game_height as u64, game_height.ilog10() as i32 + 1, pop_box_end + 3 + width_digits * 5 + 6, game_height + 2, RGB8::WHITE, false);
		
		game.draw_info();
		game.set_speed(DEFAULT_SPEED);
		game.update_macro();
		game.draw_cursor();
		
		if seed != 0 {game.random(seed);}
		
		game
	}
	/// Used to pre-calculate neighbor cells, rather than calculating them every generation.
	fn generate_neighbors(&mut self) {
		let width = self.width as usize;
		let height = self.height as usize;
		for y in 0..height {
			let a = (
				(if y > 0 {y} else {height} - 1) * width,
				y * width,
				if y+1 < height {y+1} else {0} * width,
			);
			for x in 0..width {
				let b = (
					if x > 0 {x} else {width} - 1,
					if x+1 < width {x+1} else {0},
				);
				self.neighbors.push([
					a.0 + b.0,
					a.0 + x,
					a.0 + b.1,
					a.1 + b.0,
					a.1 + b.1,
					a.2 + b.0,
					a.2 + x,
					a.2 + b.1,
				]);
			}
		}
	}
	#[inline]
	fn cell_color(state: CellState) -> RGB8 {
		match state {
			CellState::Dead => Self::CLEAR_COLOR,
			CellState::Alive => RGB8::WHITE,
			CellState::Block => RGB8::GRAY,
		}
	}
	#[inline]
	fn coord_index(&self, x: i32, y: i32) -> usize {
		((y+self.height) % self.height) as usize * self.width as usize + ((x+self.width) % self.width) as usize
	}
	#[inline]
	fn get_selection(&self) -> (i32, i32) {
		self.selection.unwrap_or((self.width, self.height))
	}
	#[inline]
	fn get_selection_end(&self) -> (i32, i32) {
		let selection = self.get_selection();
		(self.cursor_pos.0 + selection.0 - 1, self.cursor_pos.1 + selection.1 - 1)
	}
	#[inline]
	fn get_cell(&self, x: i32, y: i32) -> CellState {
		let i = self.coord_index(x, y);
		self.cells[i]
	}
	fn set_cell(&mut self, x: i32, y: i32, state: CellState) {
		let i = self.coord_index(x, y);
		if self.cells[i] == state {return}
		if self.cells[i] == CellState::Alive {self.alive_cells.swap_remove(self.alive_cells.iter().position(|x| *x == i).unwrap());}
		else if state == CellState::Alive {self.alive_cells.push(i);}
		self.cells[i] = state;
		self.buffer[i] = Self::cell_color(state);
	}
	fn fill_selection(&mut self, state: CellState) {
		let end = self.get_selection_end();
		for y in self.cursor_pos.1..=end.1 {
			for x in self.cursor_pos.0..=end.0 {
				let i = self.coord_index(x, y);
				self.cells[i] = state;
				self.buffer[i] = Self::cell_color(state);
			}
		}
		self.alive_cells.clear();
		for (i, cell) in self.cells.iter().enumerate() {
			if *cell == CellState::Alive {
				self.alive_cells.push(i)
			}
		}
	}
	// fill the grid with random cells
	fn random(&mut self, seed: u64) {
		//self.population = 0;
		self.alive_cells.clear();
		self.generation = 0;
		let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
		for i in 0..self.cells.len() {
			if rng.next_u64() & 1 == 0 {
				self.cells[i] = CellState::Dead;
				self.buffer[i] = Self::CLEAR_COLOR;
			} else {
				self.cells[i] = CellState::Alive;
				self.buffer[i] = RGB8::WHITE;
				self.alive_cells.push(i);
			}
		}
		self.draw_info();
		self.draw_cursor();
	}
	
	#[inline]
	fn set_pixel(&mut self, x: i32, y: i32, color: RGB8) {
		let i = self.coord_index(x, y);
		self.buffer[i] = color;
	}
	#[inline]
	fn set_pixel_nowrap(&mut self, x: i32, y: i32, color: RGB8) {
		self.buffer[y as usize * self.width as usize + x as usize] = color;
	}
	#[inline]
	fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: RGB8) {
		for y2 in y..y+h {
			for x2 in x..x+w {
				self.set_pixel_nowrap(x2, y2, color);
			}
		}
	}
	// set cell pixel according to cell state
	#[inline]
	fn draw_cell(&mut self, x: i32, y: i32) {
		let i = self.coord_index(x, y);
		self.buffer[i] = Self::cell_color(self.cells[i]);
	}
	fn draw_cursor(&mut self) {
		if self.cursor_hidden {return}
		const CURSOR_COLOR: RGB8 = RGB8::RED;
		const SELECT_COLOR: RGB8 = RGB8::AQUA;
		if self.selection.is_some() {
			let selection_end = self.get_selection_end();
			self.set_pixel(self.cursor_pos.0, selection_end.1, SELECT_COLOR);
			self.set_pixel(selection_end.0, self.cursor_pos.1, SELECT_COLOR);
			self.set_pixel(selection_end.0, selection_end.1, SELECT_COLOR);
			for offset in 2..=4 {
				self.set_pixel(selection_end.0 - offset, selection_end.1, SELECT_COLOR);
				self.set_pixel(selection_end.0, selection_end.1 - offset, SELECT_COLOR);
			}
			for x in (self.cursor_pos.0+3..selection_end.0-6).step_by(4) {
				self.set_pixel(x + 4, self.cursor_pos.1, SELECT_COLOR);
				self.set_pixel(x + 5, self.cursor_pos.1, SELECT_COLOR);
				self.set_pixel(x + 1, selection_end.1, SELECT_COLOR);
				self.set_pixel(x, selection_end.1, SELECT_COLOR);
			}
			for y in (self.cursor_pos.1+3..selection_end.1-6).step_by(4) {
				self.set_pixel(self.cursor_pos.0, y + 4, SELECT_COLOR);
				self.set_pixel(self.cursor_pos.0, y + 5, SELECT_COLOR);
				self.set_pixel(selection_end.0, y, SELECT_COLOR);
				self.set_pixel(selection_end.0, y + 1, SELECT_COLOR);
			}
		}
		for offset in Self::CURSOR_PIXEL_OFFSETS {
			self.set_pixel(self.cursor_pos.0 + offset, self.cursor_pos.1, CURSOR_COLOR);
			self.set_pixel(self.cursor_pos.0, self.cursor_pos.1 + offset, CURSOR_COLOR);
		}
	}
	#[inline]
	fn erase_cursor(&mut self) {
		self.erase_selection();
		for offset in Self::CURSOR_PIXEL_OFFSETS {
			self.draw_cell(self.cursor_pos.0 + offset, self.cursor_pos.1);
			self.draw_cell(self.cursor_pos.0, self.cursor_pos.1 + offset);
		}
	}
	#[inline]
	fn erase_selection(&mut self) {
		if self.selection.is_none() {return}
		let selection_end = self.get_selection_end();
		self.draw_cell(selection_end.0, selection_end.1);
		for offset in 2..=4 {
			self.draw_cell(selection_end.0 - offset, selection_end.1);
			self.draw_cell(selection_end.0, selection_end.1 - offset);
		}
		for x in self.cursor_pos.0..selection_end.0 {
			self.draw_cell(x + 4, self.cursor_pos.1);
			self.draw_cell(x, selection_end.1);
		}
		for y in self.cursor_pos.1..selection_end.1 {
			self.draw_cell(self.cursor_pos.0, y + 4);
			self.draw_cell(selection_end.0, y);
		}
	}
	fn move_cursor(&mut self, x: i32, y: i32, keep_selection: bool) {
		self.erase_cursor();
		if !keep_selection {self.selection = None;}
		self.cursor_pos = ((x+self.width) % self.width, (y+self.height) % self.height);
		self.draw_cursor();
	}
	fn set_selection(&mut self, w: i32, h: i32) {
		self.erase_selection();
		self.selection = Some((
			(w + self.width) % self.width,
			(h + self.height) % self.height,
		));
		self.draw_cursor();
	}
	
	fn draw_bitmap(&mut self, lines: &[u8], x: i32, y: i32, color: RGB8) {
		for i in 0..lines.len() {
			if lines[i] == 0 {continue}
			for b in 0..8 {
				if lines[i] & 1 << b != 0 {
					self.set_pixel_nowrap(x + i as i32, y + b, color);
				}
			}
		}
	}
	fn draw_int(&mut self, mut value: u64, digits: i32, x: i32, y: i32, color: RGB8, clear: bool) {
		let digit_bitmap = &DIGIT_BITMAP[self.digit_bitmap_range.clone()];
		let base = self.digit_bitmap_range.len() as u64;
		if clear {self.fill_rect(x, y, digits * 5, 7, Self::CLEAR_COLOR);}
		for i in (1..digits).rev() {
			self.draw_bitmap(&digit_bitmap[(value % base) as usize], x + i * 5, y, color);
			value /= base;
			if value == 0 {return}
		}
		if value < base {
			self.draw_bitmap(&digit_bitmap[value as usize], x, y, color);
			return
		}
		if value > 22 {value = 22;}
		self.draw_bitmap(&DIGIT_BITMAP[value as usize], x, y, color);
	}
	fn draw_info(&mut self) {
		const GEN_X_POS: i32 = 2 + TEXT_BITMAP_GENERATION.len() as i32 + 2;
		self.draw_int(self.generation, 8, GEN_X_POS, self.height + 2, RGB8::WHITE, true);
		const POP_X_POS: i32 = GEN_X_POS + 43 + TEXT_BITMAP_POPULATION.len() as i32 + 2;
		self.draw_int(self.alive_cells.len() as u64, 8, POP_X_POS, self.height + 2, RGB8::WHITE, true);
	}
	
	fn set_speed(&mut self, speed: i32) {
		self.speed = if speed < 1 {1}
		else if speed > MAX_SPEED {MAX_SPEED}
		else {speed};
		self.draw_int(self.speed as u64, 4, TEXT_BITMAP_GENERATION.len() as i32 + 24, self.height + 13, RGB8::WHITE, true);
	}
	fn update_macro(&mut self) {
		const X_POS: i32 = TEXT_BITMAP_GENERATION.len() as i32 + 48 + TEXT_BITMAP_MACRO.len() as i32 + 8;
		self.fill_rect(X_POS, self.height + 13, 40, 7, Self::CLEAR_COLOR);
		
		let mut i: i32 = 0;
		for cell in GLIDER.iter() {
			if *cell > 0 {
				i += *cell as i32;
				continue
			}
			let mut x = i % 3 - 1;
			let mut y = i / 3 - 1;
			if self.macro_swap_xy {(x, y) = (y, x)};
			self.set_pixel_nowrap(
				X_POS + 36 + if self.macro_flip_x {-x} else {x},
				self.height + 16 + if self.macro_flip_y {-y} else {y},
				RGB8::WHITE
			);
			i += 1;
		}
		
		if self.macro_swap_xy {self.draw_bitmap(&BITMAP_ICON_FLIP_XY, X_POS + 8, self.height + 13, RGB8 {r:0, g:128, b:255});}
		if self.macro_flip_x {self.draw_bitmap(&BITMAP_ICON_FLIP_X, X_POS + 16, self.height + 13, RGB8::RED);}
		if self.macro_flip_y {self.draw_bitmap(&BITMAP_ICON_FLIP_Y, X_POS + 24, self.height + 13, RGB8::GREEN);}
	}
	// set cells from macro
	fn place_macro(&mut self, cells: &[u8], w: i32, h: i32) {
		let mut i: i32 = 0;
		for cell in cells.iter() {
			if *cell > 0 {
				i += *cell as i32;
				continue
			}
			let mut x = i % w - w / 2;
			let mut y = i / w - h / 2;
			if self.macro_swap_xy {(x, y) = (y, x)};
			self.set_cell(
				self.cursor_pos.0 + if self.macro_flip_x {-x} else {x},
				self.cursor_pos.1 + if self.macro_flip_y {-y} else {y},
				CellState::Alive
			);
			i += 1;
		}
		self.draw_info();
		self.draw_cursor();
	}
	
	#[inline]
	fn mouse_to_pixel(&self, x: f32, y: f32) -> (i32, i32) {
		(x as i32 * self.width / self.window_width, y as i32 * (self.height + INFO_HEIGHT) / self.window_height)
	}
	fn apply_resize(&mut self, new_width: i32, new_height: i32) {
		//for pix in self.buffer.iter_mut() {
		//	*pix = self.clear_color;
		//}
		
		//self.buffer.resize((new_width * new_height) as usize, Self::CLEAR_COLOR);
		self.window_width = new_width;
		self.window_height = new_height;
	}
}

impl miniquad::EventHandler for Game {
	fn update(&mut self, _ctx: &mut GraphicsContext) {
		const MAX_GENERATIONS: f32 = MAX_SPEED as f32 / 60.0;
		const ACTUAL_SPEED_X: i32 = TEXT_BITMAP_SPEED.len() as i32 + 8;
		if self.paused {return}
		
		self.gen_queue += self.instant.elapsed().as_secs_f32() * self.speed as f32;
		self.instant = Instant::now();
		
		if self.gen_queue < 1.0 {return}
		if self.gen_queue > MAX_GENERATIONS {self.gen_queue = MAX_GENERATIONS;}
		
		let mut neighbor_counts = vec![0u8; self.cells.len()];
		for i in self.alive_cells.iter() {
			for neighbor in self.neighbors[*i].iter() {
				neighbor_counts[*neighbor] += 1;
			}
		}
		let mut new_alive_cells = Vec::new();
		for _ in 0..self.gen_queue.floor() as i32 {
			const BORN_NEIGHBORS: u8 = 3;
			const ALIVE_MIN_NEIGHBORS: u8 = 2;
			const ALIVE_MAX_NEIGHBORS: u8 = 3;
			/* Simple approach, doesn't use self.alive_cells
			let mut neighbor_counts = vec![0u8; self.cells.len()];
			// get neighbor counts
			for (i, cell) in self.cells.iter().enumerate() {
				if *cell != CellState::Alive {continue}
				for neighbor in self.neighbors[i].iter() {
					neighbor_counts[*neighbor] += 1;
				}
			}
			// process generation
			for (i, n) in neighbor_counts.iter().enumerate() {
				match self.cells[i] {
					CellState::Dead => if *n == 3 {
						self.cells[i] = CellState::Alive;
						self.buffer[i] = RGB8::WHITE;
						self.population += 1;
					},
					CellState::Alive => if *n < 2 || *n > 3 {
						self.cells[i] = CellState::Dead;
						self.buffer[i] = Self::CLEAR_COLOR;
						self.population -= 1;
					},
					_ => ()
				}
			}*/
			
			let prev_neighbor_counts = neighbor_counts.clone();
			
			let mut i = 0;
			while i < self.alive_cells.len() {
				let cell_i = self.alive_cells[i];
				// check if cell dies
				let cell_died = prev_neighbor_counts[cell_i] < ALIVE_MIN_NEIGHBORS || prev_neighbor_counts[cell_i] > ALIVE_MAX_NEIGHBORS;
				if cell_died {
					self.cells[cell_i] = CellState::Dead;
					self.buffer[cell_i] = Self::CLEAR_COLOR;
					self.alive_cells.swap_remove(i);
				} else {i += 1;}
				// check neighbors
				for neighbor in self.neighbors[cell_i].iter() {
					if cell_died {neighbor_counts[*neighbor] -= 1;}
					// cell born
					if prev_neighbor_counts[*neighbor] != BORN_NEIGHBORS || self.cells[*neighbor] != CellState::Dead {continue}
					self.cells[*neighbor] = CellState::Alive;
					self.buffer[*neighbor] = RGB8::WHITE;
					new_alive_cells.push(*neighbor);
					for neighbor2 in self.neighbors[*neighbor].iter() {
						neighbor_counts[*neighbor2] += 1;
					}
				}
			}
			self.alive_cells.append(&mut new_alive_cells);
			
			self.generation += 1;
		}
		
		self.draw_info();
		self.draw_cursor();
		
		let effective_speed = (self.gen_queue / self.instant.elapsed().as_secs_f32()) as i32;
		self.fill_rect(ACTUAL_SPEED_X, self.height + 13, 19, 7, Self::CLEAR_COLOR);
		if effective_speed < self.speed {
			self.draw_int(effective_speed as u64, 4, ACTUAL_SPEED_X, self.height + 13, RGB8::RED, false);
		}
		self.gen_queue = self.gen_queue.fract();
	}
	
	fn draw(&mut self, ctx: &mut GraphicsContext) {
		(&self.bindings.images[0]).update(ctx, self.buffer.as_bytes());
		
		ctx.begin_default_pass(miniquad::PassAction::Nothing);
		
		ctx.apply_pipeline(&self.pipeline);
		ctx.apply_bindings(&self.bindings);
		
		ctx.draw(0, 6, 1);
		
		ctx.end_render_pass();
		ctx.commit_frame();
	}
	
	fn key_down_event(
		&mut self,
		ctx: &mut GraphicsContext,
		key_code: KeyCode,
		key_mods: KeyMods,
		_repeat: bool,
	) {
		let multiplier = if key_mods.ctrl {16} else {1} * if key_mods.shift {4} else {1};
		match key_code {
			// cursor movement
			KeyCode::Right => self.move_cursor(self.cursor_pos.0 + multiplier, self.cursor_pos.1, true),
			KeyCode::Left => self.move_cursor(self.cursor_pos.0 - multiplier, self.cursor_pos.1, true),
			KeyCode::Down => self.move_cursor(self.cursor_pos.0, self.cursor_pos.1 + multiplier, true),
			KeyCode::Up => self.move_cursor(self.cursor_pos.0, self.cursor_pos.1 - multiplier, true),
			// change speed
			KeyCode::LeftBracket => self.set_speed(self.speed - multiplier),
			KeyCode::RightBracket => self.set_speed(self.speed + multiplier),
			// pause/unpause
			KeyCode::Space => {
				self.paused = !self.paused;
				self.instant = Instant::now();
			},
			// cell toggles
			KeyCode::Enter => {
				self.set_cell(
					self.cursor_pos.0, self.cursor_pos.1,
					if self.get_cell(self.cursor_pos.0, self.cursor_pos.1) == CellState::Alive {CellState::Dead}
					else {CellState::Alive}
				);
			},
			KeyCode::B => {
				if self.selection.is_some() {
					self.fill_selection(CellState::Block);
					self.draw_cursor();
				} else {
					self.set_cell(
						self.cursor_pos.0, self.cursor_pos.1,
						if self.get_cell(self.cursor_pos.0, self.cursor_pos.1) == CellState::Block {CellState::Dead}
						else {CellState::Block}
					);
				}
			},
			KeyCode::Delete | KeyCode::Backspace => if self.selection.is_some() {
				self.fill_selection(CellState::Dead);
				self.draw_cursor();
			} else if key_mods.ctrl {
				self.alive_cells.clear();
				self.generation = 0;
				self.cells.fill(CellState::Dead);
				for i in 0..self.cells.len() {self.buffer[i] = Self::CLEAR_COLOR;}
				self.draw_cursor();
			},
			// special/shortcuts
			KeyCode::A => if key_mods.ctrl {
				self.erase_selection();
				self.selection = if key_mods.shift {None} else {Some((self.width, self.height))};
				self.draw_cursor();
			} else {self.place_macro(ACORN.as_slice(), 7, 3);},
			KeyCode::Q => if key_mods.ctrl {ctx.quit();},
			KeyCode::H => if key_mods.ctrl {
				self.cursor_hidden = !self.cursor_hidden;
				if self.cursor_hidden {self.erase_cursor();} else {self.draw_cursor()}
			},
			KeyCode::R => if key_mods.ctrl {self.random(rand::random::<u64>());},
			KeyCode::Minus => if key_mods.ctrl {
				let mut scale = (self.window_width / self.width) as u32 - 1;
				if scale < 1 {scale = 1;}
				ctx.set_window_size(scale * self.width as u32, scale * (self.height + INFO_HEIGHT) as u32);
			},
			KeyCode::Equal => if key_mods.ctrl {
				let scale = (self.window_width / self.width) as u32 + 1;
				ctx.set_window_size(scale * self.width as u32, scale * (self.height + INFO_HEIGHT) as u32);
			},
			// copy/paste
			KeyCode::C => if key_mods.ctrl {
				let selection = self.get_selection();
				let mut clip_str = format!("={}x{}=", selection.0, selection.1);
				let mut byte: usize = 0;
				let mut bit: usize = 32;
				for y in self.cursor_pos.1..self.cursor_pos.1+selection.1 {
					for x in self.cursor_pos.0..self.cursor_pos.0+selection.0 {
						if self.get_cell(x, y) == CellState::Alive {
							byte |= bit;
						}
						if bit == 1 {
							clip_str.push(BASE64.chars().nth(byte).unwrap());
							byte = 0;
							bit = 32;
						} else {bit >>= 1;}
					}
				}
				if byte != 0 {
					clip_str.push(BASE64.chars().nth(byte).unwrap());
				}
				if cfg!(target_os = "linux") {
					use clipboard::ClipboardProvider;
					match clipboard::ClipboardContext::new() {
						Ok(mut clipboard) => {let _ = clipboard.set_contents(clip_str);},
						Err(_) => println!("Couldn't get clipboard"),
					}
				} else {ctx.clipboard_set(&clip_str);}
			},
			KeyCode::V => if key_mods.ctrl {
				let clip_str: &str = &ctx.clipboard_get().unwrap();
				// parse clip
				let mut size = self.get_selection();
				let mut i: usize = 0;
				let clip_bytes = clip_str.as_bytes();
				if clip_bytes[0] == b'=' {
					i = 1;
					// width value
					let width_begin = i;
					while clip_bytes[i] >= b'0' && clip_bytes[i] <= b'9' {i += 1;}
					let width_end = i;
					// height value
					i += 1;
					while clip_bytes[i] >= b'0' && clip_bytes[i] <= b'9' {i += 1;}
					let height_end = i;
					if clip_bytes[i] == b'=' {i += 1;}
					// set ends
					size.0 = clip_str[width_begin..width_end].parse::<i32>().unwrap();
					size.1 = clip_str[width_end+1..height_end].parse::<i32>().unwrap();
					if size.0 <= 0 || size.1 <= 0 {return}
				}
				// process data
				fn paste_clip(game: &mut Game, clip: &[u8], pos: (i32, i32), w: i32, h: i32) {
					let mut i = 0;
					let mut byte = 0;
					let mut bit = 0;
					for y in pos.1..pos.1+h {
						for x in pos.0..pos.0+w {
							if bit == 0 {
								byte = clip[i];
								match byte {
									b'A'..=b'Z' => byte -= b'A',
									b'a'..=b'z' => {
										byte += 26;
										byte -= b'a';
									},
									b'0'..=b'9' => byte += 52 - b'0',
									b'+' => byte = 62,
									b'/' => byte = 63,
									_ => {
										println!("Invalid character in clip!");
										return
									},
								}
								bit = 32;
								i += 1;
								if i >= clip.len() {i = 0;}
							}
							if byte & bit != 0 {
								game.set_cell(x, y, CellState::Alive);
							}
							bit >>= 1;
						}
					}
				}
				let clip = clip_str[i..].as_bytes();
				if self.selection.is_some() {
					self.fill_selection(CellState::Dead);
					let end = self.get_selection_end();
					for y in (self.cursor_pos.1..=end.1-size.1+1).step_by(size.1 as usize) {
						for x in (self.cursor_pos.0..=end.0-size.0+1).step_by(size.0 as usize) {
							paste_clip(self, clip, (x, y), size.0, size.1);
						}
					}
				} else {
					self.selection = Some(size);
					self.fill_selection(CellState::Dead);
					self.erase_selection();
					if size == (self.width, self.height) {
						self.selection = None
					}
					paste_clip(self, clip, self.cursor_pos, size.0, size.1);
				}
				self.draw_cursor();
			},
			// macros
			KeyCode::Key1 => {
				self.macro_swap_xy = !self.macro_swap_xy;
				self.update_macro();
			},
			KeyCode::Key2 => {
				self.macro_flip_x = !self.macro_flip_x;
				self.update_macro();
			},
			KeyCode::Key3 => {
				self.macro_flip_y = !self.macro_flip_y;
				self.update_macro();
			},
			KeyCode::G => if key_mods.shift {
				self.place_macro(GLIDER_GUN.as_slice(), 36, 9);
			} else {
				self.place_macro(GLIDER.as_slice(), 3, 3);
			},
			KeyCode::S => if key_mods.shift {
				self.place_macro(SPACESHIP_L.as_slice(), 7, 4);
			} else if key_mods.ctrl {
				self.place_macro(SPACESHIP_M.as_slice(), 6, 4);
			} else {
				self.place_macro(SPACESHIP_S.as_slice(), 5, 4);
			},
			_ => ()
		}
	}
	
	/*fn key_up_event(
		&mut self,
		_ctx: &mut GraphicsContext,
		key_code: KeyCode,
		_key_mods: KeyMods,
	) {
		match key_code {
			
			_ => ()
		}
	}*/
	
	/*fn mouse_motion_event(&mut self, _ctx: &mut GraphicsContext, x: f32, y: f32) {
		let _mouse_pos = self.mouse_to_pixel(x, y);
	}*/
	
	fn mouse_button_down_event(
		&mut self,
		_ctx: &mut GraphicsContext,
		button: MouseButton,
		x: f32,
		y: f32,
	) {
		let mouse_pos = self.mouse_to_pixel(x, y);
		match button {
			MouseButton::Left => {
				self.move_cursor(mouse_pos.0, mouse_pos.1, false);
			},
			MouseButton::Right => {
				self.set_selection(mouse_pos.0 - self.cursor_pos.0 + 1, mouse_pos.1 - self.cursor_pos.1 + 1);
			},
			_ => ()
		}
	}
	
	fn mouse_button_up_event(
		&mut self,
		_ctx: &mut GraphicsContext,
		button: MouseButton,
		x: f32,
		y: f32,
	) {
		let mouse_pos = self.mouse_to_pixel(x, y);
		match button {
			MouseButton::Left => {
				if mouse_pos == self.cursor_pos {return}
				self.set_selection(mouse_pos.0 - self.cursor_pos.0 + 1, mouse_pos.1 - self.cursor_pos.1 + 1);
			},
			_ => ()
		}
	}
	
	fn resize_event(&mut self, _ctx: &mut GraphicsContext, width: f32, height: f32) {
		self.apply_resize(width.round() as i32, height.round() as i32);
	}
}

/// Program to simulate Conway's Game of Life
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "
Program to simulate Conway's Game of Life

Controls:
	Arrow keys - move cursor (Ctrl / Shift to move by more)
	Space - Pause/Unpause
	Enter - Toggle cell dead/alive
	B - Toggle cell block (block acts as dead cell, but cannot become alive)
	[ ] - Change speed (Ctrl / Shift to increment by more)
	1 2 3 - Macro transformations
	Del / Backspace - Clear selected region
	Ctrl+Del / Ctrl+Backspace - Reset game
	Ctrl+R - Random state")]
struct Args {
	/// Dimensions of the game in cells (WIDTHxHEIGHT)
	#[arg(default_value_t = ("256x256").to_string())]
	size: String,
	/// Style used for numbers on the display
	#[arg(short, long, value_parser = ["normal", "hex", "7-segment", "7-segment-hex", "straight"], default_value_t = ("normal").to_string())]
	digits: String,
	/// Use a seed to generate the initial state of the game
	#[arg(short, long, default_value_t = 0)]
	seed: u64,
	/// Use random seed (overrides -s)
	#[arg(short, long, default_value_t = false)]
	random: bool,
}

fn main() {
	let mut args = Args::parse();
	
	let digit_range = match &args.digits[..] {
		"normal" => 0..10,
		"hex" => 0..16,
		"7-segment" => 23..33,
		"7-segment-hex" => 23..39,
		"straight" => 39..49,
		&_ => todo!(),
	};
	
	if args.random {args.seed = rand::random::<u64>()}
	
	let (x, y) = args.size.split_once('x').unwrap();
	
	let window_width = x.parse::<i32>().unwrap();
	let window_height = y.parse::<i32>().unwrap() + INFO_HEIGHT;
	
	let conf = miniquad::conf::Conf {
		window_title: "Conway's Game of Life".to_string(),
		window_width,
		window_height,
		..Default::default()
	};
	miniquad::start(conf, move |ctx| Box::new(Game::init(ctx, window_width, window_height, args.seed, digit_range)));
}
