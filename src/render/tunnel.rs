use crate::proto::TunnelListResponse;
use crate::render::{render_number_u32, RenderUI};
use crate::resource::ResponseType;
use crate::{proto, TemplateApp};
use eframe::emath::vec2;
use eframe::epaint::Color32;
use egui::{Rect, Ui};
use egui_extras::{Column, TableBuilder};
use std::collections::HashMap;

static PAGE_SIZE: usize = 20;
static INVALID_ITEM_ID: u32 = u32::MAX;

static GRAY: Color32 = Color32::from_rgba_premultiplied(80, 80, 80, 80);

enum OperationResult {
    None,
    Wait,
    Error(String),
}

struct CreateData {
    source: String,
    endpoint: String,
    sender: u32,
    receiver: u32,
    description: String,
}

pub struct Logic {
    key_get_list: String,
    key_remove_item: String,
    key_add_item: String,
    key_update_item: String,

    item_operation_map: HashMap<String, (u32, OperationResult)>,

    // 是否正在等待列表数据刷新
    wait_data_list: bool,
    data: Option<TunnelListResponse>,

    show_create_window: bool,
    create_data: CreateData,
}

impl Logic {
    pub fn new() -> Self {
        Self {
            key_get_list: "tunnel_list".into(),
            key_remove_item: "remove_tunnel".into(),
            key_add_item: "add_tunnel".into(),
            key_update_item: "update_tunnel".into(),
            wait_data_list: false,
            data: None,
            item_operation_map: HashMap::new(),
            show_create_window: false,
            create_data: CreateData {
                source: "".to_string(),
                endpoint: "".to_string(),
                sender: 0,
                receiver: 0,
                description: "".to_string(),
            },
        }
    }
}

impl RenderUI for Logic {
    fn render(&mut self, ctx: &egui::Context, app: &mut TemplateApp) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let need_update_page = if self.busy(app) {
                self.render_loading(ui);
                self.render_create_window(ctx, app, false)
            } else {
                self.render_create_window(ctx, app, true)
            };

            self.render_content(ui, ctx, app, need_update_page);
        });
    }

    fn reset(&mut self) {
        self.data = None;
        self.wait_data_list = false;
        self.item_operation_map.clear();
        self.show_create_window = false;
        self.create_data = CreateData {
            source: "".to_string(),
            endpoint: "".to_string(),
            sender: 0,
            receiver: 0,
            description: "".to_string(),
        };
    }
}

