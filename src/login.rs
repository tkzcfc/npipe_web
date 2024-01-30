use crate::app::RequestType;
use crate::password::password;
use crate::proto::{GeneralResponse, LoginReq};
use crate::TemplateApp;
use log::info;
use regex::Regex;

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
                                    match serde_json::from_slice::<GeneralResponse>(&response.bytes)
                                    {
                                        Ok(ack) => {
                                            if ack.code == 0 {
                                                info!("login success");
                                                app.login_success(extract_cookies(response));
                                            } else {
                                                ui.colored_label(
                                                    ui.visuals().error_fg_color,
                                                    ack.msg,
                                                );
                                            }
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

// 提取响应中的 Set-Cookie 头部
fn extract_cookies(response: &ehttp::Response) -> Vec<String> {
    response
        .headers
        .headers
        .iter()
        .filter_map(|(name, value)| {
            if name.eq_ignore_ascii_case("Set-Cookie") {
                // 使用正则表达式查找cookie名称和值
                // auth-id=mYiQZEtaR9EFh0KUXThIPfpu%2FyWu91D8IxUq2SRX6660xi57K84uv40gVA8YVYImklEngoO4njikpDr5Q6o%3D; HttpOnly; SameSite=Lax; Path=/; Max-Age=3600
                // 只捕获 auth-id=mYiQZEtaR9EFh0KUXThIPfpu%2FyWu91D8IxUq2SRX6660xi57K84uv40gVA8YVYImklEngoO4njikpDr5Q6o%3D
                let cookie_re = Regex::new(r"[^;=\s]+=[^;]*").unwrap();
                for cap in cookie_re.captures_iter(value) {
                    if cap.len() > 0 {
                        return Some(cap[0].to_owned());
                    }
                    break;
                }
                None
            } else {
                None
            }
        })
        .collect()
}
