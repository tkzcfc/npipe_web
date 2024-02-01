use crate::proto::PlayerListResponse;
use crate::render::RenderUI;
use crate::resource::ResponseType;
use crate::{proto, TemplateApp};
use eframe::epaint::Color32;
use egui::Ui;
use egui_extras::{Column, TableBuilder};

static PAGE_SIZE: u32 = 10;
static INVALID_ITEM_ID: u32 = u32::MAX;

pub struct Logic {
    key_get_list: String,
    key_remove_item: String,
    key_add_item: String,
    key_update_item: String,

    // 正在更新的id
    update_item_id: u32,

    // 是否正在等待玩家列表数据刷新
    wait_player_list: bool,
    data: Option<PlayerListResponse>,
}

impl Logic {
    pub fn new() -> Self {
        Self {
            key_get_list: "player_list".into(),
            key_remove_item: "remove_player".into(),
            key_add_item: "add_player".into(),
            key_update_item: "update_player".into(),
            wait_player_list: false,
            data: None,
            update_item_id: INVALID_ITEM_ID,
        }
    }
}

impl RenderUI for Logic {
    fn render(&mut self, ctx: &egui::Context, app: &mut TemplateApp) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_content(ui, ctx, app);
        });
    }

    fn reset(&mut self) {
        self.data = None;
        self.update_item_id = INVALID_ITEM_ID;
        self.wait_player_list = false;
    }
}

impl Logic {
    fn render_content(&mut self, ui: &mut Ui, ctx: &egui::Context, app: &mut TemplateApp) {
        let mut need_request = false;
        let mut page_number: usize = 0;
        if let Some(promise) = app.promise_map.get_mut(&self.key_get_list) {
            if let Some(result) = promise.ready_mut() {
                match result {
                    Ok(ref resource) => match &resource.response_data {
                        ResponseType::PlayerListResponse(ref player_list) => {
                            if self.wait_player_list {
                                self.wait_player_list = false;
                                self.data = Some(player_list.clone());
                            }
                        }
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

        // 玩家列表渲染
        if let Some(ref mut player_list) = self.data {
            ui.horizontal(|ui| {
                // 刷新按钮
                if ui.button("refresh").clicked() {
                    need_request = true;
                }

                // 计算页数
                page_number = player_list.cur_page_number as usize;
                let page_count = if player_list.total_count % PAGE_SIZE == 0 {
                    player_list.total_count / PAGE_SIZE
                } else {
                    player_list.total_count / PAGE_SIZE + 1
                };

                // 页数选择
                if page_count > 1
                    && egui::ComboBox::from_label("Page")
                        .selected_text(format!("{}", page_number + 1))
                        .show_index(ui, &mut page_number, page_count as usize, |i| {
                            format!("{}", i + 1)
                        })
                        .changed()
                {
                    need_request = true;
                }
            });
            ui.label(format!("total : {}", player_list.total_count));
            self.render_table(ui, app);
        }

        if need_request {
            let req = proto::PlayerListRequest {
                page_number: page_number as u32,
                page_size: PAGE_SIZE,
            };
            app.http_request(
                ctx,
                &self.key_get_list,
                None,
                serde_json::to_string(&req).unwrap().into(),
            );
            self.wait_player_list = true;
        }
    }

    fn busy(&self, app: &mut TemplateApp) -> bool {
        !app.can_request(&self.key_get_list)
            || !app.can_request(&self.key_remove_item)
            || !app.can_request(&self.key_add_item)
            || !app.can_request(&self.key_update_item)
    }

    fn render_table(&mut self, ui: &mut Ui, app: &mut TemplateApp) {
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
                    ui.strong("name");
                });
                header.col(|ui| {
                    ui.strong("password");
                });
                header.col(|ui| {
                    ui.strong("online");
                });
                header.col(|ui| {
                    ui.strong("update");
                });
            })
            .body(|mut body| {
                if let Some(ref mut player_list) = self.data {
                    for (index, player) in player_list.players.iter_mut().enumerate() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{}", index + 1));
                            });
                            row.col(|ui| {
                                if ui.button("📋").on_hover_text("copy").clicked() {
                                    ui.output_mut(|o| o.copied_text = format!("{}", player.id));
                                }
                                ui.label(format!("{}", player.id));
                            });
                            row.col(|ui| {
                                if ui.button("📋").on_hover_text("copy").clicked() {
                                    ui.output_mut(|o| o.copied_text = player.username.clone());
                                }
                                ui.label(player.username.as_str());
                            });
                            row.col(|ui| {
                                // ui.add(password(&mut player.password));
                                ui.text_edit_singleline(&mut player.password);
                            });
                            row.col(|ui| {
                                if player.online {
                                    ui.colored_label(Color32::GREEN, "online");
                                } else {
                                    ui.colored_label(ui.visuals().error_fg_color, "offline");
                                }
                            });
                            row.col(|ui| {
                                // self.render_item_update(ui, player.id, app);
                            });
                        });
                    }
                }
            });

        // if no_update_item && self.update_item_id != INVALID_ITEM_ID {
        //
        // }
    }

    fn render_item_update(&self, ui: &mut Ui, id: u32, app: &mut TemplateApp) -> u32 {
        let mut need_request = false;
        if self.update_item_id == INVALID_ITEM_ID {
            if ui.button("update").clicked() {
                need_request = true;
            }
        } else {
            if self.update_item_id != id {
                return self.update_item_id;
            }

            let value = app.promise_map.get(&self.key_update_item);
            if value.is_none() {
                return INVALID_ITEM_ID;
            }
            let promise = value.unwrap();
            if let Some(result) = promise.ready() {
                match result {
                    Ok(resource) => match &resource.response_data {
                        ResponseType::GeneralResponse(_) => {
                            // self.update_item_id = INVALID_ITEM_ID;
                        }
                        ResponseType::Error(err) => {
                            ui.horizontal(|ui| {
                                if ui.button("retry").clicked() {
                                    need_request = true;
                                }
                                ui.colored_label(ui.visuals().error_fg_color, err);
                            });
                        }
                        _ => {
                            ui.horizontal(|ui| {
                                if ui.button("retry").clicked() {
                                    need_request = true;
                                }
                                ui.colored_label(ui.visuals().error_fg_color, "Unknown error");
                            });
                        }
                    },
                    Err(error) => {
                        ui.horizontal(|ui| {
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
                        });
                    }
                }
            } else {
                ui.spinner();
            }
        }
        self.update_item_id
    }
}
