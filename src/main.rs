mod game_data {
    #[derive(Clone)]
    pub enum Card {
        Land,
        Creature(CreatureCard),
    }
    #[derive(Clone)]
    pub struct CreatureCard {
        cmc: u64,
        pow: u64,
        tou: u64,
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
                (3, 5, 4), // Steel Leaf Chamption, Wooly Thoctar
                (3, 4, 5), // Leatherback Baloth
                (3, 0, 8), // Wall of Denial, Wall of Stone
                /* Deadbridge Goliath; Phyrexian Obliterator;
                 * Polukranos, World Eater; Rampart Smasher;
                 * Spellbreaker Behemoth; Sunder Shaman;
                 * Tahngarth, First Mate; Territorial Allosaurus
                 */
                (4, 5, 5),
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
    // Either muligan or keep and return cards.
    pub enum MuliganChoice {
        Muligan,
        KeepExcept(Vec<usize>),
    }
}

mod player {
    use crate::game_data::{Card, CreatureCard, MuliganChoice};
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
    }
}
use crate::game_data::{Card, MuliganChoice};
use crate::player::Player;
use rand::prelude::*;

struct PlayerState {
    player: Player,
    deck: Vec<Card>,
    hand: Vec<Card>,
    // battlefields:
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
    fn draw(&mut self) -> DrawResult {
        if self.deck.is_empty() {
            DrawResult::Empty
        } else {
            let card = self.deck.pop().expect("Nonempty");
            self.hand.push(card);
            DrawResult::Nonempty
        }
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
        todo!()
    }
}
fn main() {
    let mut game = GameState::new(Player::LandsSuck, Player::LandsRule);
    let winner = game.play();
    println!("{:?}", winner);
}
