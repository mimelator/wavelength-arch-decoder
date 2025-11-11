// Main Application
document.addEventListener('DOMContentLoaded', () => {
    initializeApp();
});

function initializeApp() {
    setupNavigation();
    setupModals();
    setupAuth();
    checkAuthStatus();
    loadDashboard();
    setupSearch();
    setupGraph();
    // setupRepositories removed - not needed
}

// Authentication
function setupAuth() {
    // Tab switching
    const tabs = document.querySelectorAll('.auth-tab');
    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const targetTab = tab.getAttribute('data-tab');
            
            // Update tab states
            tabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            
            // Update form visibility
            document.querySelectorAll('.auth-form').forEach(form => {
                form.classList.remove('active');
            });
            
            if (targetTab === 'register') {
                document.getElementById('register-form').classList.add('active');
            } else {
                document.getElementById('login-form').classList.add('active');
            }
        });
    });
    
    // Register form
    const registerForm = document.getElementById('form-register');
    if (registerForm) {
        registerForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const email = document.getElementById('register-email').value;
            const password = document.getElementById('register-password').value;
            const resultDiv = document.getElementById('register-result');
            
            try {
                const result = await api.register(email, password);
                resultDiv.className = 'auth-result success';
                resultDiv.textContent = 'Registration successful! Redirecting to dashboard...';
                setTimeout(() => {
                    showPage('dashboard');
                    document.querySelectorAll('.nav-link').forEach(l => {
                        if (l.getAttribute('data-page') === 'dashboard') {
                            l.classList.add('active');
                        } else {
                            l.classList.remove('active');
                        }
                    });
                    loadDashboard();
                }, 1000);
            } catch (error) {
                resultDiv.className = 'auth-result error';
                // Provide more helpful error messages
                let errorMsg = error.message || 'Registration failed';
                if (errorMsg.includes('already exists') || errorMsg.includes('duplicate')) {
                    errorMsg = 'An account with this email already exists. Please login instead.';
                } else if (errorMsg.includes('API key')) {
                    errorMsg = 'Failed to generate API key. Please try again.';
                }
                resultDiv.textContent = 'Registration failed: ' + errorMsg;
                console.error('Registration error:', error);
            }
        });
    }
    
    // Login form
    const loginForm = document.getElementById('form-login');
    if (loginForm) {
        loginForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const email = document.getElementById('login-email').value;
            const password = document.getElementById('login-password').value;
            const resultDiv = document.getElementById('login-result');
            
            try {
                const result = await api.login(email, password);
                resultDiv.className = 'auth-result success';
                resultDiv.textContent = 'Login successful! Redirecting to dashboard...';
                setTimeout(() => {
                    showPage('dashboard');
                    document.querySelectorAll('.nav-link').forEach(l => {
                        if (l.getAttribute('data-page') === 'dashboard') {
                            l.classList.add('active');
                        } else {
                            l.classList.remove('active');
                        }
                    });
                    loadDashboard();
                }, 1000);
            } catch (error) {
                resultDiv.className = 'auth-result error';
                // Provide more helpful error messages
                let errorMsg = error.message || 'Login failed';
                if (errorMsg.includes('Invalid email or password')) {
                    errorMsg = 'Invalid email or password. Please check your credentials and try again.';
                } else if (errorMsg.includes('API key')) {
                    errorMsg = 'Failed to generate API key. Please try again.';
                }
                resultDiv.textContent = 'Login failed: ' + errorMsg;
                console.error('Login error:', error);
            }
        });
    }
    
    // Copy API key button
    const copyBtn = document.getElementById('btn-copy-key');
    if (copyBtn) {
        copyBtn.addEventListener('click', () => {
            const apiKey = document.getElementById('api-key-value').textContent;
            navigator.clipboard.writeText(apiKey).then(() => {
                copyBtn.textContent = 'Copied!';
                setTimeout(() => {
                    copyBtn.textContent = 'Copy';
                }, 2000);
            });
        });
    }
    
    // Continue button
    const continueBtn = document.getElementById('btn-continue');
    if (continueBtn) {
        continueBtn.addEventListener('click', () => {
            document.getElementById('api-key-display').style.display = 'none';
            showPage('dashboard');
            document.querySelector('[data-page="dashboard"]').classList.add('active');
            document.querySelector('[data-page="login"]').classList.remove('active');
            document.getElementById('nav-login').textContent = 'Logout';
            loadDashboard();
        });
    }
}

function showApiKey(apiKey) {
    document.getElementById('api-key-value').textContent = apiKey;
    document.getElementById('api-key-display').style.display = 'block';
    // Scroll to API key display
    document.getElementById('api-key-display').scrollIntoView({ behavior: 'smooth' });
}

function checkAuthStatus() {
    // API keys removed - no authentication check needed
    // Just show dashboard directly
    showPage('dashboard');
    document.querySelectorAll('.nav-link').forEach(link => {
        if (link.getAttribute('data-page') === 'dashboard') {
            link.classList.add('active');
        } else {
            link.classList.remove('active');
        }
    });
}

// Update navigation to handle logout
function setupNavigation() {
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            const page = link.getAttribute('data-page');
            
            // Handle logout
            if (link.id === 'nav-login' && link.textContent === 'Logout') {
                // API keys removed - just redirect to login page
                link.textContent = 'Login';
                showPage('login');
                document.querySelectorAll('.nav-link').forEach(l => {
                    if (l.getAttribute('data-page') === 'login') {
                        l.classList.add('active');
                    } else {
                        l.classList.remove('active');
                    }
                });
                return;
            }
            
            showPage(page);
            
            // Update active state
            navLinks.forEach(l => l.classList.remove('active'));
            link.classList.add('active');
        });
    });
}

function showPage(pageId) {
    document.querySelectorAll('.page').forEach(page => {
        page.classList.remove('active');
    });
    const targetPage = document.getElementById(pageId);
    if (targetPage) {
        targetPage.classList.add('active');
        
        // Load page-specific data
        if (pageId === 'dashboard') {
            loadDashboard();
        } else if (pageId === 'repositories') {
            loadRepositories();
        } else if (pageId === 'graph') {
            loadRepositoriesForGraph();
        }
    }
}

// Dashboard
async function loadDashboard() {
    try {
        const repos = await api.listRepositories();
        document.getElementById('stat-repositories').textContent = repos.length || 0;
        
        // Calculate stats
        let totalDeps = 0;
        let totalServices = 0;
        let totalSecurity = 0;
        
        for (const repo of repos.slice(0, 5)) {
            try {
                const deps = await api.getDependencies(repo.id);
                totalDeps += deps.length || 0;
                
                const services = await api.getServices(repo.id);
                totalServices += services.length || 0;
                
                const security = await api.getSecurityEntities(repo.id);
                totalSecurity += security.length || 0;
            } catch (e) {
                // Repository might not be analyzed yet
            }
        }
        
        document.getElementById('stat-dependencies').textContent = totalDeps;
        document.getElementById('stat-services').textContent = totalServices;
        document.getElementById('stat-security').textContent = totalSecurity;
        
        // Show recent repositories
        const recentRepos = repos.slice(0, 5);
        displayRepositories(recentRepos, 'recent-repos');
    } catch (error) {
        console.error('Failed to load dashboard:', error);
        
        // If API key is invalid, redirect to login
        if (error.message && error.message.includes('API key')) {
            // API keys removed - just show error
            showError('Failed to load dashboard: ' + error.message);
        } else {
            showError('Failed to load dashboard. ' + error.message);
        }
    }
}

// Repositories
async function loadRepositories() {
    try {
        const repos = await api.listRepositories();
        displayRepositories(repos, 'repositories-list');
    } catch (error) {
        console.error('Failed to load repositories:', error);
        showError('Failed to load repositories.');
    }
}

async function loadRepositoriesForGraph() {
    try {
        const repos = await api.listRepositories();
        const select = document.getElementById('graph-repo-select');
        select.innerHTML = '<option value="">Select Repository</option>';
        repos.forEach(repo => {
            const option = document.createElement('option');
            option.value = repo.id;
            option.textContent = repo.name;
            select.appendChild(option);
        });
    } catch (error) {
        console.error('Failed to load repositories:', error);
    }
}

