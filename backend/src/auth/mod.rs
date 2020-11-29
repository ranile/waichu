mod jwt;
mod routes;

pub use jwt::*;
pub use routes::auth as routes;

const BCRYPT_COST: u32 = 12;
