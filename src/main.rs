use std::collections::{HashMap, HashSet};

mod game_data {
    #[derive(Clone, PartialEq, Eq)]
    pub enum Card {
        Land,
        Creature(CreatureCard),
    }
    #[derive(Clone, PartialEq, Eq)]
    pub struct CreatureCard {
        cmc: u64,
        pow: u64,
        tou: u64,
    }
    impl CreatureCard {
        pub fn cmc(&self) -> u64 {
            self.cmc
        }
        pub fn pow(&self) -> u64 {
            self.pow
        }
        pub fn tou(&self) -> u64 {
            self.tou
        }
    }
    impl CreatureCard {
        pub fn try_new(cmc: u64, pow: u64, tou: u64) -> Result<Self, ()> {
            /* Questionable cards - do they have drawbacks?
             * Permeating Mass - (1, 1, 3)
             *   It's a drawback, wrapped in an upside.
             * Nullhide Ferox - (4, 6, 6)
             *   Was clearly printed with a drawback,
             *   but the drawback is irrelevant in this format.
             */
            let allowed_cpt = vec![
                (0, 1, 1), // Memnite
                (0, 0, 3), // Phyrexian Walker
                (1, 2, 2), // Icehide Golem; Isamaru, Hound of Konda
                /* Disowned Ancestor, God-Pharoh's Faithful, Kraken Hatchling,
                 * Lagonna-Band Trailblazer, Merfolk Secretkeeper,
                 * Perimeter Captain, Sidisi's Faithful, Steel Wall,
                 * Tassled Dromedary, Wall of Runes, Yoked Ox
                 */
                (1, 0, 4),
                // Bronzehide Lion, Fleecemane Lion, Kalonian Tusker, Watchwolf
                (2, 3, 3),
                (2, 1, 5), // Grizzled Leotau
                // Dragon's Eye Savants, Fortified Rampart, Wall of Tanglecord
                (2, 0, 6),
                (3, 5, 4),    // Steel Leaf Chamption, Wooly Thoctar
                (3, 4, 5),    // Leatherback Baloth
                (3, 0, 8),    // Wall of Denial, Wall of Stone
                (4, 6, 6),    // Nullhide Ferox
                (4, 2, 10),   // Indominable Ancients
                (4, 0, 13),   // Tree of Perdition, Tree of Ancients
                (5, 10, 10),  // Gigantosaurus
                (9, 11, 9),   // Void Winnower
                (9, 7, 11),   // Inkwell Leviathan
                (10, 16, 16), // Impervious Greatwurm
            ];
            if allowed_cpt.contains(&(cmc, pow, tou)) {
                Ok(CreatureCard { cmc, pow, tou })
            } else {
                Err(())
            }
        }
    }
    pub struct Creature {
        cmc: u64,
        pow: u64,
        tou: u64,
        pub tapped: bool,
    }
    impl Creature {
        pub fn new(creature_card: &CreatureCard) -> Self {
            Creature {
                cmc: creature_card.cmc,
                pow: creature_card.pow,
                tou: creature_card.tou,
                tapped: false,
            }
        }
    }
    // Either muligan or keep and return cards.
    pub enum MuliganChoice {
        Muligan,
        KeepExcept(Vec<usize>),
    }
    // The information a player has available
    pub struct PlayerView<'a> {
        pub num_turn: u64,
        pub hand: &'a Vec<Card>,
        pub num_lands: u64,
        pub creatures: &'a Vec<Creature>,
        pub deck_size: usize,
        pub oth_hand_size: usize,
        pub oth_lands: u64,
        pub oth_creatures: &'a Vec<Creature>,
        pub oth_deck_size: usize,
    }
    // Response for main phase:
    // whether to play a land,
    // indexes in hand of creatures to play
    pub struct MainPhasePlays {
        pub land: bool,
        pub cards: Vec<usize>,
    }
}

mod player {
    use crate::game_data::{Card, CreatureCard, MainPhasePlays, MuliganChoice, PlayerView};
    use std::collections::{HashMap, HashSet};

