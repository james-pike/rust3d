// animation.rs - Fixed for Bevy 0.15+ (AnimationGraph-based animation system)

use bevy::prelude::*;
use bevy::animation::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, AnimationPlayer};
use bevy::asset::Assets;

use crate::components::{Player, MoveDir, AnimationState, AnimationType, AnimationDirection};

// New resources (add these to your crate root or this file)
#[derive(Resource)]
pub struct PlayerAnimationGraph(pub Handle<AnimationGraph>);

#[derive(Resource)]
pub struct PlayerAnimationIndices {
    pub idle: AnimationNodeIndex,
    pub idle_alert: AnimationNodeIndex,

    pub walk_fwd: AnimationNodeIndex,
    pub walk_bwd: AnimationNodeIndex,
    pub walk_left: AnimationNodeIndex,
    pub walk_right: AnimationNodeIndex,
    pub walk_fwd_left: AnimationNodeIndex,
    pub walk_fwd_right: AnimationNodeIndex,
    pub walk_bwd_left: AnimationNodeIndex,
    pub walk_bwd_right: AnimationNodeIndex,

    pub walk_alert_fwd: AnimationNodeIndex,
    pub walk_alert_bwd: AnimationNodeIndex,
    pub walk_alert_left: AnimationNodeIndex,
    pub walk_alert_right: AnimationNodeIndex,
    pub walk_alert_fwd_left: AnimationNodeIndex,
    pub walk_alert_fwd_right: AnimationNodeIndex,
    pub walk_alert_bwd_left: AnimationNodeIndex,
    pub walk_alert_bwd_right: AnimationNodeIndex,

    pub run_fwd: AnimationNodeIndex,
    pub run_fwd_left: AnimationNodeIndex,
    pub run_fwd_right: AnimationNodeIndex,

    pub run_alert_fwd: AnimationNodeIndex,
    pub run_alert_fwd_left: AnimationNodeIndex,
    pub run_alert_fwd_right: AnimationNodeIndex,
}

// Component that links the player entity to its AnimationPlayer (now also tracks current node to prevent restarts)
#[derive(Component)]
pub struct AnimationPlayerLink {
    pub player_entity: Entity,
    pub current_node: Option<AnimationNodeIndex>,
}

// Load all clips and build a shared AnimationGraph (run this in Startup)
pub fn load_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("Loading animation clips and building AnimationGraph...");

    let idle = asset_server.load("player1.glb#Animation0");
    let idle_alert = asset_server.load("player1.glb#Animation1");

    let walk_fwd = asset_server.load("player1.glb#Animation21");
    let walk_bwd = asset_server.load("player1.glb#Animation18");
    let walk_left = asset_server.load("player1.glb#Animation24");
    let walk_right = asset_server.load("player1.glb#Animation25");
    let walk_fwd_left = asset_server.load("player1.glb#Animation22");
    let walk_fwd_right = asset_server.load("player1.glb#Animation23");
    let walk_bwd_left = asset_server.load("player1.glb#Animation19");
    let walk_bwd_right = asset_server.load("player1.glb#Animation20");

    let walk_alert_fwd = asset_server.load("player1.glb#Animation13");
    let walk_alert_bwd = asset_server.load("player1.glb#Animation10");
    let walk_alert_left = asset_server.load("player1.glb#Animation16");
    let walk_alert_right = asset_server.load("player1.glb#Animation17");
    let walk_alert_fwd_left = asset_server.load("player1.glb#Animation14");
    let walk_alert_fwd_right = asset_server.load("player1.glb#Animation15");
    let walk_alert_bwd_left = asset_server.load("player1.glb#Animation11");
    let walk_alert_bwd_right = asset_server.load("player1.glb#Animation12");

    let run_fwd = asset_server.load("player1.glb#Animation29");
    let run_fwd_left = asset_server.load("player1.glb#Animation30");
    let run_fwd_right = asset_server.load("player1.glb#Animation31");

    let run_alert_fwd = asset_server.load("player1.glb#Animation26");
    let run_alert_fwd_left = asset_server.load("player1.glb#Animation27");
    let run_alert_fwd_right = asset_server.load("player1.glb#Animation28");

    let mut graph = AnimationGraph::new();

    let indices = PlayerAnimationIndices {
        idle: graph.add_clip(idle.clone(), 1.0, graph.root()),
        idle_alert: graph.add_clip(idle_alert.clone(), 1.0, graph.root()),

        walk_fwd: graph.add_clip(walk_fwd.clone(), 1.0, graph.root()),
        walk_bwd: graph.add_clip(walk_bwd.clone(), 1.0, graph.root()),
        walk_left: graph.add_clip(walk_left.clone(), 1.0, graph.root()),
        walk_right: graph.add_clip(walk_right.clone(), 1.0, graph.root()),
        walk_fwd_left: graph.add_clip(walk_fwd_left.clone(), 1.0, graph.root()),
        walk_fwd_right: graph.add_clip(walk_fwd_right.clone(), 1.0, graph.root()),
        walk_bwd_left: graph.add_clip(walk_bwd_left.clone(), 1.0, graph.root()),
        walk_bwd_right: graph.add_clip(walk_bwd_right.clone(), 1.0, graph.root()),

        walk_alert_fwd: graph.add_clip(walk_alert_fwd.clone(), 1.0, graph.root()),
        walk_alert_bwd: graph.add_clip(walk_alert_bwd.clone(), 1.0, graph.root()),
        walk_alert_left: graph.add_clip(walk_alert_left.clone(), 1.0, graph.root()),
        walk_alert_right: graph.add_clip(walk_alert_right.clone(), 1.0, graph.root()),
        walk_alert_fwd_left: graph.add_clip(walk_alert_fwd_left.clone(), 1.0, graph.root()),
        walk_alert_fwd_right: graph.add_clip(walk_alert_fwd_right.clone(), 1.0, graph.root()),
        walk_alert_bwd_left: graph.add_clip(walk_alert_bwd_left.clone(), 1.0, graph.root()),
        walk_alert_bwd_right: graph.add_clip(walk_alert_bwd_right.clone(), 1.0, graph.root()),

        run_fwd: graph.add_clip(run_fwd.clone(), 1.0, graph.root()),
        run_fwd_left: graph.add_clip(run_fwd_left.clone(), 1.0, graph.root()),
        run_fwd_right: graph.add_clip(run_fwd_right.clone(), 1.0, graph.root()),

        run_alert_fwd: graph.add_clip(run_alert_fwd.clone(), 1.0, graph.root()),
        run_alert_fwd_left: graph.add_clip(run_alert_fwd_left.clone(), 1.0, graph.root()),
        run_alert_fwd_right: graph.add_clip(run_alert_fwd_right.clone(), 1.0, graph.root()),
    };

    let graph_handle = animation_graphs.add(graph);

    commands.insert_resource(PlayerAnimationGraph(graph_handle));
    commands.insert_resource(indices);

    info!("Animation graph built and inserted!");
}

