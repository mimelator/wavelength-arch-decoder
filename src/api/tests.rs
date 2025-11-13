use actix_web::{web, HttpResponse, Responder};
use crate::api::{ApiState, ErrorResponse};

/// Get all tests for a repository
pub async fn get_tests(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> impl Responder {
    let repository_id = path.into_inner();
    
    log::debug!("Fetching tests for repository: {}", repository_id);
    
    match state.test_repo.get_by_repository(&repository_id) {
        Ok(tests) => {
            log::debug!("Found {} test(s) for repository {}", tests.len(), repository_id);
            HttpResponse::Ok().json(tests)
        }
        Err(e) => {
            log::error!("Failed to fetch tests: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to fetch tests: {}", e),
            })
        }
    }
}

/// Get tests filtered by framework
pub async fn get_tests_by_framework(
    state: web::Data<ApiState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (repository_id, framework) = path.into_inner();
    
    log::debug!("Fetching tests for repository: {} with framework: {}", repository_id, framework);
    
    match state.test_repo.get_by_framework(&repository_id, &framework) {
        Ok(tests) => {
            log::debug!("Found {} test(s) for repository {} with framework {}", tests.len(), repository_id, framework);
            HttpResponse::Ok().json(tests)
        }
        Err(e) => {
            log::error!("Failed to fetch tests: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to fetch tests: {}", e),
            })
        }
    }
}