function displayRepositories(repos, containerId) {
    const container = document.getElementById(containerId);
    if (!container) return;
    
    if (repos.length === 0) {
        container.innerHTML = '<p class="text-secondary">No repositories found. Add one to get started!</p>';
        return;
    }
    
    container.innerHTML = repos.map(repo => `
        <div class="repo-item" data-repo-id="${repo.id}">
            <div class="repo-info">
                <h3>${escapeHtml(repo.name)}</h3>
                <p>${escapeHtml(repo.url)}</p>
                <p style="font-size: 0.75rem; color: var(--text-secondary);">
                    Branch: ${escapeHtml(repo.branch)} | 
                    ${repo.last_analyzed_at ? `Last analyzed: ${new Date(repo.last_analyzed_at).toLocaleDateString()}` : 'Not analyzed yet'}
                </p>
                <div class="analysis-status" style="display: none; margin-top: 0.5rem;"></div>
            </div>
            <div class="repo-actions">
                <button class="btn btn-primary btn-analyze" onclick="analyzeRepository('${repo.id}')">Analyze</button>
                <button class="btn btn-secondary" onclick="viewRepository('${repo.id}')">View</button>
            </div>
        </div>
    `).join('');
}

// Graph Visualization
function setupGraph() {
    document.getElementById('btn-load-graph').addEventListener('click', async () => {
        const repoId = document.getElementById('graph-repo-select').value;
        if (!repoId) {
            alert('Please select a repository');
            return;
        }
        await loadGraph(repoId);
    });
}

async function loadGraph(repoId) {
    try {
        const container = document.getElementById('graph-container');
        container.innerHTML = '<div class="loading"></div> Loading graph...';
        
        const graphData = await api.getGraph(repoId);
        
        if (!graphData.nodes || graphData.nodes.length === 0) {
            container.innerHTML = '<div class="graph-placeholder"><p>No graph data available. Please analyze the repository first.</p></div>';
            return;
        }
        
        // Render enhanced graph
        renderEnhancedGraph(graphData, container, repoId);
        
        // Show graph info
        document.getElementById('graph-info').innerHTML = `
            <strong>Graph Statistics:</strong>
            <ul>
                <li>Nodes: ${graphData.nodes.length}</li>
                <li>Edges: ${graphData.edges.length}</li>
                <li>Repository: ${repoId}</li>
            </ul>
        `;
    } catch (error) {
        console.error('Failed to load graph:', error);
        container.innerHTML = 
            '<div class="graph-placeholder"><p>Failed to load graph. Please try again.</p></div>';
    }
}

function renderEnhancedGraph(graphData, container, repoId = null) {
    // Map node types to better display names (handle both enum serialization and string formats)
    const nodeTypeLabels = {
        'repository': 'Repository',
        'Repository': 'Repository',
        'dependency': 'Dependency',
        'Dependency': 'Dependency',
        'service': 'Service',
        'Service': 'Service',
        'package_manager': 'Package Manager',
        'PackageManager': 'Package Manager',
        'service_provider': 'Provider',
        'ServiceProvider': 'Provider',
    };
    
    // Map edge types to readable labels
    const edgeTypeLabels = {
        'DependsOn': 'depends on',
        'depends_on': 'depends on',
        'UsesService': 'uses',
        'uses_service': 'uses',
        'HasDependency': 'has dependency',
        'has_dependency': 'has dependency',
        'UsesPackageManager': 'uses',
        'uses_package_manager': 'uses',
        'ProvidedBy': 'provided by',
        'provided_by': 'provided by',
        'RelatedTo': 'related to',
        'related_to': 'related to',
    };
    
    // Prepare nodes with enhanced information
    const nodes = graphData.nodes.map(node => {
        // Handle both node_type (from enum) and type (from GraphQL)
        let nodeType = (node.node_type || node.type || 'unknown').toLowerCase();
        // Normalize enum serialization (e.g., "Repository" -> "repository")
        if (nodeTypeLabels[node.node_type || node.type]) {
            nodeType = (node.node_type || node.type).toLowerCase();
        }
        const nodeTypeLabel = nodeTypeLabels[node.node_type || node.type] || nodeTypeLabels[nodeType] || nodeType;
        
        // Build tooltip with all properties
        let tooltip = `<strong>${escapeHtml(node.name)}</strong><br>`;
        tooltip += `<em>Type: ${nodeTypeLabel}</em><br>`;
        
        if (node.properties) {
            // Handle both object and JSON string properties
            let props = node.properties;
            if (typeof props === 'string') {
                try {
                    props = JSON.parse(props);
                } catch (e) {
                    props = {};
                }
            }
            const propEntries = Object.entries(props || {});
            if (propEntries.length > 0) {
                tooltip += '<br><strong>Properties:</strong><br>';
                propEntries.forEach(([key, value]) => {
                    tooltip += `${escapeHtml(key)}: ${escapeHtml(String(value))}<br>`;
                });
            }
        }
        
        // Build label with type indicator
        const label = `${node.name}\n(${nodeTypeLabel})`;
        
        return {
            id: node.id,
            label: label,
            title: tooltip,
            color: getNodeColor(nodeType),
            shape: getNodeShape(nodeType),
            font: {
                size: 18,
                face: 'Arial',
                multi: 'html',
            },
            borderWidth: 2,
            size: 25,
            chosen: {
                node: function(values, id, selected, hovering) {
                    if (hovering || selected) {
                        values.borderWidth = 4;
                        values.size = 35;
                        values.font.size = 20;
                    }
                }
            }
        };
    });
    
    // Prepare edges with relationship labels
    const edges = graphData.edges.map(edge => {
        // Handle both edge_type (from enum) and relationship_type
        const edgeType = edge.edge_type || edge.relationship_type || edge.type || 'RelatedTo';
        const edgeLabel = edgeTypeLabels[edgeType] || edgeTypeLabels[edgeType.toLowerCase()] || edgeType.toLowerCase().replace(/([A-Z])/g, ' $1').trim();
        
        // Build tooltip with properties
        let tooltip = `<strong>${edgeLabel}</strong>`;
        if (edge.properties) {
            // Handle both object and JSON string properties
            let props = edge.properties;
            if (typeof props === 'string') {
                try {
                    props = JSON.parse(props);
                } catch (e) {
                    props = {};
                }
            }
            const propEntries = Object.entries(props || {});
            if (propEntries.length > 0) {
                tooltip += '<br><br><strong>Properties:</strong><br>';
                propEntries.forEach(([key, value]) => {
                    tooltip += `${escapeHtml(key)}: ${escapeHtml(String(value))}<br>`;
                });
            }
        }
        
        return {
            from: edge.source_node_id || edge.source,
            to: edge.target_node_id || edge.target,
            label: edgeLabel,
            title: tooltip,
            arrows: 'to',
            color: {
                color: '#64748b',
                highlight: '#2563eb',
                hover: '#2563eb',
            },
            font: {
                size: 11,
                align: 'middle',
            },
            smooth: {
                type: 'continuous',
                roundness: 0.5,
            },
            width: 2,
        };
    });
    
    // Create network with enhanced options
    const data = { nodes, edges };
    const options = {
        nodes: {
            font: { 
                size: 18,
                face: 'Arial',
                multi: 'html',
            },
            borderWidth: 2,
            shadow: true,
            size: 25,
            scaling: {
                min: 20,
                max: 40,
            },
            margin: 20,
        },
        edges: {
            font: { 
                size: 13, 
                align: 'middle',
                background: 'white',
                strokeWidth: 2,
            },
            smooth: { 
                type: 'continuous',
                roundness: 0.5,
            },
            arrows: {
                to: {
                    enabled: true,
                    scaleFactor: 1.2,
                }
            },
            shadow: true,
            length: 300,
        },
        physics: {
            enabled: true,
            stabilization: { 
                iterations: 300,
                fit: true,
            },
            barnesHut: {
                gravitationalConstant: -3000,
                centralGravity: 0.05,
                springLength: 300,
                springConstant: 0.03,
                damping: 0.1,
            },
        },
        interaction: {
            hover: true,
            tooltipDelay: 100,
            zoomView: true,
            dragView: true,
            zoomSpeed: 0.5,
        },
        layout: {
            improvedLayout: true,
            hierarchical: {
                enabled: false,
            }
        },
    };
    
    const network = new vis.Network(container, data, options);
    
    // Store graph data and repo ID for navigation
    network.graphData = graphData;
    // Determine repo ID: use provided repoId, or check if we're in repository detail view
    network.repoId = repoId || (container.closest('.page')?.id === 'repository-detail' ? currentRepoId : null);
    
    // Add click handler to show node details
    network.on('click', function(params) {
        if (params.nodes.length > 0) {
            const nodeId = params.nodes[0];
            const node = graphData.nodes.find(n => n.id === nodeId);
            if (node) {
                showNodeDetails(node, graphData, network.repoId);
            }
        }
    });
    
    // Add hover handler for better feedback
    network.on('hoverNode', function(params) {
        container.style.cursor = 'pointer';
    });
    
    network.on('blurNode', function(params) {
        container.style.cursor = 'default';
    });
}

