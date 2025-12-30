use dioxus::prelude::*;

const _GAME_CSS: Asset = asset!("/assets/styling/game.css");

#[derive(Clone, PartialEq, Debug)]
struct Player {
    name: String,
    score: i32,
    is_eliminated: bool,
}

#[derive(Clone, PartialEq, Debug)]
enum CardType {
    Normal,
    Imposter,
}

#[derive(Clone, PartialEq, Debug)]
struct GameCard {
    card_type: CardType,
    word: String,
}

#[derive(Clone, PartialEq, Debug)]
enum GameScreen {
    Setup,
    CardView { current_player_index: usize },
    Voting,
    Elimination { eliminated_index: usize, was_imposter: bool },
    RoundEnd { imposter_found: bool, game_over: bool },
    GameScore,
}

/// Main Game component
#[component]
pub fn Game() -> Element {
    let game_screen = use_signal(|| GameScreen::Setup);
    let players = use_signal(|| Vec::<Player>::new());
    let player_count_input = use_signal(|| String::from("3"));
    let player_names = use_signal(|| Vec::<String>::new());
    let round_number = use_signal(|| 1);
    let cards = use_signal(|| Vec::<GameCard>::new());
    let imposter_index = use_signal(|| 0usize);
    
    rsx! {
        document::Stylesheet { href: _GAME_CSS }
        div { class: "game-container",
            match game_screen() {
                GameScreen::Setup => rsx! {
                    SetupScreen {
                        player_count_input,
                        player_names,
                        players,
                        game_screen,
                        round_number,
                    }
                },
                GameScreen::CardView { current_player_index } => rsx! {
                    CardViewScreen {
                        current_player_index,
                        players,
                        cards,
                        imposter_index,
                        game_screen,
                    }
                },
                GameScreen::Voting => rsx! {
                    VotingScreen {
                        players,
                        game_screen,
                        imposter_index,
                    }
                },
                GameScreen::Elimination { eliminated_index, was_imposter } => rsx! {
                    EliminationScreen {
                        players,
                        eliminated_index,
                        was_imposter,
                        game_screen,
                        round_number,
                        cards,
                        imposter_index,
                    }
                },
                GameScreen::RoundEnd { imposter_found, game_over } => rsx! {
                    RoundEndScreen {
                        players,
                        imposter_found,
                        game_over,
                        game_screen,
                        round_number,
                        cards,
                        imposter_index,
                    }
                },
                GameScreen::GameScore => rsx! {
                    GameScoreScreen {
                        players,
                        round_number,
                        game_screen,
                        cards,
                        imposter_index,
                    }
                },
            }
        }
    }
}

/// Setup screen for entering player names
#[component]
fn SetupScreen(
    mut player_count_input: Signal<String>,
    mut player_names: Signal<Vec<String>>,
    mut players: Signal<Vec<Player>>,
    mut game_screen: Signal<GameScreen>,
    mut round_number: Signal<i32>,
) -> Element {
    let player_count = player_count_input().parse::<usize>().unwrap_or(3).max(3).min(10);
    
    // Initialize player names if needed - ensure this happens before rendering
    let mut current_names = player_names();
    if current_names.len() != player_count {
        let mut names = vec![String::new(); player_count];
        for i in 0..player_count.min(current_names.len()) {
            names[i] = current_names[i].clone();
        }
        current_names = names.clone();
        player_names.set(names);
    }

    rsx! {
        div { class: "setup-screen",
            div { class: "setup-header",
                h1 { "🎮 Agent-X" }
                p { class: "subtitle", "The Social Deduction Game" }
            }
            
            div { class: "player-count-section",
                label { 
                    "👥 Number of Players"
                    span { class: "hint", "(minimum 3)" }
                }
                input {
                    r#type: "number",
                    min: "3",
                    max: "10",
                    value: "{player_count_input}",
                    oninput: move |e| {
                        player_count_input.set(e.value());
                    }
                }
            }
            
            div { class: "player-names-section",
                h2 { "✏️ Player Names" }
                div { class: "player-inputs-grid",
                    for i in 0..player_count {
                        div { class: "player-input",
                            span { class: "player-number", "{i + 1}" }
                            input {
                                r#type: "text",
                                placeholder: "Enter name...",
                                value: "{current_names.get(i).cloned().unwrap_or_default()}",
                                oninput: move |e| {
                                    let mut names = player_names();
                                    // Ensure the vector is large enough
                                    while names.len() <= i {
                                        names.push(String::new());
                                    }
                                    names[i] = e.value();
                                    player_names.set(names);
                                }
                            }
                        }
                    }
                }
            }
            
            button {
                class: "start-game-btn",
                onclick: move |_| {
                    let names = player_names();
                    if names.iter().all(|n| !n.trim().is_empty()) {
                        let new_players: Vec<Player> = names.iter().map(|name| Player {
                            name: name.clone(),
                            score: 0,
                            is_eliminated: false,
                        }).collect();
                        players.set(new_players);
                        round_number.set(1);
                        game_screen.set(GameScreen::CardView { current_player_index: 0 });
                    }
                },
                "🚀 Start Game"
            }
        }
    }
}

