use anyhow::Result;
use chrono::Utc;
use crate::storage::{
    RepositoryRepository, DependencyRepository, ServiceRepository,
    CodeElementRepository, CodeRelationshipRepository, SecurityRepository,
    ToolRepository, Repository, StoredDependency, StoredService,
};
use crate::graph::GraphBuilder;

pub struct ReportGenerator {
    repo_repo: RepositoryRepository,
    dep_repo: DependencyRepository,
    service_repo: ServiceRepository,
    code_repo: CodeElementRepository,
    code_relationship_repo: CodeRelationshipRepository,
    security_repo: SecurityRepository,
    tool_repo: ToolRepository,
    graph_builder: GraphBuilder,
}

impl ReportGenerator {
    pub fn new(
        repo_repo: RepositoryRepository,
        dep_repo: DependencyRepository,
        service_repo: ServiceRepository,
        code_repo: CodeElementRepository,
        code_relationship_repo: CodeRelationshipRepository,
        security_repo: SecurityRepository,
        tool_repo: ToolRepository,
        graph_builder: GraphBuilder,
    ) -> Self {
        ReportGenerator {
            repo_repo,
            dep_repo,
            service_repo,
            code_repo,
            code_relationship_repo,
            security_repo,
            tool_repo,
            graph_builder,
        }
    }

    pub fn generate_html_report(&self, repository_id: &str) -> Result<String> {
        // Get repository
        let repo = self.repo_repo.find_by_id(repository_id)?
            .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

        // Get all data
        let dependencies = self.dep_repo.get_by_repository(repository_id)?;
        let services = self.service_repo.get_by_repository(repository_id)?;
        let code_elements = self.code_repo.get_by_repository(repository_id)?;
        let code_relationships = self.code_relationship_repo.get_by_repository(repository_id)?;
        let security_entities = self.security_repo.get_entities(repository_id)?;
        let security_vulnerabilities = self.security_repo.get_vulnerabilities(repository_id)?;
        let tools = self.tool_repo.get_tools_by_repository(repository_id)?;

        // Get graph statistics
        let graph = self.graph_builder.build_for_repository(repository_id)?;
        let graph_stats = graph.get_statistics();

        // Generate HTML report
        let html = self.generate_html(
            &repo,
            &dependencies,
            &services,
            &code_elements,
            &code_relationships,
            &security_entities,
            &security_vulnerabilities,
            &tools,
            &graph,
            &graph_stats,
        )?;

        Ok(html)
    }

    fn generate_html(
        &self,
        repo: &Repository,
        dependencies: &[StoredDependency],
        services: &[StoredService],
        code_elements: &[crate::analysis::CodeElement],
        code_relationships: &[crate::analysis::CodeRelationship],
        security_entities: &[crate::security::SecurityEntity],
        security_vulnerabilities: &[crate::security::SecurityVulnerability],
        tools: &[crate::storage::tool_repo::StoredTool],
        graph: &crate::graph::graph::KnowledgeGraph,
        graph_stats: &crate::graph::graph::GraphStatistics,
    ) -> Result<String> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        
        // Group dependencies by package manager
        let mut deps_by_manager: std::collections::HashMap<String, Vec<&StoredDependency>> = std::collections::HashMap::new();
        for dep in dependencies {
            deps_by_manager
                .entry(dep.package_manager.clone())
                .or_insert_with(Vec::new)
                .push(dep);
        }

        // Group services by provider
        let mut services_by_provider: std::collections::HashMap<String, Vec<&StoredService>> = std::collections::HashMap::new();
        for service in services {
            services_by_provider
                .entry(service.provider.clone())
                .or_insert_with(Vec::new)
                .push(service);
        }

        // Group security entities by type
        let mut security_by_type: std::collections::HashMap<String, Vec<&crate::security::SecurityEntity>> = std::collections::HashMap::new();
        for entity in security_entities {
            let type_str = format!("{:?}", entity.entity_type);
            security_by_type
                .entry(type_str)
                .or_insert_with(Vec::new)
                .push(entity);
        }

