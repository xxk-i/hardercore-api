use std::collections::HashMap;
use super::mojang::{Profile, self};

pub struct ProfileCache {
    uuid_profile_map: HashMap<String, Profile>
}

impl ProfileCache {
    pub fn new() -> Self {
        ProfileCache { uuid_profile_map: HashMap::new() }
    }

    pub async fn get(&mut self, uuid: String) -> &Profile {
        if !self.uuid_profile_map.contains_key(&uuid) {
            let profile = mojang::resolve_uuid_to_profile(&uuid).await.unwrap();
            self.uuid_profile_map.insert(uuid.clone(), profile);
            &self.uuid_profile_map.get(&uuid).unwrap()
        }

        else {
            &self.uuid_profile_map.get(&uuid).unwrap()
        }
    }
}