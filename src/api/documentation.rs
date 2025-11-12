use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::api::{ApiState, ErrorResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationResponse {
    pub id: String,
    pub repository_id: String,
    pub file_path: String,
    pub file_name: String,
    pub doc_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content_preview: String,
    pub word_count: usize,
    pub line_count: usize,
    pub has_code_examples: bool,
    pub has_api_references: bool,
    pub has_diagrams: bool,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

pub async fn get_documentation(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let repository_id = path.into_inner();
    
    match state.documentation_repo.get_by_repository(&repository_id) {
        Ok(docs) => {
            let response: Vec<DocumentationResponse> = docs.into_iter().map(|d| {
                DocumentationResponse {
                    id: d.id,
                    repository_id: d.repository_id,
                    file_path: d.file_path,
                    file_name: d.file_name,
                    doc_type: d.doc_type,
                    title: d.title,
                    description: d.description,
                    content_preview: d.content_preview,
                    word_count: d.word_count,
                    line_count: d.line_count,
                    has_code_examples: d.has_code_examples,
                    has_api_references: d.has_api_references,
                    has_diagrams: d.has_diagrams,
                    metadata: d.metadata,
                    created_at: d.created_at,
                }
            }).collect();
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            log::error!("Failed to get documentation: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get documentation: {}", e),
            })
        }
    }
}

pub async fn get_documentation_by_type(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (repository_id, doc_type) = path.into_inner();
    
    match state.documentation_repo.get_by_type(&repository_id, &doc_type) {
        Ok(docs) => {
            let response: Vec<DocumentationResponse> = docs.into_iter().map(|d| {
                DocumentationResponse {
                    id: d.id,
                    repository_id: d.repository_id,
                    file_path: d.file_path,
                    file_name: d.file_name,
                    doc_type: d.doc_type,
                    title: d.title,
                    description: d.description,
                    content_preview: d.content_preview,
                    word_count: d.word_count,
                    line_count: d.line_count,
                    has_code_examples: d.has_code_examples,
                    has_api_references: d.has_api_references,
                    has_diagrams: d.has_diagrams,
                    metadata: d.metadata,
                    created_at: d.created_at,
                }
            }).collect();
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            log::error!("Failed to get documentation by type: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get documentation: {}", e),
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn search_documentation(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<SearchQuery>,
) -> impl Responder {
    let repository_id = path.into_inner();
    let search_query = query.into_inner().q;
    
    if search_query.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Search query cannot be empty".to_string(),
        });
    }
    
    match state.documentation_repo.search(&repository_id, &search_query) {
        Ok(docs) => {
            let response: Vec<DocumentationResponse> = docs.into_iter().map(|d| {
                DocumentationResponse {
                    id: d.id,
                    repository_id: d.repository_id,
                    file_path: d.file_path,
                    file_name: d.file_name,
                    doc_type: d.doc_type,
                    title: d.title,
                    description: d.description,
                    content_preview: d.content_preview,
                    word_count: d.word_count,
                    line_count: d.line_count,
                    has_code_examples: d.has_code_examples,
                    has_api_references: d.has_api_references,
                    has_diagrams: d.has_diagrams,
                    metadata: d.metadata,
                    created_at: d.created_at,
                }
            }).collect();
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            log::error!("Failed to search documentation: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to search documentation: {}", e),
            })
        }
    }
}

