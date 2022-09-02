use platform_dirs::AppDirs;

pub fn get_app_dirs() -> AppDirs {
    platform_dirs::AppDirs::new("Wgg".into(), false).expect("Couldn't find a home directory for config!")
}