/// Screen where players view their cards one by one
#[component]
fn CardViewScreen(
    current_player_index: usize,
    players: Signal<Vec<Player>>,
    mut cards: Signal<Vec<GameCard>>,
    mut imposter_index: Signal<usize>,
    mut game_screen: Signal<GameScreen>,
) -> Element {
    // Initialize cards for the round
    use_effect(move || {
        let player_count = players().len();
        if cards().is_empty() && player_count > 0 {
            let (new_cards, new_imposter) = generate_cards(player_count);
            cards.set(new_cards);
            imposter_index.set(new_imposter);
        }
    });

    let mut card_revealed = use_signal(|| false);
    let player_list = players();
    let cards_list = cards();
    
    if current_player_index >= player_list.len() {
        return rsx! {
            div { class: "transition-screen",
                h2 { "All players have seen their cards!" }
                button {
                    class: "proceed-btn",
                    onclick: move |_| {
                        game_screen.set(GameScreen::Voting);
                    },
                    "Proceed to Voting"
                }
            }
        };
    }
    
    // Check if cards are initialized
    if cards_list.is_empty() || current_player_index >= cards_list.len() {
        return rsx! {
            div { class: "loading-screen",
                p { "Preparing cards..." }
            }
        };
    }

    let current_player = &player_list[current_player_index];
    let current_card = &cards_list[current_player_index];

    rsx! {
        div { class: "card-view-screen",
            if !card_revealed() {
                div { class: "player-ready-screen",
                    h2 { "Pass device to:" }
                    h1 { class: "player-name", "{current_player.name}" }
                    p { class: "instruction", "⚠️ Make sure other players can't see the screen!" }
                        button {
                            class: "reveal-btn",
                            onclick: move |_| {
                                card_revealed.set(true);
                            },
                            "Reveal My Card"
                        }
                }
            } else {
                div { class: "card-revealed-screen",
                    h2 { "{current_player.name}'s Card" }
                    
                    div { 
                        class: if current_card.card_type == CardType::Imposter {
                            "game-card imposter-card"
                        } else {
                            "game-card normal-card"
                        },
                        div { class: "card-word",
                            "{current_card.word}"
                        }
                        div { class: "card-type-hint",
                            if current_card.card_type == CardType::Imposter {
                                "🎭 You are the IMPOSTER!"
                            } else {
                                "👥 You are a regular player"
                            }
                        }
                    }
                    
                    p { class: "card-instruction",
                        if current_card.card_type == CardType::Imposter {
                            "Try to blend in! Don't let others know you have the odd word."
                        } else {
                            "Find the player with the different word!"
                        }
                    }
                    
                    button {
                        class: "next-btn",
                        onclick: move |_| {
                            card_revealed.set(false);
                            game_screen.set(GameScreen::CardView {
                                current_player_index: current_player_index + 1
                            });
                        },
                        "Next Player"
                    }
                }
            }
        }
    }
}

