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

// ============================== AssetLibrary =================================

#[derive(Default)]
pub struct AssetManager<'a> {
    sprites: HashMap<String, Sprite>,
    textures: Vec<Texture<'a>>,
}

impl<'a> AssetManager<'a> {
    pub fn define_sprite(&mut self, name: &str, texture_id: usize, region: Rect) {
        self.sprites
            .insert(name.to_string(), Sprite { region, texture_id });
    }

    pub fn get_sprite(&self, name: &str) -> Option<Sprite> {
        self.sprites.get(name).copied()
    }

    pub fn get_texture_mut(&mut self, texture_id: usize) -> Option<&mut Texture<'a>> {
        self.textures.get_mut(texture_id)
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

/// Loads and holds textures.
///
/// The asset library is tied to texture creator's lifetime by SDL requirement.
pub struct AssetLibrary<'a> {
    textures: HashMap<&'a str, Texture<'a>>,
}

impl<'a> AssetLibrary<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        let mut textures = HashMap::new();

        let asset_list = ["board-cell", "card", "font"];
        for asset in asset_list {
            let texture = texture_creator.load_texture(format!("assets/{}.png", asset))?;
            textures.insert(asset, texture);
        }

        Ok(AssetLibrary { textures })
    }

    /// Retrieve a loaded texture.
    ///
    /// Textures are loaded on `AssetLibrary` initialization.
    pub fn get_texture(&self, name: &str) -> &Texture<'a> {
        // FIXME turn to infallible with map_or
        &self.textures[name]
    }

    pub fn get_texture_mut(&mut self, name: &str) -> &mut Texture<'a> {
        // FIXME turn to infallible with map_or
        self.textures.get_mut(name).unwrap()
    }
}
