use crate::TemplateApp;

pub mod channel;
pub mod login;
mod password;
pub mod player;

pub trait RenderUI {
    fn render(&mut self, ctx: &egui::Context, app: &mut TemplateApp);

    fn reset(&mut self) {}
}
