use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

use crate::card::{CardSuit, FaceRank, GameRank, Points};
use crate::player::PlayerId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PartnerKind {
    None,
    Across,
    Called,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PointsAwarded {
    // assign separately to Makers and Defenders
    Fixed(Points),
    PointsTakenWithMultiplier(Points),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptions {
    pub hand_size: u8,
    /// This might be smaller than the number of cards left after dealing.
    /// If so, it becomes the effective exhange limit. Any remaining cards in
    /// the deck are added to the nest after the exchange.
    pub nest_size: u8,
    /// The number of nest cards presented face up.
    pub nest_face_up: u8,
    pub makers_points_awarded_for_win: PointsAwarded,
    pub makers_points_awarded_for_loss: PointsAwarded,
    pub defenders_points_awarded_for_win: PointsAwarded,
    pub defenders_points_awarded_for_loss: PointsAwarded,
    pub nest_points_bonus: i16,
}

impl GameOptions {
    pub fn new() -> Self {
        Self {
            hand_size: 9,
            nest_size: 2,
            nest_face_up: 0,
            makers_points_awarded_for_win: PointsAwarded::PointsTakenWithMultiplier(1),
            makers_points_awarded_for_loss: PointsAwarded::Fixed(0),
            defenders_points_awarded_for_win: PointsAwarded::PointsTakenWithMultiplier(1),
            defenders_points_awarded_for_loss: PointsAwarded::PointsTakenWithMultiplier(1),
            nest_points_bonus: 10,
        }
    }

    fn read_contents_from_file(path: &str) -> String {
        let mut file = File::open(&path).expect("Could not open: {path}");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read to string: {path}");
        contents
    }

    pub fn read_from_yaml(path: &str) -> GameOptions {
        let contents = GameOptions::read_contents_from_file(path);

        match serde_yaml::from_str(&contents) {
            Ok(options) => options,
            Err(e) => panic!("Error creating GameOptions: {}", e),
        }
    }

    pub fn write_to_yaml(&self, path: &str) {
        let serialized = serde_yaml::to_string(self).unwrap();

        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(e) => panic!("{}", e),
        };
        write!(file, "{}", serialized).expect("File not written: {path}");
    }
}