function showNodeDetails(node, graphData, repoId = null) {
    // Find all edges connected to this node
    const connectedEdges = graphData.edges.filter(e => 
        e.source_node_id === node.id || e.target_node_id === node.id
    );
    
    // Determine which tab this node type belongs to
    const nodeType = (node.node_type || node.type || 'unknown').toLowerCase();
    const tabMapping = {
        'dependency': 'dependencies',
        'service': 'services',
        'code_element': 'code',
        'security_entity': 'security',
        'repository': 'overview',
        'package_manager': 'dependencies',
        'service_provider': 'services',
    };
    const targetTab = tabMapping[nodeType] || null;
    
    // Build details HTML
    let details = `<div class="node-details">`;
    details += `<h3>${escapeHtml(node.name)}</h3>`;
    details += `<p><strong>Type:</strong> ${escapeHtml(node.node_type || node.type || 'unknown')}</p>`;
    
    if (node.properties) {
        // Handle both object and JSON string properties
        let props = node.properties;
        if (typeof props === 'string') {
            try {
                props = JSON.parse(props);
            } catch (e) {
                props = {};
            }
        }
        const propEntries = Object.entries(props || {});
        if (propEntries.length > 0) {
            details += `<h4>Properties:</h4><ul>`;
            propEntries.forEach(([key, value]) => {
                details += `<li><strong>${escapeHtml(key)}:</strong> ${escapeHtml(String(value))}</li>`;
            });
            details += `</ul>`;
        }
    }
    
    if (connectedEdges.length > 0) {
        details += `<h4>Connections (${connectedEdges.length}):</h4><ul>`;
        connectedEdges.forEach(edge => {
            const otherNodeId = edge.source_node_id === node.id ? edge.target_node_id : edge.source_node_id;
            const otherNode = graphData.nodes.find(n => n.id === otherNodeId);
            const direction = edge.source_node_id === node.id ? '→' : '←';
            const edgeLabel = edge.edge_type || 'related to';
            if (otherNode) {
                details += `<li>${direction} ${escapeHtml(otherNode.name)} (${escapeHtml(edgeLabel)})</li>`;
            }
        });
        details += `</ul>`;
    }
    
    // Add navigation link if we're in repository detail view and have a matching tab
    let actionButtons = '<div style="margin-top: 1rem; display: flex; gap: 0.5rem;">';
    actionButtons += '<button onclick="this.closest(\'.node-details-modal\').remove()">Close</button>';
    
    if (repoId && targetTab) {
        actionButtons += `<button onclick="navigateToNodeInDetail('${repoId}', '${targetTab}', '${escapeHtml(node.name)}'); this.closest('.node-details-modal').remove();" style="background: var(--primary-color); color: white;">View in Repository Details</button>`;
    } else if (repoId) {
        // If we have a repo ID but no specific tab, just go to overview
        actionButtons += `<button onclick="navigateToNodeInDetail('${repoId}', 'overview', '${escapeHtml(node.name)}'); this.closest('.node-details-modal').remove();" style="background: var(--primary-color); color: white;">View Repository Details</button>`;
    }
    actionButtons += '</div>';
    
    details += actionButtons;
    details += `</div>`;
    
    // Show in a modal
    const detailsDiv = document.createElement('div');
    detailsDiv.className = 'node-details-modal';
    detailsDiv.innerHTML = details;
    document.body.appendChild(detailsDiv);
}

// Navigate to a specific node in the repository detail view
window.navigateToNodeInDetail = async function(repoId, tabName, nodeName) {
    // If we're not already on the repository detail page, navigate there
    const currentPage = document.querySelector('.page.active')?.id;
    if (currentPage !== 'repository-detail') {
        await window.viewRepository(repoId);
        // Wait a bit for the page to load
        await new Promise(resolve => setTimeout(resolve, 300));
    }
    
    // Switch to the appropriate tab
    switchTab(tabName, repoId);
    
    // Wait for tab content to load, then try to scroll to/highlight the item
    setTimeout(() => {
        highlightNodeInTab(tabName, nodeName);
    }, 500);
}

function highlightNodeInTab(tabName, nodeName) {
    const tabContent = document.getElementById(`tab-${tabName}`);
    if (!tabContent) return;
    
    // For code tab, expand all sections first
    if (tabName === 'code') {
        const sections = tabContent.querySelectorAll('.collapsible-section');
        sections.forEach(section => {
            const content = section.querySelector('.section-content');
            const toggle = section.querySelector('.section-toggle');
            if (content && content.style.display === 'none') {
                content.style.display = 'block';
                if (toggle) toggle.textContent = '▼';
            }
        });
    }
    
    // Find the item that matches the node name
    const items = tabContent.querySelectorAll('.detail-item');
    for (const item of items) {
        const itemName = item.querySelector('strong')?.textContent;
        if (itemName && itemName.toLowerCase().includes(nodeName.toLowerCase())) {
            // Scroll to the item
            item.scrollIntoView({ behavior: 'smooth', block: 'center' });
            
            // Highlight it temporarily
            item.style.backgroundColor = 'rgba(37, 99, 235, 0.2)';
            item.style.borderColor = 'var(--primary-color)';
            item.style.transition = 'all 0.3s';
            
            // Remove highlight after 3 seconds
            setTimeout(() => {
                item.style.backgroundColor = '';
                item.style.borderColor = '';
            }, 3000);
            
            break;
        }
    }
}

function getNodeColor(type) {
    const colors = {
        'repository': '#3b82f6',
        'dependency': '#10b981',
        'service': '#f59e0b',
        'package_manager': '#8b5cf6',
        'service_provider': '#ec4899',
        'code_element': '#8b5cf6',
        'security_entity': '#ef4444',
    };
    return colors[type] || '#64748b';
}

function getNodeShape(type) {
    const shapes = {
        'repository': 'box',
        'dependency': 'dot',
        'service': 'diamond',
        'package_manager': 'triangle',
        'service_provider': 'star',
        'code_element': 'triangle',
        'security_entity': 'star',
    };
    return shapes[type] || 'dot';
}

// Search
function setupSearch() {
    document.getElementById('btn-search').addEventListener('click', performSearch);
    document.getElementById('search-query').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            performSearch();
        }
    });
}

async function performSearch() {
    const query = document.getElementById('search-query').value.trim();
    if (!query) {
        alert('Please enter a search query');
        return;
    }
    
    const resultsContainer = document.getElementById('search-results');
    resultsContainer.innerHTML = '<div class="loading"></div> Searching...';
    
    try {
        const results = [];
        
        // Search dependencies
        if (document.getElementById('filter-dependencies').checked) {
            try {
                const deps = await api.searchDependencies(query);
                results.push(...(deps || []).map(dep => ({
                    type: 'Dependency',
                    name: dep.name,
                    version: dep.version,
                    description: `${dep.package_manager} package`,
                })));
            } catch (e) {
                console.error('Dependency search failed:', e);
            }
        }
        
        // Search services
        if (document.getElementById('filter-services').checked) {
            try {
                const services = await api.searchServices(query);
                results.push(...(services || []).map(svc => ({
                    type: 'Service',
                    name: svc.name,
                    provider: svc.provider,
                    description: `${svc.service_type} service`,
                })));
            } catch (e) {
                console.error('Service search failed:', e);
            }
        }
        
        displaySearchResults(results);
    } catch (error) {
        console.error('Search failed:', error);
        resultsContainer.innerHTML = '<p>Search failed. Please try again.</p>';
    }
}

function displaySearchResults(results) {
    const container = document.getElementById('search-results');
    
    if (results.length === 0) {
        container.innerHTML = '<p>No results found.</p>';
        return;
    }
    
    container.innerHTML = results.map(result => `
        <div class="result-item">
            <h4>${escapeHtml(result.name)} <span style="color: var(--text-secondary); font-size: 0.875rem;">(${result.type})</span></h4>
            <p>${escapeHtml(result.description || '')}</p>
            ${result.version ? `<p style="font-size: 0.75rem; color: var(--text-secondary);">Version: ${escapeHtml(result.version)}</p>` : ''}
            ${result.provider ? `<p style="font-size: 0.75rem; color: var(--text-secondary);">Provider: ${escapeHtml(result.provider)}</p>` : ''}
        </div>
    `).join('');
}

