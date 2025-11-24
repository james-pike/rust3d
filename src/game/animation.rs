// animation.rs - Animation system (currently not in use)
// NOTE: This animation system needs to be updated to match the current AnimationState component structure
// The current AnimationState only has: current_animation (AnimationType), animation_time, transition_speed
// This code expects: animation_type, direction, alert_mode which don't exist yet

use bevy::prelude::*;
use bevy::animation::{AnimationPlayer, graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex}};
use bevy::asset::Assets;

use crate::entities::components::{Player, MoveDir, AnimationState, AnimationType};

// TODO: AnimationDirection needs to be defined or this code needs to be updated
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimationDirection {
    Forward,
    Backward,
    Left,
    Right,
    ForwardLeft,
    ForwardRight,
    BackwardLeft,
    BackwardRight,
}

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

/* COMMENTED OUT - Animation system needs updating to match current component structure

// Setup system (run once at Startup to build the graph)
pub fn setup_animation_graph(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
) {
    // Load all animation clips
    let idle: Handle<AnimationClip> = asset_server.load("animations/knight_idle.glb#Animation0");
    let idle_alert: Handle<AnimationClip> = asset_server.load("animations/knight_idle_alert.glb#Animation0");

    let walk_fwd: Handle<AnimationClip> = asset_server.load("animations/knight_walk_forward.glb#Animation0");
    let walk_bwd: Handle<AnimationClip> = asset_server.load("animations/knight_walk_backward.glb#Animation0");
    let walk_left: Handle<AnimationClip> = asset_server.load("animations/knight_walk_left.glb#Animation0");
    let walk_right: Handle<AnimationClip> = asset_server.load("animations/knight_walk_right.glb#Animation0");
    let walk_fwd_left: Handle<AnimationClip> = asset_server.load("animations/knight_walk_fwd_left.glb#Animation0");
    let walk_fwd_right: Handle<AnimationClip> = asset_server.load("animations/knight_walk_fwd_right.glb#Animation0");
    let walk_bwd_left: Handle<AnimationClip> = asset_server.load("animations/knight_walk_bwd_left.glb#Animation0");
    let walk_bwd_right: Handle<AnimationClip> = asset_server.load("animations/knight_walk_bwd_right.glb#Animation0");

    let walk_alert_fwd: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_forward.glb#Animation0");
    let walk_alert_bwd: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_backward.glb#Animation0");
    let walk_alert_left: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_left.glb#Animation0");
    let walk_alert_right: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_right.glb#Animation0");
    let walk_alert_fwd_left: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_fwd_left.glb#Animation0");
    let walk_alert_fwd_right: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_fwd_right.glb#Animation0");
    let walk_alert_bwd_left: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_bwd_left.glb#Animation0");
    let walk_alert_bwd_right: Handle<AnimationClip> = asset_server.load("animations/knight_walk_alert_bwd_right.glb#Animation0");

    let run_fwd: Handle<AnimationClip> = asset_server.load("animations/knight_run_forward.glb#Animation0");
    let run_fwd_left: Handle<AnimationClip> = asset_server.load("animations/knight_run_fwd_left.glb#Animation0");
    let run_fwd_right: Handle<AnimationClip> = asset_server.load("animations/knight_run_fwd_right.glb#Animation0");

    let run_alert_fwd: Handle<AnimationClip> = asset_server.load("animations/knight_run_alert_forward.glb#Animation0");
    let run_alert_fwd_left: Handle<AnimationClip> = asset_server.load("animations/knight_run_alert_fwd_left.glb#Animation0");
    let run_alert_fwd_right: Handle<AnimationClip> = asset_server.load("animations/knight_run_alert_fwd_right.glb#Animation0");

    // Build the graph
    let mut graph = AnimationGraph::new();

    let indices = PlayerAnimationIndices {
        idle: graph.add_clip(idle.clone(), 1.0, graph.root),
        idle_alert: graph.add_clip(idle_alert.clone(), 1.0, graph.root),

        walk_fwd: graph.add_clip(walk_fwd.clone(), 1.0, graph.root),
        walk_bwd: graph.add_clip(walk_bwd.clone(), 1.0, graph.root),
        walk_left: graph.add_clip(walk_left.clone(), 1.0, graph.root),
        walk_right: graph.add_clip(walk_right.clone(), 1.0, graph.root),
        walk_fwd_left: graph.add_clip(walk_fwd_left.clone(), 1.0, graph.root),
        walk_fwd_right: graph.add_clip(walk_fwd_right.clone(), 1.0, graph.root),
        walk_bwd_left: graph.add_clip(walk_bwd_left.clone(), 1.0, graph.root),
        walk_bwd_right: graph.add_clip(walk_bwd_right.clone(), 1.0, graph.root),

        walk_alert_fwd: graph.add_clip(walk_alert_fwd.clone(), 1.0, graph.root),
        walk_alert_bwd: graph.add_clip(walk_alert_bwd.clone(), 1.0, graph.root),
        walk_alert_left: graph.add_clip(walk_alert_left.clone(), 1.0, graph.root),
        walk_alert_right: graph.add_clip(walk_alert_right.clone(), 1.0, graph.root),
        walk_alert_fwd_left: graph.add_clip(walk_alert_fwd_left.clone(), 1.0, graph.root),
        walk_alert_fwd_right: graph.add_clip(walk_alert_fwd_right.clone(), 1.0, graph.root),
        walk_alert_bwd_left: graph.add_clip(walk_alert_bwd_left.clone(), 1.0, graph.root),
        walk_alert_bwd_right: graph.add_clip(walk_alert_bwd_right.clone(), 1.0, graph.root),

        run_fwd: graph.add_clip(run_fwd.clone(), 1.0, graph.root),
        run_fwd_left: graph.add_clip(run_fwd_left.clone(), 1.0, graph.root),
        run_fwd_right: graph.add_clip(run_fwd_right.clone(), 1.0, graph.root),

        run_alert_fwd: graph.add_clip(run_alert_fwd.clone(), 1.0, graph.root),
        run_alert_fwd_left: graph.add_clip(run_alert_fwd_left.clone(), 1.0, graph.root),
        run_alert_fwd_right: graph.add_clip(run_alert_fwd_right.clone(), 1.0, graph.root),
    };

    let graph_handle = graphs.add(graph);

    commands.insert_resource(PlayerAnimationGraph(graph_handle));
    commands.insert_resource(indices);
}

// Helper function to find the AnimationPlayer in the hierarchy
pub fn link_animation_players(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    animation_players: Query<(Entity, &Parent), Added<AnimationPlayer>>,
) {
    for player_entity in &players {
        for (player_entity_child, parent) in &animation_players {
            if **parent == player_entity {
                info!("Linking AnimationPlayer {:?} to Player {:?}", player_entity_child, player_entity);
                commands.entity(player_entity_child).insert(AnimationPlayerLink {
                    player_entity,
                    current_node: None,
                });
            }
        }
    }
}

*/
