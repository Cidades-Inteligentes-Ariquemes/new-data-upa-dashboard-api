use md5;

pub fn verify_pronto_password(input_password: &str, stored_password: &str) -> bool {
    let hashed_password = format!("{:x}", md5::compute(input_password)).to_uppercase();
    hashed_password == stored_password
}

pub fn has_doctor_profile(profiles: &[crate::domain::models::auth_pronto::ProfileInfo]) -> bool {
    for profile in profiles {
        if profile.perfil_nome.to_uppercase() == "MEDICO" {
            return true;
        }
    }
    false
}