    pub enum Player {
        LandsSuck,
        LandsRule,
    }
    impl Player {
        // Make a 60 card deck
        pub fn make_deck(&mut self) -> Vec<Card> {
            match self {
                Player::LandsSuck => {
                    let memnite = CreatureCard::try_new(0, 1, 1).expect("Memnite is allowed");
                    vec![Card::Creature(memnite); 60]
                }
                Player::LandsRule => vec![Card::Land; 60],
            }
        }
        pub fn muligan_choice(
            &mut self,
            hand: &Vec<Card>,
            num_muls: usize,
            is_first: bool,
        ) -> MuliganChoice {
            match self {
                Player::LandsSuck => MuliganChoice::KeepExcept(vec![]),
                Player::LandsRule => MuliganChoice::KeepExcept(vec![]),
            }
        }
        pub fn attack(&mut self, view: PlayerView) -> Vec<usize> {
            match self {
                Player::LandsSuck => (0..view.creatures.len()).collect(),
                Player::LandsRule => vec![],
            }
        }
        pub fn block(&mut self, view: PlayerView, attackers: &Vec<usize>) -> Vec<(usize, usize)> {
            match self {
                Player::LandsSuck => todo!(),
                Player::LandsRule => vec![],
            }
        }
        pub fn order_blockers(
            &mut self,
            view: PlayerView,
            default: &HashMap<usize, Vec<usize>>,
        ) -> HashMap<usize, Vec<usize>> {
            match self {
                Player::LandsSuck => todo!(),
                Player::LandsRule => default.clone(),
            }
        }
        pub fn main_phase(&mut self, view: PlayerView) -> MainPhasePlays {
            match self {
                Player::LandsSuck => MainPhasePlays {
                    land: false,
                    cards: (0..view.hand.len()).collect(),
                },
                Player::LandsRule => MainPhasePlays {
                    land: !view.hand.is_empty(),
                    cards: vec![],
                },
            }
        }
        pub fn discard(&mut self, view: PlayerView) -> Vec<usize> {
            assert!(view.hand.len() > 7);
            match self {
                Player::LandsSuck | Player::LandsRule => (0..view.hand.len() - 7).collect(),
            }
        }
    }
}
use crate::game_data::{Card, Creature, MainPhasePlays, MuliganChoice, PlayerView};
use crate::player::Player;
use rand::prelude::*;

struct PlayerState {
    player: Player,
    deck: Vec<Card>,
    hand: Vec<Card>,
    num_lands: u64,
    creatures: Vec<Creature>,
}
#[derive(Debug, Eq, PartialEq)]
enum DrawResult {
    Empty,
    Nonempty,
}
impl PlayerState {
    fn new(mut player: Player) -> Self {
        let deck = player.make_deck();
        assert_eq!(deck.len(), 60);
        PlayerState {
            player,
            deck,
            hand: vec![],
            num_lands: 0,
            creatures: vec![],
        }
    }
    fn do_muligans(&mut self, is_first: bool) {
        let mut rng = thread_rng();
        let mut num_muls = 0;
        while num_muls < 7 {
            self.deck.shuffle(&mut rng);
            for _ in 0..7 {
                let draw_result = self.draw();
                assert_eq!(draw_result, DrawResult::Nonempty);
            }
            let perform_muligan = self.player.muligan_choice(&self.hand, num_muls, is_first);
            if let MuliganChoice::KeepExcept(remove) = perform_muligan {
                assert_eq!(remove.len(), num_muls);
                for &index in &remove {
                    assert!(index < 7);
                }
                for i in (0..7).rev() {
                    if remove.contains(&i) {
                        let card = self.hand.remove(i);
                        self.deck.insert(0, card);
                    }
                }
                assert_eq!(self.hand.len(), 7 - num_muls);
                return;
            }
            self.deck.extend(self.hand.drain(..));
            num_muls += 1;
        }
        // If mul down to 0, exit here.
        assert!(self.hand.is_empty());
    }
    fn handle_main_phase_plays(&mut self, main_phase_plays: MainPhasePlays) {
        if main_phase_plays.land {
            let land_position = self
                .hand
                .iter()
                .position(|c| c == &Card::Land)
                .expect("Player tried to play land, so land is present.");
            self.hand.iter().skip(land_position).for_each(|c| match c {
                Card::Creature(_) => panic!("Creature after land"),
                Card::Land => (),
            });
            self.hand.remove(land_position);
            self.num_lands += 1;
        }
        let total_cmc: u64 = main_phase_plays
            .cards
            .iter()
            .map(|i| {
                assert!(*i < self.hand.len());
                let card = &self.hand[*i];
                if let Card::Creature(creature_card) = card {
                    creature_card.cmc()
                } else {
                    panic!("Only cast creatures");
                }
            })
            .sum();
        assert!(total_cmc <= self.num_lands);
        main_phase_plays.cards.iter().for_each(|i| {
            let card = &self.hand[*i];
            if let Card::Creature(creature_card) = card {
                let creature = Creature::new(creature_card);
                self.creatures.push(creature);
            } else {
                panic!("Only cast creatures");
            }
        });

        let prior_number_cards = self.hand.len();
        let mut index = 0;
        self.hand.retain(|_| {
            let keep = !main_phase_plays.cards.contains(&index);
            index += 1;
            keep
        });
        assert_eq!(
            prior_number_cards,
            self.hand.len() + main_phase_plays.cards.len(),
            "Play correct number of cards"
        );
    }
    fn handle_discard(&mut self, discard_indices: Vec<usize>) {
        assert_eq!(
            discard_indices.len(),
            self.hand.len() - 7,
            "Attempt to discard correct number of cards"
        );
        let mut index = 0;
        self.hand.retain(|_| {
            let keep = !discard_indices.contains(&index);
            index += 1;
            keep
        });
        assert_eq!(
            self.hand.len(),
            7,
            "Discard correct number of cards"
        );
    }
    fn draw(&mut self) -> DrawResult {
        if self.deck.is_empty() {
            DrawResult::Empty
        } else {
            let card = self.deck.pop().expect("Nonempty");
            self.hand.push(card);
            DrawResult::Nonempty
        }
    }
    fn sort_hand(&mut self) {
        self.hand
            .sort_by_key(|card| if Card::Land == *card { 1 } else { 0 })
    }
    fn view_and_mut<'a>(
        &'a mut self,
        other_state: &'a Self,
        num_turn: u64,
    ) -> (PlayerView<'a>, &'a mut Player) {
        let view = PlayerView {
            num_turn,
            hand: &self.hand,
            num_lands: self.num_lands,
            creatures: &self.creatures,
            deck_size: self.deck.len(),
            oth_hand_size: other_state.hand.len(),
            oth_lands: other_state.num_lands,
            oth_creatures: &other_state.creatures,
            oth_deck_size: other_state.deck.len(),
        };
        (view, &mut self.player)
    }
}

