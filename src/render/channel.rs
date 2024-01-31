use crate::TemplateApp;
use eframe::emath::Align2;
use egui::{vec2, Ui};

pub fn ui(ctx: &egui::Context, app: &mut TemplateApp) {
    let pos = egui::pos2(
        ctx.screen_rect().width() * 0.75,
        ctx.screen_rect().height() * 0.5,
    );
    egui::Window::new("Channel")
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .collapsible(true)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(pos)
        // .min_size(vec2(100.0, 100.0))
        .show(ctx, |ui| render_content(ui, ctx, app));
}

fn render_content(ui: &mut Ui, ctx: &egui::Context, app: &mut TemplateApp) {}
