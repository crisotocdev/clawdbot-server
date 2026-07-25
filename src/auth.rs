use std::env;

const ADMIN_TOKEN_ENV: &str = "MOLTBOT_ADMIN_TOKEN";
const USER_TOKEN_ENV: &str = "MOLTBOT_USER_TOKEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rol {
    User,
    Admin,
}

/// Lee una variable de entorno, elimina espacios externos
/// y rechaza valores vacíos.
fn read_token(variable_name: &str) -> Option<String> {
    match env::var(variable_name) {
        Ok(value) => {
            let cleaned = value.trim();

            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned.to_string())
            }
        }
        Err(_) => None,
    }
}

fn admin_token() -> Option<String> {
    read_token(ADMIN_TOKEN_ENV)
}

fn user_token() -> Option<String> {
    read_token(USER_TOKEN_ENV)
}

/// Determina el rol asociado al token recibido.
///
/// No imprime los valores reales de los tokens.
/// En modo desarrollo solo muestra longitudes y coincidencias.
pub fn rol(token: &str) -> Option<Rol> {
    let received_token = token.trim();

    if received_token.is_empty() {
        #[cfg(debug_assertions)]
        println!("AUTH: token recibido vacío");

        return None;
    }

    let admin = admin_token();
    let user = user_token();

    let admin_matches = admin
        .as_deref()
        .is_some_and(|stored_token| stored_token == received_token);

    let user_matches = user
        .as_deref()
        .is_some_and(|stored_token| stored_token == received_token);

    #[cfg(debug_assertions)]
    {
        println!(
            "AUTH: admin cargado={}, longitud={}, coincide={}",
            admin.is_some(),
            admin.as_deref().map_or(0, str::len),
            admin_matches
        );

        println!(
            "AUTH: user cargado={}, longitud={}, coincide={}",
            user.is_some(),
            user.as_deref().map_or(0, str::len),
            user_matches
        );

        println!("AUTH: token recibido longitud={}", received_token.len());
    }

    if admin_matches {
        #[cfg(debug_assertions)]
        println!("AUTH: acceso concedido como ADMIN");

        return Some(Rol::Admin);
    }

    if user_matches {
        #[cfg(debug_assertions)]
        println!("AUTH: acceso concedido como USER");

        return Some(Rol::User);
    }

    #[cfg(debug_assertions)]
    println!("AUTH: token no coincide con ninguno de los configurados");

    None
}
