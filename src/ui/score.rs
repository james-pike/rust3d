// ui.rs
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui::{self, Align2, Color32, FontId, RichText}};
use crate::core::resources::Scores;

pub fn update_score_ui(mut contexts: EguiContexts, scores: Res<Scores>) {
    let Scores(p1_score, p2_score) = *scores;

    egui::Area::new("score".into())
        .anchor(Align2::CENTER_TOP, (0., 25.))
        .show(contexts.ctx_mut().unwrap(), |ui| {
            ui.label(
                RichText::new(format!("{p1_score} - {p2_score}"))
                    .color(Color32::WHITE)
                    .font(FontId::proportional(72.0)),
            );
        });
}