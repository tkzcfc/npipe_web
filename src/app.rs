use crate::login;
use crate::proto::GeneralResponse;
use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::epaint::FontFamily;
use log::error;
use poll_promise::Promise;
use serde_urlencoded;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Resource {
    /// HTTP response
    pub(crate) response: ehttp::Response,
    checked: bool,
}

impl Resource {
    fn from_response(_ctx: &egui::Context, response: ehttp::Response) -> Self {
        error!("text: {}", response.text().unwrap());
        Self {
            response,
            checked: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum RequestType {
    Login,
    Test,
    Logout,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    pub(crate) api_url: String,

    /// 用户名
    pub(crate) username: String,
    /// 密码
    pub(crate) password: String,
    /// 是否是暗黑主题
    pub(crate) is_dark_them: bool,
    /// 是否已登录
    pub(crate) logged_in: bool,
    /// cookies缓存
    pub(crate) cookies: Vec<String>,

    #[serde(skip)]
    pub(crate) promise_map: HashMap<RequestType, Promise<ehttp::Result<Resource>>>,

    #[serde(skip)]
    need_check: Arc<Mutex<bool>>,

    #[serde(skip)]
    pub(crate) can_modify_api_url: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:8120/api/".to_owned(),
            username: "admin".into(),
            password: "".into(),
            logged_in: false,
            is_dark_them: true,
            promise_map: HashMap::new(),
            cookies: Vec::new(),
            need_check: Arc::new(Mutex::new(false)),
            can_modify_api_url: true,
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

        load_fonts(&cc.egui_ctx);

        #[cfg(target_arch = "wasm32")]
        if let Some(url) = get_current_url() {
            app.api_url = url;
            app.can_modify_api_url = false;
        }

        app
    }

    pub fn can_request(&mut self, request_type: &RequestType) -> bool {
        if let Some(promise) = self.promise_map.get(request_type) {
            promise.ready().is_some()
        } else {
            true
        }
    }

    pub fn http_request(
        &mut self,
        ctx: &egui::Context,
        request_type: RequestType,
        path: &str,
        params: Option<HashMap<String, String>>,
        body: Vec<u8>,
    ) {
        let mut url = if let Some('/') = self.api_url.chars().last() {
            self.api_url.clone()
        } else {
            format!("{}/", self.api_url)
        };

        url.push_str(path);

        if let Some(params) = params {
            let encoded: String = serde_urlencoded::to_string(params).unwrap();
            if encoded.len() > 0 {
                url.push_str("?");
                url.push_str(&encoded);
            }
        }

        let mut request = ehttp::Request::post(url, body);

        // 使用保存的 cookies
        let is_web = cfg!(target_arch = "wasm32");
        if !is_web {
            request
                .headers
                .headers
                .push(("Cookie".into(), self.cookies.join(";")));
        }

        let need_check = self.need_check.clone();
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            *need_check.lock().unwrap() = true;
            let resource = response.map(|response| Resource::from_response(&ctx, response));
            sender.send(resource);
        });

        self.promise_map.insert(request_type, promise);
    }

    fn http_response_check(&mut self) {
        if *self.need_check.lock().unwrap() == false {
            return;
        }
        for value in self.promise_map.values_mut() {
            if let Some(result) = value.ready_mut() {
                if let Ok(resource) = result {
                    if !resource.checked {
                        resource.checked = false;
                        let ref response = resource.response;
                        if response.ok {
                            if let Ok(response) =
                                serde_json::from_slice::<GeneralResponse>(&response.bytes)
                            {
                                if response.code == 10086 {
                                    self.logout();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        *self.need_check.lock().unwrap() = false;
    }

    /// 登录成功
    pub fn login_success(&mut self, cookies: Vec<String>) {
        self.promise_map.clear();
        self.logged_in = true;
        self.cookies = cookies;
    }

    /// 登出，清理数据
    pub fn logout(&mut self) {
        self.promise_map.clear();
        self.logged_in = false;
        self.cookies.clear();
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.http_response_check();

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
                        if ui.button("Test").clicked() {
                            self.http_request(
                                ctx,
                                RequestType::Test,
                                "test_auth",
                                None,
                                Vec::new(),
                            );
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
                self.is_dark_them = ctx.style().visuals.dark_mode;
            });
        });

        if self.logged_in {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Welcome");

                if ui.button("logout").clicked() && self.can_request(&RequestType::Logout) {
                    self.http_request(ctx, RequestType::Logout, "logout", None, Vec::new());
                }

                //ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's

                // ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                //     // ui.horizontal(|ui| {
                //     //     ui.label("Write something: ");
                //     //     ui.text_edit_singleline(&mut self.addr);
                //     // });
                //     // let mut dummy = false;
                //     // ui.checkbox(&mut dummy, "checkbox");
                //     //     ui.label("Write something  : ");
                //     //     ui.text_edit_singleline(&mut self.addr);
                //
                //     ui.horizontal(|ui| {
                //         ui.label("server:");
                //         ui.text_edit_singleline(&mut self.addr);
                //     });
                // });

                // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
                // if ui.button("Login").clicked() {}
                //
                // ui.separator();
                //
                // ui.add(egui::github_link_file!(
                //     "https://github.com/emilk/eframe_template/blob/master/",
                //     "Source code."
                // ));
                // });
            });
        } else {
            login::ui(ctx, self);
        }

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
        ui.hyperlink_to("Source code", "https://github.com/tkzcfc/npipe_web");

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

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        String::from("s_chinese_fallback"),
        FontData::from_static(include_bytes!("../assets/kuaile.ttf")),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .push("s_chinese_fallback".to_owned());

    ctx.set_fonts(fonts);
}

#[cfg(target_arch = "wasm32")]
fn get_current_url() -> Option<String> {
    use url::{Position, Url};
    use web_sys::window;

    if let Some(window) = window() {
        if let Some(location) = window.location().href().ok() {
            if let Ok(mut url) = Url::parse(&location) {
                url.set_path("/api/");
                return Some(url.to_string());
            } else {
                return None;
            }
        }
    }
    None
}
