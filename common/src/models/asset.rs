use chrono::{DateTime, Utc};
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    #[serde(with = "message_type_serializer_deserializer")]
    pub mime: Mime,
    pub uuid: Uuid,
    #[allow(clippy::rc_buffer)] // this is an arc so i don't clean the actual bytes a billion times
    #[serde(skip)]
    pub bytes: Arc<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

impl Asset {
    pub fn new(bytes: Vec<u8>, mime: Mime) -> Self {
        Self {
            mime,
            uuid: Uuid::new_v4(),
            bytes: Arc::new(bytes),
            created_at: Utc::now(),
        }
    }
}

mod message_type_serializer_deserializer {
    use super::Mime;
    use serde::export::Formatter;
    use serde::{de, Deserializer, Serializer};
    use std::fmt;
    use std::str::FromStr;

    struct MimeVisitor;

    impl<'de> de::Visitor<'de> for MimeVisitor {
        type Value = Mime;

        fn expecting(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "a valid message type")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Mime::from_str(v).map_err(|e| de::Error::custom(e.to_string()))
        }
    }

    pub fn serialize<S>(input: &Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(input.to_string().as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mime, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(MimeVisitor)
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
