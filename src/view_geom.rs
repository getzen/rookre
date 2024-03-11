use std::f32::consts::PI;

use notan::math::{vec2, Vec2};

use crate::player::PlayerId;

pub const CARD_SIZE: (f32, f32) = (80., 120.); // texture is 240 x 360
pub const VIEW_CENTER: Vec2 = vec2(400., 400.);
pub const BUTTON_POS: Vec2 = vec2(400., 480.);
pub const MESSAGE_POS: (f32, f32) = (1100., 910.);

pub struct ViewGeom {}

impl ViewGeom {
    pub fn position_from(start_pos: Vec2, radians: f32, magnitude: f32) -> Vec2 {
        vec2(
            start_pos.x + radians.cos() * magnitude,
            start_pos.y + radians.sin() * magnitude,
        )
    }

    pub fn player_radians_from_center(player_id: usize, player_count: usize) -> f32 {
        player_id as f32 * PI * 2.0 / player_count as f32 + PI / 2.0
    }

    pub fn player_rotation(player_id: usize, player_count: usize) -> f32 {
        player_id as f32 * PI * 2.0 / player_count as f32
    }

    pub fn message_view_position(p_id: Option<PlayerId>, player_count: usize) -> Vec2 {
        match p_id {
            Some(p_id) => ViewGeom::bid_view_position(p_id, player_count),
            None => vec2(400., 350.),
        }
    }

    pub fn dealer_marker_position(player_id: usize, player_count: usize) -> Vec2 {
        let distance_from_center = 380.0;
        let radians = ViewGeom::player_radians_from_center(player_id, player_count);
        ViewGeom::position_from(VIEW_CENTER, radians, distance_from_center)
    }

    pub fn active_player_marker_position(player_id: usize, player_count: usize) -> Vec2 {
        let distance_from_center = 370.0;
        let radians = ViewGeom::player_radians_from_center(player_id, player_count);
        ViewGeom::position_from(VIEW_CENTER, radians, distance_from_center)
    }

    pub fn bid_view_position(player_id: usize, player_count: usize) -> Vec2 {
        let distance_from_center = 180.0;
        let radians = ViewGeom::player_radians_from_center(player_id, player_count);
        ViewGeom::position_from(VIEW_CENTER, radians, distance_from_center)
    }

    pub fn discard_panel_position() -> Vec2 {
        vec2(VIEW_CENTER.x, VIEW_CENTER.y + 180.0)
    }
}
