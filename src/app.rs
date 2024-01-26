use crate::login;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};
use serde_urlencoded;
use std::collections::HashMap;

pub struct Resource {
    /// HTTP response
    pub(crate) response: ehttp::Response,
}

impl Resource {
    fn from_response(_ctx: &egui::Context, response: ehttp::Response) -> Self {
        Self { response }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum RequestType {
    Login,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    pub(crate) addr: String,

    /// 用户名
    pub(crate) username: String,
    /// 密码
    pub(crate) password: String,
    /// 是否是暗黑主题
    pub(crate) is_dark_them: bool,
    /// token
    pub(crate) token: String,

    #[serde(skip)]
    pub(crate) promise_map: HashMap<RequestType, Promise<ehttp::Result<Resource>>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            addr: "http://127.0.0.1:8118/api".to_owned(),
            username: "admin".into(),
            password: "".into(),
            token: "".into(),
            is_dark_them: true,
            promise_map: HashMap::new(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: Self = Default::default();

        if let Some(storage) = cc.storage {
            app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        if app.is_dark_them {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        app
    }

    pub fn can_request(&mut self, request_type: &RequestType) -> bool {
        if let Some(promise) = self.promise_map.get(request_type) {
            if let Some(result) = promise.ready() {
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn http_request(
        &mut self,
        ctx: &egui::Context,
        request_type: RequestType,
        path: &str,
        params: HashMap<String, String>,
        is_post: bool,
    ) {
        let mut url = if let Some('/') = self.addr.chars().last() {
            self.addr.clone()
        } else {
            format!("{}/", self.addr)
        };

        url.push_str(path);

        let request = if is_post {
            let json_string = serde_json::to_string(&params).unwrap();
            ehttp::Request::post(url, json_string.into())
        } else {
            let encoded: String = serde_urlencoded::to_string(params).unwrap();
            if encoded.len() > 0 {
                url.push_str("?");
                url.push_str(&encoded);
            }
            ehttp::Request::get(url)
        };

        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            let resource = response.map(|response| Resource::from_response(&ctx, response));
            sender.send(resource);
        });

        self.promise_map.insert(request_type, promise);
    }

    pub fn login_success(&mut self, token: String) {
        self.promise_map.clear();
        self.token = token;
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
                self.is_dark_them = ctx.style().visuals.dark_mode;
            });
        });

        if self.token.is_empty() {
            login::ui(ctx, self);
        }

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        //         // The central panel the region left after adding TopPanel's and SidePanel's
        //         ui.heading("npipe-web");
        //
        //         ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        //             // ui.horizontal(|ui| {
        //             //     ui.label("Write something: ");
        //             //     ui.text_edit_singleline(&mut self.addr);
        //             // });
        //             // let mut dummy = false;
        //             // ui.checkbox(&mut dummy, "checkbox");
        //             //     ui.label("Write something  : ");
        //             //     ui.text_edit_singleline(&mut self.addr);
        //
        //             ui.horizontal(|ui| {
        //                 ui.label("server:");
        //                 ui.text_edit_singleline(&mut self.addr);
        //             });
        //         });
        //
        //         ui.horizontal(|ui| {
        //             ui.label("password:");
        //             ui.add(password(&mut self.password));
        //         });
        //
        //         // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
        //         if ui.button("Login").clicked() {}
        //
        //         ui.separator();
        //
        //         ui.add(egui::github_link_file!(
        //             "https://github.com/emilk/eframe_template/blob/master/",
        //             "Source code."
        //         ));
        //     });
        // });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            powered_by_egui_and_eframe(ui);
            egui::warn_if_debug_build(ui);
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
