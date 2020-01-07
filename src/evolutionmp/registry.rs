use winreg::RegKey;
use std::path::{Path, PathBuf};

pub struct Registry {
    is_steam: bool,
    install_folder: PathBuf,
    game_type: String,
    game_version: String,
    language: String,
    patch_version: String
}

impl Registry {
    pub fn read() -> Option<Registry> {
        let user_key = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        let rockstar_key = user_key.open_subkey("SOFTWARE\\Wow6432Node\\Rockstar Games").ok()?;
        if let Some(gta_key) = rockstar_key.open_subkey("GTAV").ok() {
            Some(Registry {
                is_steam: true,
                install_folder: PathBuf::from(&gta_key.get_value::<String, &str>("InstallFolderSteam").ok()?[4..]),
                game_type: "".to_string(),
                game_version: "".to_string(),
                language: "".to_string(),
                patch_version: "".to_string()
            })
        } else if let Some(gta_key) = rockstar_key.open_subkey("Grand Theft Auto V").ok() {
            Some(Registry {
                is_steam: false,
                install_folder: PathBuf::from(gta_key.get_value::<String, &str>("InstallFolder").ok()?),
                game_type: gta_key.get_value("Game Type").ok()?,
                game_version: gta_key.get_value("Game Version").ok()?,
                language: gta_key.get_value("Language").ok()?,
                patch_version: gta_key.get_value("PatchVersion").ok()?
            })
        } else {
            None
        }
    }

    pub fn is_retail_key(&self) -> bool {
        !self.is_steam
    }

    pub fn is_steam_key(&self) -> bool {
        self.is_steam
    }

    pub fn get_install_path(&self) -> PathBuf {
        self.install_folder.clone()
    }

    pub fn get_language(&self) -> &String {
        &self.language
    }
}