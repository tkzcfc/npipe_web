use crate::app::RequestType;
use crate::password::password;
use crate::TemplateApp;
use std::collections::HashMap;

pub fn ui(ctx: &egui::Context, app: &mut TemplateApp) {
    egui::Window::new("Login")
        .vscroll(true)
        .hscroll(true)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.label("server:");
                    ui.text_edit_singleline(&mut app.addr);
                });

                ui.horizontal(|ui| {
                    ui.label("username:");
                    ui.text_edit_singleline(&mut app.username);
                });

                ui.horizontal(|ui| {
                    ui.label("password:");
                    ui.add(password(&mut app.password));
                });

                ui.separator();
                if ui.button("Login").clicked() && app.can_request(&RequestType::Login) {
                    let mut params: HashMap<String, String> = HashMap::new();
                    params.insert("username".into(), app.username.clone());
                    params.insert("password".into(), app.password.clone());
                    app.http_request(ctx, RequestType::Login, "login", params);
                }

                if let Some(promise) = app.promise_map.get(&RequestType::Login) {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(resource) => {
                                app.token = resource.response.status_text.clone();
                            }
                            Err(error) => {
                                // This should only happen if the fetch API isn't available or something similar.
                                ui.colored_label(
                                    ui.visuals().error_fg_color,
                                    if error.is_empty() {
                                        "Login failed"
                                    } else {
                                        error
                                    },
                                );
                            }
                        }
                    } else {
                        ui.spinner();
                    }
                }
            });
        });
}
