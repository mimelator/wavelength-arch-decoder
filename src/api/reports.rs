use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};
use crate::report::ReportGenerator;
use crate::graph::GraphBuilder;

/// Generate HTML report for a repository
pub async fn generate_report(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let repository_id = path.into_inner();
    
    // Create graph builder
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
        state.tool_repo.clone(),
        state.code_relationship_repo.clone(),
        state.test_repo.clone(),
        state.port_repo.clone(),
        state.endpoint_repo.clone(),
    );
    
    // Create report generator
    let report_generator = ReportGenerator::new(
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
        state.code_repo.clone(),
        state.code_relationship_repo.clone(),
        state.security_repo.clone(),
        state.tool_repo.clone(),
        state.port_repo.clone(),
        state.endpoint_repo.clone(),
        graph_builder,
    );
    
    match report_generator.generate_html_report(&repository_id) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html")
            .body(html),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to generate report: {}", e),
        }),
    }
}

