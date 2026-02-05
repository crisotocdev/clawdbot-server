use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rol {
    User,
    Admin,
}

fn admin_token() -> Option<String> {
    env::var("MOLTBOT_ADMIN_TOKEN").ok()
}

fn user_token() -> Option<String> {
    env::var("MOLTBOT_USER_TOKEN").ok()
}

pub fn rol(token: &str) -> Option<Rol> {
    let t = token.trim();

    // Evita aceptar tokens vacíos o solo espacios
    if t.is_empty() {
        return None;
    }

    if let Some(a) = admin_token() {
        if t == a.trim() {
            return Some(Rol::Admin);
        }
    }

    if let Some(u) = user_token() {
        if t == u.trim() {
            return Some(Rol::User);
        }
    }

    None
}
