use crate::render::RenderUI;
use crate::TemplateApp;
use eframe::emath::{vec2, Align2};
use egui::Ui;

pub struct Logic {}

impl Logic {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderUI for Logic {
    fn render(&mut self, ctx: &egui::Context, app: &mut TemplateApp) {
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
            .show(ctx, |ui| render_content(ui, ctx, app));
    }
}

fn render_content(ui: &mut Ui, ctx: &egui::Context, app: &mut TemplateApp) {}
