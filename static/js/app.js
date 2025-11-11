// Main Application
document.addEventListener('DOMContentLoaded', () => {
    initializeApp();
});

function initializeApp() {
    setupNavigation();
    setupModals();
    loadDashboard();
    setupSearch();
    setupGraph();
    setupRepositories();
}

// Navigation
function setupNavigation() {
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            const page = link.getAttribute('data-page');
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
        showError('Failed to load dashboard. Please check your API key.');
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
        <div class="repo-item">
            <div class="repo-info">
                <h3>${escapeHtml(repo.name)}</h3>
                <p>${escapeHtml(repo.url)}</p>
                <p style="font-size: 0.75rem; color: var(--text-secondary);">
                    Branch: ${escapeHtml(repo.branch)} | 
                    ${repo.last_analyzed_at ? `Last analyzed: ${new Date(repo.last_analyzed_at).toLocaleDateString()}` : 'Not analyzed yet'}
                </p>
            </div>
            <div class="repo-actions">
                <button class="btn btn-primary" onclick="analyzeRepository('${repo.id}')">Analyze</button>
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
        
        // Prepare data for vis.js
        const nodes = graphData.nodes.map(node => ({
            id: node.id,
            label: node.name || node.id,
            title: `${node.type}: ${node.name || node.id}`,
            color: getNodeColor(node.type),
            shape: getNodeShape(node.type),
        }));
        
        const edges = graphData.edges.map(edge => ({
            from: edge.source,
            to: edge.target,
            label: edge.relationship_type || '',
            arrows: 'to',
        }));
        
        // Create network
        const data = { nodes, edges };
        const options = {
            nodes: {
                font: { size: 14 },
                borderWidth: 2,
            },
            edges: {
                font: { size: 12, align: 'middle' },
                smooth: { type: 'continuous' },
            },
            physics: {
                enabled: true,
                stabilization: { iterations: 200 },
            },
            interaction: {
                hover: true,
                tooltipDelay: 200,
            },
        };
        
        const network = new vis.Network(container, data, options);
        
        // Show graph info
        document.getElementById('graph-info').innerHTML = `
            <strong>Graph Statistics:</strong>
            <ul>
                <li>Nodes: ${nodes.length}</li>
                <li>Edges: ${edges.length}</li>
                <li>Repository: ${repoId}</li>
            </ul>
        `;
    } catch (error) {
        console.error('Failed to load graph:', error);
        document.getElementById('graph-container').innerHTML = 
            '<div class="graph-placeholder"><p>Failed to load graph. Please try again.</p></div>';
    }
}

function getNodeColor(type) {
    const colors = {
        'repository': '#3b82f6',
        'dependency': '#10b981',
        'service': '#f59e0b',
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
        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            const name = document.getElementById('repo-name').value;
            const url = document.getElementById('repo-url').value;
            const branch = document.getElementById('repo-branch').value || 'main';
            
            try {
                await api.createRepository(name, url, branch);
                modal.classList.remove('active');
                form.reset();
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
    try {
        await api.analyzeRepository(repoId);
        alert('Analysis started! This may take a few moments.');
        loadRepositories();
    } catch (error) {
        alert('Failed to start analysis: ' + error.message);
    }
};

window.viewRepository = function(repoId) {
    // Could navigate to a repository detail page
    alert('Repository detail view coming soon!');
};

