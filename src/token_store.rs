use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn token_dir() -> io::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "No pude encontrar el HOME del usuario")
    })?;

    Ok(home.join(".moltbot"))
}

pub fn token_path() -> io::Result<PathBuf> {
    Ok(token_dir()?.join("token.txt"))
}

pub fn save_token(token: &str) -> io::Result<()> {
    let dir = token_dir()?;
    fs::create_dir_all(&dir)?;

    let path = dir.join("token.txt");
    fs::write(&path, token.trim())?;

    // (Opcional) permisos seguros en Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perm = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perm)?;
    }

    Ok(())
}

pub fn load_token() -> io::Result<Option<String>> {
    let path = token_path()?;
    if !Path::new(&path).exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let t = raw.trim().to_string();
    if t.is_empty() { Ok(None) } else { Ok(Some(t)) }
}

pub fn delete_token() -> io::Result<()> {
    let path = token_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
