use notan::prelude::*;

use crate::card::CardSuit;
use crate::card::{Card, CardKind};

pub const TEX_SCALE: f32 = 2.0; // Default texture images are double size.
pub const CARD_TEX_SCALE: f32 = 3.0; // Card images are triple size.

// Colors
pub const TABLE_COLOR: Color = Color::from_rgb(0.3, 0.3, 0.3);
pub const DEEP_GREEN: Color = Color::new(0. / 255., 175. / 255., 0. / 255., 1.);
pub const LIGHT_GRAY: Color = Color::new(225. / 255., 225. / 255., 225. / 255., 1.);
pub const MED_GRAY: Color = Color::new(200. / 255., 200. / 255., 200. / 255., 1.);


pub struct ViewFn {}

impl ViewFn {
    pub fn load_card_texture(gfx: &mut Graphics, card: &Card) -> Texture {
        let builder = match card.kind {
            CardKind::Joker => gfx
                .create_texture()
                .from_image(include_bytes!("assets/cards/joker.png")),
            CardKind::Bird => gfx
                .create_texture()
                .from_image(include_bytes!("assets/cards/bird.png")),

            CardKind::Suited => {
                let int_rank = card.face_rank as usize;
                match (int_rank, card.suit) {
                    (1, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb1.png")),
                    (2, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb2.png")),
                    (3, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb3.png")),
                    (4, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb4.png")),
                    (5, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb5.png")),
                    (6, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb6.png")),
                    (7, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb7.png")),
                    (8, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb8.png")),
                    (9, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb9.png")),
                    (10, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb10.png")),
                    (11, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb11.png")),
                    (12, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb12.png")),
                    (13, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb13.png")),
                    (14, CardSuit::Club) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/clb14.png")),

                    (1, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia1.png")),
                    (2, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia2.png")),
                    (3, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia3.png")),
                    (4, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia4.png")),
                    (5, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia5.png")),
                    (6, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia6.png")),
                    (7, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia7.png")),
                    (8, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia8.png")),
                    (9, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia9.png")),
                    (10, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia10.png")),
                    (11, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia11.png")),
                    (12, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia12.png")),
                    (13, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia13.png")),
                    (14, CardSuit::Diamond) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/dia14.png")),

                    (1, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt1.png")),
                    (2, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt2.png")),
                    (3, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt3.png")),
                    (4, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt4.png")),
                    (5, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt5.png")),
                    (6, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt6.png")),
                    (7, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt7.png")),
                    (8, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt8.png")),
                    (9, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt9.png")),
                    (10, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt10.png")),
                    (11, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt11.png")),
                    (12, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt12.png")),
                    (13, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt13.png")),
                    (14, CardSuit::Heart) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/hrt14.png")),

                    (1, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd1.png")),
                    (2, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd2.png")),
                    (3, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd3.png")),
                    (4, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd4.png")),
                    (5, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd5.png")),
                    (6, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd6.png")),
                    (7, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd7.png")),
                    (8, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd8.png")),
                    (9, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd9.png")),
                    (10, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd10.png")),
                    (11, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd11.png")),
                    (12, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd12.png")),
                    (13, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd13.png")),
                    (14, CardSuit::Spade) => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/cards/spd14.png")),

                    _ => panic!("{}, {:?}", card.face_rank, card.suit),
                }
            }
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

    pub fn load_suit_disabled_texture(gfx: &mut Graphics, suit: &CardSuit) -> Texture {
        let builder = match suit {
            CardSuit::Club => gfx
                .create_texture()
                .from_image(include_bytes!("assets/club_disabled.png")),
            CardSuit::Diamond => gfx
                .create_texture()
                .from_image(include_bytes!("assets/diamond_disabled.png")),
            CardSuit::Heart => gfx
                .create_texture()
                .from_image(include_bytes!("assets/heart_disabled.png")),
            CardSuit::Spade => gfx
                .create_texture()
                .from_image(include_bytes!("assets/spade_disabled.png")),
            _ => panic!(),
        };
        builder.build().unwrap()
    }

    /*
        pub fn create_trump_view(gfx: &mut Graphics) -> TrumpView {
            let mut suit_textures = Vec::new();
            for i in 0..4 {
                let tex_builder = match i {
                    0 => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/club.png")),
                    1 => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/diamond.png")),
                    2 => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/heart.png")),
                    3 => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/spade.png")),
                    4 => gfx
                        .create_texture()
                        .from_image(include_bytes!("assets/star.png")),
                    _ => panic!(),
                };
                suit_textures.push(tex_builder.build().unwrap());
            }
            TrumpView::new((400., 400.), suit_textures)
        }

        pub fn create_bid_views(gfx: &mut Graphics, font: Font, player_count: usize) -> Vec<BidView> {
            let mut views = Vec::new();
            for p in 0..player_count {
                let pos = ViewFn::bid_view_position(p, player_count);
                let vb = BidView::new(pos, 0.0, font);
                views.push(vb);
            }
            views
        }

        pub fn create_discard_outlines(gfx: &mut Graphics, count: usize) -> Vec<Imager> {
            let texture = gfx
                .create_texture()
                .from_image(include_bytes!("assets/cards/outline.png"))
                .build()
                .unwrap();
            let mut markers = Vec::new();
            for _ in 0..count {
                let mut marker = Imager::with_size(CARD_SIZE, texture.clone());
                marker.visible = false;
                markers.push(marker);
            }
            markers
        }

        pub fn create_play_outline(gfx: &mut Graphics) -> Imager {
            let texture = gfx
                .create_texture()
                .from_image(include_bytes!("assets/cards/outline.png"))
                .build()
                .unwrap(); // 170 x 230
            let mut play_outline = Imager::with_size((85., 115.), texture);
            play_outline.visible = false;
            play_outline
        }
    */
    
}