// New system - make sure it runs after scenes are instantiated (Update or FixedUpdate)
pub fn setup_player_animation_graph(
    mut commands: Commands,
    graph: Res<PlayerAnimationGraph>,
    query: Query<Entity, (With<AnimationPlayer>, Without<AnimationGraphHandle>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(AnimationGraphHandle(graph.0.clone()));
    }
}

// Fixed link system - Children::iter() now yields Entity (no &)
pub fn link_animation_players(
    mut commands: Commands,
    players: Query<Entity, (With<Player>, Without<AnimationPlayerLink>)>,
    animation_players: Query<Entity, (With<AnimationPlayer>, Without<AnimationPlayerLink>)>,
    children: Query<&Children>,
) {
    for player_entity in &players {
        if let Ok(player_children) = children.get(player_entity) {
            for child in player_children.iter() {
                if animation_players.contains(child) {
                    commands.entity(child).insert(AnimationPlayerLink {
                        player_entity,
                        current_node: None,
                    });
                    info!("Linked AnimationPlayer {:?} to Player {:?}", child, player_entity);
                    continue;
                }

                if let Ok(grandchildren) = children.get(child) {
                    for grandchild in grandchildren.iter() {
                        if animation_players.contains(grandchild) {
                            commands.entity(grandchild).insert(AnimationPlayerLink {
                                player_entity,
                                current_node: None,
                            });
                            info!("Linked AnimationPlayer {:?} to Player {:?}", grandchild, player_entity);
                            break;
                        }
                    }
                }
            }
        }
    }
}

// Select node index (used instead of Handle<AnimationClip>)
fn select_animation_node(state: &AnimationState, indices: &PlayerAnimationIndices) -> AnimationNodeIndex {
    match state.animation_type {
        AnimationType::Idle => {
            if state.alert_mode { indices.idle_alert } else { indices.idle }
        }
        AnimationType::Walk => {
            if state.alert_mode {
                match state.direction {
                    AnimationDirection::Forward => indices.walk_alert_fwd,
                    AnimationDirection::Backward => indices.walk_alert_bwd,
                    AnimationDirection::Left => indices.walk_alert_left,
                    AnimationDirection::Right => indices.walk_alert_right,
                    AnimationDirection::ForwardLeft => indices.walk_alert_fwd_left,
                    AnimationDirection::ForwardRight => indices.walk_alert_fwd_right,
                    AnimationDirection::BackwardLeft => indices.walk_alert_bwd_left,
                    AnimationDirection::BackwardRight => indices.walk_alert_bwd_right,
                }
            } else {
                match state.direction {
                    AnimationDirection::Forward => indices.walk_fwd,
                    AnimationDirection::Backward => indices.walk_bwd,
                    AnimationDirection::Left => indices.walk_left,
                    AnimationDirection::Right => indices.walk_right,
                    AnimationDirection::ForwardLeft => indices.walk_fwd_left,
                    AnimationDirection::ForwardRight => indices.walk_fwd_right,
                    AnimationDirection::BackwardLeft => indices.walk_bwd_left,
                    AnimationDirection::BackwardRight => indices.walk_bwd_right,
                }
            }
        }
        AnimationType::Run => {
            let alert = state.alert_mode;
            match state.direction {
                AnimationDirection::Forward | AnimationDirection::Backward => {
                    if alert { indices.run_alert_fwd } else { indices.run_fwd }
                }
                AnimationDirection::Left | AnimationDirection::ForwardLeft | AnimationDirection::BackwardLeft => {
                    if alert { indices.run_alert_fwd_left } else { indices.run_fwd_left }
                }
                _ => if alert { indices.run_alert_fwd_right } else { indices.run_fwd_right },
            }
        }
    }
}

// Updated apply system - now uses the graph + node tracking (run this in Update)
pub fn apply_animations(
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationPlayerLink)>,
    player_states: Query<&AnimationState, With<Player>>,
    indices: Res<PlayerAnimationIndices>,
) {
    for (mut player, mut link) in &mut animation_players {
        if let Ok(state) = player_states.get(link.player_entity) {
            let node_index = select_animation_node(state, &indices);

            // Only play if different or initial (prevents restart when direction only changes but clip stays the same)
            if link.current_node.map_or(true, |current| current != node_index) {
                let animation_name = format!("{:?}_{:?}_alert:{}", state.animation_type, state.direction, state.alert_mode);
                info!("Playing animation: {} for player {:?}", animation_name, link.player_entity);
                player.play(node_index).repeat();
                link.current_node = Some(node_index);
            }
        }
    }
}