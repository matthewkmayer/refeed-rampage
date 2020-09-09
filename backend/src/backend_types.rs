use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ErrorResp {
    pub error: String,
}

#[derive(Serialize, Debug)]
pub struct Health {
    pub healthy: bool,
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct Login {
    pub user: String,
    pub pw: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: u32, // Required (validate_exp defaults to true in validation). Expiration time
    pub sub: String, // Optional. Subject (whom token refers to)
}

#[derive(Debug, Serialize)]
pub struct LoginResp {
    pub jwt: String,
}
