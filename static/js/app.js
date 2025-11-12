// Helper function to generate short ID for display
function getShortId(fullId) {
    if (!fullId) return 'N/A';
    // Use first 8 characters of UUID
    return fullId.substring(0, 8);
}

// Main Application
document.addEventListener('DOMContentLoaded', () => {
    initializeApp();
});

function initializeApp() {
    setupNavigation();
    setupModals();
    setupTheme();
    setupUrlHistory(); // Setup URL history navigation
    loadDashboard();
    setupSearch();
    setupGraph();
    loadVersion();
    // setupRepositories removed - not needed
    
    // Handle deep links from AI Assistant
    handleDeepLinks();
}

// Setup URL history navigation
function setupUrlHistory() {
    // Handle browser back/forward buttons
    window.addEventListener('popstate', async (event) => {
        const hash = window.location.hash;
        
        if (hash && hash.includes('repository-detail')) {
            // Parse repository detail URL
            const urlParams = new URLSearchParams(hash.split('?')[1] || '');
            const repoId = urlParams.get('repo');
            const tab = urlParams.get('tab');
            
            if (repoId) {
                // Show repository detail page first
                showPage('repository-detail', false);
                // Load repository details with optional tab
                await loadRepositoryDetail(repoId, tab);
            }
        } else if (hash) {
            // Regular page navigation
            const pageId = hash.replace('#', '').split('?')[0];
            if (pageId && document.getElementById(pageId)) {
                showPage(pageId, false);
            } else {
                showPage('dashboard', false);
            }
        } else {
            // No hash - show dashboard
            showPage('dashboard', false);
        }
    });
    
    // Initial page load - restore from URL
    const hash = window.location.hash;
    if (hash && hash.includes('repository-detail')) {
        // Will be handled by handleDeepLinks
        return;
    } else if (hash) {
        const pageId = hash.replace('#', '').split('?')[0];
        if (pageId && document.getElementById(pageId)) {
            showPage(pageId, false);
        }
    }
}

function handleDeepLinks() {
    // Check for deep link parameters in URL hash
    const hash = window.location.hash;
    if (hash && hash.includes('repository-detail')) {
        const urlParams = new URLSearchParams(hash.split('?')[1] || '');
        const repoId = urlParams.get('repo');
        const entityId = urlParams.get('entity');
        const entityType = urlParams.get('entityType');
        const tab = urlParams.get('tab');
        
        if (repoId && entityId && entityType) {
            // Navigate to repository detail page first, then open entity modal
            setTimeout(async () => {
                await viewRepository(repoId, tab);
                // Wait a bit for the page to load, then open entity detail
                setTimeout(() => {
                    showEntityDetail(repoId, entityType, entityId);
                }, 800);
            }, 300);
        } else if (repoId) {
            // Navigate to repository detail with optional tab
            setTimeout(async () => {
                await viewRepository(repoId, tab);
            }, 300);
        }
    }
}

// Theme Management
function setupTheme() {
    // Load saved theme preference
    const savedTheme = localStorage.getItem('theme') || 'light';
    setTheme(savedTheme);
    
    // Setup theme toggle button
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
        themeToggle.addEventListener('click', () => {
            const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
            const newTheme = currentTheme === 'light' ? 'dark' : 'light';
            setTheme(newTheme);
        });
    }
}

function setTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
    
    // Update theme icon
    const themeIcon = document.getElementById('theme-icon');
    if (themeIcon) {
        themeIcon.textContent = theme === 'dark' ? '☀️' : '🌙';
    }
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

