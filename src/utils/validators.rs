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
pub const ALLOWED_RESPIRATORY_DISEASES: [&str; 4] = ["normal", "covid-19", "pneumonia viral", "pneumonia bacteriana"];
pub const ALLOWED_FEEDBACKS:[&str; 2] = ["sim", "não"];

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

pub fn validate_respiratory_diseases(diseases: &[String; 2]) -> Result<(), AppError> {
    for disease in diseases {
        if !ALLOWED_RESPIRATORY_DISEASES.contains(&disease.as_str()) {
            return Err(AppError::BadRequest(
                format!(
                    "Error: '{}' is not a respiratory diseases. Allowed values are: {}",
                    disease,
                    ALLOWED_RESPIRATORY_DISEASES.join(", ")
                )
            ));
        }
    }
    Ok(())
}

pub fn validate_feedbacks(feedback: &str) -> Result<(), AppError> {
    if !ALLOWED_FEEDBACKS.contains(&feedback) {
        return Err(AppError::BadRequest(
            format!(
                "Error: '{}' is not a valid feedback. Allowed values are: {}",
                feedback,
                ALLOWED_FEEDBACKS.join(", ")
            )
        ));
    }
    Ok(())
}

pub fn is_public_route(path: &str) -> bool {
    let public_routes = [
        "/api/auth/login",
        "/api/auth/login-pronto",
        "/api/users/send-verification-code/",
        "/api/users/resend-verification-code/",
        "/api/users/confirm-verification-code",
        "/api/users/update-password-for-forgetting-user",
        "/api/swagger", // Adiciona a rota do Swagger como pública
    ];

    public_routes.iter().any(|route| path.starts_with(route))
}

pub fn routes_for_users_common(path: &str) -> bool {
    let routes_common = [
        "/api/users/feedback-respiratory-diseases",
        "/api/users/feedback-tuberculosis",
        "/api/users/update-password-by-user-common",
        "/api/prediction/predict",
        "/api/prediction/predict_tb",
        "/api/prediction/detect",
        "/api/data/number-of-appointments-per-month",
        "/api/data/number-of-appointments-per-flow",
        "/api/data/distribuition-of-patients-ages",
        "/api/data/number-of-calls-per-day-of-the-week",
        "/api/data/distribution-of-services-by-hour-group",
        "/api/data/number-of-visits-per-nurse",
        "/api/data/number-of-visits-per-doctor",
        "/api/data/average-time-in-minutes-per-doctor",
        "/api/data/heat-map-with-disease-indications",
        "/api/data/heat-map-with-the-number-of-medical-appointments-by-neighborhood"
    ];

    routes_common.iter().any(|route| path.starts_with(route))

}
