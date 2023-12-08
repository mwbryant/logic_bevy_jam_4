#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub const SQRT_NUMBER_OF_GAMES: i32 = 10;
pub const NUMBER_OF_GAMES: i32 = SQRT_NUMBER_OF_GAMES * SQRT_NUMBER_OF_GAMES;
pub const BOARD_SIZE: f32 = 30.0;
pub const BOARD_PADDING: f32 = 5.0;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_turborand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Sick game".to_string(),
                    // present_mode: bevy::window::PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            // Adds frame time diagnostics
            FrameTimeDiagnosticsPlugin,
            // Adds a system that prints diagnostics to the console
            LogDiagnosticsPlugin::default(),
            // Any plugin can register diagnostics. Uncomment this to add an entity count diagnostics:
            // bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
            // Uncomment this to add an asset count diagnostics:
            // bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<Texture>::default(),
            // Uncomment this to add system info diagnostics:
            // bevy::diagnostic::SystemInformationDiagnosticsPlugin::default()
        ))
        .add_plugins(RngPlugin::default())
        .add_systems(Startup, (setup, spawn_decks))
        .add_systems(Update, (simulate_games, place_games, print_win_rates))
        .run();
}

#[derive(Component)]
pub struct Deck {
    cards: Vec<Card>,
    health: i32,
}

#[derive(Component, Default, Debug)]
pub struct PlayArea {
    cards: [Option<Entity>; 3],
}

impl PlayArea {
    fn get_random_open_slot(&self, rng: &mut RngComponent) -> Option<usize> {
        let mut slots = vec![];
        for slot in 0..2 {
            if self.cards[slot].is_none() {
                slots.push(slot);
            }
        }
        rng.shuffle(&mut slots);
        slots.first().cloned()
    }
}

#[derive(Component, Debug)]
pub struct Card {
    damage: i32,
    health: i32,
}

fn dummy_deck() -> Deck {
    Deck {
        cards: vec![
            Card {
                damage: 3,
                health: 1,
            },
            Card {
                damage: 1,
                health: 1,
            },
            Card {
                damage: 0,
                health: 5,
            },
            Card {
                damage: 2,
                health: 1,
            },
        ],
        health: 5,
    }
}

#[derive(Component, Debug)]
pub enum Side {
    Player,
    Enemy,
    Draw,
}

#[derive(PartialEq)]
pub enum GamePhase {
    Play,
    Attack,
    Halt,
}

#[derive(Component)]
pub struct Game {
    id: i32,
    player: Entity,
    enemy: Entity,
    turn: GamePhase,
    side: Side,
    turn_count: usize,
}

fn print_win_rates(games: Query<&Game>) {
    for game in &games {
        if game.turn != GamePhase::Halt {
            // Not all games have halted
            return;
        }
    }
    let mut counts = [0, 0, 0];
    for game in &games {
        match game.side {
            Side::Player => counts[0] += 1,
            Side::Enemy => counts[1] += 1,
            Side::Draw => counts[2] += 1,
        }
    }
    info!(
        "Results: {} player wins, {} enemy wins, {} draws",
        counts[0], counts[1], counts[2]
    );
}

fn simulate_games(
    mut commands: Commands,
    mut games: Query<&mut Game>,
    mut players: Query<(&mut Deck, &mut PlayArea, &mut RngComponent)>,
    mut cards: Query<&mut Card>,
) {
    for mut game in &mut games {
        let to_play = match game.side {
            Side::Player => game.player,
            Side::Enemy => game.enemy,
            Side::Draw => {
                info!("draw");
                continue;
            }
        };
        let to_hit = match game.side {
            Side::Player => game.enemy,
            Side::Enemy => game.player,
            Side::Draw => {
                unreachable!()
            }
        };

        match game.turn {
            GamePhase::Play => {
                let (mut deck, mut play_area, mut rng) = players.get_mut(to_play).unwrap();
                rng.shuffle(&mut deck.cards);
                let card = deck.cards.pop();
                // info!("Draw! {:?}", card);

                if let Some(card) = card {
                    let slot = play_area.get_random_open_slot(&mut rng);
                    if let Some(slot) = slot {
                        // info!("Played at {}", slot);
                        play_area.cards[slot] = Some(commands.spawn(card).id());
                    } else {
                        // info!("Can't play");
                    }
                } else {
                    // info!("NO card :(");
                }
                game.turn = GamePhase::Attack;
            }
            GamePhase::Attack => {
                let [(_, play_area, _), (mut deck, mut defend_area, _)] =
                    players.get_many_mut([to_play, to_hit]).unwrap();

                for slot in 0..2 {
                    if let Some(card) = play_area.cards[slot] {
                        let card = cards.get(card).unwrap();
                        let attack = card.damage;
                        if let Some(defender) = defend_area.cards[slot] {
                            let mut card = cards.get_mut(defender).unwrap();
                            card.health -= attack;
                            if card.health >= 0 {
                                // info!("Blocked but took {} damage", attack);
                            } else {
                                // info!("Destroyed blocker");
                                commands.entity(defender).despawn_recursive();
                                defend_area.cards[slot] = None;
                            }
                        } else {
                            // info!("Attacking Directly: {}!", attack);
                            deck.health -= attack;
                            if deck.health <= 0 {
                                // info!("Winner: {:?}", game.side);
                                game.turn = GamePhase::Halt;
                                continue;
                            }
                        }
                    }
                }
                if game.turn == GamePhase::Halt {
                    continue;
                }
                game.turn_count += 1;
                if game.turn_count > 500 {
                    info!("draw");
                    game.turn = GamePhase::Halt;
                    game.side = Side::Draw;
                    continue;
                }
                game.turn = GamePhase::Play;
                game.side = match game.side {
                    Side::Player => Side::Enemy,
                    Side::Enemy => Side::Player,
                    Side::Draw => unreachable!(),
                };
            }
            GamePhase::Halt => {}
        }
    }
}

fn place_games(mut games: Query<(&mut Transform, &Game)>) {
    for (mut transform, game) in &mut games {
        let x = game.id % SQRT_NUMBER_OF_GAMES;
        let y = game.id / SQRT_NUMBER_OF_GAMES;

        transform.translation = Vec3::new(
            x as f32 * (BOARD_SIZE + BOARD_PADDING),
            y as f32 * (BOARD_SIZE + BOARD_PADDING),
            0.0,
        );
    }
}

fn spawn_decks(
    mut commands: Commands,
    mut global_rng: ResMut<GlobalRng>,
    asset_server: Res<AssetServer>,
) {
    for id in 0..NUMBER_OF_GAMES {
        let player = commands
            .spawn((
                dummy_deck(),
                Side::Player,
                PlayArea::default(),
                RngComponent::from(&mut global_rng),
            ))
            .id();
        let enemy = commands
            .spawn((
                dummy_deck(),
                Side::Enemy,
                PlayArea::default(),
                RngComponent::from(&mut global_rng),
            ))
            .id();
        commands
            .spawn((
                Game {
                    id,
                    player,
                    enemy,
                    turn: GamePhase::Play,
                    side: Side::Player,
                    turn_count: 0,
                },
                SpriteBundle {
                    texture: asset_server.load("icon.png"),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(BOARD_SIZE)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ))
            .push_children(&[player, enemy]);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
