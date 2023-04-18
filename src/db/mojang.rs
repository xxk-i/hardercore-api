use serde::{Deserialize};
use reqwest;
use base64::{Engine as _, engine::{self, general_purpose}};


#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Skin {
    url: String
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Textures {
    SKIN: Skin
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct DecodedValue {
   timestamp: u64, 
   profileId: String,
   profileName: String,
   textures: Textures
}

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

impl Profile {
    
    // TODO this probably can crash cause not all players have custom skins lol
    pub fn get_skin_url(&self) -> String {
        let encoded = self.properties.get(0).unwrap().value.clone();
        let decoded = general_purpose::STANDARD.decode(encoded).unwrap();

        let decoded_string = String::from_utf8(decoded).unwrap();

        let decoded_value: DecodedValue = serde_json::from_str(&decoded_string).unwrap();
        decoded_value.textures.SKIN.url
    }
}

pub async fn resolve_uuid_to_profile(uuid: &String) -> Result<Profile, Box<dyn std::error::Error>> {
    let url = String::from("https://sessionserver.mojang.com/session/minecraft/profile/");
    let response: Profile = reqwest::get(format!("{}{}", url, uuid))
    .await?
    .json::<Profile>()
    .await?;

    println!("Retrieved profile for {}", response.name);
    
    Ok(response)
}