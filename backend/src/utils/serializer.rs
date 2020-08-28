use uuid::Uuid;
use serde::Serializer;

pub fn serialize_to_uuid<T, S>(field: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: GetUuid {
    return s.serialize_str(&field.uuid().to_string());
}

pub trait GetUuid {
    fn uuid(&self) -> Uuid;
}
