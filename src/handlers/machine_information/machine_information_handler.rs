use actix_web::{web, HttpResponse};
use crate::{
    application::machine_information_service::MachineInformationService,
    AppError,
};

pub async fn get_machine_information(
    service: web::Data<MachineInformationService>
) -> Result<HttpResponse, AppError> {
    service.get_machine_information().await
}