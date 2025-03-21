use common::{ActiveItem, ActiveItemKind, ClientId, PickupKind, ServerMessage, map::Map};
use std::collections::HashMap;

use crate::client::Client;
use crate::server::client_handler::{ClientManagerHandle, SendTo};

const SHELL_SPEED: f32 = 0.2;

#[derive(Debug, Default)]
pub struct GameState {
    map: Map,

    active_items: Vec<ActiveItem>,
    coin_states: Vec<bool>,
    item_box_states: Vec<bool>,
}

impl GameState {
    pub fn active_items(&self) -> &[ActiveItem] {
        &self.active_items
    }

    pub fn from_map(map: Map) -> Self {
        let coin_states = vec![true; map.coins.len()];
        let item_box_states = vec![true; map.item_spawns.len()];
        Self {
            map,

            active_items: Vec::new(),
            coin_states,
            item_box_states,
        }
    }

    pub fn add_item(
        &mut self,
        mut item: ActiveItem,
        client: &Client,
        clients: &HashMap<ClientId, Client>,
    ) {
        match &mut item.kind {
            ActiveItemKind::RedShell { target, .. } => {
                let start_pos = client.state.track_pos;
                struct Nearest {
                    id: ClientId,
                    distance_segments: usize,
                    distance_progress: f32,
                }
                let mut nearest_found: Option<Nearest> = None;

                for (id, other) in clients {
                    if *id == client.id() {
                        continue;
                    }

                    let mut target_segment = other.state.track_pos.segment;
                    if target_segment < start_pos.segment
                        || start_pos.progress > other.state.track_pos.progress
                    {
                        target_segment += self.map.track.path.len()
                    }
                    let segment_diff = target_segment - start_pos.segment;
                    let progress_diff = other.state.track_pos.progress - start_pos.progress;

                    let candidate = Nearest {
                        id: *id,
                        distance_segments: segment_diff,
                        distance_progress: progress_diff,
                    };

                    if let Some(ref nearest) = nearest_found {
                        if candidate.distance_segments < nearest.distance_segments
                            || (candidate.distance_segments == nearest.distance_segments
                                && candidate.distance_progress < nearest.distance_progress)
                        {
                            nearest_found = Some(candidate);
                        }
                    } else {
                        nearest_found = Some(candidate);
                    }
                }

                *target = nearest_found.map(|n| n.id).unwrap_or(ClientId::invalid());
            }

            ActiveItemKind::GreenShell { .. } => {}
            ActiveItemKind::Banana => {}
        }

        self.active_items.push(item);
    }

    // returns true if the pickup was picked up
    pub fn pickup(&mut self, kind: PickupKind, index: usize) -> bool {
        let array = match kind {
            PickupKind::Coin => &mut self.coin_states,
            PickupKind::ItemBox => &mut self.item_box_states,
        };

        let mut success = false;
        if index < array.len() {
            success = array[index];
            array[index] = false;
        }
        success
    }

    pub fn respawn_pickup(&mut self, kind: PickupKind, index: usize) {
        let array = match kind {
            PickupKind::Coin => &mut self.coin_states,
            PickupKind::ItemBox => &mut self.item_box_states,
        };

        if index < array.len() {
            array[index] = true;
        }
    }

    pub async fn tick(
        &mut self,
        players: &mut HashMap<ClientId, Client>,
        client_handler: ClientManagerHandle,
    ) {
        async fn notify_hit(client_handler: &ClientManagerHandle, player: ClientId) {
            client_handler
                .send(SendTo::InGameAll, ServerMessage::HitByItem { player })
                .await;
        }

        for i in (0..self.active_items.len()).rev() {
            let mut remove = false;

            let item = &mut self.active_items[i];
            match &mut item.kind {
                ActiveItemKind::GreenShell { direction } => {
                    item.pos += *direction * SHELL_SPEED;
                }

                ActiveItemKind::RedShell { target, velocity } => {
                    if *target == ClientId::invalid() {
                        item.pos += *velocity;
                    } else if let Some(target) = players.get(&target) {
                        // TODO: follow track until we get close enough

                        let target_pos = target.state.pos;
                        let direction = target_pos - item.pos;
                        let target_velocity = direction.normalize() * SHELL_SPEED;
                        let acceleration = (target_velocity - *velocity) * 0.1;
                        *velocity += acceleration;

                        item.pos += *velocity;
                    } else {
                        remove = true;
                    }
                }

                ActiveItemKind::Banana => {}
            }

            match item.kind {
                ActiveItemKind::Banana | ActiveItemKind::GreenShell { .. } => {
                    for player in players.values_mut() {
                        if (player.state.pos - item.pos).length() < 0.5 {
                            notify_hit(&client_handler, player.id()).await;
                            remove = true;
                        }
                    }
                }

                ActiveItemKind::RedShell { target, .. } => {
                    if let Some(target) = players.get(&target) {
                        if (target.state.pos - item.pos).length() < 0.1 {
                            notify_hit(&client_handler, target.id()).await;
                            remove = true;
                        }
                    }
                }
            }

            if remove {
                self.active_items.swap_remove(i);
            }
        }
    }
}
