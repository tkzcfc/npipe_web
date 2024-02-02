use crate::proto::GeneralResponse;
use crate::render;
use crate::render::RenderUI;
use crate::resource::Resource;
use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::epaint::FontFamily;
use poll_promise::Promise;
use serde_urlencoded;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct SubPage {
    render: Rc<RefCell<dyn RenderUI>>,
    name: String,
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
    pub(crate) promise_map: HashMap<String, Promise<ehttp::Result<Resource>>>,

    #[serde(skip)]
    need_check: Arc<Mutex<bool>>,

    #[serde(skip)]
    pub(crate) can_modify_api_url: bool,

    #[serde(skip)]
    sub_pages: Vec<SubPage>,
    #[serde(skip)]
    cur_page_index: usize,
    #[serde(skip)]
    login_ui: Rc<RefCell<dyn RenderUI>>,
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
            login_ui: Rc::new(RefCell::new(render::login::Logic::new())),
            cur_page_index: 0,
            sub_pages: vec![
                SubPage {
                    name: "👥Player".into(),
                    render: Rc::new(RefCell::new(render::player::Logic::new())),
                },
                SubPage {
                    name: "🔀Channel".into(),
                    render: Rc::new(RefCell::new(render::channel::Logic::new())),
                },
            ],
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

    pub fn can_request(&mut self, request_type: &String) -> bool {
        if let Some(promise) = self.promise_map.get(request_type) {
            promise.ready().is_some()
        } else {
            true
        }
    }

    pub fn http_request_ex(
        &mut self,
        ctx: &egui::Context,
        path: &str,
        key: String,
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

        let path_moved = path.to_string();
        let need_check = self.need_check.clone();
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            *need_check.lock().unwrap() = true;
            let resource =
                response.map(|response| Resource::from_response(&ctx, response, path_moved));
            sender.send(resource);
        });

        self.promise_map.insert(key, promise);
    }

    pub fn http_request(
        &mut self,
        ctx: &egui::Context,
        path: &str,
        params: Option<HashMap<String, String>>,
        body: Vec<u8>,
    ) {
        self.http_request_ex(ctx, path, path.to_string(), params, body);
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
                        } else if response.status == 401 {
                            self.logout();
                            break;
                        }
                    }
                }
            }
        }

        *self.need_check.lock().unwrap() = false;
    }

    /// 登录成功
    pub fn login_success(&mut self, cookies: Vec<String>) {
        for ref page in &self.sub_pages {
            page.render.borrow_mut().reset();
        }
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

                ui.menu_button("Tools", |ui| {
                    if !is_web {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if self.logged_in {
                            if ui.button("Logout").clicked() && self.can_request(&"logout".into()) {
                                self.http_request(ctx, "logout", None, Vec::new());
                            }
                        }
                    }
                    if ui.button("Test").clicked() {
                        self.http_request(ctx, "test_auth", None, Vec::new());
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_dark_light_mode_buttons(ui);
                self.is_dark_them = ctx.style().visuals.dark_mode;
            });

            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                for (index, page) in self.sub_pages.iter().enumerate() {
                    if ui
                        .selectable_label(self.cur_page_index == index, &page.name)
                        .clicked()
                    {
                        self.cur_page_index = index;
                    }
                }
            });
        });

        if self.logged_in {
            if let Some(page) = self.sub_pages.get(self.cur_page_index) {
                page.render.clone().borrow_mut().render(ctx, self);
            }
        } else {
            self.login_ui.clone().borrow_mut().render(ctx, self);
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

/// 根据平台自动寻找字体
// https://github.com/emilk/egui/issues/3060
//
// pub fn configure_fonts(ctx: &eframe::egui::Context) -> Option<()> {
//     let font_file = find_cjk_font()?;
//     let font_name = font_file.split('/').last()?.split('.').next()?.to_string();
//     let font_file_bytes = std::fs::read(font_file).ok()?;
//
//     let font_data = eframe::egui::FontData::from_owned(font_file_bytes);
//     let mut font_def = eframe::egui::FontDefinitions::default();
//     font_def.font_data.insert(font_name.to_string(), font_data);
//
//     let font_family = eframe::epaint::FontFamily::Proportional;
//     font_def.families.get_mut(&font_family)?.insert(0, font_name);
//
//     ctx.set_fonts(font_def);
//     Some(())
// }
//
// fn find_cjk_font() -> Option<String> {
//     use std::path::PathBuf;
//     #[cfg(unix)]
//     {
//         use std::process::Command;
//         // linux/macOS command: fc-list
//         let output = Command::new("sh").arg("-c").arg("fc-list").output().ok()?;
//         let stdout = std::str::from_utf8(&output.stdout).ok()?;
//         #[cfg(target_os = "macos")]
//             let font_line = stdout
//             .lines()
//             .find(|line| line.contains("Regular") && line.contains("Hiragino Sans GB"))
//             .unwrap_or("/System/Library/Fonts/Hiragino Sans GB.ttc");
//         #[cfg(target_os = "linux")]
//             let font_line = stdout
//             .lines()
//             .find(|line| line.contains("Regular") && line.contains("CJK"))
//             .unwrap_or("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc");
//
//         let font_path = font_line.split(':').next()?.trim();
//         Some(font_path.to_string())
//     }
//     #[cfg(windows)]
//     {
//         let font_file = {
//             // c:/Windows/Fonts/msyh.ttc
//             let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok()?);
//             font_path.push("Fonts");
//             font_path.push("msyh.ttc");
//             font_path.to_str()?.to_string().replace("\\", "/")
//         };
//         Some(font_file)
//     }
// }

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
