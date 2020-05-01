use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct Image {
    pub location: String,
    pub id: String,
    pub is_loaded: bool,
}

impl Image {
    pub fn new(id: String, location: String) -> Image {
        Image {
            location,
            id,
            is_loaded: false,
        }
    }
}

impl Clone for Image {
    fn clone(&self) -> Image {
        Image::new(self.id.clone(), self.location.clone())
    }
}

// ================================================================================================
// == Serde Serialization for parsing input files.                                               ==
// ================================================================================================
#[derive(Serialize, Deserialize)]
struct SerializableImage {
    name: String,
    location: String,
}

impl Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializableImage {
            name: self.id.clone(),
            location: self.location.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|SerializableImage { name, location }| Image::new(name, location))
    }
}
