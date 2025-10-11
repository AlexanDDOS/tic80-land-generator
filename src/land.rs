use crate::tic80::*;
use crate::trace;
use itertools::Itertools;
use noise::{Simplex, NoiseFn};

// Common internal functions
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-x))
}

/// Struct to describe a land chunk with size of 8x8 pixels,
/// which is stored as a u64 value in 8 sequential MAP cells 
pub struct LandChunk {
    map_x: i32,
    map_y: i32,
}

impl LandChunk {
    pub fn new(map_x: i32, map_y: i32) -> Self {
        Self{map_x, map_y}
    }
    
    /// Get the fullinness of a chunk pixel at `(x, y)`
    pub fn get(&self, x: i32, y: i32) -> bool {
        let val = mget(self.map_x + y, self.map_y);
        ((val >> x) & 1) != 0
    }

    /// Fill or empty a chunk pixel at `(x, y)`
    pub fn set(&self, x: i32, y: i32, state: bool) {
        let mut val = mget(self.map_x + y, self.map_y);
        val = if state {
            val | (1 << x)
        } else {
            val & !(1 << x)
        };
        mset(self.map_x + y, self.map_y, val);
    }
    
    /// Get the full mask of the chunk
    pub fn get_mask(&self) -> u64 {
        let mut mask = 0u64;
        for i in 0..8 {
            let val = mget(self.map_x + i, self.map_y) as u64;
            mask = (mask << 8) | val;
        }
        return mask;
    }
    
    /// Set the full mask of the chunk
    pub fn set_mask(&self, mut mask: u64) {
        for i in 0..8 {
            let val = (mask & 0xff) as i32;
            mset(self.map_x + i, self.map_y, val);
            mask = mask >> 8;
        }
    }

    /// Return `true` if all chunk pixels are empty
    pub fn empty(&self) -> bool {
        self.get_mask() == 0u64
    }

    /// Return `true` if all chunk pixels are full
    pub fn full(&self) -> bool {
        self.get_mask() == !0u64
    }

    /// Draw the chunk with at `(x, y)` a given tile texture ID and scale factor
    pub fn draw(&self, where_x: i32, where_y:i32, tile: i32, scale: i32) {
        if !self.empty() {
            if self.full() {
                // Just use spr() for optimized rendering
                // spr(tile, where_x, where_y, SpriteOptions::default());
                spr(tile, where_x, where_y, SpriteOptions{scale, ..Default::default()});
            } else {
                // Draw every chunk pixel manually
                unsafe {
                    let tile_addr4 = (TILES as i32 + tile * 32) * 2;
                    for (x, y ) in (0..8).cartesian_product(0..8) {                      
                        if self.get(x, y) {
                            // Gather the tile's pixel colors and put it on the screen
                            let color = peek4(tile_addr4 + y * 8 + x);
                            // pix(where_x + x, where_y + y, color);
                            rect(where_x + x * scale, where_y + y * scale, scale, scale, color);
                        }
                    }
                }
            }
        }
    }
}

/// Land texture description
pub struct LandTexture {
    pub spr_id: i32, // ID of the first texture sprite/tile
    pub width: i32,  // Texture width
    pub height: i32, // Texture height
}

impl LandTexture {
    #[inline]
    pub fn tile(&self, x: i32, y: i32) -> i32 {
        self.spr_id + (y % self.height) * 16 + (x % self.width)
    }
}

/// Chunk address offest for reservation
const LAND_CHUNK_ADDR_RESERVE: i32 = 0x10;

// WARNING: Total land size (width * height * 8) may not exceed 32,640 bytes (the map memory size),
// as the map memory is used to load/share lands
pub struct Land {
    width: i32,    // Land total width in chuncks
    height: i32,   // Land total height in chuncks
    seed: u32,     // Seed used to generate the land
    covered: bool, // Covered land flag
    texture: LandTexture,
    water_height: i32,
}

impl Land {
    /// Empty land constructor
    pub fn new(width: i32, height: i32, texture: LandTexture) -> Self {
        assert!(width * height * 8 + LAND_CHUNK_ADDR_RESERVE <= 32640);
        let water_height = height * 8 - 8;
        let land = Self{width, height, texture, water_height, seed: 0, covered: false};
        land.save_in_map();
        return land;
    }

    /// Construct a land from data in the MAP memory
    pub fn from_map() -> Self {
        let width = mget(0, 0);
        let height = mget(1, 0);
        let mut seed = 0u32;
        for i in 0..4 {
            seed = (seed << 8) | (mget(2 + i, 0) as u32);
        }
        let texture = LandTexture {
            spr_id: mget(6, 0),
            width: mget(7, 0) >> 4, 
            height: mget(7, 0) & 0x0f
        };
        let water_height = height * 8 - 8;
        let covered = (mget(8, 0) & 0x01) != 0;
        Self{width, height, texture, water_height, seed, covered}
    }

    /// Construct a land from MAP data unless they are invalid.
    /// Otherwise make an empty land from the given arguments.
    pub fn from_map_or_new(width: i32, height: i32, texture: LandTexture) -> Self {
        // Data check (TODO: use CRC for better validation)
        let (map_width, map_height) = (mget(0, 0), mget(1, 0));
        if map_width == 0 || map_height == 0 {
            Self::new(width, height, texture)
        } else {
            Self::from_map()
        }
    }

