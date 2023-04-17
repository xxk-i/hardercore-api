use serde::{Deserialize};
use serde_json;
use reqwest;

#[derive(Deserialize, Debug)]
pub struct Properties {
    pub name: String,
    pub value: String,
    pub signature: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub properties: Vec<Properties>,
}

pub async fn resolve_uuid_to_profile(uuid: &String) -> Result<Profile, Box<dyn std::error::Error>> {
    let URL = String::from("https://sessionserver.mojang.com/session/minecraft/profile/");
    let response: Profile = reqwest::get(format!("{}{}", URL, uuid))
    .await?
    .json::<Profile>()
    .await?;
    
    Ok(response)
}