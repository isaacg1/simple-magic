use std::collections::{HashMap, HashSet};

mod game_data {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Card {
        Land,
        Creature(CreatureCard),
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct CreatureCard {
        cmc: u64,
        pow: u64,
        tou: u64,
    }
    impl CreatureCard {
        pub fn cmc(&self) -> u64 {
            self.cmc
        }
        #[allow(dead_code)]
        pub fn pow(&self) -> u64 {
            self.pow
        }
        #[allow(dead_code)]
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
    #[allow(dead_code)]
    #[derive(Debug)]
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
        #[allow(dead_code)]
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
    // Either muligan or keep and return cards.
    #[allow(dead_code)]
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
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum Player {
        LandsSuck,
        MemnitesDontBlock,
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
                Player::MemnitesDontBlock => {
                    let memnite = CreatureCard::try_new(0, 1, 1).expect("Memnite is allowed");
                    vec![Card::Creature(memnite); 60]
                }
                Player::LandsRule => vec![Card::Land; 60],
            }
        }
        pub fn muligan_choice(
            &mut self,
            _hand: &Vec<Card>,
            _num_muls: usize,
            _is_first: bool,
        ) -> MuliganChoice {
            match self {
                Player::LandsSuck | Player::MemnitesDontBlock | Player::LandsRule => {
                    MuliganChoice::KeepExcept(vec![])
                }
            }
        }
        pub fn attack(&mut self, view: PlayerView) -> Vec<usize> {
            match self {
                Player::LandsSuck | Player::MemnitesDontBlock => {
                    (0..view.creatures.len()).collect()
                }
                Player::LandsRule => vec![],
            }
        }
        pub fn block(&mut self, view: PlayerView, attackers: &Vec<usize>) -> Vec<(usize, usize)> {
            match self {
                Player::LandsSuck => {
                    let mut blockers = vec![];
                    let mut has_been_blocked = vec![];
                    let mut num_matched = 0;
                    let num_available = view.creatures.iter().filter(|c| !c.tapped).count() as u64;
                    while num_matched < num_available {
                        let best_block = view
                            .oth_creatures
                            .iter()
                            .enumerate()
                            .filter(|(i, c)| {
                                c.tapped
                                    && c.tou() <= num_available - num_matched
                                    && !has_been_blocked.contains(i)
                            })
                            .max_by_key(|(_, c)| c.tou());
                        if let Some((best_block_index, best_block_creature)) = best_block {
                            assert!(attackers.contains(&best_block_index));
                            let num_block = best_block_creature.tou();
                            for creature_number in num_matched..num_matched + num_block {
                                let blocker_index = view
                                    .creatures
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, c)| !c.tapped)
                                    .nth(creature_number as usize)
                                    .expect("Enough blockers available")
                                    .0;
                                blockers.push((blocker_index, best_block_index))
                            }
                            num_matched += num_block;
                            has_been_blocked.push(best_block_index);
                        } else {
                            break;
                        }
                    }
                    blockers
                }
                Player::MemnitesDontBlock | Player::LandsRule => vec![],
            }
        }
        pub fn order_blockers(
            &mut self,
            view: PlayerView,
            default_ordering: &HashMap<usize, Vec<usize>>,
        ) -> HashMap<usize, Vec<usize>> {
            match self {
                Player::LandsSuck => {
                    let mut ordering = HashMap::new();
                    for (&attacker, blockers) in default_ordering {
                        let mut blockers = blockers.clone();
                        blockers.sort_by_key(|&b| view.oth_creatures[b].tou());
                        ordering.insert(attacker, blockers);
                    }
                    ordering
                }
                Player::MemnitesDontBlock | Player::LandsRule => default_ordering.clone(),
            }
        }
        pub fn main_phase(&mut self, view: PlayerView) -> MainPhasePlays {
            match self {
                Player::LandsSuck | Player::MemnitesDontBlock => MainPhasePlays {
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
                Player::LandsSuck | Player::LandsRule | Player::MemnitesDontBlock => {
                    (0..view.hand.len() - 7).collect()
                }
            }
        }
    }
}
use crate::game_data::{Card, Creature, MainPhasePlays, MuliganChoice, PlayerView};
use crate::player::Player;
use rand::prelude::*;

