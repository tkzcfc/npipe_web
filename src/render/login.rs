use super::password::password;
use crate::proto;
use crate::resource::ResponseType;
use crate::TemplateApp;
use egui::{Align2, Ui};
use once_cell::sync::Lazy;
use regex::Regex;

static KEY_LOGIN: Lazy<String> = Lazy::new(|| String::from("login"));

pub fn ui(ctx: &egui::Context, app: &mut TemplateApp) {
    let pos = egui::pos2(
        ctx.screen_rect().width() * 0.5,
        ctx.screen_rect().height() * 0.5,
    );
    egui::Window::new("Login")
        .vscroll(false)
        .hscroll(false)
        .resizable(false)
        .collapsible(false)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .show(ctx, |ui| render_content(ui, ctx, app));
}

fn render_content(ui: &mut Ui, ctx: &egui::Context, app: &mut TemplateApp) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        if app.can_modify_api_url {
            ui.horizontal(|ui| {
                ui.label("api url:");
                ui.text_edit_singleline(&mut app.api_url);
            });
        }

        ui.horizontal(|ui| {
            ui.label("username:");
            ui.text_edit_singleline(&mut app.username);
        });

        ui.horizontal(|ui| {
            ui.label("password:");
            ui.add(password(&mut app.password));
        });

        ui.separator();
        if ui.button("Login").clicked() && app.can_request(&KEY_LOGIN) {
            let req = proto::LoginReq {
                username: app.username.clone(),
                password: app.password.clone(),
            };
            app.http_request(
                ctx,
                KEY_LOGIN.as_str(),
                None,
                serde_json::to_string(&req).unwrap().into(),
            );
        }

        if let Some(promise) = app.promise_map.get(&*KEY_LOGIN) {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(resource) => match &resource.response_data {
                        ResponseType::GeneralResponse(_) => {
                            app.login_success(extract_cookies(&resource.response));
                        }
                        ResponseType::Error(err) => {
                            ui.colored_label(ui.visuals().error_fg_color, err);
                        }
                        _ => {
                            ui.colored_label(ui.visuals().error_fg_color, "Unknown error");
                        }
                    },
                    Err(error) => {
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
