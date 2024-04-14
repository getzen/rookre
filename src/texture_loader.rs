use std::collections::HashMap;

use notan::prelude::*;

pub const TEX_SCALE: f32 = 2.0; // Default texture images are double size.
pub const CARD_TEX_SCALE: f32 = 3.0; // Card images are triple size.

pub struct TextureLoader {
    assets: HashMap<String, Asset<Texture>>,
    pub assets_completed: bool,
    textures: HashMap<String, Texture>,
}

impl TextureLoader {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            assets_completed: false,
            textures: HashMap::new(),
        }
    }

    fn asset_path(path: &str) -> String {
        let base = if cfg!(target_arch = "wasm32") {
            // browsers
            "./assets"
        } else {
            "./src/assets" // development: debug & release
        };
        format!("{base}/{path}")
    }

    // pub fn load_assets(&mut self, assets: &mut Assets, names: &[String]) {
    //     for name in names {
    //         let adj_name = format!("{}.png", name);
    //         let path = TextureLoader::asset_path(&adj_name);
    //         let ass_tex: Asset<Texture> = assets.load_asset(&path).unwrap();
    //         self.assets.insert(name.to_string(), ass_tex);
    //     }
    // }

    pub fn load_assets(&mut self, assets: &mut Assets, names: &[String]) {
        for name in names {
            let adj_name = format!("{}.png", name);
            let path = TextureLoader::asset_path(&adj_name);
            let ass_tex: Asset<Texture> = assets.load_asset(&path).unwrap();
            self.assets.insert(name.clone(), ass_tex);
        }
    }

    /// Returns true if asset loading is completed.
    pub fn update(&mut self) -> bool {
        let ids: Vec<String> = self.assets.keys().cloned().collect();

        for id in &ids {
            if let Some(item) = self.assets.get(id) {
                if item.is_loaded() {
                    let asset_tex = self.assets.remove(id).unwrap();
                    let tex = asset_tex.lock().unwrap().clone();
                    self.textures.insert(id.clone(), tex);
                }
            }
        }
        self.assets.is_empty()
    }

    pub fn get_tex(&self, name: &str) -> Option<&Texture> {
        let tex = self.textures.get(name);
        match tex {
            Some(_) => tex,
            None => {
                println!("Can't find texture: {}", name);
                None
            }
        }
    }
}
