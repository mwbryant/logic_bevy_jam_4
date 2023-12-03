// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_decks))
        .add_systems(Update, (simulate_games))
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
}

pub enum GamePhase {
    Play,
    Attack,
    Halt,
}

#[derive(Component)]
pub struct Game {
    player: Entity,
    enemy: Entity,
    turn: GamePhase,
    side: Side,
}

fn simulate_games(
    mut commands: Commands,
    mut games: Query<&mut Game>,
    mut players: Query<(&mut Deck, &mut PlayArea)>,
    mut cards: Query<(&mut Card)>,
) {
    for mut game in &mut games {
        let to_play = match game.side {
            Side::Player => game.player,
            Side::Enemy => game.enemy,
        };
        let to_hit = match game.side {
            Side::Player => game.enemy,
            Side::Enemy => game.player,
        };

        match game.turn {
            GamePhase::Play => {
                let (mut deck, mut play_area) = players.get_mut(to_play).unwrap();
                let card = deck.cards.pop();
                info!("Draw! {:?}", card);

                if let Some(card) = card {
                    play_area.cards[0] = Some(commands.spawn(card).id());
                } else {
                    info!("NO card :(");
                }
                game.turn = GamePhase::Attack;
            }
            GamePhase::Attack => {
                let [(_, play_area), (mut deck, mut defend_area)] =
                    players.get_many_mut([to_play, to_hit]).unwrap();

                for slot in 0..2 {
                    if let Some(card) = play_area.cards[slot] {
                        let card = cards.get(card).unwrap();
                        let attack = card.damage;
                        if let Some(defender) = defend_area.cards[slot] {
                            let mut card = cards.get_mut(defender).unwrap();
                            card.health -= attack;
                            if card.health >= 0 {
                                info!("Blocked but took {} damage", attack);
                            } else {
                                info!("Destroyed blocker");
                                commands.entity(defender).despawn_recursive();
                                defend_area.cards[slot] = None;
                            }
                        } else {
                            info!("Attacking Directly: {}!", attack);
                            deck.health -= attack;
                            if deck.health <= 0 {
                                info!("Winner: {:?}", game.side);
                                game.turn = GamePhase::Halt;
                                return;
                            }
                        }
                    }
                }

                game.turn = GamePhase::Play;
                game.side = match game.side {
                    Side::Player => Side::Enemy,
                    Side::Enemy => Side::Player,
                };
            }
            GamePhase::Halt => {}
        }
    }
}

fn spawn_decks(mut commands: Commands) {
    let player = commands
        .spawn((dummy_deck(), Side::Player, PlayArea::default()))
        .id();
    let enemy = commands
        .spawn((dummy_deck(), Side::Enemy, PlayArea::default()))
        .id();
    commands.spawn(
        (Game {
            player,
            enemy,
            turn: GamePhase::Play,
            side: Side::Player,
        }),
    );
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("icon.png"),
        ..Default::default()
    });
}