/// Voting screen where all players collectively decide who to evict
#[component]
fn VotingScreen(
    mut players: Signal<Vec<Player>>,
    mut game_screen: Signal<GameScreen>,
    imposter_index: Signal<usize>,
) -> Element {
    let player_list = players();
    
    // Only show non-eliminated players
    let active_indices: Vec<usize> = player_list.iter()
        .enumerate()
        .filter(|(_, p)| !p.is_eliminated)
        .map(|(i, _)| i)
        .collect();
    
    // Pre-compute player data for the loop
    let player_data: Vec<(usize, String)> = active_indices.iter()
        .map(|&idx| (idx, player_list[idx].name.clone()))
        .collect();
    
    rsx! {
        div { class: "voting-screen",
            h1 { "🗳️ Group Decision Time!" }
            
            div { class: "voting-instructions",
                p { "Discuss among all players and decide who to evict." }
                p { class: "hint", "Tap on the player card you all agreed to evict." }
            }
            
            div { class: "players-voting-list",
                for &(player_idx, ref player_name) in player_data.iter() {
                    div { class: "player-voting-card",
                        div { class: "player-info",
                            h3 { "{player_name}" }
                        }
                        button {
                            class: "evict-btn",
                            onclick: move |_| {
                                let was_imposter = player_idx == imposter_index();
                                game_screen.set(GameScreen::Elimination { 
                                    eliminated_index: player_idx,
                                    was_imposter 
                                });
                            },
                            "Evict"
                        }
                    }
                }
            }
        }
    }
}

/// Screen showing elimination results
#[component]
fn EliminationScreen(
    mut players: Signal<Vec<Player>>,
    eliminated_index: usize,
    was_imposter: bool,
    mut game_screen: Signal<GameScreen>,
    mut round_number: Signal<i32>,
    mut cards: Signal<Vec<GameCard>>,
    imposter_index: Signal<usize>,
) -> Element {
    let player_list = players();
    let eliminated_player = &player_list[eliminated_index];
    let active_count = player_list.iter().filter(|p| !p.is_eliminated).count();
    
    rsx! {
        div { class: "elimination-screen",
            h1 { "🗳️ Player Eliminated" }
            
            div { class: "elimination-result",
                p { class: "eliminated-player",
                    "{eliminated_player.name} has been evicted!"
                }
                p { class: "players-remaining",
                    "{active_count - 1} players remaining"
                }
            }
            
            div { class: "action-buttons",
                button {
                    class: "continue-btn",
                    onclick: move |_| {
                        let mut updated_players = players();
                        // Eliminate the player
                        updated_players[eliminated_index].is_eliminated = true;
                        
                        // Check if imposter was eliminated
                        if was_imposter {
                            // Imposter found - civilians win!
                            // ALL civilians get points, even if they were eliminated before
                            for (i, player) in updated_players.iter_mut().enumerate() {
                                if i != imposter_index() {
                                    player.score += 10;
                                }
                            }
                            players.set(updated_players);
                            game_screen.set(GameScreen::RoundEnd { 
                                imposter_found: true,
                                game_over: true 
                            });
                        } else {
                            // Check if only 2 players remain
                            let remaining_count = updated_players.iter()
                                .filter(|p| !p.is_eliminated)
                                .count();
                            
                            players.set(updated_players);
                            
                            if remaining_count <= 2 {
                                // Imposter wins!
                                let mut final_players = players();
                                final_players[imposter_index()].score += 20;
                                players.set(final_players);
                                game_screen.set(GameScreen::RoundEnd { 
                                    imposter_found: false,
                                    game_over: true 
                                });
                            } else {
                                // Continue to next voting round
                                round_number.set(round_number() + 1);
                                game_screen.set(GameScreen::Voting);
                            }
                        }
                    },
                    "Continue"
                }
            }
        }
    }
}

