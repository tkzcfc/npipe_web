use crate::app::RequestType;
use crate::password::password;
use crate::proto::{LoginAck, LoginReq};
use crate::TemplateApp;

pub fn ui(ctx: &egui::Context, app: &mut TemplateApp) {
    egui::Window::new("Login")
        .vscroll(false)
        .hscroll(false)
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
                    let req = LoginReq {
                        username: app.username.clone(),
                        password: app.password.clone(),
                    };
                    app.http_request(
                        ctx,
                        RequestType::Login,
                        "login",
                        None,
                        serde_json::to_string(&req).unwrap().into(),
                    );
                }

                if let Some(promise) = app.promise_map.get(&RequestType::Login) {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(resource) => {
                                let ref response = resource.response;
                                if response.ok {
                                    match serde_json::from_slice::<LoginAck>(&response.bytes) {
                                        Ok(ack) => {
                                            app.login_success(ack.token);
                                        }
                                        Err(err) => {
                                            ui.colored_label(
                                                ui.visuals().error_fg_color,
                                                err.to_string(),
                                            );
                                        }
                                    }
                                } else {
                                    ui.colored_label(
                                        ui.visuals().error_fg_color,
                                        format!(
                                            "status:       {} ({})",
                                            response.status, response.status_text
                                        ),
                                    );
                                }
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
