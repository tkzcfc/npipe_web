use crate::proto::{GeneralResponse, LoginReq, PlayerListResponse};
use crate::resource::ResponseType;
use crate::TemplateApp;
use eframe::emath::Align2;
use egui::{vec2, Ui};
use egui_extras::{Column, TableBuilder};
use log::info;
use once_cell::sync::Lazy;

static KEY_PLAYER_LIST: Lazy<String> = Lazy::new(|| String::from("player_list"));

pub fn ui(ctx: &egui::Context, app: &mut TemplateApp) {
    let pos = egui::pos2(
        ctx.screen_rect().width() * 0.25,
        ctx.screen_rect().height() * 0.5,
    );
    egui::Window::new("Player")
        .default_size(vec2(512.0, 512.0))
        .resizable(true)
        .vscroll(true)
        .hscroll(true)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(pos)
        .show(ctx, |ui| render_content(ui, ctx, app));
}

fn render_content(ui: &mut Ui, ctx: &egui::Context, app: &mut TemplateApp) {
    let mut need_request = false;
    if let Some(promise) = app.promise_map.get_mut(&*KEY_PLAYER_LIST) {
        if let Some(result) = promise.ready_mut() {
            match result {
                Ok(ref mut resource) => match &mut resource.response_data {
                    ResponseType::PlayerListResponse(player_list) => {}
                    ResponseType::Error(err) => {
                        ui.colored_label(ui.visuals().error_fg_color, err);
                    }
                    _ => {
                        ui.colored_label(ui.visuals().error_fg_color, "Unknown error");
                    }
                },
                Err(error) => {
                    if ui.button("retry").clicked() {
                        need_request = true;
                    }

                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        if error.is_empty() {
                            "Request failed"
                        } else {
                            error
                        },
                    );
                }
            }
        } else {
            ui.spinner();
        }
    } else {
        ui.spinner();
        need_request = true;
    }

    if need_request {
        app.http_request(ctx, KEY_PLAYER_LIST.as_str(), None, Vec::new());
    }
}

fn render_table() {}