#[derive(Debug)]
struct PlayerState {
    player: Player,
    deck: Vec<Card>,
    hand: Vec<Card>,
    num_lands: u64,
    creatures: Vec<Creature>,
    life: i64,
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
            life: 20,
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
        assert_eq!(self.hand.len(), 7, "Discard correct number of cards");
    }
    fn die(&mut self, dead_creatures: Vec<usize>) {
        let prior_number_creatures = self.creatures.len();
        let mut index = 0;
        self.creatures.retain(|_| {
            let keep = !dead_creatures.contains(&index);
            index += 1;
            keep
        });
        assert_eq!(
            prior_number_creatures,
            self.creatures.len() + dead_creatures.len(),
            "Correct number of creatures die"
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
    fn untap(&mut self) {
        for creature in &mut self.creatures {
            creature.tapped = false
        }
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
    fn print_player(&self, is_current_player: bool) {
        print!(
            "L: {}, C: {}, P: {:?}",
            self.life,
            self.deck.len(),
            self.player
        );
        if is_current_player {
            print!("   <<<");
        }
        println!();
    }
    fn print_hand(&self) {
        print!("H: ");
        for card in &self.hand {
            match card {
                Card::Creature(cc) => print!("{}/{}/{} ", cc.cmc(), cc.pow(), cc.tou()),
                Card::Land => print!("Land "),
            }
        }
        println!();
    }
    fn print_battlefield(&self) {
        print!("B: {} lands    ", self.num_lands);
        for creature in &self.creatures {
            print!(
                "{}/{}/{}{} ",
                creature.cmc(),
                creature.pow(),
                creature.tou(),
                if creature.tapped { "t" } else { "u" }
            )
        }
        println!();
    }
}

#[derive(Debug)]
enum Winner {
    Player1,
    Player2,
}
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
enum Printout {
    PrintAndPause,
    Print,
    Nothing,
}
#[derive(Debug)]
struct GameState {
    player_states: [PlayerState; 2],
    num_turn: u64,
    current_player_index: usize,
    printout: Printout,
}
impl GameState {
    #[allow(dead_code)]
    fn new_with_flip(player1: Player, player2: Player, printout: Printout) -> Self {
        let mut rng = thread_rng();
        let player1_first = rng.gen::<f64>() < 0.5;
        if player1_first {
            GameState::new(player1, player2, printout)
        } else {
            GameState::new(player2, player1, printout)
        }
    }
    fn new(player1: Player, player2: Player, printout: Printout) -> Self {
        GameState {
            player_states: [PlayerState::new(player1), PlayerState::new(player2)],
            num_turn: 1,
            current_player_index: 0,
            printout,
        }
    }
    fn play(&mut self) -> Winner {
        for (i, player_state) in self.player_states.iter_mut().enumerate() {
            player_state.do_muligans(i == 0);
        }
        loop {
            let num_turn = self.num_turn;
            let current_player_index = self.current_player_index;
            // Untap
            let (current_state, _) = self.states_mut(current_player_index);
            current_state.untap();
            if !(num_turn == 1 && current_player_index == 0) {
                self.handle_printout("Untap");
            }
            // Draw step
            if !(num_turn == 1 && current_player_index == 0) {
                let draw_result = self.player_states[current_player_index].draw();
                if let DrawResult::Empty = draw_result {
                    self.handle_printout("Game over due to decking");
                    // Game over due to decking
                    return if current_player_index == 0 {
                        Winner::Player2
                    } else {
                        Winner::Player1
                    };
                }
            }
            self.player_states[current_player_index].sort_hand();
            self.handle_printout("Draw");
            // Current player attacks
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
            if !attackers.is_empty() {
                self.handle_printout("Attack");
            }
            // Other player blocks
            let (current_state, other_state) = self.states_mut(current_player_index);
            let (other_view, other_player) = other_state.view_and_mut(&current_state, num_turn);
            let blocking_pairs = other_player.block(other_view, &attackers);
            let mut blockers = HashSet::new();
            let mut blocking_arrangement = HashMap::new();
            for (blocker, attacker) in blocking_pairs {
                assert!(attacker < current_state.creatures.len());
                assert!(blocker < other_state.creatures.len());
                assert!(
                    !other_state.creatures[blocker].tapped,
                    "Tapped creatures can't block"
                );
                assert!(!blockers.contains(&blocker), "Creatures can't block twice");
                blockers.insert(blocker);
                blocking_arrangement
                    .entry(attacker)
                    .or_insert(vec![])
                    .push(blocker);
            }
            // Current player orders blockers
            let (current_view, current_player) = current_state.view_and_mut(&other_state, num_turn);
            let ordered_blockers =
                current_player.order_blockers(current_view, &blocking_arrangement);
            assert_eq!(
                blocking_arrangement.len(),
                ordered_blockers.len(),
                "Same number of attackers"
            );
            for (attacker, blockers) in &ordered_blockers {
                assert!(blocking_arrangement.contains_key(attacker));
                let default_blockers = &blocking_arrangement[attacker];
                assert_eq!(
                    blockers.len(),
                    default_blockers.len(),
                    "Same number of blockers"
                );
                blockers
                    .iter()
                    .for_each(|i| assert!(default_blockers.contains(i)));
                default_blockers
                    .iter()
                    .for_each(|i| assert!(blockers.contains(i)));
            }
            let mut all_blockers = ordered_blockers;
            // Add in unblocked attackers
            for &attacker in &attackers {
                all_blockers.entry(attacker).or_insert(vec![]);
            }
            // Damage, check for dead creatures, lethal damage
            let mut dead_attackers = vec![];
            let mut dead_blockers = vec![];
            for (&attacker, blockers) in &all_blockers {
                let attacker_pow = current_state.creatures[attacker].pow();
                if blockers.is_empty() {
                    other_state.life -= attacker_pow as i64
                } else {
                    let mut attacker_damage_remaining = attacker_pow;
                    for &blocker in blockers {
                        let blocker_tou = other_state.creatures[blocker].tou();
                        if blocker_tou > attacker_damage_remaining {
                            break;
                        } else {
                            attacker_damage_remaining -= blocker_tou;
                            dead_blockers.push(blocker)
                        }
                    }
                    let blocker_damage_total: u64 = blockers
                        .iter()
                        .map(|&b| other_state.creatures[b].pow())
                        .sum();
                    let attacker_tou = current_state.creatures[attacker].tou();
                    if blocker_damage_total >= attacker_tou {
                        dead_attackers.push(attacker)
                    }
                }
            }
            current_state.die(dead_attackers);
            other_state.die(dead_blockers);

            if other_state.life <= 0 {
                self.handle_printout("Game over due to life");
                // Game over due to life loss
                return if current_player_index == 0 {
                    Winner::Player1
                } else {
                    Winner::Player2
                };
            }
            if !attackers.is_empty() {
                self.handle_printout("Damage");
            }
            // Main phase
            let (current_state, other_state) = self.states_mut(current_player_index);
            let (view, player) = current_state.view_and_mut(&other_state, num_turn);
            let main_phase_plays = player.main_phase(view);
            current_state.handle_main_phase_plays(main_phase_plays);
            self.handle_printout("Main phase");

            // Discard
            let (current_state, other_state) = self.states_mut(current_player_index);
            if current_state.hand.len() > 7 {
                let (view, player) = current_state.view_and_mut(&other_state, num_turn);
                let discard_indices = player.discard(view);
                current_state.handle_discard(discard_indices);
                self.handle_printout("Discard");
            }
            // Switch current player, increment turn number as appropriate
            self.current_player_index = 1 - self.current_player_index;
            if self.current_player_index == 0 {
                self.num_turn += 1;
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
    fn handle_printout(&self, phase: &str) {
        if let Printout::Nothing = self.printout {
            return;
        }
        println!("{} {}", phase, self.num_turn);
        let state0 = &self.player_states[0];
        let state1 = &self.player_states[1];
        state0.print_player(self.current_player_index == 0);
        state0.print_hand();
        state0.print_battlefield();
        state1.print_battlefield();
        state1.print_hand();
        state1.print_player(self.current_player_index == 1);

        if let Printout::PrintAndPause = self.printout {
            use std::io::{stdin, stdout, Write};
            let mut s = String::new();
            println!("Enter to continue");
            stdout().flush().expect("Flushed");
            stdin().read_line(&mut s).expect("Continued");
        }
    }
}
fn main() {
    for (player1, player2) in vec![
        (Player::LandsRule, Player::LandsRule),
        (Player::LandsRule, Player::LandsSuck),
        (Player::LandsSuck, Player::LandsSuck),
        (Player::LandsSuck, Player::MemnitesDontBlock),
        (Player::MemnitesDontBlock, Player::LandsSuck),
        (Player::MemnitesDontBlock, Player::MemnitesDontBlock),
    ] {
        let mut game = GameState::new(player1, player2, Printout::Nothing);
        let winner = game.play();
        let player1 = &game.player_states[0].player;
        let player2 = &game.player_states[1].player;
        println!(
            "{:?} v {:?}: {:?} ({}) wins",
            player1, player2,
            match winner {
                Winner::Player1 => player1,
                Winner::Player2 => player2,
            },
            match winner {
                Winner::Player1 => 0,
                Winner::Player2 => 1,
            }
        );
        println!(
            "On turn {} of {:?} ({}), life {} v {}",
            game.num_turn,
            game.player_states[game.current_player_index].player,
            game.current_player_index,
            game.player_states[0].life,
            game.player_states[1].life
        );
        println!()
    }
}
