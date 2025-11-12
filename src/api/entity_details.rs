use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};
use std::collections::HashMap;

/// Get entity details and relationships
/// Supports: dependency, service, code_element, security_entity
pub async fn get_entity_details(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<(String, String, String)>, // (repo_id, entity_type, entity_id)
) -> impl Responder {
    let (repo_id, entity_type, entity_id) = path.into_inner();
    
    let mut details: HashMap<String, serde_json::Value> = HashMap::new();
    
    match entity_type.as_str() {
        "dependency" => {
            // Get dependency details
            if let Ok(deps) = state.dep_repo.get_by_repository(&repo_id) {
                if let Some(dep) = deps.iter().find(|d| d.id == entity_id) {
                    details.insert("entity".to_string(), serde_json::to_value(dep).unwrap());
                    
                    // Get related dependencies (same package manager)
                    let related: Vec<_> = deps.iter()
                        .filter(|d| d.package_manager == dep.package_manager && d.id != entity_id)
                        .take(10)
                        .map(|d| serde_json::to_value(d).unwrap())
                        .collect();
                    details.insert("related_dependencies".to_string(), serde_json::json!(related));
                }
            }
        },
        "service" => {
            // Get service details
            if let Ok(services) = state.service_repo.get_by_repository(&repo_id) {
                if let Some(service) = services.iter().find(|s| s.id == entity_id) {
                    details.insert("entity".to_string(), serde_json::to_value(service).unwrap());
                    
                    // Get related services (same provider)
                    let related: Vec<_> = services.iter()
                        .filter(|s| s.provider == service.provider && s.id != entity_id)
                        .take(10)
                        .map(|s| serde_json::to_value(s).unwrap())
                        .collect();
                    details.insert("related_services".to_string(), serde_json::json!(related));
                    
                    // Get API keys that match this service
                    if let Ok(security_entities) = state.security_repo.get_entities(&repo_id) {
                        // Filter for API keys that match this service
                        let matching_api_keys: Vec<_> = security_entities.iter()
                            .filter(|e| {
                                // Only API keys
                                use crate::security::SecurityEntityType;
                                if !matches!(e.entity_type, SecurityEntityType::ApiKey) {
                                    return false;
                                }
                                
                                // Match by provider
                                if e.provider == service.provider {
                                    return true;
                                }
                                
                                // Also check if the API key's related_services contains this service name
                                if let Some(related_services) = e.configuration.get("related_services") {
                                    if let Some(services_array) = related_services.as_array() {
                                        for svc_name in services_array {
                                            if let Some(name_str) = svc_name.as_str() {
                                                if name_str == service.name {
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                false
                            })
                            .take(20)
                            .map(|e| serde_json::to_value(e).unwrap())
                            .collect();
                        details.insert("api_keys".to_string(), serde_json::json!(matching_api_keys));
                    }
                }
            }
        },
        "code_element" => {
            // Get code element details
            if let Ok(elements) = state.code_repo.get_by_repository(&repo_id) {
                if let Some(element) = elements.iter().find(|e| e.id == entity_id) {
                    details.insert("entity".to_string(), serde_json::to_value(element).unwrap());
                    
                    // Get code calls (callers and callees)
                    if let Ok(calls) = state.code_repo.get_calls(&repo_id) {
                        let callers: Vec<_> = calls.iter()
                            .filter(|c| c.callee_id == entity_id)
                            .map(|c| {
                                elements.iter()
                                    .find(|e| e.id == c.caller_id)
                                    .map(|e| serde_json::json!({
                                        "id": e.id,
                                        "name": e.name,
                                        "file_path": e.file_path,
                                        "line_number": c.line_number,
                                        "call_type": c.call_type
                                    }))
                            })
                            .filter_map(|x| x)
                            .collect();
                        details.insert("callers".to_string(), serde_json::json!(callers));
                        
                        let callees: Vec<_> = calls.iter()
                            .filter(|c| c.caller_id == entity_id)
                            .map(|c| {
                                elements.iter()
                                    .find(|e| e.id == c.callee_id)
                                    .map(|e| serde_json::json!({
                                        "id": e.id,
                                        "name": e.name,
                                        "file_path": e.file_path,
                                        "line_number": c.line_number,
                                        "call_type": c.call_type
                                    }))
                            })
                            .filter_map(|x| x)
                            .collect();
                        details.insert("callees".to_string(), serde_json::json!(callees));
                    }
                    
                    // Get related elements (same file)
                    let related: Vec<_> = elements.iter()
                        .filter(|e| e.file_path == element.file_path && e.id != entity_id)
                        .take(10)
                        .map(|e| serde_json::to_value(e).unwrap())
                        .collect();
                    details.insert("related_elements".to_string(), serde_json::json!(related));
                    
                    // Get code relationships (services and dependencies used by this element)
                    if let Ok(relationships) = state.code_relationship_repo.get_by_code_element(&entity_id) {
                        let mut related_services = Vec::new();
                        let mut related_dependencies = Vec::new();
                        
                        for rel in &relationships {
                            match rel.target_type {
                                crate::analysis::RelationshipTargetType::Service => {
                                    if let Ok(services) = state.service_repo.get_by_repository(&repo_id) {
                                        if let Some(service) = services.iter().find(|s| s.id == rel.target_id) {
                                            related_services.push(serde_json::json!({
                                                "id": service.id,
                                                "name": service.name,
                                                "provider": service.provider,
                                                "service_type": service.service_type,
                                                "relationship_type": rel.relationship_type,
                                                "confidence": rel.confidence,
                                                "evidence": rel.evidence
                                            }));
                                        }
                                    }
                                },
                                crate::analysis::RelationshipTargetType::Dependency => {
                                    if let Ok(deps) = state.dep_repo.get_by_repository(&repo_id) {
                                        if let Some(dep) = deps.iter().find(|d| d.id == rel.target_id) {
                                            related_dependencies.push(serde_json::json!({
                                                "id": dep.id,
                                                "name": dep.name,
                                                "version": dep.version,
                                                "package_manager": dep.package_manager,
                                                "relationship_type": rel.relationship_type,
                                                "confidence": rel.confidence,
                                                "evidence": rel.evidence
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                        
                        details.insert("related_services".to_string(), serde_json::json!(related_services));
                        details.insert("related_dependencies".to_string(), serde_json::json!(related_dependencies));
                    }
                }
            }
        },
        "security_entity" => {
            // Get security entity details
            if let Ok(entities) = state.security_repo.get_entities(&repo_id) {
                if let Some(entity) = entities.iter().find(|e| e.id == entity_id) {
                    details.insert("entity".to_string(), serde_json::to_value(entity).unwrap());
                    
                    // Get relationships
                    if let Ok(relationships) = state.security_repo.get_relationships(&repo_id) {
                        let related: Vec<_> = relationships.iter()
                            .filter(|r| r.source_entity_id == entity_id || r.target_entity_id == entity_id)
                            .map(|r| {
                                let related_id = if r.source_entity_id == entity_id {
                                    &r.target_entity_id
                                } else {
                                    &r.source_entity_id
                                };
                                entities.iter()
                                    .find(|e| e.id == *related_id)
                                    .map(|e| serde_json::json!({
                                        "entity": e,
                                        "relationship_type": r.relationship_type,
                                        "permissions": r.permissions,
                                        "condition": r.condition
                                    }))
                            })
                            .filter_map(|x| x)
                            .collect();
                        details.insert("relationships".to_string(), serde_json::json!(related));
                    }
                    
                    // Get vulnerabilities
                    if let Ok(vulnerabilities) = state.security_repo.get_vulnerabilities(&repo_id) {
                        let entity_vulns: Vec<_> = vulnerabilities.iter()
                            .filter(|v| v.entity_id == entity_id)
                            .map(|v| serde_json::to_value(v).unwrap())
                            .collect();
                        details.insert("vulnerabilities".to_string(), serde_json::json!(entity_vulns));
                    }
                    
                    // Get related entities (same provider)
                    let related_entities: Vec<_> = entities.iter()
                        .filter(|e| e.provider == entity.provider && e.id != entity_id)
                        .take(10)
                        .map(|e| serde_json::to_value(e).unwrap())
                        .collect();
                    details.insert("related_entities".to_string(), serde_json::json!(related_entities));
                }
            }
        },
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Unknown entity type: {}", entity_type),
            });
        }
    }
    
    if details.is_empty() {
        HttpResponse::NotFound().json(ErrorResponse {
            error: "Entity not found".to_string(),
        })
    } else {
        HttpResponse::Ok().json(details)
    }
}