/// Screen showing round results
#[component]
fn RoundEndScreen(
    mut players: Signal<Vec<Player>>,
    imposter_found: bool,
    game_over: bool,
    mut game_screen: Signal<GameScreen>,
    mut round_number: Signal<i32>,
    mut cards: Signal<Vec<GameCard>>,
    imposter_index: Signal<usize>,
) -> Element {
    let player_list = players();
    let imposter_name = &player_list[imposter_index()].name;

    rsx! {
        div { class: "round-end-screen",
            h1 {
                if imposter_found {
                    "✅ Civilians Win!"
                } else {
                    "😈 Imposter Wins!"
                }
            }
            
            div { class: "round-result",
                p { class: "imposter-reveal",
                    "The imposter was: {imposter_name}"
                }
                
                if imposter_found {
                    p { class: "result-message",
                        "🎉 All civilians get 10 points!"
                    }
                } else {
                    p { class: "result-message",
                        "😈 The imposter gets 20 points!"
                    }
                }
            }
            
            div { class: "action-buttons",
                button {
                    class: "view-scores-btn",
                    onclick: move |_| {
                        game_screen.set(GameScreen::GameScore);
                    },
                    "View Final Scores"
                }
                
                button {
                    class: "new-game-btn",
                    onclick: move |_| {
                        game_screen.set(GameScreen::Setup);
                    },
                    "New Game"
                }
            }
        }
    }
}

/// Screen showing all player scores
#[component]
fn GameScoreScreen(
    players: Signal<Vec<Player>>,
    round_number: Signal<i32>,
    mut game_screen: Signal<GameScreen>,
    mut cards: Signal<Vec<GameCard>>,
    imposter_index: Signal<usize>,
) -> Element {
    let mut sorted_players = players();
    sorted_players.sort_by(|a, b| b.score.cmp(&a.score));

    rsx! {
        div { class: "score-screen",
            h1 { "🏆 Scoreboard" }
            p { class: "round-info", "After Round {round_number()}" }
            
            div { class: "scoreboard",
                for (rank, player) in sorted_players.iter().enumerate() {
                    div { 
                        class: if rank == 0 { "score-card winner" } else { "score-card" },
                        div { class: "rank", "#{rank + 1}" }
                        div { class: "player-score-info",
                            h3 { "{player.name}" }
                            p { class: "score", "{player.score} points" }
                        }
                        if rank == 0 {
                            span { class: "winner-badge", "👑" }
                        }
                    }
                }
            }
            
            div { class: "action-buttons",
                button {
                    class: "next-round-btn",
                    onclick: move |_| {
                        // Reset all player states for new round
                        let mut updated_players = players();
                        for player in updated_players.iter_mut() {
                            player.is_eliminated = false; // Reset eliminations for new round
                        }
                        players.set(updated_players);
                        cards.set(Vec::new());
                        round_number.set(round_number() + 1);
                        game_screen.set(GameScreen::CardView { current_player_index: 0 });
                    },
                    "Play Next Round"
                }
                
                button {
                    class: "new-game-btn",
                    onclick: move |_| {
                        game_screen.set(GameScreen::Setup);
                    },
                    "New Game"
                }
            }
        }
    }
}

