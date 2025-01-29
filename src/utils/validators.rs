use::regex::Regex;
use lazy_static::lazy_static;
use crate::AppError;

// Validação de e-mail
lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
}

pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

// Validação campos

pub const ALLOWED_PROFILES: [&str; 2] = ["Administrador", "Usuario Comum"];
pub const ALLOWED_APPS: [&str; 2] = ["xpredict", "upavision"];

pub fn validate_profile(profile: &str) -> Result<(), AppError> {
    if !ALLOWED_PROFILES.contains(&profile) {
        return Err(AppError::BadRequest(
            format!(
                "Error: '{}' is not a valid profile. Allowed values are: {}",
                profile,
                ALLOWED_PROFILES.join(", ")
            )
        ));
    }
    Ok(())
}

pub fn validate_applications(applications: &[String]) -> Result<(), AppError> {
    for app in applications {
        if !ALLOWED_APPS.contains(&app.as_str()) {
            return Err(AppError::BadRequest(
                format!(
                    "Error: '{}' is not a valid application. Allowed values are: {}", 
                    app,
                    ALLOWED_APPS.join(", ")
                )
            ));
        }
    }
    Ok(())
}