// Modals
function setupModals() {
    const modal = document.getElementById('modal-add-repo');
    const btn = document.getElementById('btn-add-repo');
    const closeBtns = document.querySelectorAll('.modal-close');
    
    if (btn) {
        btn.addEventListener('click', () => {
            modal.classList.add('active');
        });
    }
    
    closeBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            modal.classList.remove('active');
        });
    });
    
    // Form submission
    const form = document.getElementById('form-add-repo');
    if (form) {
        // Show/hide auth value field based on auth type
        const authTypeSelect = document.getElementById('repo-auth-type');
        const authValueGroup = document.getElementById('auth-value-group');
        const authValueInput = document.getElementById('repo-auth-value');
        const authHint = document.getElementById('auth-hint');
        
        if (authTypeSelect) {
            authTypeSelect.addEventListener('change', () => {
                const authType = authTypeSelect.value;
                if (authType) {
                    authValueGroup.style.display = 'block';
                    authValueInput.required = true;
                    switch(authType) {
                        case 'token':
                            authHint.textContent = 'Enter your personal access token (GitHub/GitLab/Bitbucket)';
                            authValueInput.type = 'password';
                            authValueInput.placeholder = 'ghp_xxxxxxxxxxxxx';
                            break;
                        case 'ssh_key':
                            authHint.textContent = 'Enter the path to your SSH private key file';
                            authValueInput.type = 'text';
                            authValueInput.placeholder = '/path/to/id_rsa';
                            break;
                        case 'username_password':
                            authHint.textContent = 'Enter username:password (will be base64 encoded)';
                            authValueInput.type = 'password';
                            authValueInput.placeholder = 'username:password';
                            break;
                    }
                } else {
                    authValueGroup.style.display = 'none';
                    authValueInput.required = false;
                }
            });
        }
        
        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            const name = document.getElementById('repo-name').value;
            const url = document.getElementById('repo-url').value;
            const branch = document.getElementById('repo-branch').value || 'main';
            const authType = document.getElementById('repo-auth-type').value;
            let authValue = document.getElementById('repo-auth-value').value;
            
            // Encode username:password as base64
            if (authType === 'username_password' && authValue) {
                authValue = btoa(authValue);
            }
            
            try {
                await api.createRepository(
                    name,
                    url,
                    branch,
                    authType || undefined,
                    authValue || undefined
                );
                modal.classList.remove('active');
                form.reset();
                authValueGroup.style.display = 'none';
                loadRepositories();
                showPage('repositories');
            } catch (error) {
                alert('Failed to create repository: ' + error.message);
            }
        });
    }
}

// Utility functions
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showError(message) {
    // Simple error display - could be enhanced with a toast notification
    alert(message);
}

// Global functions for onclick handlers
window.analyzeRepository = async function(repoId) {
    // Find the repository item to show progress
    const repoItem = document.querySelector(`[data-repo-id="${repoId}"]`);
    const analyzeBtn = repoItem?.querySelector('.btn-analyze');
    const statusDiv = repoItem?.querySelector('.analysis-status');
    
    // Show loading state
    if (analyzeBtn) {
        analyzeBtn.disabled = true;
        analyzeBtn.textContent = 'Analyzing...';
    }
    
    if (statusDiv) {
        statusDiv.innerHTML = '<div class="analysis-progress">Starting analysis...</div>';
        statusDiv.style.display = 'block';
    }
    
    const steps = [
        'Fetching repository information...',
        'Initializing crawler...',
        'Cloning/updating repository...',
        'Extracting dependencies...',
        'Detecting services...',
        'Building knowledge graph...',
        'Analyzing code structure...',
        'Analyzing security configuration...',
        'Finalizing...'
    ];
    
    let currentStep = 0;
    const updateProgress = () => {
        if (statusDiv && currentStep < steps.length) {
            const progress = ((currentStep + 1) / steps.length * 100).toFixed(0);
            statusDiv.innerHTML = `
                <div class="analysis-progress">
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: ${progress}%"></div>
                    </div>
                    <div class="progress-text">Step ${currentStep + 1}/${steps.length}: ${steps[currentStep]}</div>
                </div>
            `;
        }
    };
    
    // Simulate progress updates (since we can't get real-time updates from the server)
    const progressInterval = setInterval(() => {
        if (currentStep < steps.length - 1) {
            currentStep++;
            updateProgress();
        }
    }, 2000); // Update every 2 seconds
    
    try {
        console.log(`Starting analysis for repository ${repoId}...`);
        const result = await api.analyzeRepository(repoId);
        
        clearInterval(progressInterval);
        
        if (statusDiv) {
            statusDiv.innerHTML = `
                <div class="analysis-success">
                    <strong>✓ Analysis Complete!</strong>
                    <div class="analysis-results">
                        <div>📦 ${result.results?.total_dependencies || 0} dependencies</div>
                        <div>🔌 ${result.results?.services_found || 0} services</div>
                        <div>📝 ${result.results?.code_elements_found || 0} code elements</div>
                        <div>🔒 ${result.results?.security_entities_found || 0} security entities</div>
                    </div>
                </div>
            `;
        }
        
        if (analyzeBtn) {
            analyzeBtn.disabled = false;
            analyzeBtn.textContent = 'Re-analyze';
        }
        
        // Reload repositories to show updated status
        setTimeout(() => {
            loadRepositories();
        }, 2000);
        
        console.log('Analysis result:', result);
    } catch (error) {
        clearInterval(progressInterval);
        
        console.error('Analysis error:', error);
        
        if (statusDiv) {
            statusDiv.innerHTML = `
                <div class="analysis-error">
                    <strong>✗ Analysis Failed</strong>
                    <div class="error-details">${escapeHtml(error.message)}</div>
                </div>
            `;
        }
        
        if (analyzeBtn) {
            analyzeBtn.disabled = false;
            analyzeBtn.textContent = 'Analyze';
        }
        
        alert('Failed to start analysis: ' + error.message);
    }
};

window.viewRepository = async function(repoId) {
    // Show repository detail page
    showPage('repository-detail');
    
    // Update navigation
    document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.remove('active');
    });
    
    // Load repository details
    await loadRepositoryDetail(repoId);
};

let currentRepoId = null;

async function loadRepositoryDetail(repoId) {
    currentRepoId = repoId;
    
    try {
        // Load repository info
        const repo = await api.getRepository(repoId);
        
        // Update header
        document.getElementById('repo-detail-name').textContent = repo.name;
        document.getElementById('repo-detail-title').textContent = repo.name;
        document.getElementById('repo-detail-url').textContent = repo.url;
        document.getElementById('repo-detail-branch').textContent = `Branch: ${repo.branch || 'main'}`;
        document.getElementById('repo-detail-last-analyzed').textContent = repo.last_analyzed_at 
            ? `Last analyzed: ${new Date(repo.last_analyzed_at).toLocaleString()}`
            : 'Not analyzed yet';
        
        // Setup analyze button
        const analyzeBtn = document.getElementById('btn-analyze-detail');
        analyzeBtn.onclick = () => {
            window.analyzeRepository(repoId);
            setTimeout(() => loadRepositoryDetail(repoId), 5000);
        };
        
        // Setup tabs
        setupRepositoryTabs(repoId);
        
        // Load overview stats
        await loadRepositoryOverview(repoId);
        
        // Load initial tab (overview)
        switchTab('overview', repoId);
        
    } catch (error) {
        console.error('Failed to load repository details:', error);
        alert('Failed to load repository details: ' + error.message);
    }
}

function setupRepositoryTabs(repoId) {
    const tabs = document.querySelectorAll('.repo-tab');
    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.getAttribute('data-tab');
            switchTab(tabName, repoId);
        });
    });
}

function switchTab(tabName, repoId) {
    // Update tab states
    document.querySelectorAll('.repo-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.repo-tab-content').forEach(c => c.classList.remove('active'));
    
    document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
    document.getElementById(`tab-${tabName}`).classList.add('active');
    
    // Load tab content
    switch(tabName) {
        case 'overview':
            loadRepositoryOverview(repoId);
            break;
        case 'dependencies':
            loadDependencies(repoId);
            break;
        case 'services':
            loadServices(repoId);
            break;
        case 'code':
            loadCodeElements(repoId);
            break;
        case 'security':
            loadSecurity(repoId);
            break;
        case 'graph':
            loadRepositoryGraph(repoId);
            break;
    }
}

async function loadRepositoryOverview(repoId) {
    try {
        const [deps, services, code, security] = await Promise.all([
            api.getDependencies(repoId).catch(() => []),
            api.getServices(repoId).catch(() => []),
            api.getCodeElements(repoId).catch(() => []),
            api.getSecurityEntities(repoId).catch(() => [])
        ]);
        
        document.getElementById('stat-deps-count').textContent = deps.length || 0;
        document.getElementById('stat-services-count').textContent = services.length || 0;
        document.getElementById('stat-code-count').textContent = code.length || 0;
        document.getElementById('stat-security-count').textContent = security.length || 0;
    } catch (error) {
        console.error('Failed to load overview:', error);
    }
}

