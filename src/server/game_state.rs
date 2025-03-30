use common::{
    ActiveItemKind, ClientId, MAP_SCALE, PickupKind, ServerMessage,
    map::{Map, TrackPosition},
    map_coord_to_world,
    types::*,
    world_coord_to_map,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use parry2d::shape::Polyline;

use crate::client::Client;
use crate::server::client_handler::{ClientManagerHandle, SendTo};

const SHELL_SPEED: f32 = 0.45;

#[derive(Debug, Default)]
pub struct GameState {
    map: Map,
    colliders: Vec<Polyline>,

    active_items: Vec<ActiveItem>,
    coin_states: Vec<bool>,
    item_box_states: Vec<bool>,
}

#[derive(Debug)]
struct ActiveItem {
    pos: Vec2,
    rot: f32,
    owner: ClientId,
    spawn_time: Instant,
    state: ActiveItemState,
}

#[derive(Debug)]
enum ActiveItemState {
    GreenShell { roll: f32, bounces: u8 },
    RedShell { target: RedShellTarget, roll: f32 },
    Banana,
}

#[derive(Debug)]
enum RedShellTarget {
    None,
    Player {
        target_id: ClientId,
        track_pos: TrackPosition,
        on_track: bool,
    },
}

impl ActiveItem {
    fn new(
        owner: &Client,
        map: &Map,
        clients: &HashMap<ClientId, Client>,
        kind: ActiveItemKind,
    ) -> Self {
        let state = match kind {
            ActiveItemKind::Banana => ActiveItemState::Banana,

            ActiveItemKind::GreenShell { roll } => ActiveItemState::GreenShell { roll, bounces: 4 },

            ActiveItemKind::RedShell { roll } => {
                let mut start_pos = owner.state.track_pos;
                map.track.advance_position(50.0, &mut start_pos);

                struct Nearest {
                    id: ClientId,
                    distance_segments: usize,
                    distance_progress: f32,
                    track_pos: TrackPosition,
                    full_lap: bool,
                }
                let mut nearest_found: Option<Nearest> = None;

                for (id, other) in clients {
                    if *id == owner.id() {
                        continue;
                    }

                    let mut target_segment = other.state.track_pos.segment;
                    let mut full_lap = false;
                    if target_segment < start_pos.segment
                        || start_pos.progress > other.state.track_pos.progress
                    {
                        target_segment += map.track.path.len();
                        full_lap = true;
                    }
                    if start_pos.segment == 0 {
                        full_lap = true;
                    }

                    let segment_diff = target_segment - start_pos.segment;
                    let progress_diff = other.state.track_pos.progress - start_pos.progress;

                    let candidate = Nearest {
                        id: *id,
                        distance_segments: segment_diff,
                        distance_progress: progress_diff,
                        track_pos: other.state.track_pos,
                        full_lap,
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

                let target = nearest_found
                    .map(|n| {
                        let mut track_pos = owner.state.track_pos;

                        track_pos.lap = if n.full_lap {
                            n.track_pos.lap.saturating_sub(1)
                        } else {
                            n.track_pos.lap
                        };

                        RedShellTarget::Player {
                            target_id: n.id,
                            track_pos,
                            on_track: false,
                        }
                    })
                    .unwrap_or(RedShellTarget::None);

                ActiveItemState::RedShell { roll, target }
            }
        };

        Self {
            pos: owner.state.pos,
            rot: owner.state.rot,
            owner: owner.id(),
            spawn_time: Instant::now(),
            state,
        }
    }

    pub fn update(
        &mut self,
        map: &Map,
        colliders: &Vec<Polyline>,
        clients: &HashMap<ClientId, Client>,
    ) -> bool {
        use parry2d::{
            math::{Isometry, Vector},
            shape::Ball,
        };

        match &mut self.state {
            ActiveItemState::GreenShell { roll, bounces: _ } => {
                self.pos += Vec2::new(self.rot.to_radians().cos(), self.rot.to_radians().sin())
                    * SHELL_SPEED;
                *roll += 20.0;
            }
            ActiveItemState::RedShell { target, roll } => {
                *roll += 20.0;
                match target {
                    RedShellTarget::None => {
                        self.pos +=
                            Vec2::new(self.rot.to_radians().cos(), self.rot.to_radians().sin())
                                * SHELL_SPEED;
                    }
                    RedShellTarget::Player {
                        target_id,
                        track_pos,
                        on_track,
                    } => {
                        if let Some(target) = clients.get(target_id) {
                            let mut future_pos = *track_pos;
                            map.track
                                .advance_position(SHELL_SPEED * 4.0 * MAP_SCALE, &mut future_pos);

                            if target.state.track_pos < future_pos {
                                *on_track = false;

                                let direction = (target.state.pos - self.pos).normalize();
                                self.pos += direction * SHELL_SPEED;
                            } else {
                                let mut advance = (SHELL_SPEED / 2.0) * MAP_SCALE;
                                if !*on_track {
                                    advance *= 0.75;
                                }

                                let target_pos = map.track.advance_position(advance, track_pos);
                                let target_pos = map_coord_to_world(target_pos);

                                let direction = target_pos - self.pos;
                                self.pos += direction.normalize() * SHELL_SPEED;

                                self.rot = direction.y.atan2(direction.x).to_degrees();

                                let distance = (self.pos - target_pos).length();
                                if distance < SHELL_SPEED {
                                    *on_track = true;
                                }
                            }
                        } else {
                            *target = RedShellTarget::None;
                        }
                    }
                }
            }
            ActiveItemState::Banana => {}
        }

        let collider = Ball::new(6.0);
        let collider_pos = Isometry::new(nalgebra::zero(), 0.0);
        let new_pos_map = world_coord_to_map(self.pos);
        let own_pos = Isometry::new(Vector::new(new_pos_map.x, new_pos_map.y), 0.0);
        for other in colliders {
            use parry2d::query;
            if let Ok(Some(contact)) =
                query::contact(&own_pos, &collider, &collider_pos, other, nalgebra::zero())
            {
                let translation_map =
                    Vec2::new(contact.normal2.x, contact.normal2.y) * contact.dist * 1.1;
                let translation = map_coord_to_world(translation_map);

                self.pos -= translation;
                // reflect
                let forward = Vec2::new(self.rot.to_radians().cos(), self.rot.to_radians().sin());
                let normal = Vec2::new(contact.normal2.x, contact.normal2.y);

                let dot = forward.dot(normal);
                let new_forward = forward - 2.0 * dot * normal;
                self.rot = new_forward.y.atan2(new_forward.x).to_degrees();

                match &mut self.state {
                    ActiveItemState::GreenShell { bounces, .. } => {
                        *bounces -= 1;
                        return *bounces == 0;
                    }
                    ActiveItemState::RedShell { .. } => {
                        return true;
                    }

                    ActiveItemState::Banana => {}
                }
            }
        }

        false
    }

    pub fn check_collision(&self, player: &Client) -> bool {
        let now = Instant::now();
        if self.owner == player.id() && now - self.spawn_time < Duration::from_millis(200) {
            return false;
        }

        let distance = (self.pos - player.state.pos).length();
        distance < 0.5
    }
}

impl GameState {
    pub fn active_items(&self) -> Vec<common::ActiveItem> {
        self.active_items
            .iter()
            .map(|item| common::ActiveItem {
                pos: item.pos,
                rot: item.rot,
                kind: match item.state {
                    ActiveItemState::GreenShell { roll, .. } => {
                        common::ActiveItemKind::GreenShell { roll }
                    }
                    ActiveItemState::RedShell { roll, .. } => {
                        common::ActiveItemKind::RedShell { roll }
                    }
                    ActiveItemState::Banana => common::ActiveItemKind::Banana,
                },
            })
            .collect()
    }

    pub fn from_map(map: Map) -> Self {
        let coin_states = vec![true; map.coins.len()];
        let item_box_states = vec![true; map.item_spawns.len()];

        let colliders = map
            .colliders
            .iter()
            .map(|c| {
                let points = c
                    .shape
                    .iter()
                    .map(|p| nalgebra::Point2::new(p.x, p.y))
                    .chain(std::iter::once(nalgebra::Point2::new(
                        c.shape[0].x,
                        c.shape[0].y,
                    )))
                    .collect();
                Polyline::new(points, None)
            })
            .collect();

        Self {
            map,
            colliders,

            active_items: Vec::new(),
            coin_states,
            item_box_states,
        }
    }

    pub fn add_item(
        &mut self,
        kind: ActiveItemKind,
        owner: &Client,
        clients: &HashMap<ClientId, Client>,
    ) {
        let item = ActiveItem::new(owner, &self.map, clients, kind);
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
        // use parry2d::{
        //     math::{Isometry, Vector},
        //     shape::Ball,
        // };
        // let player_collider = Ball::new((4.0 / MAP_SCALE) * 2.0);
        //
        // for (i, player) in players.values().enumerate() {
        //     let player_pos =
        //         Isometry::new(Vector::new(player.state.pos.x, player.state.pos.y), 0.0);
        //     for other in players.values().skip(i + 1) {
        //         let other_pos =
        //             Isometry::new(Vector::new(other.state.pos.x, other.state.pos.y), 0.0);
        //         if let Ok(Some(contact)) = parry2d::query::contact(
        //             &player_pos,
        //             &player_collider,
        //             &other_pos,
        //             &player_collider,
        //             nalgebra::zero(),
        //         ) {
        //             client_handler
        //                 .send(
        //                     SendTo::InGameOnly(player.id()),
        //                     ServerMessage::PlayerCollision {
        //                         depth: -contact.dist / 2.0,
        //                         normal: Vec2::new(contact.normal2.x, contact.normal2.y),
        //
        //                         other_velocity: other.state.vel,
        //                         other_rotation: other.state.rot,
        //                     },
        //                 )
        //                 .await;
        //
        //             client_handler
        //                 .send(
        //                     SendTo::InGameOnly(other.id()),
        //                     ServerMessage::PlayerCollision {
        //                         normal: Vec2::new(contact.normal1.x, contact.normal1.y),
        //                         depth: -contact.dist / 2.0,
        //
        //                         other_velocity: player.state.vel,
        //                         other_rotation: player.state.rot,
        //                     },
        //                 )
        //                 .await;
        //         }
        //     }
        // }

        for i in (0..self.active_items.len()).rev() {
            let item = &mut self.active_items[i];

            let mut remove = item.update(&self.map, &self.colliders, &players);

            for player in players.values_mut() {
                if item.check_collision(player) {
                    client_handler
                        .send(
                            SendTo::InGameAll,
                            ServerMessage::HitByItem {
                                player: player.id(),
                            },
                        )
                        .await;
                    remove = true;
                }
            }

            if remove {
                self.active_items.swap_remove(i);
            }
        }
    }
}
