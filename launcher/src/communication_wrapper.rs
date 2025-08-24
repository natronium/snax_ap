use archipelago_rs::{
    client::{ArchipelagoClient, ArchipelagoError},
    protocol::{ClientMessage, LocationInfo, LocationScouts, ReceivedItems, ServerMessage},
};
use std::collections::HashMap;
use tokio::{
    runtime::Builder,
    select,
    sync::mpsc::{Receiver, Sender, channel},
};

pub(crate) struct ConnectionInfo {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) slot_name: String,
    pub(crate) password: Option<String>,
}

pub(crate) struct CommunicationWrapper {
    pub(crate) channel_out: Sender<APMessage>,
    pub(crate) channel_in: Receiver<SnaxMessage>,
}

impl CommunicationWrapper {
    pub(crate) fn start(connection_info: ConnectionInfo) -> Self {
        let (send_by_ap, recv_by_snax) = channel::<SnaxMessage>(100);
        let (send_by_snax, mut recv_by_ap) = channel::<APMessage>(100);

        std::thread::Builder::new().name("AP Communication Thread".to_string()).spawn(move || {
            let runtime = Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Could not build tokio runtime");

            let mut client = runtime
                .block_on(ArchipelagoClient::new(&format!(
                    "{}:{}",
                    connection_info.host, connection_info.port
                )))
                .expect("could not connect to archipelago");
            let result = runtime
                .block_on(client.connect(
                    "Bugsnax",
                    &connection_info.slot_name,
                    connection_info.password.as_deref(),
                    Some(0b111),
                    vec!["AP".to_string()],
                ))
                .expect("Could not connect to slot Bugsnax:Player1");

            if let Some(msg) =
                CommunicationWrapper::handle_message_from_ap(Ok(ServerMessage::Connected(result)))
            {
                send_by_ap
                    .blocking_send(msg)
                    .expect("Could not send {msg:?} on send_by_ap channel");
            }

            runtime.block_on(async move {
                loop {
                    select! {
                        result = client.recv() => {
                            if let Some(result) = result.transpose() && let Some(msg) = CommunicationWrapper::handle_message_from_ap(result){
                                send_by_ap.send(msg).await.expect("Could not send {msg:?} on send_by_ap channel");
                            }
                        }
                        result = recv_by_ap.recv() => {
                            CommunicationWrapper::handle_message_from_snax(&mut client, result).await;
                        }
                    }
                }
            });
        }).expect("Could not start AP Communication thread!");

        CommunicationWrapper {
            channel_in: recv_by_snax,
            channel_out: send_by_snax,
        }
    }

    pub(crate) fn handle_message_from_ap(
        msg: Result<ServerMessage, ArchipelagoError>,
    ) -> Option<SnaxMessage> {
        let message_from_ap = msg.expect("Archipelago error (TODO: Handle ArchipelagoError)");
        match message_from_ap {
            ServerMessage::Connected(connected) => Some(SnaxMessage::Connected {}),
            ServerMessage::ConnectionRefused(connection_refused) => {
                Some(SnaxMessage::Disconnected {
                    error: format!("Connection Refused: {connection_refused:?}").to_string(),
                })
            }
            ServerMessage::LocationInfo(location_info) => Some(SnaxMessage::LocationScouts {
                location_to_items: CommunicationWrapper::convert_location_info_to_dict(
                    &location_info,
                ),
            }),
            ServerMessage::ReceivedItems(received_items) => Some(SnaxMessage::ItemsReceived {
                items_received: CommunicationWrapper::convert_received_items_to_dict(
                    &received_items,
                ),
                index: received_items.index,
            }),

            ServerMessage::Print(_)
            | ServerMessage::PrintJSON(_)
            | ServerMessage::Bounced(_)
            | ServerMessage::DataPackage(_)
            | ServerMessage::RoomInfo(_)
            | ServerMessage::RoomUpdate(_)
            | ServerMessage::SetReply(_)
            | ServerMessage::Retrieved(_)
            | ServerMessage::InvalidPacket(_) => None,
        }
    }

    pub(crate) async fn handle_message_from_snax(
        client: &mut ArchipelagoClient,
        result: Option<APMessage>,
    ) {
        if let Some(result) = result {
            match result {
                APMessage::LocationsToCheck {
                    location_ids: locations_id,
                } => client.location_checks(locations_id).await,
                APMessage::GoalCompletion {} => {
                    client
                        .status_update(archipelago_rs::protocol::ClientStatus::ClientGoal)
                        .await
                }
                APMessage::LocationsToScout { location_ids } => {
                    client
                        .send(ClientMessage::LocationScouts(LocationScouts {
                            locations: location_ids,
                            create_as_hint: 0,
                        }))
                        .await
                }
            }
            .expect("TODO: handle ArchipelagoErrors");
        }
    }

    pub(crate) fn convert_location_info_to_dict(info: &LocationInfo) -> HashMap<i64, String> {
        let mut location_scouts = HashMap::new();
        for network_item in info.locations.iter() {
            location_scouts.insert(
                network_item.location,
                format!(
                    "{}'s {}",
                    network_item.player.to_string(),
                    network_item.item.to_string()
                ),
            );
        }
        location_scouts
    }

    pub(crate) fn convert_received_items_to_dict(
        received_items: &ReceivedItems,
    ) -> HashMap<i64, usize> {
        let mut items_received = HashMap::new();
        for item in received_items.items.iter() {
            *items_received.entry(item.item).or_default() += 1;
        }
        items_received
    }
}

/// Messages sent to archipelago by the player doing things in Bugsnax
pub enum APMessage {
    LocationsToScout { location_ids: Vec<i64> },
    LocationsToCheck { location_ids: Vec<i64> },
    GoalCompletion {},
}

/// Messages sent to Bugsnax by archipelago
#[derive(Debug)]
pub enum SnaxMessage {
    Connected {},
    Disconnected {
        error: String,
    },
    LocationScouts {
        location_to_items: HashMap<i64, String>,
    },
    ItemsReceived {
        items_received: HashMap<i64, usize>,
        index: i32,
    },
}
