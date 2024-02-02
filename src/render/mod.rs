use crate::TemplateApp;
use egui::Ui;

pub mod channel;
pub mod login;
mod password;
pub mod player;

pub trait RenderUI {
    fn render(&mut self, ctx: &egui::Context, app: &mut TemplateApp);

    fn reset(&mut self) {}
}

pub fn render_number_u32(ui: &mut Ui, number: &mut u32) {
    let mut str = format!("{}", number);
    if ui.text_edit_singleline(&mut str).changed() {
        if let Ok(value) = str.parse::<u32>() {
            *number = value;
        }
    }
}
