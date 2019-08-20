pub mod damage_history;
pub mod missile;

use amethyst::{
    ecs::{Component, DenseVecStorage, FlaggedStorage, NullStorage, ReaderId, VecStorage},
    network::NetEvent,
};
use shrinkwraprs::Shrinkwrap;

use std::time::{Duration, Instant};

use crate::{
    actions::{
        mob::MobAction,
        player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
    },
    math::{Vector2, ZeroVector},
    net::{ConnectionIdentifier, EncodedMessage},
};
use crate::net::EntityNetIdentifier;

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct WorldPosition {
    #[shrinkwrap(main_field)]
    pub position: Vector2,
}

impl WorldPosition {
    pub fn new(position: Vector2) -> Self {
        Self { position }
    }
}

impl Component for WorldPosition {
    type Storage = VecStorage<Self>;
}

pub struct Player {
    pub health: f32,
    pub velocity: Vector2,
    pub walking_direction: Vector2,
    pub looking_direction: Vector2,
    pub radius: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            health: 100.0,
            velocity: Vector2::zero(),
            walking_direction: Vector2::new(0.0, 1.0),
            looking_direction: Vector2::new(0.0, 1.0),
            radius: 20.0,
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct PlayerActions {
    pub walk_actions: Vec<PlayerWalkAction>,
    pub look_actions: Vec<PlayerLookAction>,
    pub cast_actions: Vec<PlayerCastAction>,
    pub last_spell_cast: Duration,
}

impl Component for PlayerActions {
    type Storage = DenseVecStorage<Self>;
}

pub struct Monster {
    pub health: f32,
    pub attack_damage: f32,
    pub destination: Vector2,
    pub velocity: Vector2,
    pub action: MobAction,
    pub name: String,
    pub radius: f32,
}

impl Component for Monster {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct Dead;

impl Component for Dead {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}

pub struct NetConnectionModel {
    pub id: ConnectionIdentifier,
    pub reader: ReaderId<NetEvent<EncodedMessage>>,
    pub created_at: Instant,
    pub last_pinged_at: Instant,
}

impl NetConnectionModel {
    pub fn new(id: ConnectionIdentifier, reader: ReaderId<NetEvent<EncodedMessage>>) -> Self {
        Self {
            id,
            reader,
            created_at: Instant::now(),
            last_pinged_at: Instant::now(),
        }
    }
}

impl Component for NetConnectionModel {
    type Storage = DenseVecStorage<Self>;
}

pub struct EntityNetMetadata {
    pub id: EntityNetIdentifier,
}

impl Component for EntityNetMetadata {
    type Storage = VecStorage<Self>;
}
