use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rol {
    User,
    Admin,
}

fn admin_token() -> Option<String> {
    env::var("CLAWDBOT_ADMIN_TOKEN").ok()
}

fn user_token() -> Option<String> {
    env::var("CLAWDBOT_USER_TOKEN").ok()
}

pub fn rol(token: &str) -> Option<Rol> {
    let t = token.trim();

    let admin = admin_token();
    let user = user_token();


    if let Some(a) = admin {
        if t == a {
            return Some(Rol::Admin);
        }
    }

    if let Some(u) = user {
        if t == u {
            return Some(Rol::User);
        }
    }

    None
}
