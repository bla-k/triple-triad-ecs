use std::collections::HashMap;

use sdl2::{
    EventPump,
    image::LoadTexture,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};

// =============================== SdlSystems ==================================

/// Initializes and holds SDL canvas, event pump and texture creator.
pub struct SdlSystems {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub texture_creator: TextureCreator<WindowContext>,
}

impl SdlSystems {
    const TITLE: &str = "Triple Triad";
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    pub fn init() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;

        let window = video
            .window(Self::TITLE, Self::WIDTH, Self::HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();

        let event_pump = sdl.event_pump()?;

        Ok(SdlSystems {
            canvas,
            event_pump,
            texture_creator,
        })
    }
}

// ============================== AssetManager =================================

#[derive(Default)]
pub struct AssetManager<'a> {
    font: Font,
    sprites: HashMap<String, Sprite>,
    textures: Vec<Texture<'a>>,
}

impl<'a> AssetManager<'a> {
    // Font texture layout, TODO move to file
    pub const GLYPH_HEIGHT: i32 = 22;
    pub const GLYPH_WIDTH: i32 = 18;

    const GLYPH_FG_ORIGIN: (i32, i32) = (0, 22);
    const GLYPH_BG_ORIGIN: (i32, i32) = (0, 66);

    const GLYPHS_PER_ROW: i32 = 6;

    const GLYPH_FG_A: (i32, i32) = (72, 44);
    const GLYPH_BG_A: (i32, i32) = (72, 88);

    pub fn define_sprite(&mut self, name: &str, texture_id: usize, region: Rect) {
        self.sprites
            .insert(name.to_string(), Sprite { region, texture_id });
    }

    pub fn get_font(&mut self) -> Option<(&Font, &mut Texture<'a>)> {
        let Some(texture) = self.textures.get_mut(self.font.texture_id) else {
            eprintln!("ERR: missing font texture");
            return None;
        };

        Some((&self.font, texture))
    }

    pub fn get_sprite(&self, name: &str) -> Option<Sprite> {
        self.sprites.get(name).copied()
    }

    pub fn get_texture_mut(&mut self, texture_id: usize) -> Option<&mut Texture<'a>> {
        self.textures.get_mut(texture_id)
    }

    #[rustfmt::skip]
    pub fn load_font(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        path: &str,
    ) -> Result<(), String> {
        let texture = texture_creator.load_texture(path)?;
        let mut glyphs = vec![None; 256];

        for j in 0i32..=9 {
            let glyph_idx = '0' as usize + j as usize;

            let x_incr = (Self::GLYPH_WIDTH * j) % (Self::GLYPH_WIDTH * Self::GLYPHS_PER_ROW);
            let y_incr = Self::GLYPH_FG_ORIGIN.1 * (j / Self::GLYPHS_PER_ROW);

            let fg_x = Self::GLYPH_FG_ORIGIN.0 + x_incr;
            let fg_y = Self::GLYPH_FG_ORIGIN.1 + y_incr;

            let bg_x = Self::GLYPH_BG_ORIGIN.0 + x_incr;
            let bg_y = Self::GLYPH_BG_ORIGIN.1 + y_incr;

            glyphs[glyph_idx] = Some((
                Rect::new(fg_x, fg_y, Self::GLYPH_WIDTH as u32, Self::GLYPH_HEIGHT as u32),
                Rect::new(bg_x, bg_y, Self::GLYPH_WIDTH as u32, Self::GLYPH_HEIGHT as u32),
            ));
        }

        // NOTE 'A' is a one off for now. The complete bitmap font will be implemented in future
        glyphs['A' as usize] = Some((
            Rect::new(Self::GLYPH_FG_A.0, Self::GLYPH_FG_A.1, Self::GLYPH_WIDTH as u32, Self::GLYPH_HEIGHT as u32),
            Rect::new(Self::GLYPH_BG_A.0, Self::GLYPH_BG_A.1, Self::GLYPH_WIDTH as u32, Self::GLYPH_HEIGHT as u32),
        ));

        let texture_id = self.textures.len();
        let font = Font { glyphs, texture_id };

        self.textures.push(texture);
        self.font = font;

        Ok(())
    }

    pub fn load_texture(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        path: &str,
    ) -> Result<usize, String> {
        self.textures.push(texture_creator.load_texture(path)?);

        Ok(self.textures.len() - 1)
    }
}

#[derive(Clone, Copy)]
pub struct Sprite {
    pub region: Rect,
    pub texture_id: usize,
}

#[derive(Default)]
pub struct Font {
    // (fg, bg) glyphs for regular and bold font. Index is ASCII char value.
    pub glyphs: Vec<Option<(Rect, Rect)>>,
    pub texture_id: usize,
}