function showPage(pageId, updateHistory = true) {
    document.querySelectorAll('.page').forEach(page => {
        page.classList.remove('active');
    });
    const targetPage = document.getElementById(pageId);
    if (targetPage) {
        targetPage.classList.add('active');
        
        // Update URL history
        if (updateHistory) {
            const url = new URL(window.location);
            url.hash = pageId === 'dashboard' ? '' : `#${pageId}`;
            history.pushState({ page: pageId }, '', url);
        }
        
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
                <button class="btn btn-danger" onclick="deleteRepository('${repo.id}', '${escapeHtml(repo.name)}')" title="Delete repository">Delete</button>
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
        'test': 'Test',
        'Test': 'Test',
        'test_framework': 'Test Framework',
        'TestFramework': 'Test Framework',
    };
    
    // Map edge types to readable labels (empty string = no label shown)
    const edgeTypeLabels = {
        'DependsOn': 'depends on',
        'depends_on': 'depends on',
        'UsesService': '',  // Hide "uses" - obvious from connection
        'uses_service': '',
        'HasDependency': '',  // Hide "has dependency" - too generic
        'has_dependency': '',
        'UsesPackageManager': '',  // Hide "uses" - obvious
        'uses_package_manager': '',
        'ProvidedBy': 'by',  // Shorten to just "by"
        'provided_by': 'by',
        'HasTest': '',  // Hide "has test" - obvious
        'has_test': '',
        'TestUsesFramework': 'uses',  // Show "uses" for test frameworks
        'test_uses_framework': 'uses',
        'TestTestsCode': 'tests',  // Show "tests" relationship
        'test_tests_code': 'tests',
        'RelatedTo': '',  // Hide generic relationships
        'related_to': '',
    };
    
    // Helper function to check if a node type should be shown
    const isNodeTypeEnabled = (nodeType) => {
        const typeLower = (nodeType || '').toLowerCase();
        if (typeLower.includes('repository')) return enabledNodeTypes.repository;
        if (typeLower.includes('dependency') || typeLower.includes('package')) return enabledNodeTypes.dependency;
        if (typeLower.includes('service') || typeLower.includes('provider')) return enabledNodeTypes.service;
        if (typeLower.includes('code') || typeLower.includes('function') || typeLower.includes('class')) return enabledNodeTypes.code;
        if (typeLower.includes('security')) return enabledNodeTypes.security;
        if (typeLower.includes('test_framework') || typeLower.includes('testframework')) return enabledNodeTypes.test_framework;
        if (typeLower.includes('test') && !typeLower.includes('framework')) return enabledNodeTypes.test;
        return true; // Default to showing unknown types
    };
    
    // Filter nodes based on enabled types
    const filteredNodes = graphData.nodes.filter(node => {
        const nodeType = (node.node_type || node.type || 'unknown').toLowerCase();
        return isNodeTypeEnabled(nodeType);
    });
    
    // Get IDs of filtered nodes for edge filtering
    const enabledNodeIds = new Set(filteredNodes.map(n => n.id));
    
    // Filter edges to only include those connecting enabled nodes
    const filteredEdges = graphData.edges.filter(edge => {
        const sourceId = edge.source_node_id || edge.source;
        const targetId = edge.target_node_id || edge.target;
        return enabledNodeIds.has(sourceId) && enabledNodeIds.has(targetId);
    });
    
    // Prepare nodes with enhanced information
    const nodes = filteredNodes.map(node => {
        // Handle both node_type (from enum) and type (from GraphQL)
        let nodeType = (node.node_type || node.type || 'unknown').toLowerCase();
        // Normalize enum serialization (e.g., "Repository" -> "repository")
        if (nodeTypeLabels[node.node_type || node.type]) {
            nodeType = (node.node_type || node.type).toLowerCase();
        }
        const nodeTypeLabel = nodeTypeLabels[node.node_type || node.type] || nodeTypeLabels[nodeType] || nodeType;
        
        // Build label - remove redundant type indicator for cleaner display
        // Type is already indicated by color and shape
        const label = node.name;
        
        // Get text color based on theme
        const isDarkMode = document.documentElement.getAttribute('data-theme') === 'dark';
        const textColor = isDarkMode ? '#e2e8f0' : '#1e293b';
        
        return {
            id: node.id,
            label: label,
            color: getNodeColor(nodeType),
            font: {
                size: 18,
                face: 'Arial',
                color: textColor,
                multi: false,  // Don't use HTML for labels, just plain text
            },
            shape: getNodeShape(nodeType),
            borderWidth: 2,
            size: 25,
            chosen: {
                node: function(values, id, selected, hovering) {
                    if (hovering || selected) {
                        if (values) {
                            values.borderWidth = 4;
                            values.size = 35;
                            if (values.font) {
                                values.font.size = 20;
                            }
                        }
                    }
                }
            }
        };
    });
    
    // Prepare edges with relationship labels (using filtered edges)
    const edges = filteredEdges.map(edge => {
        // Handle both edge_type (from enum) and relationship_type
        const edgeType = edge.edge_type || edge.relationship_type || edge.type || 'RelatedTo';
        const edgeLabel = edgeTypeLabels[edgeType] || edgeTypeLabels[edgeType.toLowerCase()] || '';
        
        // Only show label if it's not empty (filter out generic/obvious relationships)
        const label = edgeLabel || undefined;
        
        // Get text color based on theme
        const isDarkMode = document.documentElement.getAttribute('data-theme') === 'dark';
        const textColor = isDarkMode ? '#cbd5e1' : '#475569';
        
        return {
            id: edge.id,
            from: edge.source_node_id || edge.source,
            to: edge.target_node_id || edge.target,
            label: label,  // Only show label if not empty/undefined
            arrows: 'to',
            color: {
                color: '#64748b',
                highlight: '#2563eb',
                hover: '#2563eb',
            },
            font: {
                size: 11,
                align: 'middle',
                color: textColor,
                background: isDarkMode ? '#1e293b' : 'white',
                strokeWidth: isDarkMode ? 2 : 1,
                strokeColor: isDarkMode ? '#1e293b' : 'white',
            },
            smooth: {
                type: 'continuous',
                roundness: 0.5,
            },
            width: 2,
        };
    });
    
    // Create network with enhanced options (using filtered data)
    const data = { nodes, edges };
    
    // Store filtered graph data for node details
    const filteredGraphData = {
        nodes: filteredNodes,
        edges: filteredEdges
    };
    
    // Get theme-aware colors
    const isDarkMode = document.documentElement.getAttribute('data-theme') === 'dark';
    const nodeTextColor = isDarkMode ? '#e2e8f0' : '#1e293b';
    const edgeTextColor = isDarkMode ? '#cbd5e1' : '#475569';
    const edgeLabelBg = isDarkMode ? '#1e293b' : 'white';
    
    const options = {
        nodes: {
            font: { 
                size: 18,
                face: 'Arial',
                color: nodeTextColor,
                multi: false,  // Plain text, not HTML
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
                color: edgeTextColor,
                background: edgeLabelBg,
                strokeWidth: isDarkMode ? 2 : 1,
                strokeColor: edgeLabelBg,
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
            hover: false,  // Disable hover tooltips
            tooltipDelay: 0,
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
    
    // Store graph data and repo ID for navigation (use filtered data for node lookup)
    network.graphData = filteredGraphData;
    // Determine repo ID: use provided repoId, or check if we're in repository detail view
    network.repoId = repoId || (container.closest('.page')?.id === 'repository-detail' ? currentRepoId : null);
    
    // Add click handler to show node details
    network.on('click', function(params) {
        if (params.nodes.length > 0) {
            const nodeId = params.nodes[0];
            const node = filteredGraphData.nodes.find(n => n.id === nodeId);
            if (node) {
                showNodeDetails(node, filteredGraphData, network.repoId);
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
    // Normalize node type - handle both enum serialization formats
    let nodeType = (node.node_type || node.type || 'unknown').toLowerCase();
    
    // Handle variations: "serviceprovider" -> "service_provider", "packagemanager" -> "package_manager"
    // Check for variations without underscores first
    if (nodeType === 'serviceprovider' || nodeType.includes('serviceprovider')) {
        nodeType = 'service_provider';
    } else if (nodeType === 'service_provider' || nodeType.includes('service_provider')) {
        nodeType = 'service_provider';
    } else if (nodeType === 'packagemanager' || nodeType.includes('packagemanager')) {
        nodeType = 'package_manager';
    } else if (nodeType === 'package_manager' || nodeType.includes('package_manager')) {
        nodeType = 'package_manager';
    } else if (nodeType === 'codeelement' || nodeType.includes('codeelement')) {
        nodeType = 'code_element';
    } else if (nodeType === 'code_element' || nodeType.includes('code_element')) {
        nodeType = 'code_element';
    } else if (nodeType === 'securityentity' || nodeType.includes('securityentity')) {
        nodeType = 'security_entity';
    } else if (nodeType === 'security_entity' || nodeType.includes('security_entity')) {
        nodeType = 'security_entity';
    } else if (nodeType === 'testframework' || nodeType.includes('testframework') || nodeType.includes('test_framework')) {
        nodeType = 'test_framework';
    } else if (nodeType === 'test' || (nodeType.includes('test') && !nodeType.includes('framework'))) {
        nodeType = 'test';
    }
    
    const tabMapping = {
        'dependency': 'dependencies',
        'service': 'services',
        'code_element': 'code',
        'security_entity': 'security',
        'repository': 'overview',
        'package_manager': 'dependencies',
        'service_provider': 'services',
        'test': 'tests',
        'test_framework': 'tests',
    };
    const targetTab = tabMapping[nodeType] || null;
    
    console.log('[NODE] showNodeDetails - raw node type:', node.node_type || node.type, 'normalized:', nodeType, 'targetTab:', targetTab, 'repoId:', repoId);
    
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
        console.log('[NODE] Creating button for:', { repoId, targetTab, nodeName: node.name });
        actionButtons += `<button onclick="navigateToNodeInDetail('${repoId}', '${targetTab}', '${escapeHtml(node.name)}'); this.closest('.node-details-modal').remove();" style="background: var(--primary-color); color: white;">View in Repository Details</button>`;
    } else if (repoId) {
        // If we have a repo ID but no specific tab, just go to overview
        console.log('[NODE] Creating button for overview:', { repoId, nodeName: node.name });
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
    console.log('[NAV] navigateToNodeInDetail called:', { repoId, tabName, nodeName });
    
    // If we're not already on the repository detail page, navigate there with the target tab
    const currentPage = document.querySelector('.page.active')?.id;
    console.log('[NAV] Current page:', currentPage);
    
    if (currentPage !== 'repository-detail') {
        console.log('[NAV] Not on detail page, calling viewRepository with tab:', tabName);
        await window.viewRepository(repoId, tabName);
        // Wait a bit for the page to load
        await new Promise(resolve => setTimeout(resolve, 500));
    } else {
        // We're already on the detail page, just switch tabs
        console.log('[NAV] Already on detail page, switching to tab:', tabName);
        switchTab(tabName, repoId);
    }
    
    // Wait for tab content to load, then try to find and navigate to the entity
    setTimeout(async () => {
        console.log('[NAV] Finding entity after tab switch:', { repoId, tabName, nodeName });
        await findAndNavigateToEntity(repoId, tabName, nodeName);
    }, 800);
}

async function findAndNavigateToEntity(repoId, tabName, nodeName) {
    try {
        let entityId = null;
        let entityType = null;
        
        // Map tab names to entity types
        const tabToEntityType = {
            'dependencies': 'dependency',
            'services': 'service',
            'code': 'code_element',
            'security': 'security_entity',
        };
        entityType = tabToEntityType[tabName];
        
        if (!entityType) {
            // If no entity type mapping, just scroll to the item
            highlightNodeInTab(tabName, nodeName);
            return;
        }
        
        // Try to find the entity by fetching the list and searching
        let entities = [];
        try {
            switch (entityType) {
                case 'dependency':
                    entities = await api.getDependencies(repoId);
                    break;
                case 'service':
                    entities = await api.getServices(repoId);
                    break;
                case 'code_element':
                    entities = await api.getCodeElements(repoId);
                    break;
                case 'security_entity':
                    entities = await api.getSecurityEntities(repoId);
                    break;
            }
        } catch (error) {
            console.error('Failed to fetch entities:', error);
            // Fall back to highlighting by name
            highlightNodeInTab(tabName, nodeName);
            return;
        }
        
        // Search for the entity by name (case-insensitive, partial match)
        const searchName = nodeName.toLowerCase();
        const matchingEntity = entities.find(entity => {
            const entityName = (entity.name || '').toLowerCase();
            return entityName === searchName || entityName.includes(searchName) || searchName.includes(entityName);
        });
        
        if (matchingEntity && matchingEntity.id) {
            // Found the entity - navigate to its detail view
            showEntityDetail(repoId, entityType, matchingEntity.id);
        } else {
            // Entity not found by exact/partial name match - try to scroll to it
            highlightNodeInTab(tabName, nodeName);
        }
    } catch (error) {
        console.error('Error finding entity:', error);
        // Fall back to highlighting by name
        highlightNodeInTab(tabName, nodeName);
    }
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
    // Get colors from CSS variables to support theme switching
    const root = document.documentElement;
    const computedStyle = getComputedStyle(root);
    
    const getColor = (varName, fallback) => {
        return computedStyle.getPropertyValue(varName).trim() || fallback;
    };
    
    const typeLower = (type || '').toLowerCase();
    const isDarkMode = document.documentElement.getAttribute('data-theme') === 'dark';
    
    // Map node types to entity colors
    let background, border;
    if (typeLower.includes('repository')) {
        background = getColor('--entity-repository', '#f59e0b');
        border = isDarkMode ? '#fbbf24' : '#d97706';
    } else if (typeLower.includes('dependency') || typeLower.includes('package')) {
        background = getColor('--entity-dependency', '#3b82f6');
        border = isDarkMode ? '#60a5fa' : '#2563eb';
    } else if (typeLower.includes('service') || typeLower.includes('provider')) {
        background = getColor('--entity-service', '#10b981');
        border = isDarkMode ? '#34d399' : '#059669';
    } else if (typeLower.includes('code') || typeLower.includes('function') || typeLower.includes('class')) {
        background = getColor('--entity-code', '#8b5cf6');
        border = isDarkMode ? '#a78bfa' : '#7c3aed';
    } else if (typeLower.includes('security') || typeLower.includes('iam') || typeLower.includes('lambda') || typeLower.includes('s3')) {
        background = getColor('--entity-security', '#ef4444');
        border = isDarkMode ? '#f87171' : '#dc2626';
    } else if (typeLower.includes('test_framework') || typeLower.includes('testframework')) {
        background = getColor('--entity-test-framework', '#f472b6');
        border = isDarkMode ? '#f9a8d4' : '#ec4899';
    } else if (typeLower.includes('test') && !typeLower.includes('framework')) {
        background = getColor('--entity-test', '#ec4899');
        border = isDarkMode ? '#f9a8d4' : '#db2777';
    } else {
        background = getColor('--secondary-color', '#64748b');
        border = isDarkMode ? '#94a3b8' : '#475569';
    }
    
    return {
        background: background,
        border: border,
        highlight: {
            background: border,
            border: background,
        },
        hover: {
            background: border,
            border: background,
        },
    };
}

function getNodeShape(type) {
    const typeLower = (type || '').toLowerCase();
    const shapes = {
        'repository': 'box',
        'dependency': 'dot',
        'service': 'diamond',
        'package_manager': 'triangle',
        'service_provider': 'star',
        'code_element': 'triangle',
        'security_entity': 'star',
        'test': 'square',
        'test_framework': 'triangleDown',
    };
    
    // Handle variations
    if (typeLower.includes('test_framework') || typeLower.includes('testframework')) {
        return 'triangleDown';
    } else if (typeLower.includes('test') && !typeLower.includes('framework')) {
        return 'square';
    }
    
    return shapes[typeLower] || shapes[type] || 'dot';
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
    console.log('Starting analysis for repository', repoId);
    
    // Find the repository item to show progress (could be on dashboard or detail page)
    const repoItem = document.querySelector(`[data-repo-id="${repoId}"]`);
    const analyzeBtn = repoItem?.querySelector('.btn-analyze') || document.getElementById('btn-analyze-detail');
    const statusDiv = repoItem?.querySelector('.analysis-status') || document.getElementById('analysis-status-detail');
    
    console.log('Found elements:', { repoItem: !!repoItem, analyzeBtn: !!analyzeBtn, statusDiv: !!statusDiv });
    
    // Show loading state
    if (analyzeBtn) {
        analyzeBtn.disabled = true;
        analyzeBtn.textContent = 'Analyzing...';
    }
    
    if (statusDiv) {
        // Make sure it's visible and prominent
        statusDiv.style.display = 'block';
        statusDiv.style.visibility = 'visible';
        statusDiv.style.opacity = '1';
        statusDiv.innerHTML = `
            <div class="analysis-progress">
                <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.5rem;">
                    <div class="spinner" style="width: 16px; height: 16px; border: 2px solid rgba(37, 99, 235, 0.3); border-top-color: var(--primary-color); border-radius: 50%; animation: spin 1s linear infinite;"></div>
                    <strong>Starting analysis...</strong>
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: 5%"></div>
                </div>
                <div style="font-size: 0.85rem; color: var(--text-secondary); margin-top: 0.5rem;">
                    ⏳ Initializing... This may take several minutes for large repositories.
                </div>
            </div>
        `;
        // Scroll status div into view if needed
        statusDiv.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        console.log('Status div updated and made visible');
    } else {
        console.error('Status div not found! Available IDs:', {
            'analysis-status-detail': !!document.getElementById('analysis-status-detail'),
            'btn-analyze-detail': !!document.getElementById('btn-analyze-detail')
        });
    }
    
    // Poll for real progress updates from the server
    let progressPollInterval = null;
    let pollAttempts = 0;
    const maxPollAttempts = 600; // Stop polling after 10 minutes (600 * 1 second)
    
    const updateProgressFromServer = async () => {
        try {
            const progress = await api.getAnalysisProgress(repoId);
            
            if (statusDiv) {
                const progressPercent = progress.progress_percent || 0;
                const stepName = progress.step_name || 'Processing...';
                const statusMessage = progress.status_message || 'Analysis in progress';
                const currentStep = progress.current_step || 0;
                const totalSteps = progress.total_steps || 9;
                
                // Format elapsed time
                const elapsed = new Date(progress.last_updated) - new Date(progress.started_at);
                const elapsedSeconds = Math.floor(elapsed / 1000);
                const elapsedMinutes = Math.floor(elapsedSeconds / 60);
                const elapsedDisplay = elapsedMinutes > 0 
                    ? `${elapsedMinutes}m ${elapsedSeconds % 60}s`
                    : `${elapsedSeconds}s`;
                
                // Ensure status div is visible
                statusDiv.style.display = 'block';
                statusDiv.style.visibility = 'visible';
                statusDiv.style.opacity = '1';
                
                statusDiv.innerHTML = `
                    <div class="analysis-progress">
                        <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.5rem;">
                            <div class="spinner" style="width: 16px; height: 16px; border: 2px solid rgba(37, 99, 235, 0.3); border-top-color: var(--primary-color); border-radius: 50%; animation: spin 1s linear infinite;"></div>
                            <strong>Analysis in Progress</strong>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: ${progressPercent.toFixed(1)}%"></div>
                        </div>
                        <div class="progress-text">
                            Step ${currentStep}/${totalSteps}: ${escapeHtml(stepName)}
                        </div>
                        <div style="font-size: 0.85rem; color: var(--text-secondary); margin-top: 0.5rem;">
                            ${escapeHtml(statusMessage)}
                        </div>
                        <div style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem;">
                            ⏱️ Elapsed: ${elapsedDisplay} | ${progressPercent.toFixed(1)}% complete
                        </div>
                        ${progress.details ? `
                            <div style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border-radius: 4px;">
                                ${Object.entries(progress.details).map(([key, value]) => 
                                    `<div><strong>${escapeHtml(key)}:</strong> ${escapeHtml(String(value))}</div>`
                                ).join('')}
                            </div>
                        ` : ''}
                    </div>
                `;
                statusDiv.style.display = 'block';
                statusDiv.style.visibility = 'visible';
            }
            
            // Stop polling if analysis is complete or failed
            if (progress.step_name === 'Complete' || progress.step_name === 'Failed' || progress.progress_percent >= 100) {
                if (progressPollInterval) {
                    clearInterval(progressPollInterval);
                    progressPollInterval = null;
                }
                
                // If complete, wait a moment then reload
                if (progress.step_name === 'Complete') {
                    if (statusDiv) {
                        statusDiv.style.display = 'block';
                        statusDiv.style.visibility = 'visible';
                        statusDiv.style.opacity = '1';
                        statusDiv.innerHTML = `
                            <div class="analysis-success" style="background: rgba(34, 197, 94, 0.1); border: 1px solid #22c55e; color: #22c55e; padding: 0.75rem; border-radius: 0.375rem;">
                                <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.5rem;">
                                    <span style="font-size: 1.2rem;">✓</span>
                                    <strong style="font-size: 1rem;">Analysis Complete!</strong>
                                </div>
                                <div class="progress-bar">
                                    <div class="progress-fill" style="width: 100%; background: #22c55e;"></div>
                                </div>
                                <div style="font-size: 0.85rem; color: var(--text-secondary); margin-top: 0.5rem;">
                                    Reloading repository data...
                                </div>
                            </div>
                        `;
                    }
                    setTimeout(() => {
                        loadRepositories();
                        if (document.getElementById('repository-detail')?.classList.contains('active')) {
                            loadRepositoryDetail(repoId);
                        }
                    }, 2000);
                }
                return false; // Stop polling
            }
            
            pollAttempts = 0; // Reset attempts on successful poll
            return true; // Continue polling
        } catch (error) {
            pollAttempts++;
            
            // If we get a 404, the analysis might not have started yet, keep trying
            if (error.message && error.message.includes('404')) {
                if (statusDiv) {
                    statusDiv.style.display = 'block';
                    statusDiv.style.visibility = 'visible';
                    statusDiv.style.opacity = '1';
                    statusDiv.innerHTML = `
                        <div class="analysis-progress">
                            <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.5rem;">
                                <div class="spinner" style="width: 16px; height: 16px; border: 2px solid rgba(37, 99, 235, 0.3); border-top-color: var(--primary-color); border-radius: 50%; animation: spin 1s linear infinite;"></div>
                                <strong>Starting analysis...</strong>
                            </div>
                            <div class="progress-bar">
                                <div class="progress-fill" style="width: 10%"></div>
                            </div>
                            <div class="progress-text">Waiting for analysis to begin...</div>
                            <div style="font-size: 0.85rem; color: var(--text-secondary); margin-top: 0.5rem;">
                                ⏳ Initializing server-side analysis...
                            </div>
                        </div>
                    `;
                }
                return pollAttempts < 10; // Try for 10 seconds before giving up
            }
            
            // For other errors, log but keep trying for a bit
            console.warn('Failed to get progress:', error);
            if (pollAttempts >= maxPollAttempts) {
                if (statusDiv) {
                    statusDiv.innerHTML = `
                        <div class="analysis-error">
                            <strong>⚠ Progress tracking unavailable</strong>
                            <div class="error-details">Unable to fetch progress updates. Analysis may still be running.</div>
                        </div>
                    `;
                }
                return false; // Stop polling
            }
            return true; // Continue polling
        }
    };
    
    // Start polling immediately, then every 1 second
    updateProgressFromServer();
    progressPollInterval = setInterval(async () => {
        const shouldContinue = await updateProgressFromServer();
        if (!shouldContinue && progressPollInterval) {
            clearInterval(progressPollInterval);
            progressPollInterval = null;
        }
    }, 1000); // Poll every second for real-time updates
    
    try {
        console.log(`Starting analysis for repository ${repoId}...`);
        const result = await api.analyzeRepository(repoId);
        
        // Clear the polling interval - progress updates will handle completion
        if (progressPollInterval) {
            clearInterval(progressPollInterval);
            progressPollInterval = null;
        }
        
        // The progress polling will handle showing completion
        // But if the API call completes immediately, show results
        if (result.results) {
            if (statusDiv) {
                // Ensure we display numbers, not objects or arrays
                const totalDeps = typeof result.results.total_dependencies === 'number' 
                    ? result.results.total_dependencies 
                    : (Array.isArray(result.results.total_dependencies) 
                        ? result.results.total_dependencies.length 
                        : 0);
                const servicesFound = typeof result.results.services_found === 'number' 
                    ? result.results.services_found 
                    : 0;
                const codeElementsFound = typeof result.results.code_elements_found === 'number' 
                    ? result.results.code_elements_found 
                    : 0;
                const securityEntitiesFound = typeof result.results.security_entities_found === 'number' 
                    ? result.results.security_entities_found 
                    : 0;
                
                statusDiv.innerHTML = `
                    <div class="analysis-success">
                        <strong>✓ Analysis Complete!</strong>
                        <div class="analysis-results">
                            <div>📦 ${totalDeps} dependencies</div>
                            <div>🔌 ${servicesFound} services</div>
                            <div>📝 ${codeElementsFound} code elements</div>
                            <div>🔒 ${securityEntitiesFound} security entities</div>
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
                if (document.getElementById('repository-detail')?.classList.contains('active')) {
                    loadRepositoryDetail(repoId);
                }
            }, 2000);
        }
        
        console.log('Analysis started:', result);
    } catch (error) {
        // Clear the polling interval
        if (progressPollInterval) {
            clearInterval(progressPollInterval);
            progressPollInterval = null;
        }
        
        console.error('Analysis error:', error);
        
        if (statusDiv) {
            statusDiv.innerHTML = `
                <div class="analysis-error">
                    <strong>✗ Analysis Failed</strong>
                    <div class="error-details">${escapeHtml(error.message)}</div>
                </div>
            `;
            statusDiv.style.display = 'block';
            statusDiv.style.visibility = 'visible';
        }
        
        if (analyzeBtn) {
            analyzeBtn.disabled = false;
            analyzeBtn.textContent = 'Analyze';
        }
        
        alert('Failed to start analysis: ' + error.message);
    }
};

window.viewRepository = async function(repoId, initialTab = null) {
    console.log('[VIEW] viewRepository called:', { repoId, initialTab });
    
    // Update URL with repository ID and optional tab
    const url = new URL(window.location);
    url.hash = `#repository-detail?repo=${repoId}${initialTab ? `&tab=${initialTab}` : ''}`;
    history.pushState({ page: 'repository-detail', repoId, tab: initialTab }, '', url);
    
    // Show repository detail page (don't update history - already done above)
    showPage('repository-detail', false);
    
    // Update navigation
    document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.remove('active');
    });
    
    // Load repository details with optional initial tab
    await loadRepositoryDetail(repoId, initialTab);
};

// Delete repository function
window.deleteRepository = async function(repoId, repoName) {
    const confirmed = confirm(
        `Are you sure you want to delete "${repoName}"?\n\n` +
        `This will permanently delete:\n` +
        `- Repository registration\n` +
        `- All dependencies\n` +
        `- All services\n` +
        `- All code elements\n` +
        `- All security entities\n` +
        `- All graph data\n` +
        `- Cached repository files\n\n` +
        `This action cannot be undone.`
    );
    
    if (!confirmed) {
        return;
    }
    
    try {
        await api.deleteRepository(repoId);
        
        // Show success message
        alert(`Repository "${repoName}" has been deleted successfully.`);
        
        // If we're on the repository detail page, go back to repositories list
        const currentPage = document.querySelector('.page.active')?.id;
        if (currentPage === 'repository-detail') {
            showPage('repositories');
        }
        
        // Reload repositories list
        await loadRepositories();
        
        // Reload dashboard if visible
        if (currentPage === 'dashboard') {
            await loadDashboard();
        }
        
        // Reload graph repository list
        await loadRepositoriesForGraph();
        
    } catch (error) {
        console.error('Failed to delete repository:', error);
        alert(`Failed to delete repository: ${error.message}`);
    }
};

async function generateReport(repoId, repoName) {
    try {
        // Show loading indicator
        const reportBtn = document.getElementById('btn-generate-report');
        const originalText = reportBtn.textContent;
        reportBtn.disabled = true;
        reportBtn.textContent = 'Generating...';
        
        // Fetch the HTML report
        const html = await api.getReport(repoId);
        
        // Create a new window and write the HTML
        const reportWindow = window.open('', '_blank');
        if (!reportWindow) {
            alert('Please allow pop-ups to generate the report');
            reportBtn.disabled = false;
            reportBtn.textContent = originalText;
            return;
        }
        
        reportWindow.document.write(html);
        reportWindow.document.close();
        
        // Reset button
        reportBtn.disabled = false;
        reportBtn.textContent = originalText;
        
    } catch (error) {
        console.error('Failed to generate report:', error);
        alert(`Failed to generate report: ${error.message}`);
        const reportBtn = document.getElementById('btn-generate-report');
        reportBtn.disabled = false;
        reportBtn.textContent = '📄 Generate Report';
    }
}

let currentRepoId = null;
let currentRepoData = null; // Store full repository data for file path resolution

// Helper function to get the local file path for a repository
function getRepositoryLocalPath(repo) {
    if (!repo || !repo.url) return null;
    
    const url = repo.url;
    
    // Handle file:// URLs
    if (url.startsWith('file://')) {
        let path = url.replace(/^file:\/\//, '');
        // Handle triple slash (file:///) for absolute paths
        if (path.startsWith('//') && path.length > 2) {
            path = path.substring(1);
        }
        return path;
    }
    
    // Handle absolute paths
    if (url.startsWith('/')) {
        return url;
    }
    
    // Handle relative paths
    if (url.startsWith('./') || url.startsWith('../')) {
        return url;
    }
    
    // For remote URLs, construct cache path
    // Extract repo name from URL (e.g., github.com/user/repo -> user-repo)
    try {
        const urlObj = new URL(url.replace(/^git@/, 'https://').replace(/\.git$/, ''));
        const pathParts = urlObj.pathname.split('/').filter(p => p);
        if (pathParts.length >= 2) {
            const repoName = `${pathParts[pathParts.length - 2]}-${pathParts[pathParts.length - 1]}`;
            // Default cache path (matches backend default)
            return `./cache/repos/${repoName}`;
        }
    } catch (e) {
        // Not a valid URL, might be a local path
        return url;
    }
    
    return null;
}

// Helper function to create a file link
function createFileLink(filePath, lineNumber = null, repoData = null) {
    if (!filePath) return '';
    
    const repoPath = repoData ? getRepositoryLocalPath(repoData) : null;
    if (!repoPath) return '';
    
    // Construct full path
    // file_path is relative to repo root, so combine with repo path
    let fullPath;
    if (filePath.startsWith('/')) {
        // Absolute path - use as is (might be from absolute repo path)
        fullPath = filePath;
    } else if (filePath.startsWith('./') || filePath.startsWith('../')) {
        // Relative path - resolve relative to repo path
        fullPath = filePath;
    } else {
        // Relative to repo root - combine with repo path
        const repoPathNormalized = repoPath.replace(/\\/g, '/').replace(/\/$/, '');
        const filePathNormalized = filePath.replace(/\\/g, '/').replace(/^\//, '');
        fullPath = `${repoPathNormalized}/${filePathNormalized}`;
    }
    
    // Normalize path separators and ensure absolute path for VS Code
    let normalizedPath = fullPath.replace(/\\/g, '/');
    
    // Ensure absolute path for VS Code (needs leading slash on Unix, drive letter on Windows)
    if (!normalizedPath.match(/^[A-Za-z]:/) && !normalizedPath.startsWith('/')) {
        // Try to make it absolute - if repoPath is absolute, use it
        if (repoPath && (repoPath.startsWith('/') || repoPath.match(/^[A-Za-z]:/))) {
            normalizedPath = fullPath;
        } else {
            // Can't determine absolute path, skip VS Code link
            normalizedPath = null;
        }
    }
    
    if (!normalizedPath) {
        // Fallback: just show file path without links
        return '';
    }
    
    // Create editor link using configurable protocol (default: vscode)
    // Supported protocols: vscode, vscode-insiders, cursor, code, sublime, atom, etc.
    const editorPath = normalizedPath.startsWith('/') || normalizedPath.match(/^[A-Za-z]:/)
        ? normalizedPath
        : `/${normalizedPath}`;
    
    // Map common editor protocols to their URL schemes
    const editorProtocolMap = {
        'vscode': 'vscode://file',
        'vscode-insiders': 'vscode-insiders://file',
        'cursor': 'cursor://file',
        'code': 'code://file',
        'sublime': 'subl://file',
        'atom': 'atom://file',
        'webstorm': 'webstorm://open?file=',
        'idea': 'idea://open?file=',
    };
    
    const editorScheme = editorProtocolMap[editorProtocol] || `${editorProtocol}://file`;
    const editorLink = `${editorScheme}${editorPath}${lineNumber ? `:${lineNumber}` : ''}`;
    const editorName = editorProtocol === 'vscode' ? 'VS Code' : 
                      editorProtocol === 'cursor' ? 'Cursor' :
                      editorProtocol === 'vscode-insiders' ? 'VS Code Insiders' :
                      editorProtocol.charAt(0).toUpperCase() + editorProtocol.slice(1);
    
    // Create file:// link for macOS Finder (needs triple slash for absolute paths)
    // macOS Finder: file:///absolute/path (three slashes)
    // Windows: file:///C:/path (three slashes)
    const fileLink = normalizedPath.match(/^[A-Za-z]:/)
        ? `file:///${normalizedPath.replace(/\\/g, '/')}` // Windows: file:///C:/path
        : `file://${normalizedPath}`; // Unix/macOS: file:///path (already has leading slash, becomes file:///path)
    
    return `
        <a href="${editorLink}" 
           onclick="event.stopPropagation(); return true;"
           title="Open in ${editorName}${lineNumber ? ` (line ${lineNumber})` : ''}"
           style="margin-left: 0.5rem; color: var(--primary-color); text-decoration: none; font-size: 0.875rem; white-space: nowrap;">
            🔗 Open in Editor
        </a>
        <a href="#" 
           onclick="event.stopPropagation(); 
                    const path = '${normalizedPath}';
                    if (navigator.clipboard && navigator.clipboard.writeText) {
                        navigator.clipboard.writeText(path).then(() => {
                            alert('Path copied to clipboard!\\n\\n' + path + '\\n\\nOn macOS: Press Cmd+Shift+G in Finder and paste the path.\\nOn Windows: Paste the path in File Explorer address bar.');
                        }).catch(() => {
                            // Fallback: select text
                            const textarea = document.createElement('textarea');
                            textarea.value = path;
                            textarea.style.position = 'fixed';
                            textarea.style.opacity = '0';
                            document.body.appendChild(textarea);
                            textarea.select();
                            try {
                                document.execCommand('copy');
                                alert('Path copied to clipboard!\\n\\n' + path + '\\n\\nOn macOS: Press Cmd+Shift+G in Finder and paste the path.\\nOn Windows: Paste the path in File Explorer address bar.');
                            } catch(e) {
                                prompt('Copy this path:', path);
                            }
                            document.body.removeChild(textarea);
                        });
                    } else {
                        // Fallback: prompt to copy
                        prompt('Copy this path (Cmd+C / Ctrl+C):', path);
                    }
                    return false;"
           title="Copy path to clipboard (macOS: Cmd+Shift+G in Finder, Windows: paste in address bar)"
           style="margin-left: 0.5rem; color: var(--text-secondary); text-decoration: none; font-size: 0.875rem; white-space: nowrap; cursor: pointer;">
            📁 Show in Finder
        </a>
    `;
}

async function loadRepositoryDetail(repoId, initialTab = null) {
    console.log('[LOAD] loadRepositoryDetail called:', { repoId, initialTab });
    currentRepoId = repoId;
    
    // Clear all cached data when switching repositories
    allDocumentation = [];
    
    try {
        // Load repository info
        const repo = await api.getRepository(repoId);
        currentRepoData = repo; // Store repo data for file path resolution
        
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
        
        // Setup delete button
        const deleteBtn = document.getElementById('btn-delete-detail');
        deleteBtn.onclick = () => {
            deleteRepository(repoId, repo.name);
        };
        
        // Setup report generation button
        const reportBtn = document.getElementById('btn-generate-report');
        reportBtn.onclick = () => {
            generateReport(repoId, repo.name);
        };
        
        // Setup tabs
        setupRepositoryTabs(repoId);
        
        // Load overview stats
        await loadRepositoryOverview(repoId);
        
        // Load initial tab (use provided tab or default to overview)
        const tabToLoad = initialTab || 'overview';
        console.log('[LOAD] Switching to tab:', tabToLoad, '(initialTab:', initialTab, ')');
        switchTab(tabToLoad, repoId);
        
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

function switchTab(tabName, repoId, updateHistory = true) {
    console.log('[TAB] switchTab called:', { tabName, repoId });
    
    // Update URL with tab parameter
    if (updateHistory && repoId) {
        const url = new URL(window.location);
        url.hash = `#repository-detail?repo=${repoId}&tab=${tabName}`;
        history.pushState({ page: 'repository-detail', repoId, tab: tabName }, '', url);
    }
    
    // Update tab states
    document.querySelectorAll('.repo-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.repo-tab-content').forEach(c => c.classList.remove('active'));
    
    const tabButton = document.querySelector(`[data-tab="${tabName}"]`);
    const tabContent = document.getElementById(`tab-${tabName}`);
    
    console.log('[TAB] Tab button found:', !!tabButton, 'Tab content found:', !!tabContent);
    
    if (tabButton) {
        tabButton.classList.add('active');
    } else {
        console.warn('[TAB] Tab button not found for:', tabName);
    }
    
    if (tabContent) {
        tabContent.classList.add('active');
    } else {
        console.warn('[TAB] Tab content not found for:', tabName);
    }
    
    // Load tab content
    switch(tabName) {
        case 'overview':
            console.log('[TAB] Loading overview');
            loadRepositoryOverview(repoId);
            break;
        case 'dependencies':
            console.log('[TAB] Loading dependencies');
            loadDependencies(repoId);
            break;
        case 'services':
            console.log('[TAB] Loading services');
            loadServices(repoId);
            break;
        case 'code':
            console.log('[TAB] Loading code');
            loadCodeElements(repoId);
            break;
        case 'security':
            console.log('[TAB] Loading security');
            loadSecurity(repoId);
            break;
        case 'tools':
            console.log('[TAB] Loading tools');
            loadTools(repoId);
            break;
        case 'tests':
            console.log('[TAB] Loading tests');
            loadTests(repoId);
            break;
        case 'documentation':
            console.log('[TAB] Loading documentation');
            loadDocumentation(repoId);
            break;
        case 'graph':
            console.log('[TAB] Loading graph');
            loadRepositoryGraph(repoId);
            break;
        default:
            console.warn('[TAB] Unknown tab name:', tabName);
    }
}

async function loadRepositoryOverview(repoId) {
    try {
        const [deps, services, code, security, tools, tests] = await Promise.all([
            api.getDependencies(repoId).catch(() => []),
            api.getServices(repoId).catch(() => []),
            api.getCodeElements(repoId).catch(() => []),
            api.getSecurityEntities(repoId).catch(() => []),
            api.getTools(repoId).catch(() => []),
            api.getTests(repoId).catch(() => [])
        ]);
        
        document.getElementById('stat-deps-count').textContent = deps.length || 0;
        document.getElementById('stat-services-count').textContent = services.length || 0;
        document.getElementById('stat-code-count').textContent = code.length || 0;
        document.getElementById('stat-security-count').textContent = security.length || 0;
        // Note: Tests count not shown in overview stats yet
    } catch (error) {
        console.error('Failed to load overview:', error);
    }
}

// Store dependencies for filtering
let allDependencies = [];
let currentDependenciesGroupBy = 'package_manager';

async function loadDependencies(repoId) {
    currentRepoId = repoId;
    const container = document.getElementById('dependencies-list');
    container.innerHTML = '<p class="loading-text">Loading dependencies...</p>';
    
    try {
        const deps = await api.getDependencies(repoId);
        allDependencies = deps;
        
        if (deps.length === 0) {
            container.innerHTML = '<p>No dependencies found. Run analysis first.</p>';
            return;
        }
        
        // Populate filter dropdowns
        populateDependencyFilters(deps);
        
        // Initial render
        filterAndRenderDependencies();
        
        // Setup filter event listeners
        setupDependencyFilters();
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load dependencies: ${escapeHtml(error.message)}</p>`;
    }
}

function populateDependencyFilters(deps) {
    const packageManagerSelect = document.getElementById('dependencies-filter-package-manager');
    const packageManagers = [...new Set(deps.map(d => d.package_manager).filter(Boolean))].sort();
    
    // Clear existing options except "All"
    packageManagerSelect.innerHTML = '<option value="">All Package Managers</option>';
    packageManagers.forEach(pm => {
        const option = document.createElement('option');
        option.value = pm;
        option.textContent = pm;
        packageManagerSelect.appendChild(option);
    });
}

function setupDependencyFilters() {
    const searchInput = document.getElementById('dependencies-search');
    const groupBySelect = document.getElementById('dependencies-group-by');
    const packageManagerFilter = document.getElementById('dependencies-filter-package-manager');
    const typeFilter = document.getElementById('dependencies-filter-type');
    
    searchInput.addEventListener('input', () => filterAndRenderDependencies());
    packageManagerFilter.addEventListener('change', () => filterAndRenderDependencies());
    typeFilter.addEventListener('change', () => filterAndRenderDependencies());
    groupBySelect.addEventListener('change', () => {
        currentDependenciesGroupBy = groupBySelect.value;
        filterAndRenderDependencies();
    });
}

function filterAndRenderDependencies() {
    const searchTerm = document.getElementById('dependencies-search').value.toLowerCase();
    const packageManagerFilter = document.getElementById('dependencies-filter-package-manager').value;
    const typeFilter = document.getElementById('dependencies-filter-type').value;
    
    let filtered = allDependencies.filter(dep => {
        const matchesSearch = !searchTerm || 
            dep.name.toLowerCase().includes(searchTerm) ||
            (dep.version && dep.version.toLowerCase().includes(searchTerm)) ||
            (dep.file_path && dep.file_path.toLowerCase().includes(searchTerm));
        const matchesPackageManager = !packageManagerFilter || dep.package_manager === packageManagerFilter;
        const matchesType = !typeFilter || 
            (typeFilter === 'dev' && dep.is_dev) ||
            (typeFilter === 'prod' && !dep.is_dev);
        return matchesSearch && matchesPackageManager && matchesType;
    });
    
    // Update filter info
    const filterInfo = document.getElementById('dependencies-filter-info');
    if (filterInfo) {
        const activeFilters = [];
        if (searchTerm) activeFilters.push(`search: "${searchTerm}"`);
        if (packageManagerFilter) activeFilters.push(`package manager: ${packageManagerFilter}`);
        if (typeFilter) activeFilters.push(`type: ${typeFilter}`);
        filterInfo.textContent = activeFilters.length > 0 
            ? `Showing ${filtered.length} of ${allDependencies.length} dependencies (${activeFilters.join(', ')})`
            : `Showing ${filtered.length} dependencies`;
    }
    
    // Group and render
    const container = document.getElementById('dependencies-list');
    if (filtered.length === 0) {
        container.innerHTML = '<p>No dependencies match the current filters.</p>';
        return;
    }
    
    let html = '';
    if (currentDependenciesGroupBy === 'none') {
        html = '<div class="detail-items">';
        html += filtered.map(dep => `
            <div class="detail-item clickable" onclick="showEntityDetail('${currentRepoId}', 'dependency', '${dep.id}')">
                <div class="detail-item-header">
                    <strong>${escapeHtml(dep.name)}</strong>
                    <div style="display: flex; gap: 0.5rem; align-items: center;">
                        <span class="entity-id-badge" title="Entity ID: ${dep.id}">ID: ${getShortId(dep.id)}</span>
                        <span class="detail-badge">${escapeHtml(dep.version || 'unknown')}</span>
                    </div>
                </div>
                <div class="detail-meta">
                    ${dep.package_manager ? `<span>${escapeHtml(dep.package_manager)}</span>` : ''}
                    ${dep.is_dev ? '<span class="badge badge-secondary">dev</span>' : ''}
                </div>
                ${dep.file_path ? `<p class="detail-meta">Found in: <code>${escapeHtml(dep.file_path)}</code>${createFileLink(dep.file_path, null, currentRepoData)}</p>` : ''}
            </div>
        `).join('');
        html += '</div>';
    } else {
        const grouped = {};
        filtered.forEach(dep => {
            let key = 'unknown';
            if (currentDependenciesGroupBy === 'package_manager') {
                key = dep.package_manager || 'unknown';
            } else if (currentDependenciesGroupBy === 'type') {
                key = dep.is_dev ? 'Dev Dependencies' : 'Production Dependencies';
            } else if (currentDependenciesGroupBy === 'file') {
                key = dep.file_path || 'unknown';
            }
            if (!grouped[key]) grouped[key] = [];
            grouped[key].push(dep);
        });
        
        html = Object.entries(grouped).map(([key, depsList]) => `
            <div class="detail-section">
                <h4>${escapeHtml(key)}</h4>
                <div class="detail-items">
                    ${depsList.map(dep => `
                        <div class="detail-item clickable" onclick="showEntityDetail('${currentRepoId}', 'dependency', '${dep.id}')">
                            <div class="detail-item-header">
                                <strong>${escapeHtml(dep.name)}</strong>
                                <div style="display: flex; gap: 0.5rem; align-items: center;">
                                    <span class="entity-id-badge" title="Entity ID: ${dep.id}">ID: ${getShortId(dep.id)}</span>
                                    <span class="detail-badge">${escapeHtml(dep.version || 'unknown')}</span>
                                </div>
                            </div>
                            ${dep.file_path && currentDependenciesGroupBy !== 'file' ? `<p class="detail-meta">Found in: <code>${escapeHtml(dep.file_path)}</code>${createFileLink(dep.file_path, null, currentRepoData)}</p>` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `).join('');
    }
    
    container.innerHTML = html;
}

// Store services for filtering
let allServices = [];
let currentServicesGroupBy = 'provider';

async function loadServices(repoId) {
    currentRepoId = repoId;
    const container = document.getElementById('services-list');
    container.innerHTML = '<p class="loading-text">Loading services...</p>';
    
    try {
        const services = await api.getServices(repoId);
        allServices = services;
        
        if (services.length === 0) {
            container.innerHTML = '<p>No services found. Run analysis first.</p>';
            return;
        }
        
        // Populate filter dropdowns
        populateServiceFilters(services);
        
        // Initial render
        filterAndRenderServices();
        
        // Setup filter event listeners
        setupServiceFilters();
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load services: ${escapeHtml(error.message)}</p>`;
    }
}

function populateServiceFilters(services) {
    const providerSelect = document.getElementById('services-filter-provider');
    const typeSelect = document.getElementById('services-filter-type');
    
    const providers = [...new Set(services.map(s => s.provider).filter(Boolean))].sort();
    const types = [...new Set(services.map(s => s.service_type).filter(Boolean))].sort();
    
    // Clear existing options except "All"
    providerSelect.innerHTML = '<option value="">All Providers</option>';
    providers.forEach(provider => {
        const option = document.createElement('option');
        option.value = provider;
        option.textContent = provider;
        providerSelect.appendChild(option);
    });
    
    typeSelect.innerHTML = '<option value="">All Types</option>';
    types.forEach(type => {
        const option = document.createElement('option');
        option.value = type;
        option.textContent = type;
        typeSelect.appendChild(option);
    });
}

function setupServiceFilters() {
    const searchInput = document.getElementById('services-search');
    const groupBySelect = document.getElementById('services-group-by');
    const providerFilter = document.getElementById('services-filter-provider');
    const typeFilter = document.getElementById('services-filter-type');
    
    searchInput.addEventListener('input', () => filterAndRenderServices());
    providerFilter.addEventListener('change', () => filterAndRenderServices());
    typeFilter.addEventListener('change', () => filterAndRenderServices());
    groupBySelect.addEventListener('change', () => {
        currentServicesGroupBy = groupBySelect.value;
        filterAndRenderServices();
    });
}

function filterAndRenderServices() {
    const searchTerm = document.getElementById('services-search').value.toLowerCase();
    const providerFilter = document.getElementById('services-filter-provider').value;
    const typeFilter = document.getElementById('services-filter-type').value;
    
    let filtered = allServices.filter(svc => {
        const matchesSearch = !searchTerm || 
            svc.name.toLowerCase().includes(searchTerm) ||
            (svc.provider && svc.provider.toLowerCase().includes(searchTerm)) ||
            (svc.service_type && svc.service_type.toLowerCase().includes(searchTerm)) ||
            (svc.file_path && svc.file_path.toLowerCase().includes(searchTerm));
        const matchesProvider = !providerFilter || svc.provider === providerFilter;
        const matchesType = !typeFilter || svc.service_type === typeFilter;
        return matchesSearch && matchesProvider && matchesType;
    });
    
    // Update filter info
    const filterInfo = document.getElementById('services-filter-info');
    if (filterInfo) {
        const activeFilters = [];
        if (searchTerm) activeFilters.push(`search: "${searchTerm}"`);
        if (providerFilter) activeFilters.push(`provider: ${providerFilter}`);
        if (typeFilter) activeFilters.push(`type: ${typeFilter}`);
        filterInfo.textContent = activeFilters.length > 0 
            ? `Showing ${filtered.length} of ${allServices.length} services (${activeFilters.join(', ')})`
            : `Showing ${filtered.length} services`;
    }
    
    // Group and render
    const container = document.getElementById('services-list');
    if (filtered.length === 0) {
        container.innerHTML = '<p>No services match the current filters.</p>';
        return;
    }
    
    let html = '';
    if (currentServicesGroupBy === 'none') {
        html = '<div class="detail-items">';
        html += filtered.map(svc => `
            <div class="detail-item clickable" onclick="showEntityDetail('${currentRepoId}', 'service', '${svc.id}')">
                <div class="detail-item-header">
                    <strong>${escapeHtml(svc.name)}</strong>
                    <div style="display: flex; gap: 0.5rem; align-items: center;">
                        <span class="entity-id-badge" title="Entity ID: ${svc.id}">ID: ${getShortId(svc.id)}</span>
                        <span class="detail-badge">${escapeHtml(svc.service_type || 'service')}</span>
                    </div>
                </div>
                <div class="detail-meta">
                    ${svc.provider ? `<span>${escapeHtml(svc.provider)}</span>` : ''}
                </div>
                ${svc.file_path ? `<p class="detail-meta">Found in: <code>${escapeHtml(svc.file_path)}</code>${createFileLink(svc.file_path, svc.line_number, currentRepoData)}</p>` : ''}
            </div>
        `).join('');
        html += '</div>';
    } else {
        const grouped = {};
        filtered.forEach(svc => {
            let key = 'unknown';
            if (currentServicesGroupBy === 'provider') {
                key = svc.provider || 'unknown';
            } else if (currentServicesGroupBy === 'type') {
                key = svc.service_type || 'unknown';
            } else if (currentServicesGroupBy === 'file') {
                key = svc.file_path || 'unknown';
            }
            if (!grouped[key]) grouped[key] = [];
            grouped[key].push(svc);
        });
        
        html = Object.entries(grouped).map(([key, svcList]) => `
            <div class="detail-section">
                <h4>${escapeHtml(key)}</h4>
                <div class="detail-items">
                    ${svcList.map(svc => `
                        <div class="detail-item clickable" onclick="showEntityDetail('${currentRepoId}', 'service', '${svc.id}')">
                            <div class="detail-item-header">
                                <strong>${escapeHtml(svc.name)}</strong>
                                <div style="display: flex; gap: 0.5rem; align-items: center;">
                                    <span class="entity-id-badge" title="Entity ID: ${svc.id}">ID: ${getShortId(svc.id)}</span>
                                    <span class="detail-badge">${escapeHtml(svc.service_type || 'service')}</span>
                                </div>
                            </div>
                            ${svc.provider && currentServicesGroupBy !== 'provider' ? `<p class="detail-meta">Provider: ${escapeHtml(svc.provider)}</p>` : ''}
                            ${svc.file_path && currentServicesGroupBy !== 'file' ? `<p class="detail-meta">Found in: <code>${escapeHtml(svc.file_path)}</code>${createFileLink(svc.file_path, svc.line_number, currentRepoData)}</p>` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        `).join('');
    }
    
    container.innerHTML = html;
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

// Helper function to detect if a file is a build artifact
function isBuildArtifact(filePath) {
    if (!filePath) return false;
    const pathLower = filePath.toLowerCase();
    return pathLower.includes('.next/') ||
           pathLower.includes('/dist/') ||
           pathLower.includes('/build/') ||
           pathLower.includes('/out/') ||
           pathLower.includes('/.nuxt/') ||
           pathLower.includes('node_modules/') ||
           pathLower.includes('/target/') ||
           pathLower.endsWith('.min.js') ||
           pathLower.endsWith('.bundle.js') ||
           pathLower.endsWith('.chunk.js');
}

// Helper function to get source mapping hint for build artifacts
function getSourceMappingHint(filePath) {
    if (!isBuildArtifact(filePath)) return '';
    
    const pathLower = filePath.toLowerCase();
    let hint = '';
    
    if (pathLower.includes('.next/')) {
        hint = '<p class="build-artifact-warning" style="color: var(--warning-color, #f59e0b); font-size: 0.875rem; margin-top: 0.5rem; padding: 0.5rem; background: rgba(245, 158, 11, 0.1); border-radius: 0.25rem; border-left: 3px solid var(--warning-color, #f59e0b);">⚠️ <strong>Build Artifact:</strong> This is a Next.js compiled file. To find the source code, check your <code>src/</code> or <code>pages/</code> directory. Use browser DevTools source maps to map back to original files.</p>';
    } else if (pathLower.includes('/dist/') || pathLower.includes('/build/')) {
        hint = '<p class="build-artifact-warning" style="color: var(--warning-color, #f59e0b); font-size: 0.875rem; margin-top: 0.5rem; padding: 0.5rem; background: rgba(245, 158, 11, 0.1); border-radius: 0.25rem; border-left: 3px solid var(--warning-color, #f59e0b);">⚠️ <strong>Build Artifact:</strong> This is a compiled file. Check your source files in <code>src/</code> directory.</p>';
    }
    
    return hint;
}

function renderCodeElementsList(elements, showAll = false) {
    // Sort elements by name
    const sorted = [...elements].sort((a, b) => a.name.localeCompare(b.name));
    
    // If showing all and there are many, limit initial display
    const maxInitial = 50;
    const shouldPaginate = !showAll && sorted.length > maxInitial;
    const displayElements = shouldPaginate ? sorted.slice(0, maxInitial) : sorted;
    
    let html = '<div class="detail-items">';
    html += displayElements.map(el => {
        const isArtifact = isBuildArtifact(el.file_path);
        const sourceHint = getSourceMappingHint(el.file_path);
        return `
        <div class="detail-item clickable" data-code-name="${escapeHtml(el.name.toLowerCase())}" onclick="showEntityDetail('${currentRepoId || ''}', 'code_element', '${el.id}')">
            <div class="detail-item-header">
                <strong>${escapeHtml(el.name)}</strong>
                <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
                    <span class="entity-id-badge" title="Entity ID: ${el.id}">ID: ${getShortId(el.id)}</span>
                    <span class="detail-badge">${escapeHtml(el.language || 'unknown')}</span>
                    ${isArtifact ? '<span class="detail-badge" style="background: var(--warning-color, #f59e0b); color: white;">Build Artifact</span>' : ''}
                </div>
            </div>
            ${el.file_path ? `<p class="detail-meta">File: <code>${escapeHtml(el.file_path)}</code>${el.line_number ? ` (line ${el.line_number})` : ''}${createFileLink(el.file_path, el.line_number, currentRepoData)}</p>` : ''}
            ${sourceHint}
            ${el.element_type ? `<p class="detail-meta" style="font-size: 0.75rem; color: var(--text-secondary);">Type: ${escapeHtml(el.element_type)}</p>` : ''}
        </div>
    `;
    }).join('');
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
                    <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
                        <span class="entity-id-badge" title="Entity ID: ${entity.id}">ID: ${getShortId(entity.id)}</span>
                        <span class="detail-badge ${keyType === 'hardcoded' ? 'badge-critical' : 'badge-info'}">${escapeHtml(keyType)}</span>
                        <span class="detail-badge">${escapeHtml(provider)}</span>
                        ${hasVulns ? '<span class="detail-badge badge-warning">⚠ Vulnerable</span>' : ''}
                    </div>
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
                <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
                    <span class="entity-id-badge" title="Entity ID: ${entity.id}">ID: ${getShortId(entity.id)}</span>
                    ${entity.provider ? `<span class="detail-badge">${escapeHtml(entity.provider)}</span>` : ''}
                    ${hasVulns ? '<span class="detail-badge badge-warning">⚠ Vulnerable</span>' : ''}
                </div>
            </div>
            ${entity.arn ? `<p class="detail-meta"><strong>ARN:</strong> <code>${escapeHtml(entity.arn)}</code></p>` : ''}
            ${entity.file_path ? `<p class="detail-meta"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${entity.line_number ? `:${entity.line_number}` : ''}${createFileLink(entity.file_path, entity.line_number, currentRepoData)}</p>` : ''}
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

// Store enabled node types for graph filtering
let enabledNodeTypes = {
    repository: true,
    dependency: true,
    service: true,
    code: false, // Code elements off by default
    security: true,
    test: false, // Tests off by default
    test_framework: false, // Test frameworks off by default
};

async function loadRepositoryGraph(repoId) {
    const container = document.getElementById('repo-graph-container');
    container.innerHTML = '<p class="loading-text">Loading graph...</p>';
    
    // Setup node type toggles
    setupGraphNodeTypeToggles(repoId);
    
    try {
        const graph = await api.getGraph(repoId);
        
        if (!graph.nodes || graph.nodes.length === 0) {
            container.innerHTML = '<p>No graph data available. Run analysis first.</p>';
            return;
        }
        
        // Store graph data globally for filtering
        window.currentGraphData = graph;
        window.currentGraphRepoId = repoId;
        
        // Render enhanced graph with repository ID
        renderEnhancedGraph(graph, container, repoId);
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load graph: ${escapeHtml(error.message)}</p>`;
    }
}

function setupGraphNodeTypeToggles(repoId) {
    const toggles = document.querySelectorAll('.node-type-toggle');
    
    // Set initial state
    toggles.forEach(toggle => {
        const nodeType = toggle.getAttribute('data-node-type');
        toggle.checked = enabledNodeTypes[nodeType] || false;
        
        // Add change listener
        toggle.addEventListener('change', function() {
            const nodeType = this.getAttribute('data-node-type');
            enabledNodeTypes[nodeType] = this.checked;
            
            // Re-render graph with filtered nodes
            if (window.currentGraphData && window.currentGraphRepoId) {
                const container = document.getElementById('repo-graph-container');
                renderEnhancedGraph(window.currentGraphData, container, window.currentGraphRepoId);
            }
        });
    });
}

// Entity Detail Modal Functions
// currentRepoId is declared earlier in the file

function showEntityDetail(repoId, entityType, entityId) {
    // Validate entity ID - ensure it's not a short ID (UUIDs are typically 36 chars with dashes, or 32 without)
    if (!entityId || (entityId.length < 32 && !entityId.includes('-'))) {
        console.error('[ENTITY] Invalid entity ID (too short):', entityId, 'Length:', entityId.length);
        const modal = document.getElementById('entity-detail-modal');
        const title = document.getElementById('entity-detail-title');
        const body = document.getElementById('entity-detail-body');
        modal.style.display = 'flex';
        title.textContent = 'Error';
        body.innerHTML = `<p class="error-text">Invalid entity ID: "${escapeHtml(entityId)}" (length: ${entityId.length}).<br><br>This appears to be a short ID. Please click on the entity name or row, not just the ID badge.</p>`;
        return;
    }
    
    currentRepoId = repoId;
    const modal = document.getElementById('entity-detail-modal');
    const title = document.getElementById('entity-detail-title');
    const body = document.getElementById('entity-detail-body');
    
    modal.style.display = 'flex';
    title.textContent = 'Loading...';
    body.innerHTML = '<p class="loading-text">Loading entity details...</p>';
    
    console.log('[ENTITY] Loading entity details:', { repoId, entityType, entityId: entityId.substring(0, 8) + '...' });
    
    api.getEntityDetails(repoId, entityType, entityId)
        .then(details => {
            title.textContent = getEntityTitle(entityType, details.entity);
            body.innerHTML = renderEntityDetails(entityType, details);
        })
        .catch(error => {
            console.error('[ENTITY] Failed to load entity details:', error);
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
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${createFileLink(entity.file_path, null, currentRepoData)}</div>`;
    return html;
}

function renderServiceDetails(entity, details) {
    let html = '';
    html += `<div class="detail-item"><strong>Name:</strong> ${escapeHtml(entity.name)}</div>`;
    html += `<div class="detail-item"><strong>Provider:</strong> ${escapeHtml(entity.provider)}</div>`;
    html += `<div class="detail-item"><strong>Type:</strong> ${escapeHtml(entity.service_type)}</div>`;
    html += `<div class="detail-item"><strong>Confidence:</strong> ${(entity.confidence * 100).toFixed(1)}%</div>`;
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${entity.line_number ? `:${entity.line_number}` : ''}${createFileLink(entity.file_path, entity.line_number, currentRepoData)}</div>`;
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
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>:${entity.line_number}${createFileLink(entity.file_path, entity.line_number, currentRepoData)}</div>`;
    
    // Add build artifact warning if applicable
    const sourceHint = getSourceMappingHint(entity.file_path);
    if (sourceHint) {
        html += `<div class="detail-item">${sourceHint}</div>`;
    }
    
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
    html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(entity.file_path)}</code>${entity.line_number ? `:${entity.line_number}` : ''}${createFileLink(entity.file_path, entity.line_number, currentRepoData)}</div>`;
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
    
    // Dependencies (from code relationships)
    if (details.related_dependencies && details.related_dependencies.length > 0) {
        html += '<div class="detail-section"><h3>Related Dependencies</h3><div class="related-items">';
        details.related_dependencies.forEach(dep => {
            const confidence = dep.confidence ? ` (${(dep.confidence * 100).toFixed(0)}% confidence)` : '';
            const evidence = dep.evidence ? `<br><small class="text-muted">${escapeHtml(dep.evidence)}</small>` : '';
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'dependency', '${dep.id}')">
                <strong>${escapeHtml(dep.name)}</strong> <code>${escapeHtml(dep.version || 'unknown')}</code>${confidence}${evidence}
            </div>`;
        });
        html += '</div></div>';
    }
    
    // Services (from code relationships)
    if (details.related_services && details.related_services.length > 0) {
        html += '<div class="detail-section"><h3>Related Services</h3><div class="related-items">';
        details.related_services.forEach(svc => {
            const confidence = svc.confidence ? ` (${(svc.confidence * 100).toFixed(0)}% confidence)` : '';
            const evidence = svc.evidence ? `<br><small class="text-muted">${escapeHtml(svc.evidence)}</small>` : '';
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'service', '${svc.id}')">
                <div><strong>${escapeHtml(svc.name)}</strong></div>
                <div>${escapeHtml(svc.provider)}${confidence}</div>
                ${evidence}
            </div>`;
        });
        html += '</div></div>';
    }
    
    // API Keys (for services)
    if (details.api_keys && details.api_keys.length > 0) {
        html += '<div class="detail-section"><h3>API Keys</h3><div class="related-items">';
        details.api_keys.forEach(key => {
            const config = typeof key.configuration === 'string' ? JSON.parse(key.configuration) : key.configuration;
            const keyName = config.key_name || key.name;
            const keyType = config.key_type || 'unknown';
            const provider = config.provider || key.provider || 'generic';
            html += `<div class="related-item clickable" onclick="showEntityDetail('${currentRepoId}', 'security_entity', '${key.id}')">
                <strong>${escapeHtml(keyName)}</strong> 
                <span class="detail-badge ${keyType === 'hardcoded' ? 'badge-critical' : 'badge-info'}">${escapeHtml(keyType)}</span>
                <span class="detail-badge">${escapeHtml(provider)}</span>
                ${key.file_path ? `<br><small><code>${escapeHtml(key.file_path)}</code>${key.line_number ? `:${key.line_number}` : ''}</small>` : ''}
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
window.showToolDetail = showToolDetail;

// Store tools for filtering
let allTools = [];
let currentToolsGroupBy = 'category';

async function loadTools(repoId) {
    currentRepoId = repoId;
    const container = document.getElementById('tools-list');
    container.innerHTML = '<p class="loading-text">Loading tools...</p>';
    
    try {
        const tools = await api.getTools(repoId);
        allTools = tools;
        
        if (tools.length === 0) {
            container.innerHTML = '<p>No tools found. Run analysis first.</p>';
            return;
        }
        
        // Get repo data for file linking
        let repoData = null;
        try {
            repoData = await api.getRepository(repoId).catch(() => null);
        } catch (e) {
            // Ignore - will use null repoData
        }
        
        // Setup filters
        setupToolFilters(tools, repoData);
        
        // Initial render
        filterAndRenderTools(repoData);
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load tools: ${escapeHtml(error.message)}</p>`;
    }
}

function setupToolFilters(tools, repoData = null) {
    const searchInput = document.getElementById('tools-search');
    const categoryFilter = document.getElementById('tools-category-filter');
    const typeFilter = document.getElementById('tools-type-filter');
    
    // Store repoData for use in filterAndRenderTools
    window.currentToolsRepoData = repoData;
    
    // Setup event listeners
    searchInput.addEventListener('input', () => {
        api.getRepository(currentRepoId).then(rd => filterAndRenderTools(rd)).catch(() => filterAndRenderTools(repoData));
    });
    categoryFilter.addEventListener('change', () => {
        api.getRepository(currentRepoId).then(rd => filterAndRenderTools(rd)).catch(() => filterAndRenderTools(repoData));
    });
    typeFilter.addEventListener('change', () => {
        api.getRepository(currentRepoId).then(rd => filterAndRenderTools(rd)).catch(() => filterAndRenderTools(repoData));
    });
}

function filterAndRenderTools(repoData = null) {
    const searchTerm = document.getElementById('tools-search')?.value.toLowerCase() || '';
    const categoryFilter = document.getElementById('tools-category-filter')?.value || '';
    const typeFilter = document.getElementById('tools-type-filter')?.value || '';
    
    // Use provided repoData or try to get from window
    const repoDataToUse = repoData || window.currentToolsRepoData || null;
    
    let filtered = allTools.filter(tool => {
        const matchesSearch = !searchTerm || 
            tool.name.toLowerCase().includes(searchTerm) ||
            (tool.tool_type && tool.tool_type.toLowerCase().includes(searchTerm)) ||
            (tool.category && tool.category.toLowerCase().includes(searchTerm)) ||
            (tool.file_path && tool.file_path.toLowerCase().includes(searchTerm));
        const matchesCategory = !categoryFilter || tool.category === categoryFilter;
        const matchesType = !typeFilter || tool.tool_type === typeFilter;
        return matchesSearch && matchesCategory && matchesType;
    });
    
    // Update filter info
    const filterInfo = document.getElementById('tools-filter-info');
    if (filterInfo) {
        const activeFilters = [];
        if (searchTerm) activeFilters.push(`search: "${searchTerm}"`);
        if (categoryFilter) activeFilters.push(`category: ${categoryFilter}`);
        if (typeFilter) activeFilters.push(`type: ${typeFilter}`);
        filterInfo.textContent = activeFilters.length > 0 
            ? `Showing ${filtered.length} of ${allTools.length} tools (${activeFilters.join(', ')})`
            : `Showing ${filtered.length} tools`;
    }
    
    // Group tools
    const grouped = {};
    filtered.forEach(tool => {
        const groupKey = tool.category || 'other';
        if (!grouped[groupKey]) grouped[groupKey] = [];
        grouped[groupKey].push(tool);
    });
    
    // Render
    const container = document.getElementById('tools-list');
    if (Object.keys(grouped).length === 0) {
        container.innerHTML = '<p>No tools match the current filters.</p>';
        return;
    }
    
    const sortedGroups = Object.keys(grouped).sort();
    container.innerHTML = sortedGroups.map(category => {
        const categoryTools = grouped[category];
        const categoryLabel = category.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
        return `
            <div class="detail-group">
                <h4 class="detail-group-header">${escapeHtml(categoryLabel)} <span class="detail-group-count">${categoryTools.length}</span></h4>
                <div class="detail-group-content">
                    ${categoryTools.map(tool => renderToolItem(tool, repoDataToUse)).join('')}
                </div>
            </div>
        `;
    }).join('');
}

function renderToolItem(tool, repoData = null) {
    const filePath = tool.file_path ? tool.file_path.replace(/^\.\/cache\/repos\/[^\/]+\//, '') : '';
    const fileLink = filePath ? createFileLink(tool.file_path, tool.line_number, repoData) : '';
    return `
        <div class="detail-item clickable" onclick="showToolDetail('${currentRepoId || ''}', '${tool.id}')">
            <div class="detail-item-header">
                <strong>${escapeHtml(tool.name)}</strong>
                <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
                    <span class="entity-id-badge" title="Tool ID: ${tool.id}">ID: ${getShortId(tool.id)}</span>
                    <span class="detail-badge">${escapeHtml(tool.tool_type || 'unknown')}</span>
                    <span class="detail-badge">${escapeHtml(tool.category || 'other')}</span>
                    ${tool.version ? `<span class="detail-badge badge-info">v${escapeHtml(tool.version)}</span>` : ''}
                </div>
            </div>
            <div class="detail-meta">
                ${filePath ? `<p><strong>File:</strong> <code>${escapeHtml(filePath)}</code>${tool.line_number ? `:${tool.line_number}` : ''} ${fileLink}</p>` : ''}
                <p><strong>Detection Method:</strong> ${escapeHtml(tool.detection_method || 'unknown')}</p>
                ${tool.confidence ? `<p><strong>Confidence:</strong> ${(tool.confidence * 100).toFixed(0)}%</p>` : ''}
            </div>
        </div>
    `;
}

async function showToolDetail(repoId, toolId) {
    currentRepoId = repoId;
    const modal = document.getElementById('entity-detail-modal');
    const title = document.getElementById('entity-detail-title');
    const body = document.getElementById('entity-detail-body');
    
    modal.style.display = 'flex';
    title.textContent = 'Loading...';
    body.innerHTML = '<p class="loading-text">Loading tool details...</p>';
    
    try {
        const tool = allTools.find(t => t.id === toolId);
        if (!tool) {
            title.textContent = 'Error';
            body.innerHTML = '<p class="error-text">Tool not found</p>';
            return;
        }
        
        const scripts = await api.getToolScripts(repoId, toolId).catch(() => []);
        
        // Get repo data for file linking
        let repoData = null;
        try {
            repoData = await api.getRepository(repoId).catch(() => null);
        } catch (e) {
            // Ignore - will use null repoData
        }
        
        const filePath = tool.file_path ? tool.file_path.replace(/^\.\/cache\/repos\/[^\/]+\//, '') : '';
        let config = {};
        try {
            config = tool.configuration ? (typeof tool.configuration === 'string' ? JSON.parse(tool.configuration) : tool.configuration) : {};
        } catch (e) {
            config = {};
        }
        
        title.textContent = `${escapeHtml(tool.name)} (${escapeHtml(tool.category || 'tool')})`;
        
        let html = '<div class="entity-detail-content">';
        
        // Basic Information
        html += '<div class="detail-section"><h3>Details</h3><div class="detail-grid">';
        html += `<div class="detail-item"><strong>ID:</strong> <code>${escapeHtml(tool.id)}</code></div>`;
        html += `<div class="detail-item"><strong>Type:</strong> ${escapeHtml(tool.tool_type || 'unknown')}</div>`;
        html += `<div class="detail-item"><strong>Category:</strong> ${escapeHtml(tool.category || 'other')}</div>`;
        if (tool.version) {
            html += `<div class="detail-item"><strong>Version:</strong> ${escapeHtml(tool.version)}</div>`;
        }
        if (filePath) {
            const fileLink = createFileLink(tool.file_path, tool.line_number, repoData);
            html += `<div class="detail-item"><strong>File:</strong> <code>${escapeHtml(filePath)}</code>${tool.line_number ? `:${tool.line_number}` : ''} ${fileLink}</div>`;
        }
        html += `<div class="detail-item"><strong>Detection Method:</strong> ${escapeHtml(tool.detection_method || 'unknown')}</div>`;
        if (tool.confidence) {
            html += `<div class="detail-item"><strong>Confidence:</strong> ${(tool.confidence * 100).toFixed(0)}%</div>`;
        }
        html += '</div></div>';
        
        // Scripts
        if (scripts.length > 0) {
            html += '<div class="detail-section"><h3>Scripts (' + scripts.length + ')</h3><div class="related-items">';
            scripts.forEach(script => {
                html += `<div class="related-item">`;
                html += `<strong>${escapeHtml(script.name || 'unnamed')}</strong>`;
                if (script.description) {
                    html += `<p>${escapeHtml(script.description)}</p>`;
                }
                if (script.command) {
                    html += `<code>${escapeHtml(script.command)}</code>`;
                }
                html += `</div>`;
            });
            html += '</div></div>';
        }
        
        // Configuration
        if (Object.keys(config).length > 0) {
            html += '<div class="detail-section"><h3>Configuration</h3>';
            html += `<pre>${escapeHtml(JSON.stringify(config, null, 2))}</pre>`;
            html += '</div>';
        }
        
        html += '</div>';
        body.innerHTML = html;
    } catch (error) {
        console.error('Failed to load tool details:', error);
        title.textContent = 'Error';
        body.innerHTML = `<p class="error-text">Failed to load tool details: ${escapeHtml(error.message)}</p>`;
    }
}

// Global editor protocol (default: vscode)
let editorProtocol = 'vscode';

async function loadVersion() {
    try {
        const response = await api.getVersion();
        const versionElement = document.getElementById('app-version');
        if (versionElement && response.version) {
            versionElement.textContent = `v${response.version}`;
            console.log('Version loaded:', response.version);
        }
        // Store editor protocol from config
        if (response.editor_protocol) {
            editorProtocol = response.editor_protocol;
            console.log('Editor protocol:', editorProtocol);
        }
    } catch (error) {
        console.error('Failed to load version:', error);
        // Keep the fallback version from HTML if API fails
    }
}

// Store documentation for filtering
let allDocumentation = [];

// Store tests for filtering
let allTests = [];

async function loadDocumentation(repoId) {
    currentRepoId = repoId;
    const container = document.getElementById('documentation-list');
    container.innerHTML = '<p class="loading-text">Loading documentation...</p>';
    
    // Clear previous documentation data
    allDocumentation = [];
    
    try {
        const docs = await api.getDocumentation(repoId);
        // Double-check: filter by repository_id as a safety measure
        allDocumentation = docs.filter(doc => doc.repository_id === repoId);
        
        if (allDocumentation.length === 0) {
            container.innerHTML = '<p>No documentation files found. Documentation indexing is experimental and may not be available for all repositories.</p>';
            return;
        }
        
        // Get repo data for file linking
        let repoData = null;
        try {
            repoData = await api.getRepository(repoId).catch(() => null);
        } catch (e) {
            // Ignore - will use null repoData
        }
        
        // Setup filter event listeners
        const searchInput = document.getElementById('documentation-search');
        const typeFilter = document.getElementById('documentation-type-filter');
        
        if (searchInput) {
            searchInput.oninput = () => {
                api.getRepository(repoId).then(rd => filterAndRenderDocumentation(repoId, rd)).catch(() => filterAndRenderDocumentation(repoId, repoData));
            };
        }
        if (typeFilter) {
            typeFilter.onchange = () => {
                api.getRepository(repoId).then(rd => filterAndRenderDocumentation(repoId, rd)).catch(() => filterAndRenderDocumentation(repoId, repoData));
            };
        }
        
        // Initial render
        filterAndRenderDocumentation(repoId, repoData);
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load documentation: ${escapeHtml(error.message)}</p>`;
    }
}

function filterAndRenderDocumentation(repoId, repoData = null) {
    const searchTerm = document.getElementById('documentation-search')?.value.toLowerCase() || '';
    const typeFilter = document.getElementById('documentation-type-filter')?.value || '';
    
    // Ensure we only show documentation for the current repository
    const repoIdToUse = repoId || currentRepoId;
    
    let filtered = allDocumentation.filter(doc => {
        // Safety check: ensure documentation belongs to current repository
        if (repoIdToUse && doc.repository_id !== repoIdToUse) {
            return false;
        }
        const matchesSearch = !searchTerm || 
            doc.file_name.toLowerCase().includes(searchTerm) ||
            (doc.title && doc.title.toLowerCase().includes(searchTerm)) ||
            (doc.description && doc.description.toLowerCase().includes(searchTerm)) ||
            doc.content_preview.toLowerCase().includes(searchTerm);
        const matchesType = !typeFilter || doc.doc_type === typeFilter;
        return matchesSearch && matchesType;
    });
    
    // Update filter info
    const filterInfo = document.getElementById('documentation-filter-info');
    if (filterInfo) {
        const activeFilters = [];
        if (searchTerm) activeFilters.push(`search: "${searchTerm}"`);
        if (typeFilter) activeFilters.push(`type: ${typeFilter}`);
        filterInfo.textContent = activeFilters.length > 0 
            ? `Showing ${filtered.length} of ${allDocumentation.length} files (${activeFilters.join(', ')})`
            : `Showing ${filtered.length} documentation files`;
    }
    
    // Group by type
    const grouped = {};
    filtered.forEach(doc => {
        const groupKey = doc.doc_type || 'Other';
        if (!grouped[groupKey]) grouped[groupKey] = [];
        grouped[groupKey].push(doc);
    });
    
    // Render
    const container = document.getElementById('documentation-list');
    if (Object.keys(grouped).length === 0) {
        container.innerHTML = '<p>No documentation files match the current filters.</p>';
        return;
    }
    
    const sortedGroups = Object.keys(grouped).sort();
    container.innerHTML = sortedGroups.map(type => {
        const typeDocs = grouped[type];
        return `
            <div class="detail-group">
                <h4 class="detail-group-header">${escapeHtml(type)} <span class="detail-group-count">${typeDocs.length}</span></h4>
                <div class="detail-group-content">
                    ${typeDocs.map(doc => renderDocumentationItem(doc, repoData)).join('')}
                </div>
            </div>
        `;
    }).join('');
}

async function loadTests(repoId) {
    currentRepoId = repoId;
    const container = document.getElementById('tests-list');
    container.innerHTML = '<p class="loading-text">Loading tests...</p>';
    
    // Clear previous test data
    allTests = [];
    
    try {
        const tests = await api.getTests(repoId);
        // Filter by repository_id as a safety measure
        allTests = tests.filter(test => test.repository_id === repoId);
        
        if (allTests.length === 0) {
            container.innerHTML = '<p>No tests found. Run analysis first.</p>';
            return;
        }
        
        // Get repo data for file linking
        let repoData = null;
        try {
            repoData = await api.getRepository(repoId).catch(() => null);
        } catch (e) {
            // Ignore - will use null repoData
        }
        
        // Setup filters
        setupTestFilters();
        
        // Initial render
        filterAndRenderTests(repoData);
    } catch (error) {
        container.innerHTML = `<p class="error-text">Failed to load tests: ${escapeHtml(error.message)}</p>`;
    }
}

function setupTestFilters() {
    const searchInput = document.getElementById('tests-search');
    const frameworkFilter = document.getElementById('tests-framework-filter');
    const languageFilter = document.getElementById('tests-language-filter');
    
    // Setup event listeners
    if (searchInput) {
        searchInput.addEventListener('input', () => {
            api.getRepository(currentRepoId).then(repoData => filterAndRenderTests(repoData)).catch(() => filterAndRenderTests(null));
        });
    }
    if (frameworkFilter) {
        frameworkFilter.addEventListener('change', () => {
            api.getRepository(currentRepoId).then(repoData => filterAndRenderTests(repoData)).catch(() => filterAndRenderTests(null));
        });
    }
    if (languageFilter) {
        languageFilter.addEventListener('change', () => {
            api.getRepository(currentRepoId).then(repoData => filterAndRenderTests(repoData)).catch(() => filterAndRenderTests(null));
        });
    }
}

function filterAndRenderTests(repoData = null) {
    const searchTerm = document.getElementById('tests-search')?.value.toLowerCase() || '';
    const frameworkFilter = document.getElementById('tests-framework-filter')?.value || '';
    const languageFilter = document.getElementById('tests-language-filter')?.value || '';
    
    let filtered = allTests.filter(test => {
        const matchesSearch = !searchTerm || 
            test.name.toLowerCase().includes(searchTerm) ||
            (test.test_framework && test.test_framework.toLowerCase().includes(searchTerm)) ||
            (test.file_path && test.file_path.toLowerCase().includes(searchTerm)) ||
            (test.suite_name && test.suite_name.toLowerCase().includes(searchTerm));
        const matchesFramework = !frameworkFilter || test.test_framework === frameworkFilter;
        const matchesLanguage = !languageFilter || test.language === languageFilter;
        return matchesSearch && matchesFramework && matchesLanguage;
    });
    
    // Update filter info
    const filterInfo = document.getElementById('tests-filter-info');
    if (filterInfo) {
        const activeFilters = [];
        if (searchTerm) activeFilters.push(`search: "${searchTerm}"`);
        if (frameworkFilter) activeFilters.push(`framework: ${frameworkFilter}`);
        if (languageFilter) activeFilters.push(`language: ${languageFilter}`);
        filterInfo.textContent = activeFilters.length > 0 
            ? `Showing ${filtered.length} of ${allTests.length} tests (${activeFilters.join(', ')})`
            : `Showing ${filtered.length} test(s)`;
    }
    
    // Group by framework
    const grouped = {};
    filtered.forEach(test => {
        const groupKey = test.test_framework || 'unknown';
        if (!grouped[groupKey]) grouped[groupKey] = [];
        grouped[groupKey].push(test);
    });
    
    // Render
    const container = document.getElementById('tests-list');
    if (Object.keys(grouped).length === 0) {
        container.innerHTML = '<p>No tests match the current filters.</p>';
        return;
    }
    
    const sortedGroups = Object.keys(grouped).sort();
    container.innerHTML = sortedGroups.map(framework => {
        const frameworkTests = grouped[framework];
        return `
            <div class="detail-group">
                <h4 class="detail-group-header">${escapeHtml(framework)} <span class="detail-group-count">${frameworkTests.length}</span></h4>
                <div class="detail-group-content">
                    ${frameworkTests.map(test => renderTestItem(test, repoData)).join('')}
                </div>
            </div>
        `;
    }).join('');
}

function renderTestItem(test, repoData = null) {
    let assertions = [];
    let setupMethods = [];
    let teardownMethods = [];
    let parameters = [];
    
    try {
        assertions = JSON.parse(test.assertions || '[]');
        setupMethods = JSON.parse(test.setup_methods || '[]');
        teardownMethods = JSON.parse(test.teardown_methods || '[]');
        parameters = JSON.parse(test.parameters || '[]');
    } catch (e) {
        console.warn('Failed to parse test JSON fields:', e);
    }
    
    const fileLink = createFileLink(test.file_path, test.line_number, repoData);
    
    return `
        <div class="detail-item" data-test-id="${test.id}">
            <div class="detail-item-header">
                <span class="entity-id-badge">${getShortId(test.id)}</span>
                <h4 class="detail-item-title">${escapeHtml(test.name)}</h4>
                <span class="detail-item-badge">${escapeHtml(test.test_framework)}</span>
                ${test.test_type ? `<span class="detail-item-badge">${escapeHtml(test.test_type)}</span>` : ''}
                <span class="detail-item-badge">${escapeHtml(test.language)}</span>
            </div>
            <div class="detail-item-content">
                ${test.suite_name ? `<div class="detail-item-row"><strong>Suite:</strong> ${escapeHtml(test.suite_name)}</div>` : ''}
                <div class="detail-item-row">
                    <strong>File:</strong> ${fileLink}
                    ${test.line_number ? ` <span class="text-muted">(line ${test.line_number})</span>` : ''}
                </div>
                ${test.signature ? `<div class="detail-item-row"><strong>Signature:</strong> <code>${escapeHtml(test.signature)}</code></div>` : ''}
                ${parameters.length > 0 ? `<div class="detail-item-row"><strong>Parameters:</strong> ${parameters.map(p => `<code>${escapeHtml(p)}</code>`).join(', ')}</div>` : ''}
                ${test.return_type ? `<div class="detail-item-row"><strong>Return Type:</strong> <code>${escapeHtml(test.return_type)}</code></div>` : ''}
                ${assertions.length > 0 ? `<div class="detail-item-row"><strong>Assertions:</strong> ${assertions.length} assertion(s)</div>` : ''}
                ${setupMethods.length > 0 ? `<div class="detail-item-row"><strong>Setup:</strong> ${setupMethods.map(m => `<code>${escapeHtml(m)}</code>`).join(', ')}</div>` : ''}
                ${teardownMethods.length > 0 ? `<div class="detail-item-row"><strong>Teardown:</strong> ${teardownMethods.map(m => `<code>${escapeHtml(m)}</code>`).join(', ')}</div>` : ''}
                ${test.doc_comment ? `<div class="detail-item-row"><strong>Documentation:</strong> <pre>${escapeHtml(test.doc_comment)}</pre></div>` : ''}
            </div>
        </div>
    `;
}

function renderDocumentationItem(doc, repoData = null) {
    const badges = [];
    if (doc.has_code_examples) badges.push('<span class="detail-badge badge-info">Code Examples</span>');
    if (doc.has_api_references) badges.push('<span class="detail-badge badge-info">API References</span>');
    if (doc.has_diagrams) badges.push('<span class="detail-badge badge-info">Diagrams</span>');
    
    const fileLink = createFileLink(doc.file_path, null, repoData);
    
    return `
        <div class="detail-item">
            <div class="detail-item-header">
                <strong>${escapeHtml(doc.file_name)}</strong>
                <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
                    <span class="entity-id-badge" title="Documentation ID: ${doc.id}">ID: ${getShortId(doc.id)}</span>
                    <span class="detail-badge">${escapeHtml(doc.doc_type)}</span>
                    ${badges.join('')}
                </div>
            </div>
            <div class="detail-meta">
                <p><strong>Path:</strong> <code>${escapeHtml(doc.file_path)}</code> ${fileLink}</p>
                ${doc.title ? `<p><strong>Title:</strong> ${escapeHtml(doc.title)}</p>` : ''}
                ${doc.description ? `<p><strong>Description:</strong> ${escapeHtml(doc.description)}</p>` : ''}
                <p><strong>Stats:</strong> ${doc.word_count} words, ${doc.line_count} lines</p>
                <div style="margin-top: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border-radius: 4px; font-size: 0.85rem;">
                    <strong>Preview:</strong>
                    <pre style="margin-top: 0.25rem; white-space: pre-wrap; word-wrap: break-word; max-height: 150px; overflow-y: auto;">${escapeHtml(doc.content_preview)}</pre>
                </div>
            </div>
        </div>
    `;
}

