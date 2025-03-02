use dotenv_codegen::dotenv;

pub const TOASTER_API_BASE: &str = dotenv!("TOASTER_API_BASE");
pub const TOASTER_API_KEY: &str = dotenv!("TOASTER_API_KEY");
pub const TOASTER_API_MODEL: &str = dotenv!("TOASTER_API_MODEL");

#[cfg(target_family = "unix")]
pub const TOASTER_DATA_PATH: &str = "~/.config/toasterai/data/root";
#[cfg(target_family = "unix")]
pub const TOASTER_TMP_DATA_PATH: &str = "~/.config/toasterai/data/tmp";
