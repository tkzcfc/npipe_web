use crate::proto;

pub enum ResponseType {
    Error(String),
    GeneralResponse(proto::GeneralResponse),
    PlayerListResponse(proto::PlayerListResponse),
    TunnelListResponse(proto::TunnelListResponse),
}

pub struct Resource {
    /// HTTP response
    pub(crate) response: ehttp::Response,
    pub(crate) checked: bool,
    pub(crate) response_data: ResponseType,
}

impl Resource {
    pub(crate) fn from_response(
        _ctx: &egui::Context,
        response: ehttp::Response,
        path: String,
    ) -> Self {
        let response_data = if response.ok {
            match path.as_str() {
                "player_list" => {
                    match serde_json::from_slice::<proto::PlayerListResponse>(&response.bytes) {
                        Ok(data) => ResponseType::PlayerListResponse(data),
                        Err(err) => {
                            ResponseType::Error(format!("json decode: {}", err.to_string()))
                        }
                    }
                }
                "tunnel_list" => {
                    match serde_json::from_slice::<proto::TunnelListResponse>(&response.bytes) {
                        Ok(data) => ResponseType::TunnelListResponse(data),
                        Err(err) => {
                            ResponseType::Error(format!("json decode: {}", err.to_string()))
                        }
                    }
                }
                _ => match serde_json::from_slice::<proto::GeneralResponse>(&response.bytes) {
                    Ok(data) => {
                        if data.code == 0 {
                            ResponseType::GeneralResponse(data)
                        } else {
                            ResponseType::Error(format!("code:{} ({})", data.code, data.msg))
                        }
                    }
                    Err(err) => ResponseType::Error(format!("json decode: {}", err.to_string())),
                },
            }
        } else {
            if let Some(text) = response.text() {
                ResponseType::Error(format!(
                    "status:{} ({})\nerror:        {}",
                    response.status, response.status_text, text
                ))
            } else {
                ResponseType::Error(format!(
                    "status:{} ({})",
                    response.status, response.status_text
                ))
            }
        };

        Self {
            response,
            checked: false,
            response_data,
        }
    }
}
