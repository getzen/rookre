use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

use crate::card::{CardSuit, FaceRank, GameRank, Points};
use crate::player::PlayerId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeckKind {
    Standard52,
    Standard53,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PartnerKind {
    None,
    Across,
    Called,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BiddingKind {
    None,
    Points,
    Tricks,
    TricksAndSuit,
    Euchre,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BiddingProgression {
    OneTimeAround,
    OneBidderLeft,
    Simultaneous,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrumpPicking {
    None,
    Fixed(CardSuit),
    Random,
    TurnOverCard,
    WithBid,
    AfterBid,
    Euchre,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PointsAwarded {
    // assign separately to Makers and Defenders
    Fixed(Points),
    PointsTakenWithMultiplier(Points),
    //PointsBid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NestPointsOption {
    CardPoints,
    Fixed(Points),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptions {
    pub player_count: PlayerId,
    pub deck_kind: DeckKind,
    pub remove_ranks: Vec<FaceRank>,
    //pub remove_cards: Vec<(FaceRank, char)>,
    pub bird_joker_rank: GameRank,
    pub bird_joker_points: Points,
    // TODO: Re-think this. It's not fine-grained enough, eg Red 1 = best card.
    //pub face_rank_to_game_rank_changes: Vec<(FaceRank, GameRank)>,
    pub face_rank_points: Vec<(FaceRank, Points)>,
    //pub partner_kind: PartnerKind,
    pub hand_size: u8,
    /// This might be smaller than the number of cards left after dealing.
    /// If so, it becomes the effective exhange limit. Any remaining cards in
    /// the deck are added to the nest after the exchange.
    pub nest_size: u8,
    /// The number of nest cards presented face up.
    pub nest_face_up: u8,
    pub bidding_kind: BiddingKind,
    //pub bidding_progression: BiddingProgression,
    //pub bid_increment: usize,
    //pub trump_picking: TrumpPicking,
    pub makers_points_awarded_for_win: PointsAwarded,
    pub makers_points_awarded_for_loss: PointsAwarded,
    pub defenders_points_awarded_for_win: PointsAwarded,
    pub defenders_points_awarded_for_loss: PointsAwarded,
    pub nest_points_awarded: NestPointsOption,
}

impl GameOptions {
    pub fn new() -> Self {
        Self {
            // Anything other than 4 will not work properly.
            player_count: 4,
            deck_kind: DeckKind::Standard53,
            remove_ranks: vec![2, 3, 4],
            //remove_cards: vec![],
            bird_joker_rank: 10.5, // default = 16.0
            bird_joker_points: 20,
            //face_rank_to_game_rank_changes: vec![(1, 14.0)], // eg (1, 15.0)
            face_rank_points: vec![(5, 5), (10, 10), (14, 10)],
            //partner_kind: PartnerKind::Across,
            hand_size: 9,
            nest_size: 5,
            nest_face_up: 2,
            bidding_kind: BiddingKind::Euchre,
            //bidding_progression: BiddingProgression::OneBidderLeft,
            //bid_increment: 0,
            //trump_picking: TrumpPicking::Euchre,
            makers_points_awarded_for_win: PointsAwarded::PointsTakenWithMultiplier(1),
            makers_points_awarded_for_loss: PointsAwarded::Fixed(0),
            defenders_points_awarded_for_win: PointsAwarded::PointsTakenWithMultiplier(2),
            defenders_points_awarded_for_loss: PointsAwarded::Fixed(0),
            nest_points_awarded: NestPointsOption::CardPoints,
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