        // Count code elements by type
        let mut code_by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for element in code_elements {
            let type_str = format!("{:?}", element.element_type);
            *code_by_type.entry(type_str).or_insert(0) += 1;
        }

        let mut html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Architecture Report: {}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 40px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            border-radius: 8px;
        }}
        h1 {{
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
            margin-bottom: 30px;
        }}
        h2 {{
            color: #34495e;
            margin-top: 40px;
            margin-bottom: 20px;
            border-left: 4px solid #3498db;
            padding-left: 15px;
        }}
        h3 {{
            color: #555;
            margin-top: 25px;
            margin-bottom: 15px;
        }}
        .metadata {{
            background: #ecf0f1;
            padding: 20px;
            border-radius: 5px;
            margin-bottom: 30px;
        }}
        .metadata p {{
            margin: 5px 0;
        }}
        .stat-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        .stat-card {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }}
        .stat-card h3 {{
            color: white;
            font-size: 2em;
            margin: 0;
        }}
        .stat-card p {{
            margin-top: 10px;
            opacity: 0.9;
        }}
        .section {{
            margin-bottom: 40px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
            background: white;
        }}
        th {{
            background: #34495e;
            color: white;
            padding: 12px;
            text-align: left;
        }}
        td {{
            padding: 10px;
            border-bottom: 1px solid #ddd;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        .badge {{
            display: inline-block;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.85em;
            font-weight: bold;
        }}
        .badge-primary {{
            background: #3498db;
            color: white;
        }}
        .badge-success {{
            background: #27ae60;
            color: white;
        }}
        .badge-warning {{
            background: #f39c12;
            color: white;
        }}
        .badge-danger {{
            background: #e74c3c;
            color: white;
        }}
        .vulnerability {{
            background: #fee;
            border-left: 4px solid #e74c3c;
            padding: 15px;
            margin: 10px 0;
            border-radius: 4px;
        }}
        .footer {{
            margin-top: 50px;
            padding-top: 20px;
            border-top: 2px solid #ecf0f1;
            text-align: center;
            color: #7f8c8d;
            font-size: 0.9em;
        }}
        .group-header {{
            background: #ecf0f1;
            padding: 10px 15px;
            font-weight: bold;
            margin-top: 20px;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 Architecture Report: {}</h1>
        
        <div class="metadata">
            <p><strong>Repository URL:</strong> {}</p>
            <p><strong>Branch:</strong> {}</p>
            <p><strong>Last Analyzed:</strong> {}</p>
            <p><strong>Report Generated:</strong> {}</p>
        </div>

        <div class="stat-grid">
            <div class="stat-card">
                <h3>{}</h3>
                <p>Dependencies</p>
            </div>
            <div class="stat-card">
                <h3>{}</h3>
                <p>Services</p>
            </div>
            <div class="stat-card">
                <h3>{}</h3>
                <p>Code Elements</p>
            </div>
            <div class="stat-card">
                <h3>{}</h3>
                <p>Security Entities</p>
            </div>
            <div class="stat-card">
                <h3>{}</h3>
                <p>Tools</p>
            </div>
            <div class="stat-card">
                <h3>{}</h3>
                <p>Graph Nodes</p>
            </div>
            <div class="stat-card">
                <h3>{}</h3>
                <p>Graph Edges</p>
            </div>
            <div class="stat-card">
                <h3>{}</h3>
                <p>Vulnerabilities</p>
            </div>
        </div>

        <div class="section">
            <h2>📦 Dependencies</h2>
            <p>Total dependencies found: <strong>{}</strong></p>
"#,
            repo.name,
            repo.name,
            repo.url,
            repo.branch,
            repo.last_analyzed_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "Never".to_string()),
            now,
            dependencies.len(),
            services.len(),
            code_elements.len(),
            security_entities.len(),
            tools.len(),
            graph.nodes.len(),
            graph.edges.len(),
            security_vulnerabilities.len(),
            dependencies.len(),
        );

        // Add dependencies grouped by package manager
        let mut deps_html = String::new();
        for (manager, deps) in deps_by_manager.iter() {
            deps_html.push_str(&format!(
                r#"
            <div class="group-header">{} ({})</div>
            <table>
                <thead>
                    <tr>
                        <th>Package Name</th>
                        <th>Version</th>
                        <th>Type</th>
                    </tr>
                </thead>
                <tbody>
"#,
                manager, deps.len()
            ));
            for dep in deps.iter().take(50) {
                deps_html.push_str(&format!(
                    r#"                    <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td><span class="badge badge-primary">{}</span></td>
                </tr>
"#,
                    dep.name,
                    dep.version,
                    if dep.is_dev { "dev" } else { "prod" }
                ));
            }
            if deps.len() > 50 {
                deps_html.push_str(&format!(
                    r#"                    <tr><td colspan="3"><em>... and {} more</em></td></tr>"#,
                    deps.len() - 50
                ));
            }
            deps_html.push_str("                </tbody>\n            </table>\n");
        }
        html.push_str(&deps_html);

        // Add services
        html.push_str(&format!(
            r#"
        </div>

        <div class="section">
            <h2>🔌 Services</h2>
            <p>Total services found: <strong>{}</strong></p>
"#,
            services.len()
        ));

        let mut services_html = String::new();
        for (provider, svcs) in services_by_provider.iter() {
            services_html.push_str(&format!(
                r#"
            <div class="group-header">{} ({})</div>
            <table>
                <thead>
                    <tr>
                        <th>Service Name</th>
                        <th>Type</th>
                        <th>Found In</th>
                    </tr>
                </thead>
                <tbody>
"#,
                provider, svcs.len()
            ));
            for service in svcs.iter().take(30) {
                services_html.push_str(&format!(
                    r#"                    <tr>
                        <td>{}</td>
                        <td><span class="badge badge-success">{}</span></td>
                        <td><code>{}</code></td>
                    </tr>
"#,
                    service.name,
                    service.service_type,
                    service.file_path
                ));
            }
            if svcs.len() > 30 {
                services_html.push_str(&format!(
                    r#"                    <tr><td colspan="3"><em>... and {} more</em></td></tr>"#,
                    svcs.len() - 30
                ));
            }
            services_html.push_str("                </tbody>\n            </table>\n");
        }
        html.push_str(&services_html);

        // Add code structure
        html.push_str(&format!(
            r#"
        </div>

        <div class="section">
            <h2>📝 Code Structure</h2>
            <p>Total code elements found: <strong>{}</strong></p>
"#,
            code_elements.len()
        ));

        let mut code_html = String::new();
        code_html.push_str("<table><thead><tr><th>Type</th><th>Count</th></tr></thead><tbody>");
        for (element_type, count) in code_by_type.iter() {
            code_html.push_str(&format!(
                r#"<tr><td>{}</td><td><strong>{}</strong></td></tr>"#,
                element_type, count
            ));
        }
        code_html.push_str("</tbody></table>");

        if !code_relationships.is_empty() {
            code_html.push_str(&format!(
                r#"
            <h3>Code Relationships</h3>
            <p>Total relationships found: <strong>{}</strong></p>
            <p><em>Code elements are automatically linked to services and dependencies they use.</em></p>
"#,
                code_relationships.len()
            ));
        }
        html.push_str(&code_html);

        // Add security
        html.push_str(&format!(
            r#"
        </div>

        <div class="section">
            <h2>🔒 Security Analysis</h2>
            <p>Total security entities found: <strong>{}</strong></p>
"#,
            security_entities.len()
        ));

        let mut security_html = String::new();
        for (entity_type, entities) in security_by_type.iter() {
            security_html.push_str(&format!(
                r#"
            <div class="group-header">{} ({})</div>
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Provider</th>
                        <th>File</th>
                    </tr>
                </thead>
                <tbody>
"#,
                entity_type, entities.len()
            ));
            for entity in entities.iter().take(20) {
                security_html.push_str(&format!(
                    r#"                    <tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td><code>{}</code></td>
                    </tr>
"#,
                    entity.name,
                    &entity.provider,
                    entity.file_path
                ));
            }
            if entities.len() > 20 {
                security_html.push_str(&format!(
                    r#"                    <tr><td colspan="3"><em>... and {} more</em></td></tr>"#,
                    entities.len() - 20
                ));
            }
            security_html.push_str("                </tbody>\n            </table>\n");
        }
        html.push_str(&security_html);

        // Add vulnerabilities
        if !security_vulnerabilities.is_empty() {
            html.push_str(&format!(
                r#"
            <h3>⚠️ Security Vulnerabilities</h3>
            <p>Found <strong>{}</strong> potential security issues:</p>
"#,
                security_vulnerabilities.len()
            ));
            for vuln in security_vulnerabilities.iter().take(20) {
                html.push_str(&format!(
                    r#"
            <div class="vulnerability">
                <strong>{}</strong><br>
                <em>Severity: {}</em><br>
                <code>{}</code><br>
                <p>{}</p>
            </div>
"#,
                    vuln.vulnerability_type,
                    format!("{:?}", vuln.severity),
                    vuln.file_path,
                    vuln.description
                ));
            }
            if security_vulnerabilities.len() > 20 {
                html.push_str(&format!(
                    r#"<p><em>... and {} more vulnerabilities</em></p>"#,
                    security_vulnerabilities.len() - 20
                ));
            }
        }

        // Add tools
        html.push_str(&format!(
            r#"
        </div>

        <div class="section">
            <h2>🛠️ Developer Tools</h2>
            <p>Total tools found: <strong>{}</strong></p>
            <table>
                <thead>
                    <tr>
                        <th>Tool Name</th>
                        <th>Type</th>
                        <th>Found In</th>
                    </tr>
                </thead>
                <tbody>
"#,
            tools.len()
        ));
        for tool in tools.iter().take(50) {
            html.push_str(&format!(
                r#"                    <tr>
                        <td>{}</td>
                        <td><span class="badge badge-warning">{}</span></td>
                        <td><code>{}</code></td>
                    </tr>
"#,
                tool.name,
                tool.tool_type,
                tool.file_path
            ));
        }
        if tools.len() > 50 {
            html.push_str(&format!(
                r#"                    <tr><td colspan="3"><em>... and {} more</em></td></tr>"#,
                tools.len() - 50
            ));
        }
        html.push_str("                </tbody>\n            </table>\n        </div>");

        // Add graph statistics
        html.push_str(&format!(
            r#"
        <div class="section">
            <h2>🕸️ Knowledge Graph</h2>
            <p>The knowledge graph contains <strong>{}</strong> nodes and <strong>{}</strong> edges, representing the complete architecture of this repository.</p>
            <p><strong>Node Types:</strong></p>
            <ul>
"#,
            graph_stats.total_nodes, graph_stats.total_edges
        ));
        for (node_type, count) in graph_stats.nodes_by_type.iter() {
            html.push_str(&format!("                <li><strong>{}</strong>: {}</li>\n", node_type, count));
        }
        html.push_str("            </ul>\n        </div>");

        // Footer - Read version from VERSION file
        let version = std::fs::read_to_string("VERSION")
            .unwrap_or_else(|_| "0.6.3".to_string())
            .trim()
            .to_string();
        
        html.push_str(&format!(
            r#"
        <div class="footer">
            <p>Generated by <strong>Wavelength Architecture Decoder</strong> v{}</p>
            <p>For more information, visit: <a href="https://github.com/mimelator/wavelength-arch-decoder">GitHub Repository</a></p>
        </div>
    </div>
</body>
</html>
"#,
            version
        ));

        Ok(html)
    }
}