    /// Save data in the MAP memory
    pub fn save_in_map(&self) {
        mset(0, 0, self.width & 0xff);
        mset(1, 0, self.height & 0xff);
        for i in 0..4 {
            let val = (self.seed >> (i * 8)) as i32;
            mset(5 - i, 0, val & 0xff);
        }
        let texture_size = (self.texture.width << 4) | self.texture.height;
        let flags= self.covered as i32;
        mset(6, 0, self.texture.spr_id);
        mset(7, 0, texture_size);
        mset(8, 0, flags);
    }

    /// Return size of the land in pixels
    pub fn size(&self) -> (i32, i32) {
        (self.width * 8, self.height * 8)
    }

    /// Get the current water height
    pub fn water_height(&self) -> i32 {
        self.water_height
    }
    
    /// Check if point (x, y) is in the land boundaries
    pub fn in_bounds(&self, x: i32, y:i32) -> bool {
        (x >= 0 && x < self.width * 8) && (y >= 0 && y < self.height * 8)
    }
    
    /// Get the chunk at land point (x, y)
    pub fn chunk(&self, x: i32, y:i32) -> Option<LandChunk> {
        let (chunk_x, chunk_y) = (x / 8, y / 8);
        if self.in_bounds(x, y) {
            let addr = LAND_CHUNK_ADDR_RESERVE + (chunk_y * self.width + chunk_x) * 8;
            Some(LandChunk::new(addr % 240, addr / 240))
        } else {
            None // Point is out of bounds
        }
    }

    /// Get the fullinness of a land pixel at `(x, y)`
    pub fn get(&self, x: i32, y:i32) -> bool {
        let chunk = self.chunk(x, y);
        match chunk {
            Some(ch) => ch.get(x % 8, y % 8),
            None => self.covered // Points out of bounds are considered empty unless the level is covered
        }
    }

    /// Fill or empty a land pixel at `(x, y)`
    pub fn set(&self, x: i32, y: i32, state: bool) {
        if let Some(chunk) = self.chunk(x, y) {
            chunk.set(x % 8, y % 8, state);
        }
    }

    /// Set the state of pixels inside a circle
    pub fn set_circle(&self, x: i32, y: i32, r: i32, state: bool) {
        let r2 = r * r;
        for (i, j) in (-r..r).cartesian_product(-r..r) {
            if i*i + j*j <= r2 {
                self.set(x + i, y + j, state);
            }
        }
    }

    /// Return an iterator over the two dimensions of land with the step of 8 (chunk size)
    fn chunk_coordinates(&self) -> impl Iterator<Item=(i32, i32)> {
        let x_range = (0..self.width * 8).step_by(8);
        let y_range = (0..self.height * 8).step_by(8);
        return x_range.cartesian_product(y_range);
    }

    /// Draw the land and water
    pub fn draw(&self, offset_x: i32, offset_y: i32, scale: i32) {
        // Land chunks
        for (x, y) in self.chunk_coordinates() {
            if let Some(chunk) = self.chunk(x, y) {
                let where_x = offset_x + x * scale;
                let where_y = offset_y + y * scale;
                let tile = self.texture.tile(x / 8, y / 8);
                chunk.draw(where_x, where_y, tile, scale);
            }
        }
        // Water
        let water_height = offset_y + self.water_height;
        if water_height < 137 {
            let water_depth = 137 - water_height;
            rect(0, water_height, 240, water_depth, 10);
        }
    }
    
    /// Clear the entire land
    pub fn clear(&self) {
        for (x, y) in self.chunk_coordinates() {
            if let Some(chunk) = self.chunk(x, y) {
                chunk.set_mask(0);
            }
        }
    }

    /// Function that suppresses altitude at the land board.
    /// NOTE: both `x` and `board_w` are normalized to the land width.
    fn board_constrain(x: f64, board_w: f64) -> f64 {
        if x < board_w {
            let dx = 6.0 * (x / board_w); // from 0.0 to 6.0
            return sigmoid(dx) * 2.0 - 1.0;
        } else if x > 1.0 - board_w {
           let dx = 6.0 * ((1.0 - x) / board_w); // from 6.0 to 0.0
            return sigmoid(dx) * 2.0 - 1.0;
        }
        return 1.0; // No suppression
    }

    /// Get generation seed
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Set generation seed
    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }

    /// Generate a random land using Simplex noise
    pub fn generate(&self) {
        self.clear();
        let (land_w, land_h) = self.size();
        let land_h_f64 = (self.water_height - 2) as f64;
        let simplex = Simplex::new(self.seed);
        for x in 0..land_w {
            let x_norm = (x as f64) / (land_w as f64);
            let (k1, k2) = (3.0 + simplex.get([2.0, -1.0]), 5.0 + simplex.get([-1.0, 2.0]));
            let point = [x_norm * k1, x_norm * k2];
            let h0 = 0.4 + 0.25 * (simplex.get(point) + 1.0);
            let board_w = 0.1 + 0.1 * simplex.get([1.0, -1.0]);
            let constrain = Land::board_constrain(x_norm, board_w);
            let h = land_h_f64 * (1.0 - (h0 * constrain));
            let y_start = std::cmp::min(h as i32, land_h - 1);
            for y in y_start..land_h {
                self.set(x, y, true);
            }
        }
    }
}