async function loadDependencies(repoId) {
    const container = document.getElementById('dependencies-list');
    container.innerHTML = '<p class="loading-text">Loading dependencies...</p>';
    
    try {
        const deps = await api.getDependencies(repoId);
        
        if (deps.length === 0) {
            container.innerHTML = '<p>No dependencies found. Run analysis first.</p>';
            return;
        }
        
        // Group by package manager
        const grouped = {};
        deps.forEach(dep => {
            const pm = dep.package_manager || 'unknown';
            if (!grouped[pm]) grouped[pm] = [];
            grouped[pm].push(dep);
        });
        
        container.innerHTML = Object.entries(grouped).map(([pm, depsList]) => `
            <div class="detail-section">
                <h4>${escapeHtml(pm)}</h4>
                <div class="detail-items">
                    ${depsList.map(dep => `
                        <div class="detail-item clickable" onclick="showEntityDetail('${repoId}', 'dependency', '${dep.id}')">
                            <div class="detail-item-header">
                                <strong>${escapeHtml(dep.name)}</strong>
                                <span class="detail-badge">${escapeHtml(dep.version || 'unknown')}</span>
                            </div>
                            ${dep.file_path ? `<p class="detail-meta">Found in: ${escapeHtml(dep.file_path)}</p>` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `).join('');
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load dependencies: ${escapeHtml(error.message)}</p>`;
    }
}

async function loadServices(repoId) {
    const container = document.getElementById('services-list');
    container.innerHTML = '<p class="loading-text">Loading services...</p>';
    
    try {
        const services = await api.getServices(repoId);
        
        if (services.length === 0) {
            container.innerHTML = '<p>No services found. Run analysis first.</p>';
            return;
        }
        
        // Group by provider
        const grouped = {};
        services.forEach(svc => {
            const provider = svc.provider || 'unknown';
            if (!grouped[provider]) grouped[provider] = [];
            grouped[provider].push(svc);
        });
        
        container.innerHTML = Object.entries(grouped).map(([provider, svcList]) => `
            <div class="detail-section">
                <h4>${escapeHtml(provider)}</h4>
                <div class="detail-items">
                    ${svcList.map(svc => `
                        <div class="detail-item clickable" onclick="showEntityDetail('${repoId}', 'service', '${svc.id}')">
                            <div class="detail-item-header">
                                <strong>${escapeHtml(svc.name)}</strong>
                                <span class="detail-badge">${escapeHtml(svc.service_type || 'service')}</span>
                            </div>
                            ${svc.configuration ? `<p class="detail-meta">Config: ${escapeHtml(JSON.stringify(svc.configuration).substring(0, 100))}...</p>` : ''}
                            ${svc.file_path ? `<p class="detail-meta">Found in: ${escapeHtml(svc.file_path)}</p>` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `).join('');
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load services: ${escapeHtml(error.message)}</p>`;
    }
}

// Store code elements for filtering
let allCodeElements = [];
let currentCodeGroupBy = 'type';

async function loadCodeElements(repoId) {
    currentRepoId = repoId;
    const container = document.getElementById('code-list');
    container.innerHTML = '<p class="loading-text">Loading code structure...</p>';
    
    try {
        const elements = await api.getCodeElements(repoId);
        allCodeElements = elements;
        
        if (elements.length === 0) {
            container.innerHTML = '<p>No code elements found. Run analysis first.</p>';
            return;
        }
        
        // Setup filters
        setupCodeFilters(elements);
        
        // Render code elements
        renderCodeElements(elements);
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load code elements: ${escapeHtml(error.message)}</p>`;
    }
}

function setupCodeFilters(elements) {
    // Populate type filter
    const typeFilter = document.getElementById('code-filter-type');
    const types = [...new Set(elements.map(el => el.element_type || 'unknown'))].sort();
    typeFilter.innerHTML = '<option value="">All Types</option>' + 
        types.map(type => `<option value="${escapeHtml(type)}">${escapeHtml(type)}</option>`).join('');
    
    // Populate language filter
    const langFilter = document.getElementById('code-filter-language');
    const languages = [...new Set(elements.map(el => el.language || 'unknown'))].sort();
    langFilter.innerHTML = '<option value="">All Languages</option>' + 
        languages.map(lang => `<option value="${escapeHtml(lang)}">${escapeHtml(lang)}</option>`).join('');
    
    // Setup event listeners
    const searchInput = document.getElementById('code-search');
    const groupBySelect = document.getElementById('code-group-by');
    
    searchInput.addEventListener('input', () => filterAndRenderCode());
    typeFilter.addEventListener('change', () => filterAndRenderCode());
    langFilter.addEventListener('change', () => filterAndRenderCode());
    groupBySelect.addEventListener('change', () => {
        currentCodeGroupBy = groupBySelect.value;
        filterAndRenderCode();
    });
}

function filterAndRenderCode() {
    const searchTerm = document.getElementById('code-search').value.toLowerCase();
    const typeFilter = document.getElementById('code-filter-type').value;
    const langFilter = document.getElementById('code-filter-language').value;
    
    let filtered = allCodeElements.filter(el => {
        const matchesSearch = !searchTerm || 
            el.name.toLowerCase().includes(searchTerm) ||
            (el.file_path && el.file_path.toLowerCase().includes(searchTerm));
        const matchesType = !typeFilter || el.element_type === typeFilter;
        const matchesLang = !langFilter || el.language === langFilter;
        return matchesSearch && matchesType && matchesLang;
    });
    
    // Update filter info
    const infoDiv = document.getElementById('code-filter-info');
    if (filtered.length !== allCodeElements.length) {
        infoDiv.textContent = `Showing ${filtered.length} of ${allCodeElements.length} code elements`;
        infoDiv.style.display = 'block';
    } else {
        infoDiv.style.display = 'none';
    }
    
    renderCodeElements(filtered);
}

function renderCodeElements(elements) {
    const container = document.getElementById('code-list');
    
    if (elements.length === 0) {
        container.innerHTML = '<p>No code elements match the current filters.</p>';
        return;
    }
    
    let html = '';
    
    if (currentCodeGroupBy === 'none') {
        // No grouping - show all in a single list with pagination
        html = renderCodeElementsList(elements, false);
    } else {
        // Group elements
        const grouped = {};
        elements.forEach(el => {
            let key;
            if (currentCodeGroupBy === 'type') {
                key = el.element_type || 'unknown';
            } else if (currentCodeGroupBy === 'language') {
                key = el.language || 'unknown';
            } else if (currentCodeGroupBy === 'file') {
                key = el.file_path ? el.file_path.split('/').slice(0, -1).join('/') || 'root' : 'unknown';
            }
            
            if (!grouped[key]) grouped[key] = [];
            grouped[key].push(el);
        });
        
        // Sort groups
        const sortedGroups = Object.entries(grouped).sort((a, b) => {
            if (currentCodeGroupBy === 'file') {
                return a[0].localeCompare(b[0]);
            }
            return a[0].localeCompare(b[0]);
        });
        
        html = sortedGroups.map(([groupKey, elList]) => {
            const groupTitle = currentCodeGroupBy === 'file' 
                ? groupKey === 'root' ? 'Root Directory' : groupKey
                : groupKey;
            const count = elList.length;
            return `
                <div class="detail-section collapsible-section">
                    <h4 class="section-header" onclick="toggleSection(this)">
                        <span class="section-toggle">▼</span>
                        ${escapeHtml(groupTitle)} <span class="section-count">(${count})</span>
                    </h4>
                    <div class="section-content">
                        ${renderCodeElementsList(elList, true)}
                    </div>
                </div>
            `;
        }).join('');
    }
    
    container.innerHTML = html;
}

function renderCodeElementsList(elements, showAll = false) {
    // Sort elements by name
    const sorted = [...elements].sort((a, b) => a.name.localeCompare(b.name));
    
    // If showing all and there are many, limit initial display
    const maxInitial = 50;
    const shouldPaginate = !showAll && sorted.length > maxInitial;
    const displayElements = shouldPaginate ? sorted.slice(0, maxInitial) : sorted;
    
    let html = '<div class="detail-items">';
    html += displayElements.map(el => `
        <div class="detail-item clickable" data-code-name="${escapeHtml(el.name.toLowerCase())}" onclick="showEntityDetail('${currentRepoId || ''}', 'code_element', '${el.id}')">
            <div class="detail-item-header">
                <strong>${escapeHtml(el.name)}</strong>
                <span class="detail-badge">${escapeHtml(el.language || 'unknown')}</span>
            </div>
            ${el.file_path ? `<p class="detail-meta">File: ${escapeHtml(el.file_path)}${el.line_number ? ` (line ${el.line_number})` : ''}</p>` : ''}
            ${el.element_type ? `<p class="detail-meta" style="font-size: 0.75rem; color: var(--text-secondary);">Type: ${escapeHtml(el.element_type)}</p>` : ''}
        </div>
    `).join('');
    html += '</div>';
    
    if (shouldPaginate) {
        html += `
            <div class="pagination-controls">
                <p style="text-align: center; color: var(--text-secondary); margin-top: 1rem;">
                    Showing ${maxInitial} of ${sorted.length} elements
                </p>
                <button class="btn btn-secondary" onclick="loadMoreCodeElements()" style="width: 100%; margin-top: 0.5rem;">
                    Show All ${sorted.length} Elements
                </button>
            </div>
        `;
    }
    
    return html;
}

window.toggleSection = function(header) {
    const section = header.closest('.collapsible-section');
    const content = section.querySelector('.section-content');
    const toggle = header.querySelector('.section-toggle');
    
    if (content.style.display === 'none') {
        content.style.display = 'block';
        toggle.textContent = '▼';
    } else {
        content.style.display = 'none';
        toggle.textContent = '▶';
    }
};

window.loadMoreCodeElements = function() {
    currentCodeGroupBy = 'none';
    document.getElementById('code-group-by').value = 'none';
    filterAndRenderCode();
};

// Store security data for filtering
let allSecurityEntities = [];
let allSecurityVulnerabilities = [];
let securityVulnMap = {};
let currentSecurityGroupBy = 'type';

async function loadSecurity(repoId) {
    currentRepoId = repoId;
    const container = document.getElementById('security-list');
    container.innerHTML = '<p class="loading-text">Loading security information...</p>';
    
    try {
        const entities = await api.getSecurityEntities(repoId);
        const vulnerabilities = await api.getSecurityVulnerabilities(repoId);
        
        allSecurityEntities = entities;
        allSecurityVulnerabilities = vulnerabilities;
        
        // Create vulnerability map by entity_id
        securityVulnMap = {};
        vulnerabilities.forEach(vuln => {
            if (!securityVulnMap[vuln.entity_id]) securityVulnMap[vuln.entity_id] = [];
            securityVulnMap[vuln.entity_id].push(vuln);
        });
        
        if (entities.length === 0) {
            container.innerHTML = '<p>No security entities found. Run analysis first.</p>';
            return;
        }
        
        // Setup filters
        setupSecurityFilters(entities, vulnerabilities);
        
        // Render security entities
        renderSecurityEntities(entities);
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load security entities: ${escapeHtml(error.message)}</p>`;
    }
}

function setupSecurityFilters(entities, vulnerabilities) {
    // Populate type filter
    const typeFilter = document.getElementById('security-filter-type');
    const types = [...new Set(entities.map(e => e.entity_type || 'unknown'))].sort();
    typeFilter.innerHTML = '<option value="">All Types</option>' + 
        types.map(type => {
            const label = type.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
            return `<option value="${escapeHtml(type)}">${escapeHtml(label)}</option>`;
        }).join('');
    
    // Populate provider filter
    const providerFilter = document.getElementById('security-filter-provider');
    const providers = [...new Set(entities.map(e => e.provider || 'unknown'))].sort();
    providerFilter.innerHTML = '<option value="">All Providers</option>' + 
        providers.map(provider => `<option value="${escapeHtml(provider)}">${escapeHtml(provider)}</option>`).join('');
    
    // Setup event listeners
    const searchInput = document.getElementById('security-search');
    const groupBySelect = document.getElementById('security-group-by');
    
    searchInput.addEventListener('input', () => filterAndRenderSecurity());
    typeFilter.addEventListener('change', () => filterAndRenderSecurity());
    providerFilter.addEventListener('change', () => filterAndRenderSecurity());
    document.getElementById('security-filter-severity').addEventListener('change', () => filterAndRenderSecurity());
    groupBySelect.addEventListener('change', () => {
        currentSecurityGroupBy = groupBySelect.value;
        filterAndRenderSecurity();
    });
}

function filterAndRenderSecurity() {
    const searchTerm = document.getElementById('security-search').value.toLowerCase();
    const typeFilter = document.getElementById('security-filter-type').value;
    const providerFilter = document.getElementById('security-filter-provider').value;
    const severityFilter = document.getElementById('security-filter-severity').value;
    
    let filtered = allSecurityEntities.filter(entity => {
        // Search filter
        const matchesSearch = !searchTerm || 
            entity.name.toLowerCase().includes(searchTerm) ||
            (entity.file_path && entity.file_path.toLowerCase().includes(searchTerm)) ||
            (entity.arn && entity.arn.toLowerCase().includes(searchTerm));
        
        // Type filter
        const matchesType = !typeFilter || entity.entity_type === typeFilter;
        
        // Provider filter
        const matchesProvider = !providerFilter || entity.provider === providerFilter;
        
        // Severity filter (check if entity has vulnerabilities with matching severity)
        let matchesSeverity = true;
        if (severityFilter) {
            const entityVulns = securityVulnMap[entity.id] || [];
            matchesSeverity = entityVulns.some(v => v.severity.toLowerCase() === severityFilter);
        }
        
        return matchesSearch && matchesType && matchesProvider && matchesSeverity;
    });
    
    // Update filter info
    const infoDiv = document.getElementById('security-filter-info');
    if (filtered.length !== allSecurityEntities.length) {
        infoDiv.textContent = `Showing ${filtered.length} of ${allSecurityEntities.length} security entities`;
        infoDiv.style.display = 'block';
    } else {
        infoDiv.style.display = 'none';
    }
    
    renderSecurityEntities(filtered);
}

function renderSecurityEntities(entities) {
    const container = document.getElementById('security-list');
    
    if (entities.length === 0) {
        container.innerHTML = '<p>No security entities match the current filters.</p>';
        return;
    }
    
    let html = '';
    
    if (currentSecurityGroupBy === 'none') {
        // No grouping - show all in a single list with pagination
        html = renderSecurityEntitiesList(entities, false);
    } else {
        // Group entities
        const grouped = {};
        entities.forEach(entity => {
            let key;
            if (currentSecurityGroupBy === 'type') {
                key = entity.entity_type || 'unknown';
            } else if (currentSecurityGroupBy === 'provider') {
                key = entity.provider || 'unknown';
            } else if (currentSecurityGroupBy === 'file') {
                key = entity.file_path ? entity.file_path.split('/').slice(0, -1).join('/') || 'root' : 'unknown';
            } else if (currentSecurityGroupBy === 'severity') {
                const entityVulns = securityVulnMap[entity.id] || [];
                if (entityVulns.length === 0) {
                    key = 'none';
                } else {
                    // Use highest severity
                    const severities = ['critical', 'high', 'medium', 'low', 'info'];
                    const highestSeverity = severities.find(s => 
                        entityVulns.some(v => v.severity.toLowerCase() === s)
                    ) || 'info';
                    key = highestSeverity;
                }
            }
            
            if (!grouped[key]) grouped[key] = [];
            grouped[key].push(entity);
        });
        
        // Sort groups
        const sortedGroups = Object.entries(grouped).sort((a, b) => {
            if (currentSecurityGroupBy === 'severity') {
                const severityOrder = { 'critical': 0, 'high': 1, 'medium': 2, 'low': 3, 'info': 4, 'none': 5 };
                return (severityOrder[a[0]] || 99) - (severityOrder[b[0]] || 99);
            }
            return a[0].localeCompare(b[0]);
        });
        
        html = sortedGroups.map(([groupKey, entityList]) => {
            let groupTitle;
            if (currentSecurityGroupBy === 'type') {
                groupTitle = groupKey.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
            } else if (currentSecurityGroupBy === 'severity') {
                groupTitle = groupKey === 'none' ? 'No Vulnerabilities' : 
                    groupKey.charAt(0).toUpperCase() + groupKey.slice(1) + ' Severity';
            } else if (currentSecurityGroupBy === 'file') {
                groupTitle = groupKey === 'root' ? 'Root Directory' : groupKey;
            } else {
                groupTitle = groupKey;
            }
            const count = entityList.length;
            return `
                <div class="detail-section collapsible-section">
                    <h4 class="section-header" onclick="toggleSection(this)">
                        <span class="section-toggle">▼</span>
                        ${escapeHtml(groupTitle)} <span class="section-count">(${count})</span>
                    </h4>
                    <div class="section-content">
                        ${renderSecurityEntitiesList(entityList, true)}
                    </div>
                </div>
            `;
        }).join('');
    }
    
    container.innerHTML = html;
}

function renderSecurityEntitiesList(entities, showAll = false) {
    // Sort entities by name
    const sorted = [...entities].sort((a, b) => a.name.localeCompare(b.name));
    
    // If showing all and there are many, limit initial display
    const maxInitial = 50;
    const shouldPaginate = !showAll && sorted.length > maxInitial;
    const displayEntities = shouldPaginate ? sorted.slice(0, maxInitial) : sorted;
    
    let html = '<div class="detail-items">';
    html += displayEntities.map(entity => {
        // Parse configuration if it's a string
        let config = entity.configuration;
        if (typeof config === 'string') {
            try {
                config = JSON.parse(config);
            } catch (e) {
                config = {};
            }
        }
        
        const entityVulns = securityVulnMap[entity.id] || [];
        const hasVulns = entityVulns.length > 0;
        const type = entity.entity_type || 'unknown';
        
        // Special handling for API keys
        if (type === 'ApiKey') {
            const keyName = config.key_name || entity.name;
            const keyType = config.key_type || 'unknown';
            const provider = config.provider || entity.provider || 'generic';
            const usedByCount = config.used_by_count || 0;
            const serviceCount = config.service_count || 0;
            const usedByElements = config.used_by_elements || [];
            const relatedServices = config.related_services || [];
            const valuePreview = config.value_preview;
            
            return `
            <div class="detail-item clickable ${hasVulns ? 'has-vulnerability' : ''}" onclick="showEntityDetail('${currentRepoId || ''}', 'security_entity', '${entity.id}')">
                <div class="detail-item-header">
                    <strong>${escapeHtml(keyName)}</strong>
                    <span class="detail-badge ${keyType === 'hardcoded' ? 'badge-critical' : 'badge-info'}">${escapeHtml(keyType)}</span>
                    <span class="detail-badge">${escapeHtml(provider)}</span>
                    ${hasVulns ? '<span class="detail-badge badge-warning">⚠ Vulnerable</span>' : ''}
                </div>
                <div class="detail-meta">
                    ${entity.file_path ? `<p><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${entity.line_number ? `:${entity.line_number}` : ''}</p>` : ''}
                    ${valuePreview ? `<p><strong>Value Preview:</strong> <code>${escapeHtml(valuePreview)}</code></p>` : ''}
                    ${usedByCount > 0 ? `<p><strong>Used by:</strong> ${usedByCount} code element(s)</p>` : ''}
                    ${serviceCount > 0 ? `<p><strong>Related Services:</strong> ${serviceCount}</p>` : ''}
                    ${usedByElements.length > 0 ? `<p class="detail-small"><strong>Code Elements:</strong> ${usedByElements.slice(0, 5).map(id => `<code>${escapeHtml(id.substring(0, 8))}...</code>`).join(', ')}${usedByElements.length > 5 ? ' ...' : ''}</p>` : ''}
                    ${relatedServices.length > 0 ? `<p class="detail-small"><strong>Services:</strong> ${relatedServices.slice(0, 5).map(s => escapeHtml(s)).join(', ')}${relatedServices.length > 5 ? ' ...' : ''}</p>` : ''}
                </div>
                ${hasVulns ? `
                <div class="vulnerability-list">
                    ${entityVulns.map(v => `
                        <div class="vulnerability-item severity-${v.severity.toLowerCase()}">
                            <strong>${escapeHtml(v.vulnerability_type)}</strong>
                            <p>${escapeHtml(v.description)}</p>
                            <p class="vulnerability-recommendation">💡 ${escapeHtml(v.recommendation)}</p>
                        </div>
                    `).join('')}
                </div>
                ` : ''}
            </div>
            `;
        }
        
        // Default rendering for other entity types
        return `
        <div class="detail-item clickable ${hasVulns ? 'has-vulnerability' : ''}" onclick="showEntityDetail('${currentRepoId || ''}', 'security_entity', '${entity.id}')">
            <div class="detail-item-header">
                <strong>${escapeHtml(entity.name)}</strong>
                ${entity.provider ? `<span class="detail-badge">${escapeHtml(entity.provider)}</span>` : ''}
                ${hasVulns ? '<span class="detail-badge badge-warning">⚠ Vulnerable</span>' : ''}
            </div>
            ${entity.arn ? `<p class="detail-meta"><strong>ARN:</strong> <code>${escapeHtml(entity.arn)}</code></p>` : ''}
            ${entity.file_path ? `<p class="detail-meta"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${entity.line_number ? `:${entity.line_number}` : ''}</p>` : ''}
            ${hasVulns ? `
            <div class="vulnerability-list">
                ${entityVulns.map(v => `
                    <div class="vulnerability-item severity-${v.severity.toLowerCase()}">
                        <strong>${escapeHtml(v.vulnerability_type)}</strong>
                        <p>${escapeHtml(v.description)}</p>
                        <p class="vulnerability-recommendation">💡 ${escapeHtml(v.recommendation)}</p>
                    </div>
                `).join('')}
            </div>
            ` : ''}
        </div>
        `;
    }).join('');
    html += '</div>';
    
    if (shouldPaginate) {
        html += `
            <div class="pagination-controls">
                <p style="text-align: center; color: var(--text-secondary); margin-top: 1rem;">
                    Showing ${maxInitial} of ${sorted.length} entities
                </p>
                <button class="btn btn-secondary" onclick="loadMoreSecurityEntities()" style="width: 100%; margin-top: 0.5rem;">
                    Show All ${sorted.length} Entities
                </button>
            </div>
        `;
    }
    
    return html;
}

window.loadMoreSecurityEntities = function() {
    currentSecurityGroupBy = 'none';
    document.getElementById('security-group-by').value = 'none';
    filterAndRenderSecurity();
};

async function loadRepositoryGraph(repoId) {
    const container = document.getElementById('repo-graph-container');
    container.innerHTML = '<p class="loading-text">Loading graph...</p>';
    
    try {
        const graph = await api.getGraph(repoId);
        
        if (!graph.nodes || graph.nodes.length === 0) {
            container.innerHTML = '<p>No graph data available. Run analysis first.</p>';
            return;
        }
        
        // Render enhanced graph with repository ID
        renderEnhancedGraph(graph, container, repoId);
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load graph: ${escapeHtml(error.message)}</p>`;
    }
}

// Entity Detail Modal Functions
// currentRepoId is declared earlier in the file

function showEntityDetail(repoId, entityType, entityId) {
    currentRepoId = repoId;
    const modal = document.getElementById('entity-detail-modal');
    const title = document.getElementById('entity-detail-title');
    const body = document.getElementById('entity-detail-body');
    
    modal.style.display = 'flex';
    title.textContent = 'Loading...';
    body.innerHTML = '<p class="loading-text">Loading entity details...</p>';
    
    api.getEntityDetails(repoId, entityType, entityId)
        .then(details => {
            title.textContent = getEntityTitle(entityType, details.entity);
            body.innerHTML = renderEntityDetails(entityType, details);
        })
        .catch(error => {
            title.textContent = 'Error';
            body.innerHTML = `<p class="error-text">Failed to load entity details: ${escapeHtml(error.message)}</p>`;
        });
}

function closeEntityDetailModal() {
    document.getElementById('entity-detail-modal').style.display = 'none';
}

function getEntityTitle(entityType, entity) {
    if (!entity) return 'Entity Details';
    
    switch (entityType) {
        case 'dependency':
            return `${entity.name} (${entity.version})`;
        case 'service':
            return `${entity.name} (${entity.provider})`;
        case 'code_element':
            return `${entity.name} (${entity.element_type})`;
        case 'security_entity':
            return `${entity.name} (${entity.entity_type})`;
        default:
            return 'Entity Details';
    }
}

function renderEntityDetails(entityType, details) {
    const entity = details.entity;
    if (!entity) return '<p>Entity not found</p>';
    
    let html = '<div class="entity-detail-content">';
    
    // Render entity metadata
    html += '<div class="detail-section">';
    html += '<h3>Details</h3>';
    html += '<div class="detail-grid">';
    
    switch (entityType) {
        case 'dependency':
            html += renderDependencyDetails(entity, details);
            break;
        case 'service':
            html += renderServiceDetails(entity, details);
            break;
        case 'code_element':
            html += renderCodeElementDetails(entity, details);
            break;
        case 'security_entity':
            html += renderSecurityEntityDetails(entity, details);
            break;
    }
    
    html += '</div></div>';
    
    // Render relationships
    html += renderRelationships(entityType, details);
    
    html += '</div>';
    return html;
}

function renderDependencyDetails(entity, details) {
    let html = '';
    html += `<div class="detail-item"><strong>Name:</strong> ${escapeHtml(entity.name)}</div>`;
    html += `<div class="detail-item"><strong>Version:</strong> <code>${escapeHtml(entity.version)}</code></div>`;
    html += `<div class="detail-item"><strong>Package Manager:</strong> ${escapeHtml(entity.package_manager)}</div>`;
    html += `<div class="detail-item"><strong>Dev Dependency:</strong> ${entity.is_dev ? 'Yes' : 'No'}</div>`;
    html += `<div class="detail-item"><strong>Optional:</strong> ${entity.is_optional ? 'Yes' : 'No'}</div>`;
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code></div>`;
    return html;
}

function renderServiceDetails(entity, details) {
    let html = '';
    html += `<div class="detail-item"><strong>Name:</strong> ${escapeHtml(entity.name)}</div>`;
    html += `<div class="detail-item"><strong>Provider:</strong> ${escapeHtml(entity.provider)}</div>`;
    html += `<div class="detail-item"><strong>Type:</strong> ${escapeHtml(entity.service_type)}</div>`;
    html += `<div class="detail-item"><strong>Confidence:</strong> ${(entity.confidence * 100).toFixed(1)}%</div>`;
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${entity.line_number ? `:${entity.line_number}` : ''}</div>`;
    if (entity.configuration) {
        try {
            const config = typeof entity.configuration === 'string' ? JSON.parse(entity.configuration) : entity.configuration;
            html += `<div class="detail-item"><strong>Configuration:</strong><pre>${escapeHtml(JSON.stringify(config, null, 2))}</pre></div>`;
        } catch (e) {
            html += `<div class="detail-item"><strong>Configuration:</strong> ${escapeHtml(entity.configuration)}</div>`;
        }
    }
    return html;
}

function renderCodeElementDetails(entity, details) {
    let html = '';
    html += `<div class="detail-item"><strong>Name:</strong> ${escapeHtml(entity.name)}</div>`;
    html += `<div class="detail-item"><strong>Type:</strong> ${escapeHtml(entity.element_type)}</div>`;
    html += `<div class="detail-item"><strong>Language:</strong> ${escapeHtml(entity.language)}</div>`;
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>:${entity.line_number}</div>`;
    if (entity.signature) html += `<div class="detail-item"><strong>Signature:</strong> <code>${escapeHtml(entity.signature)}</code></div>`;
    if (entity.visibility) html += `<div class="detail-item"><strong>Visibility:</strong> ${escapeHtml(entity.visibility)}</div>`;
    if (entity.parameters && entity.parameters.length > 0) {
        html += `<div class="detail-item"><strong>Parameters:</strong> ${entity.parameters.map(p => `<code>${escapeHtml(p)}</code>`).join(', ')}</div>`;
    }
    if (entity.return_type) html += `<div class="detail-item"><strong>Return Type:</strong> <code>${escapeHtml(entity.return_type)}</code></div>`;
    if (entity.doc_comment) html += `<div class="detail-item"><strong>Documentation:</strong><pre>${escapeHtml(entity.doc_comment)}</pre></div>`;
    return html;
}

function renderSecurityEntityDetails(entity, details) {
    let html = '';
    html += `<div class="detail-item"><strong>Name:</strong> ${escapeHtml(entity.name)}</div>`;
    html += `<div class="detail-item"><strong>Type:</strong> ${escapeHtml(entity.entity_type)}</div>`;
    html += `<div class="detail-item"><strong>Provider:</strong> ${escapeHtml(entity.provider)}</div>`;
    if (entity.arn) html += `<div class="detail-item"><strong>ARN:</strong> <code>${escapeHtml(entity.arn)}</code></div>`;
    if (entity.region) html += `<div class="detail-item"><strong>Region:</strong> ${escapeHtml(entity.region)}</div>`;
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${entity.line_number ? `:${entity.line_number}` : ''}</div>`;
    if (entity.configuration) {
        try {
            const config = typeof entity.configuration === 'string' ? JSON.parse(entity.configuration) : entity.configuration;
            html += `<div class="detail-item"><strong>Configuration:</strong><pre>${escapeHtml(JSON.stringify(config, null, 2))}</pre></div>`;
        } catch (e) {
            html += `<div class="detail-item"><strong>Configuration:</strong> ${escapeHtml(entity.configuration)}</div>`;
        }
    }
    return html;
}

function renderRelationships(entityType, details) {
    let html = '';
    
    // Dependencies
    if (details.related_dependencies && details.related_dependencies.length > 0) {
        html += '<div class="detail-section"><h3>Related Dependencies</h3><div class="related-items">';
        details.related_dependencies.forEach(dep => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'dependency', '${dep.id}')">
                <strong>${escapeHtml(dep.name)}</strong> <code>${escapeHtml(dep.version)}</code>
            </div>`;
        });
        html += '</div></div>';
    }
    
    // Services
    if (details.related_services && details.related_services.length > 0) {
        html += '<div class="detail-section"><h3>Related Services</h3><div class="related-items">';
        details.related_services.forEach(svc => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'service', '${svc.id}')">
                <strong>${escapeHtml(svc.name)}</strong> (${escapeHtml(svc.provider)})
            </div>`;
        });
        html += '</div></div>';
    }
    
    // Code Elements
    if (details.callers && details.callers.length > 0) {
        html += '<div class="detail-section"><h3>Callers</h3><div class="related-items">';
        details.callers.forEach(caller => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'code_element', '${caller.id}')">
                <strong>${escapeHtml(caller.name)}</strong> <code>${escapeHtml(caller.file_path)}</code>:${caller.line_number}
            </div>`;
        });
        html += '</div></div>';
    }
    
    if (details.callees && details.callees.length > 0) {
        html += '<div class="detail-section"><h3>Callees</h3><div class="related-items">';
        details.callees.forEach(callee => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'code_element', '${callee.id}')">
                <strong>${escapeHtml(callee.name)}</strong> <code>${escapeHtml(callee.file_path)}</code>:${callee.line_number}
            </div>`;
        });
        html += '</div></div>';
    }
    
    if (details.related_elements && details.related_elements.length > 0) {
        html += '<div class="detail-section"><h3>Related Elements (Same File)</h3><div class="related-items">';
        details.related_elements.forEach(el => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'code_element', '${el.id}')">
                <strong>${escapeHtml(el.name)}</strong> (${escapeHtml(el.element_type)})
            </div>`;
        });
        html += '</div></div>';
    }
    
    // Security Entities
    if (details.relationships && details.relationships.length > 0) {
        html += '<div class="detail-section"><h3>Security Relationships</h3><div class="related-items">';
        details.relationships.forEach(rel => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'security_entity', '${rel.entity.id}')">
                <strong>${escapeHtml(rel.entity.name)}</strong> - ${escapeHtml(rel.relationship_type)}
                ${rel.permissions && rel.permissions.length > 0 ? `<br><small>Permissions: ${rel.permissions.join(', ')}</small>` : ''}
            </div>`;
        });
        html += '</div></div>';
    }
    
    if (details.vulnerabilities && details.vulnerabilities.length > 0) {
        html += '<div class="detail-section"><h3>Vulnerabilities</h3><div class="related-items">';
        details.vulnerabilities.forEach(vuln => {
            html += `<div class="related-item vulnerability-item severity-${vuln.severity.toLowerCase()}">
                <strong>${escapeHtml(vuln.vulnerability_type)}</strong>
                <p>${escapeHtml(vuln.description)}</p>
                <p class="vulnerability-recommendation">💡 ${escapeHtml(vuln.recommendation)}</p>
            </div>`;
        });
        html += '</div></div>';
    }
    
    if (details.related_entities && details.related_entities.length > 0) {
        html += '<div class="detail-section"><h3>Related Entities (Same Provider)</h3><div class="related-items">';
        details.related_entities.forEach(e => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'security_entity', '${e.id}')">
                <strong>${escapeHtml(e.name)}</strong> (${escapeHtml(e.entity_type)})
            </div>`;
        });
        html += '</div></div>';
    }
    
    if (details.related_security_entities && details.related_security_entities.length > 0) {
        html += '<div class="detail-section"><h3>Related Security Entities</h3><div class="related-items">';
        details.related_security_entities.forEach(e => {
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'security_entity', '${e.id}')">
                <strong>${escapeHtml(e.name)}</strong> (${escapeHtml(e.entity_type)})
            </div>`;
        });
        html += '</div></div>';
    }
    
    return html;
}

// Make functions globally available
window.showEntityDetail = showEntityDetail;
window.closeEntityDetailModal = closeEntityDetailModal;