#[derive(Debug)]
enum Winner {
    Player1,
    Player2,
}
struct GameState {
    player_states: [PlayerState; 2],
}
impl GameState {
    // TODO: Add a printout flag
    fn new(player1: Player, player2: Player) -> Self {
        GameState {
            player_states: [PlayerState::new(player1), PlayerState::new(player2)],
        }
    }
    fn play(&mut self) -> Winner {
        let mut rng = thread_rng();
        let player1_first = rng.gen::<f64>() < 0.5;
        for (i, player_state) in self.player_states.iter_mut().enumerate() {
            let first = (i == 0) == player1_first;
            player_state.do_muligans(first);
        }
        let mut current_player_index = if player1_first { 0 } else { 1 };
        let first_player_index = current_player_index;
        let mut is_first_turn = true;
        let mut num_turn = 1;
        loop {
            // TODO: Untap
            todo!();
            // Draw step
            if !is_first_turn {
                let draw_result = self.player_states[current_player_index].draw();
                if let DrawResult::Empty = draw_result {
                    // Game over due to decking
                    return if current_player_index == 0 {
                        Winner::Player2
                    } else {
                        Winner::Player1
                    };
                }
            }
            self.player_states[current_player_index].sort_hand();
            // TODO: Current player attacks
            let (current_state, other_state) = self.states_mut(current_player_index);
            let (current_view, current_player) = current_state.view_and_mut(&other_state, num_turn);
            let attackers = current_player.attack(current_view);
            for &attacker in &attackers {
                assert!(attacker < current_state.creatures.len());
                assert!(
                    !current_state.creatures[attacker].tapped,
                    "No double attacks"
                );
                current_state.creatures[attacker].tapped = true;
            }
            // TODO: Other player blocks
            let (other_view, other_player) = other_state.view_and_mut(&current_state, num_turn);
            let blocking_pairs = other_player.block(other_view, &attackers);
            let mut blockers = HashSet::new();
            let mut blocking_arrangement = HashMap::new();
            for (blocker, attacker) in blocking_pairs {
                assert!(attacker < current_state.creatures.len());
                assert!(blocker < other_state.creatures.len());
                assert!(!blockers.contains(&blocker), "Creatures can't block twice");
                blockers.insert(blocker);
                blocking_arrangement
                    .entry(attacker)
                    .or_insert(vec![])
                    .push(blocker);
            }
            // TODO: Current player orders blockers
            let (current_view, current_player) = current_state.view_and_mut(&other_state, num_turn);
            let ordered_blockers =
                current_player.order_blockers(current_view, &blocking_arrangement);
            // TODO: Check for dead creatures, lethal damage
            todo!();
            // Main phase
            let (current_state, other_state) = self.states_mut(current_player_index);
            let (view, player) = current_state.view_and_mut(&other_state, num_turn);
            let main_phase_plays = player.main_phase(view);
            current_state.handle_main_phase_plays(main_phase_plays);

            // Discard
            if current_state.hand.len() > 7 {
                let (view, player) = current_state.view_and_mut(&other_state, num_turn);
                let discard_indices = player.discard(view);
                current_state.handle_discard(discard_indices);
            }
            // Switch current player, increment turn number as appropriate
            current_player_index = 1 - current_player_index;
            if current_player_index == first_player_index {
                num_turn += 1;
            }
        }
    }
    fn states_mut(&mut self, current_player: usize) -> (&mut PlayerState, &mut PlayerState) {
        let (first_state, rest) = self
            .player_states
            .split_first_mut()
            .expect("Multiple players");
        let second_state = &mut rest[0];
        if current_player == 0 {
            (first_state, second_state)
        } else {
            (second_state, first_state)
        }
    }
}
fn main() {
    let mut game = GameState::new(Player::LandsSuck, Player::LandsRule);
    let winner = game.play();
    println!("{:?}", winner);
}
