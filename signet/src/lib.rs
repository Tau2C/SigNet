use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Frame {
    Register {
        id: String,
    },
    Open {
        conn_id: String,
        target: String,
        port: u16,
    },
    Data {
        conn_id: String,
        data: String,
    },
    Close {
        conn_id: String,
    },
}
