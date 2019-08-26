#[cfg(feature = "client")]
use amethyst::{
    ecs::ReadExpect,
    prelude::{SimpleTrans, StateEvent, Trans},
};
use amethyst::{
    ecs::{World, WriteExpect, WriteStorage},
    prelude::{GameData, SimpleState, StateData},
    shred::SystemData,
};

#[cfg(feature = "client")]
use ha_client_shared::ecs::factories::PlayerClientFactory;
#[cfg(feature = "client")]
use ha_client_shared::{
    ecs::{factories::CameraFactory, resources::MultiplayerRoomState},
    utils::{self, animation},
};
#[cfg(not(feature = "client"))]
use ha_core::net::server_message::ServerMessagePayload;
#[cfg(not(feature = "client"))]
use ha_core::net::NetConnection;
use ha_core::{
    actions::monster_spawn::{Count, SpawnAction, SpawnActions, SpawnType},
    ecs::{
        components::EntityNetMetadata,
        resources::{
            net::{EntityNetMetadataService, MultiplayerGameState},
            GameEngineState, GameLevelState,
        },
        system_data::time::GameTimeService,
    },
};

use crate::ecs::factories::{LandscapeFactory, PlayerFactory};
#[cfg(not(feature = "client"))]
use crate::utils::net::broadcast_message_reliable;

#[derive(Default)]
pub struct PlayingState;

impl SimpleState for PlayingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        log::info!("PlayingState started");
        let world = data.world;
        *world.write_resource::<GameEngineState>() = GameEngineState::Playing;

        world.add_resource(SpawnActions(Vec::new()));
        world.add_resource(GameLevelState::default());

        GameTimeService::fetch(&world.res).set_level_started_at();

        initialize_players(world);

        {
            let mut spawn_actions = world.write_resource::<SpawnActions>();
            spawn_actions.0.append(&mut vec![
                SpawnAction {
                    monsters: Count {
                        entity: "Ghoul".to_owned(),
                        num: 1,
                    },
                    spawn_type: SpawnType::Borderline,
                },
                SpawnAction {
                    monsters: Count {
                        entity: "Ghoul".to_owned(),
                        num: 5,
                    },
                    spawn_type: SpawnType::Random,
                },
            ]);
        }

        world.exec(|mut landscape_factory: LandscapeFactory| landscape_factory.create());
    }

    #[cfg(feature = "client")]
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let world = data.world;
        utils::handle_window_event(&world, &event);
        Trans::None
    }

    #[cfg(feature = "client")]
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        animation::start_hero_animations(data.world);
        Trans::None
    }
}

#[cfg(feature = "client")]
fn initialize_players(world: &mut World) {
    let mut main_player = None;

    world.exec(
        |(
            mut player_factory,
            mut player_client_factory,
            mut entity_net_metadata,
            mut entity_net_metadata_service,
            mut multiplayer_room_state,
            multiplayer_game_state,
        ): (
            PlayerFactory,
            PlayerClientFactory,
            WriteStorage<EntityNetMetadata>,
            WriteExpect<EntityNetMetadataService>,
            WriteExpect<MultiplayerRoomState>,
            ReadExpect<MultiplayerGameState>,
        )| {
            if !multiplayer_game_state.is_playing {
                let player_entity = player_factory.create();
                player_client_factory.create(player_entity, true);
                main_player = Some(player_entity);
            }

            for player in &multiplayer_game_state.players {
                let player_entity = player_factory.create();
                entity_net_metadata_service.set_net_id(player_entity, player.entity_net_id);
                entity_net_metadata
                    .insert(
                        player_entity,
                        EntityNetMetadata {
                            id: player.entity_net_id,
                            spawned_frame_number: 0,
                        },
                    )
                    .expect("Expected to insert EntityNetMetadata component");

                if player.connection_id == multiplayer_room_state.connection_id {
                    player_client_factory.create(player_entity, true);
                    multiplayer_room_state.player_net_id = player.entity_net_id;
                    main_player = Some(player_entity);
                } else {
                    player_client_factory.create(player_entity, false);
                }
            }
        },
    );

    let main_player = main_player.expect("Expected an initialized main player");
    world.exec(move |mut camera_factory: CameraFactory| {
        camera_factory.create(main_player);
    });
}

#[cfg(not(feature = "client"))]
fn initialize_players(world: &mut World) {
    world.exec(
        |(
            mut player_factory,
            mut entity_net_metadata,
            mut entity_net_metadata_service,
            mut multiplayer_game_state,
            mut net_connections,
        ): (
            PlayerFactory,
            WriteStorage<EntityNetMetadata>,
            WriteExpect<EntityNetMetadataService>,
            WriteExpect<MultiplayerGameState>,
            WriteStorage<NetConnection>,
        )| {
            let player_net_identifiers = multiplayer_game_state
                .players
                .iter_mut()
                .map(|player| {
                    let player_entity = player_factory.create();
                    let entity_net_id =
                        entity_net_metadata_service.register_new_entity(player_entity);
                    player.entity_net_id = entity_net_id;
                    entity_net_metadata
                        .insert(
                            player_entity,
                            EntityNetMetadata {
                                id: entity_net_id,
                                spawned_frame_number: 0,
                            },
                        )
                        .expect("Expected to insert EntityNetMetadata component");
                    entity_net_id
                })
                .collect();
            broadcast_message_reliable(
                &mut net_connections,
                &ServerMessagePayload::StartGame(player_net_identifiers),
            );
        },
    );
}
