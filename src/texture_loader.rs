use std::collections::HashMap;

use notan::prelude::*;

use crate::card::{Card, CardSuit};

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
        self.textures.get(name)
    }

    pub fn load_card_texture(gfx: &mut Graphics, card: &Card) -> Texture {
        let builder = match card.suit {
            CardSuit::Club => match card.face_rank {
                1 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb1.png")),
                2 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb2.png")),
                3 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb3.png")),
                4 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb4.png")),
                5 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb5.png")),
                6 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb6.png")),
                7 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb7.png")),
                8 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb8.png")),
                9 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb9.png")),
                10 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clb10.png")),
                11 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clbJ.png")),
                12 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clbQ.png")),
                13 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clbK.png")),
                14 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/clbA.png")),
                _ => panic!("Can't load tex for rank {}", card.face_rank),
            },
            CardSuit::Diamond => match card.face_rank {
                1 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia1.png")),
                2 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia2.png")),
                3 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia3.png")),
                4 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia4.png")),
                5 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia5.png")),
                6 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia6.png")),
                7 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia7.png")),
                8 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia8.png")),
                9 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia9.png")),
                10 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/dia10.png")),
                11 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/diaJ.png")),
                12 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/diaQ.png")),
                13 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/diaK.png")),
                14 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/diaA.png")),
                _ => panic!("Can't load tex for rank {}", card.face_rank),
            },
            CardSuit::Heart => match card.face_rank {
                1 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt1.png")),
                2 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt2.png")),
                3 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt3.png")),
                4 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt4.png")),
                5 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt5.png")),
                6 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt6.png")),
                7 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt7.png")),
                8 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt8.png")),
                9 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt9.png")),
                10 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrt10.png")),
                11 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrtJ.png")),
                12 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrtQ.png")),
                13 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrtK.png")),
                14 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/hrtA.png")),
                _ => panic!("Can't load tex for rank {}", card.face_rank),
            },
            CardSuit::Spade => match card.face_rank {
                1 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd1.png")),
                2 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd2.png")),
                3 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd3.png")),
                4 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd4.png")),
                5 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd5.png")),
                6 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd6.png")),
                7 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd7.png")),
                8 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd8.png")),
                9 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd9.png")),
                10 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spd10.png")),
                11 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spdJ.png")),
                12 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spdQ.png")),
                13 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spdK.png")),
                14 => gfx
                    .create_texture()
                    .from_image(include_bytes!("assets/cards/spdA.png")),
                _ => panic!("Can't load tex for rank {}", card.face_rank),
            },
            CardSuit::Joker => gfx
                .create_texture()
                .from_image(include_bytes!("assets/cards/joker.png")),
        };

        builder.build().unwrap()
    }

    pub fn load_suit_texture(gfx: &mut Graphics, suit: &CardSuit) -> Texture {
        let builder = match suit {
            CardSuit::Club => gfx
                .create_texture()
                .from_image(include_bytes!("assets/club.png")),
            CardSuit::Diamond => gfx
                .create_texture()
                .from_image(include_bytes!("assets/diamond.png")),
            CardSuit::Heart => gfx
                .create_texture()
                .from_image(include_bytes!("assets/heart.png")),
            CardSuit::Spade => gfx
                .create_texture()
                .from_image(include_bytes!("assets/spade.png")),
            _ => panic!(),
        };
        builder.build().unwrap()
    }

    pub fn load_suit_mouse_over_texture(gfx: &mut Graphics, suit: &CardSuit) -> Texture {
        let builder = match suit {
            CardSuit::Club => gfx
                .create_texture()
                .from_image(include_bytes!("assets/club_mouse_over.png")),
            CardSuit::Diamond => gfx
                .create_texture()
                .from_image(include_bytes!("assets/diamond_mouse_over.png")),
            CardSuit::Heart => gfx
                .create_texture()
                .from_image(include_bytes!("assets/heart_mouse_over.png")),
            CardSuit::Spade => gfx
                .create_texture()
                .from_image(include_bytes!("assets/spade_mouse_over.png")),
            _ => panic!(),
        };
        builder.build().unwrap()
    }
}