/// Helper function to generate cards for the round
fn generate_cards(player_count: usize) -> (Vec<GameCard>, usize) {
    use getrandom::getrandom;
    
    // Extended word pairs (civilian word, imposter word)
    // These should be similar but different enough to create interesting discussions
    let word_pairs = vec![
        ("Coffee", "Tea"),
        ("Cat", "Dog"),
        ("Sun", "Moon"),
        ("Ocean", "Sea"),
        ("Mountain", "Hill"),
        ("River", "Stream"),
        ("Book", "Magazine"),
        ("Car", "Truck"),
        ("Pizza", "Burger"),
        ("Apple", "Orange"),
        ("Winter", "Autumn"),
        ("Guitar", "Piano"),
        ("Soccer", "Basketball"),
        ("Movie", "TV Show"),
        ("Rain", "Snow"),
        ("Lion", "Tiger"),
        ("Hotel", "Motel"),
        ("Ship", "Boat"),
        ("Forest", "Jungle"),
        ("Lake", "Pond"),
        ("Bread", "Toast"),
        ("Juice", "Smoothie"),
        ("Doctor", "Nurse"),
        ("Teacher", "Professor"),
        ("Phone", "Tablet"),
        ("Laptop", "Desktop"),
        ("Watch", "Clock"),
        ("Shirt", "Blouse"),
        ("Shoes", "Boots"),
        ("Hat", "Cap"),
        ("Painting", "Drawing"),
        ("Park", "Garden"),
        ("Airport", "Station"),
        ("Restaurant", "Cafe"),
        ("Mall", "Market"),
        ("Beach", "Shore"),
        ("Valley", "Canyon"),
        ("Cloud", "Mist"),
        ("Thunder", "Lightning"),
        ("Sunrise", "Sunset"),
        ("Spring", "Summer"),
        ("Breakfast", "Brunch"),
        ("Dinner", "Supper"),
        ("Pen", "Pencil"),
        ("Paper", "Notebook"),
        ("Email", "Letter"),
        ("Photo", "Picture"),
        ("Song", "Music"),
        ("Dance", "Ballet"),
        ("Running", "Jogging"),
        ("Swimming", "Diving"),
        ("Bicycle", "Motorcycle"),
        ("Bus", "Train"),
        ("Plane", "Helicopter"),
        ("Rocket", "Spaceship"),
        ("Castle", "Palace"),
        ("Tower", "Building"),
        ("Bridge", "Tunnel"),
        ("Road", "Highway"),
        ("City", "Town"),
        ("Village", "Hamlet"),
        ("King", "Emperor"),
        ("Queen", "Princess"),
        ("Knight", "Warrior"),
        ("Wizard", "Sorcerer"),
        ("Dragon", "Dinosaur"),
        ("Eagle", "Hawk"),
        ("Whale", "Dolphin"),
        ("Shark", "Fish"),
        ("Snake", "Lizard"),
        ("Spider", "Insect"),
        ("Rose", "Tulip"),
        ("Tree", "Plant"),
        ("Grass", "Weed"),
        ("Diamond", "Crystal"),
        ("Gold", "Silver"),
        ("Ring", "Bracelet"),
        ("Necklace", "Chain"),
        ("Candle", "Lamp"),
        ("Fire", "Flame"),
        ("Ice", "Snow"),
        ("Desert", "Wasteland"),
        ("Island", "Peninsula"),
        ("Volcano", "Mountain"),
        ("Cave", "Cavern"),
        ("Treasure", "Jewel"),
        ("Pirate", "Sailor"),
        ("Hero", "Champion"),
        ("Villain", "Criminal"),
        ("Mystery", "Secret"),
        ("Adventure", "Journey"),
        ("Story", "Tale"),
        ("Legend", "Myth"),
        ("Ghost", "Spirit"),
        ("Angel", "Fairy"),
        ("Monster", "Creature"),
        ("Robot", "Android"),
        ("Alien", "Extraterrestrial"),
        ("Planet", "Star"),
        ("Galaxy", "Universe"),
        ("Comet", "Meteor"),
    ];
    
    // Get random bytes for word pair selection
    let mut buf_word = [0u8; 8];
    let _ = getrandom(&mut buf_word);
    let random_word = u64::from_le_bytes(buf_word);
    
    // Get SEPARATE random bytes for imposter selection (ensures true randomness)
    let mut buf_imposter = [0u8; 8];
    let _ = getrandom(&mut buf_imposter);
    let random_imposter = u64::from_le_bytes(buf_imposter);
    
    // Select random word pair
    let pair_index = (random_word as usize) % word_pairs.len();
    let (normal_word, imposter_word) = word_pairs[pair_index];
    
    // Select random imposter index (using separate random value)
    let imposter_idx = (random_imposter as usize) % player_count;
    
    let mut cards = Vec::new();
    for i in 0..player_count {
        if i == imposter_idx {
            cards.push(GameCard {
                card_type: CardType::Imposter,
                word: imposter_word.to_string(),
            });
        } else {
            cards.push(GameCard {
                card_type: CardType::Normal,
                word: normal_word.to_string(),
            });
        }
    }
    
    (cards, imposter_idx)
}