impl Logic {
    fn render_content(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        app: &mut TemplateApp,
        need_update_page: bool,
    ) {
        let mut need_request = need_update_page;
        let mut cur_page_number: usize = 0;
        if let Some(promise) = app.promise_map.get_mut(&self.key_get_list) {
            if let Some(result) = promise.ready_mut() {
                match result {
                    Ok(ref resource) => match &resource.response_data {
                        ResponseType::TunnelListResponse(ref tunnel_list) => {
                            if self.wait_data_list {
                                self.wait_data_list = false;
                                self.data = Some(tunnel_list.clone());
                                self.item_operation_map.clear();
                            }
                        }
                        ResponseType::Error(err) => {
                            if ui.button("retry").clicked() {
                                need_request = true;
                            }
                            ui.colored_label(ui.visuals().error_fg_color, err);
                        }
                        _ => {
                            if ui.button("retry").clicked() {
                                need_request = true;
                            }
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
            }
        } else {
            need_request = true;
        }

        // 玩家列表渲染
        if let Some(ref mut tunnel_list) = self.data {
            ui.horizontal(|ui| {
                // 刷新按钮
                if ui.button("🔃").clicked() {
                    need_request = true;
                }

                // 计算页数
                cur_page_number = tunnel_list.cur_page_number as usize;
                let mut page_count = if tunnel_list.total_count % PAGE_SIZE == 0 {
                    tunnel_list.total_count / PAGE_SIZE
                } else {
                    tunnel_list.total_count / PAGE_SIZE + 1
                };

                if page_count <= 0 {
                    page_count = 1;
                }

                if cur_page_number > 0 && page_count <= cur_page_number {
                    cur_page_number = page_count - 1;
                    need_request = true;
                }

                // 页数选择
                if page_count > 1
                    && egui::ComboBox::from_label("Page")
                        .selected_text(format!("{}", cur_page_number + 1))
                        .show_index(ui, &mut cur_page_number, page_count, |i| {
                            format!("{}", i + 1)
                        })
                        .changed()
                {
                    need_request = true;
                }
            });

            ui.horizontal(|ui| {
                if ui.button("new tunnel").clicked() {
                    self.show_create_window = true;
                }
                ui.label(format!("total : {}", tunnel_list.total_count));
            });

            self.render_table(ui, ctx, app);
        }

        // 请求列表数据
        if need_request {
            let req = proto::TunnelListRequest {
                page_number: cur_page_number,
                page_size: PAGE_SIZE,
            };
            app.http_request(
                ctx,
                &self.key_get_list,
                None,
                serde_json::to_string(&req).unwrap().into(),
            );
            self.wait_data_list = true;
        }
    }

    fn render_loading(&self, ui: &mut Ui) {
        ui.painter().rect_filled(ui.max_rect(), 0.0, GRAY);

        egui::Spinner::new().paint_at(
            ui,
            Rect::from_center_size(ui.max_rect().center(), vec2(30.0, 30.0)),
        );

        // 屏蔽下层输入
        ui.interact(
            ui.min_rect(),
            egui::Id::new("Some Id"),
            egui::Sense::click(),
        );
    }

    fn render_table(&mut self, ui: &mut Ui, ctx: &egui::Context, app: &mut TemplateApp) {
        let mut need_update_item_info = None;
        let mut need_remove_item_info = None;

        let update_item_operation = self.item_operation_map.get(&self.key_update_item);
        let remove_item_operation = self.item_operation_map.get(&self.key_remove_item);

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .min_scrolled_height(0.0);
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("index");
                });
                header.col(|ui| {
                    ui.strong("id");
                });
                header.col(|ui| {
                    ui.strong("source");
                });
                header.col(|ui| {
                    ui.strong("endpoint");
                });
                header.col(|ui| {
                    ui.strong("enabled");
                });
                header.col(|ui| {
                    ui.strong("sender");
                });
                header.col(|ui| {
                    ui.strong("receiver");
                });
                header.col(|ui| {
                    ui.strong("description");
                });
                header.col(|ui| {
                    ui.strong("update");
                });
                header.col(|ui| {
                    ui.strong("remove");
                });
            })
            .body(|mut body| {
                if let Some(ref mut item_list) = self.data {
                    for (index, item) in item_list.tunnels.iter_mut().enumerate() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{}", index + 1));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", item.id));
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut item.source);
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut item.endpoint);
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut item.enabled, "enabled");
                            });
                            row.col(|ui| {
                                render_number_u32(ui, &mut item.sender);
                            });
                            row.col(|ui| {
                                render_number_u32(ui, &mut item.receiver);
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut item.description);
                            });
                            row.col(|ui| {
                                if update_item_operation.is_none() {
                                    if ui.button("🔄update").clicked() {
                                        need_update_item_info = Some(item.clone());
                                    }
                                    return;
                                }

                                let (item_id, operation_result) = &update_item_operation.unwrap();

                                if item_id == &INVALID_ITEM_ID {
                                    if ui.button("🔄update").clicked() {
                                        need_update_item_info = Some(item.clone());
                                    }
                                } else {
                                    if item_id != &item.id {
                                        return;
                                    } else {
                                        match operation_result {
                                            OperationResult::Error(message) => {
                                                if ui.button("🔄retry").clicked() {
                                                    need_update_item_info = Some(item.clone());
                                                }
                                                ui.colored_label(
                                                    ui.visuals().error_fg_color,
                                                    message,
                                                );
                                            }
                                            _ => {
                                                ui.spinner();
                                            }
                                        }
                                    }
                                }
                            });
                            row.col(|ui| {
                                // 没有正在删除的元素
                                if remove_item_operation.is_none()
                                    || remove_item_operation.unwrap().0 == INVALID_ITEM_ID
                                {
                                    if let Some(operation_result) = update_item_operation {
                                        // 正在更新的元素不是当前元素，显示删除按钮
                                        if operation_result.0 != item.id && ui.button("✖").clicked()
                                        {
                                            need_remove_item_info = Some(item.id);
                                        }
                                    } else {
                                        if ui.button("✖").clicked() {
                                            need_remove_item_info = Some(item.id);
                                        }
                                    }
                                } else {
                                    // 正在删除其他元素
                                    if remove_item_operation.unwrap().0 != item.id {
                                        return;
                                    }

                                    match &remove_item_operation.unwrap().1 {
                                        OperationResult::Error(message) => {
                                            if ui.button("✖").clicked() {
                                                need_remove_item_info = Some(item.id);
                                            }
                                            ui.colored_label(ui.visuals().error_fg_color, message);
                                        }
                                        _ => {
                                            ui.spinner();
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            });

        // 更新操作
        if let Some(info) = need_update_item_info {
            self.item_operation_map.insert(
                self.key_update_item.clone(),
                (info.id, OperationResult::Wait),
            );
            let req = proto::TunnelUpdateReq {
                id: info.id,
                source: info.source,
                endpoint: info.endpoint,
                enabled: info.enabled as u8,
                sender: info.sender,
                receiver: info.receiver,
                description: info.description,
            };
            app.http_request(
                ctx,
                &self.key_update_item,
                None,
                serde_json::to_string(&req).unwrap().into(),
            )
        }

        // 删除操作
        if let Some(info) = need_remove_item_info {
            self.item_operation_map
                .insert(self.key_remove_item.clone(), (info, OperationResult::Wait));
            let req = proto::TunnelRemoveReq { id: info };
            app.http_request(
                ctx,
                &self.key_remove_item,
                None,
                serde_json::to_string(&req).unwrap().into(),
            )
        }
    }

    fn render_create_window(
        &mut self,
        ctx: &egui::Context,
        app: &mut TemplateApp,
        enabled: bool,
    ) -> bool {
        let mut request_finish = false;
        egui::Window::new("New Tunnel")
            .vscroll(true)
            .hscroll(true)
            .resizable(true)
            .collapsible(true)
            .open(&mut self.show_create_window)
            .enabled(enabled)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("source     :");
                    ui.text_edit_singleline(&mut self.create_data.source);
                });

                ui.horizontal(|ui| {
                    ui.label("endpoint   :");
                    ui.text_edit_singleline(&mut self.create_data.endpoint);
                });
                ui.horizontal(|ui| {
                    ui.label("sender     :");
                    render_number_u32(ui, &mut self.create_data.sender);
                });
                ui.horizontal(|ui| {
                    ui.label("receiver   :");
                    render_number_u32(ui, &mut self.create_data.receiver);
                });
                ui.horizontal(|ui| {
                    ui.label("description:");
                    ui.text_edit_singleline(&mut self.create_data.description);
                });
                ui.separator();
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    if ui.button("   ok   ").clicked() && app.can_request(&self.key_add_item) {
                        let req = proto::TunnelAddReq {
                            source: self.create_data.source.clone(),
                            endpoint: self.create_data.endpoint.clone(),
                            enabled: 1,
                            sender: self.create_data.sender,
                            receiver: self.create_data.receiver,
                            description: self.create_data.description.clone(),
                        };
                        app.http_request(
                            ctx,
                            &self.key_add_item,
                            None,
                            serde_json::to_string(&req).unwrap().into(),
                        );
                    }

                    if let Some(promise) = app.promise_map.get(&self.key_add_item) {
                        if let Some(result) = promise.ready() {
                            match result {
                                Ok(resource) => match &resource.response_data {
                                    ResponseType::GeneralResponse(_) => {
                                        request_finish = true;
                                        app.promise_map.remove(&self.key_add_item);
                                    }
                                    ResponseType::Error(err) => {
                                        ui.colored_label(ui.visuals().error_fg_color, err);
                                    }
                                    _ => {
                                        ui.colored_label(
                                            ui.visuals().error_fg_color,
                                            "Unknown error",
                                        );
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
            });

        if request_finish {
            self.show_create_window = false;
            if let Some(data) = &mut self.data {
                data.total_count = data.total_count + 1;
                if data.tunnels.len() < PAGE_SIZE {
                    return true;
                }
            }
        }
        false
    }

    fn busy(&mut self, app: &mut TemplateApp) -> bool {
        let mut removed_id = None;
        for (key, (id, operation_result)) in &mut self.item_operation_map {
            if *id != INVALID_ITEM_ID {
                let promise_option = app.promise_map.get(key);
                if let Some(promise) = promise_option {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(resource) => match &resource.response_data {
                                ResponseType::GeneralResponse(_) => {
                                    if key == &self.key_remove_item {
                                        removed_id = Some(*id);
                                    }
                                    *operation_result = OperationResult::None;
                                    *id = INVALID_ITEM_ID;
                                }
                                ResponseType::Error(err) => {
                                    *operation_result = OperationResult::Error(err.clone());
                                }
                                _ => {
                                    *operation_result =
                                        OperationResult::Error("Unknown error".into());
                                }
                            },
                            Err(error) => {
                                *operation_result = OperationResult::Error(if error.is_empty() {
                                    "Request failed".into()
                                } else {
                                    error.to_string()
                                });
                            }
                        }
                    } else {
                        *operation_result = OperationResult::Wait;
                    }
                } else {
                    *operation_result = OperationResult::None;
                    *id = INVALID_ITEM_ID;
                }
            }
        }

        if let Some(removed_id) = removed_id {
            if let Some(data) = &mut self.data {
                data.tunnels.retain(|x| x.id != removed_id);
                data.total_count -= 1;
            }
        }

        !app.can_request(&self.key_get_list) || !app.can_request(&self.key_add_item)
    }
}
