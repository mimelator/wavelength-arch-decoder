const API_BASE_URL = window.location.origin;

let currentRepositoryId = null;
let chatHistory = [];

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    loadRepositories();
    setupEventListeners();
});

async function loadRepositories() {
    try {
        // Use proxy endpoint on same origin to avoid CORS issues
        console.log('Loading repositories from:', API_BASE_URL);
        const response = await fetch(`${API_BASE_URL}/api/v1/repositories`);
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const repos = await response.json();
        console.log('Loaded repositories:', repos);
        
        if (!Array.isArray(repos)) {
            throw new Error('Invalid response format - expected array');
        }
        
        const select = document.getElementById('repository-select');
        select.innerHTML = '<option value="">Select a repository...</option>';
        
        if (repos.length === 0) {
            const option = document.createElement('option');
            option.value = '';
            option.textContent = 'No repositories found';
            option.disabled = true;
            select.appendChild(option);
            return;
        }
        
        repos.forEach(repo => {
            const option = document.createElement('option');
            option.value = repo.id;
            // Handle file:// URLs better
            let displayName = repo.name;
            if (repo.url) {
                const urlParts = repo.url.split('/');
                const lastPart = urlParts[urlParts.length - 1];
                if (lastPart && lastPart !== repo.name) {
                    displayName = `${repo.name} (${lastPart})`;
                }
            }
            option.textContent = displayName;
            select.appendChild(option);
        });
        
        console.log(`Successfully loaded ${repos.length} repositories`);
    } catch (error) {
        console.error('Failed to load repositories:', error);
        const select = document.getElementById('repository-select');
        select.innerHTML = '<option value="">Error loading repositories</option>';
        const errorOption = document.createElement('option');
        errorOption.value = '';
        errorOption.textContent = `Error: ${error.message}`;
        errorOption.disabled = true;
        select.appendChild(errorOption);
    }
}

function setupEventListeners() {
    const repoSelect = document.getElementById('repository-select');
    const queryInput = document.getElementById('query-input');
    const sendButton = document.getElementById('send-button');
    const clearButton = document.getElementById('clear-chat');

    repoSelect.addEventListener('change', (e) => {
        currentRepositoryId = e.target.value;
        sendButton.disabled = !currentRepositoryId || !queryInput.value.trim();
        if (currentRepositoryId) {
            addSystemMessage(`Selected repository: ${e.target.options[e.target.selectedIndex].textContent}`);
        }
    });

    queryInput.addEventListener('input', () => {
        sendButton.disabled = !currentRepositoryId || !queryInput.value.trim();
    });

    queryInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            if (!sendButton.disabled) {
                sendQuery();
            }
        }
    });

    sendButton.addEventListener('click', sendQuery);
    clearButton.addEventListener('click', clearChat);
}

async function sendQuery() {
    const queryInput = document.getElementById('query-input');
    const query = queryInput.value.trim();
    
    if (!query || !currentRepositoryId) return;

    // Add user message
    addMessage('user', query);
    queryInput.value = '';
    document.getElementById('send-button').disabled = true;

    // Show loading
    const loadingId = addMessage('assistant', '', true);

    try {
        const response = await fetch(`${API_BASE_URL}/api/v1/ai/query`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                repository_id: currentRepositoryId,
                query: query,
                max_results: 10,
                include_graph: true
            })
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        
        // Remove loading message
        removeMessage(loadingId);
        
        // Add assistant response
        addAssistantMessage(data);
        
        // Save to history
        chatHistory.push({
            query,
            response: data,
            timestamp: new Date()
        });

    } catch (error) {
        console.error('Query failed:', error);
        removeMessage(loadingId);
        addMessage('assistant', `Error: ${error.message}`, false, true);
    } finally {
        document.getElementById('send-button').disabled = !currentRepositoryId;
    }
}

function addMessage(role, text, isLoading = false, isError = false) {
    const messagesContainer = document.getElementById('chat-messages');
    const messageId = `msg-${Date.now()}-${Math.random()}`;
    
    const messageDiv = document.createElement('div');
    messageDiv.id = messageId;
    messageDiv.className = `message ${role}`;
    
    const avatar = document.createElement('div');
    avatar.className = 'message-avatar';
    avatar.textContent = role === 'user' ? '👤' : '🤖';
    
    const content = document.createElement('div');
    content.className = 'message-content';
    
    if (isLoading) {
        const loadingDiv = document.createElement('div');
        loadingDiv.className = 'loading';
        loadingDiv.innerHTML = '<div class="loading-spinner"></div><span>Thinking...</span>';
        content.appendChild(loadingDiv);
    } else if (isError) {
        const errorDiv = document.createElement('div');
        errorDiv.className = 'error-message';
        errorDiv.textContent = text;
        content.appendChild(errorDiv);
    } else {
        const textDiv = document.createElement('div');
        textDiv.className = 'message-text';
        textDiv.textContent = text;
        content.appendChild(textDiv);
    }
    
    messageDiv.appendChild(avatar);
    messageDiv.appendChild(content);
    
    messagesContainer.appendChild(messageDiv);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
    
    return messageId;
}

function addSystemMessage(text) {
    const messagesContainer = document.getElementById('chat-messages');
    const messageDiv = document.createElement('div');
    messageDiv.className = 'message assistant';
    messageDiv.style.opacity = '0.7';
    
    const avatar = document.createElement('div');
    avatar.className = 'message-avatar';
    avatar.textContent = 'ℹ️';
    
    const content = document.createElement('div');
    content.className = 'message-content';
    content.style.fontSize = '0.9rem';
    content.style.color = 'var(--text-secondary)';
    content.textContent = text;
    
    messageDiv.appendChild(avatar);
    messageDiv.appendChild(content);
    messagesContainer.appendChild(messageDiv);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
}

function addAssistantMessage(data) {
    const messagesContainer = document.getElementById('chat-messages');
    const messageId = `msg-${Date.now()}-${Math.random()}`;
    
    const messageDiv = document.createElement('div');
    messageDiv.id = messageId;
    messageDiv.className = 'message assistant';
    
    const avatar = document.createElement('div');
    avatar.className = 'message-avatar';
    avatar.textContent = '🤖';
    
    const content = document.createElement('div');
    content.className = 'message-content';
    
    // Main answer
    const answerDiv = document.createElement('div');
    answerDiv.className = 'message-text';
    answerDiv.textContent = data.answer || 'No answer provided.';
    content.appendChild(answerDiv);
    
    // Sources
    if (data.sources && data.sources.length > 0) {
        const sourcesSection = document.createElement('div');
        sourcesSection.className = 'sources-section';
        
        const sourcesTitle = document.createElement('div');
        sourcesTitle.className = 'sources-title';
        sourcesTitle.innerHTML = `<span>📚</span><span>Sources (${data.sources.length})</span>`;
        sourcesSection.appendChild(sourcesTitle);
        
        data.sources.forEach(source => {
            const sourceItem = document.createElement('div');
            sourceItem.className = 'source-item';
            
            const nameRow = document.createElement('div');
            nameRow.style.display = 'flex';
            nameRow.style.alignItems = 'center';
            nameRow.style.gap = '0.5rem';
            nameRow.style.marginBottom = '0.25rem';
            
            const name = document.createElement('strong');
            if (source.deep_link) {
                // Make name a clickable link
                const link = document.createElement('a');
                link.href = source.deep_link;
                link.textContent = source.name || source.id || 'Unknown';
                link.style.color = 'var(--primary-color)';
                link.style.textDecoration = 'none';
                link.style.cursor = 'pointer';
                link.title = 'View in Architecture Decoder';
                link.addEventListener('click', (e) => {
                    e.preventDefault();
                    window.open(source.deep_link, '_blank');
                });
                name.appendChild(link);
            } else {
                name.textContent = source.name || source.id || 'Unknown';
            }
            nameRow.appendChild(name);
            
            if (source.deep_link) {
                const linkIcon = document.createElement('span');
                linkIcon.textContent = '🔗';
                linkIcon.style.fontSize = '0.8em';
                linkIcon.style.cursor = 'pointer';
                linkIcon.title = 'Open in Architecture Decoder';
                linkIcon.addEventListener('click', () => {
                    window.open(source.deep_link, '_blank');
                });
                nameRow.appendChild(linkIcon);
            }
            
            sourceItem.appendChild(nameRow);
            
            if (source.type) {
                const type = document.createElement('span');
                type.textContent = `Type: ${source.type}`;
                type.style.color = 'var(--text-secondary)';
                type.style.fontSize = '0.8em';
                type.style.marginLeft = '0';
                type.style.marginTop = '0.25rem';
                sourceItem.appendChild(type);
            }
            
            if (source.file_path) {
                const path = document.createElement('div');
                path.className = 'source-path';
                path.textContent = source.file_path;
                sourceItem.appendChild(path);
            }
            
            if (source.version) {
                const version = document.createElement('div');
                version.textContent = `Version: ${source.version}`;
                version.style.color = 'var(--text-secondary)';
                version.style.fontSize = '0.8em';
                version.style.marginTop = '0.25rem';
                sourceItem.appendChild(version);
            }
            
            sourcesSection.appendChild(sourceItem);
        });
        
        content.appendChild(sourcesSection);
    }
    
    // Graph statistics
    if (data.graph_context && data.graph_context.statistics) {
        const graphSection = document.createElement('div');
        graphSection.className = 'graph-section';
        
        const graphTitle = document.createElement('div');
        graphTitle.className = 'sources-title';
        graphTitle.innerHTML = `<span>🕸️</span><span>Knowledge Graph</span>`;
        graphSection.appendChild(graphTitle);
        
        const stats = document.createElement('div');
        stats.className = 'graph-stats';
        
        const statsData = data.graph_context.statistics;
        if (statsData.total_nodes) {
            stats.appendChild(createStatItem(statsData.total_nodes, 'Nodes'));
        }
        if (statsData.total_edges) {
            stats.appendChild(createStatItem(statsData.total_edges, 'Edges'));
        }
        if (statsData.node_types) {
            Object.entries(statsData.node_types).forEach(([type, count]) => {
                stats.appendChild(createStatItem(count, type));
            });
        }
        
        graphSection.appendChild(stats);
        content.appendChild(graphSection);
    }
    
    // Related entities
    if (data.related_entities && Object.keys(data.related_entities).length > 0) {
        const relatedSection = document.createElement('div');
        relatedSection.className = 'sources-section';
        
        const relatedTitle = document.createElement('div');
        relatedTitle.className = 'sources-title';
        relatedTitle.innerHTML = `<span>🔗</span><span>Related Entities</span>`;
        relatedSection.appendChild(relatedTitle);
        
        Object.entries(data.related_entities).forEach(([type, items]) => {
            if (items && items.length > 0) {
                const typeDiv = document.createElement('div');
                typeDiv.style.marginTop = '0.5rem';
                typeDiv.innerHTML = `<strong>${type}:</strong> ${items.join(', ')}`;
                relatedSection.appendChild(typeDiv);
            }
        });
        
        content.appendChild(relatedSection);
    }
    
    // Meta information
    const meta = document.createElement('div');
    meta.className = 'message-meta';
    if (data.intent) {
        meta.textContent = `Intent: ${data.intent}`;
    }
    content.appendChild(meta);
    
    messageDiv.appendChild(avatar);
    messageDiv.appendChild(content);
    messagesContainer.appendChild(messageDiv);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
    
    return messageId;
}

function createStatItem(value, label) {
    const item = document.createElement('div');
    item.className = 'stat-item';
    
    const valueDiv = document.createElement('div');
    valueDiv.className = 'stat-value';
    valueDiv.textContent = value;
    
    const labelDiv = document.createElement('div');
    labelDiv.className = 'stat-label';
    labelDiv.textContent = label;
    
    item.appendChild(valueDiv);
    item.appendChild(labelDiv);
    return item;
}

function removeMessage(messageId) {
    const message = document.getElementById(messageId);
    if (message) {
        message.remove();
    }
}

function clearChat() {
    const messagesContainer = document.getElementById('chat-messages');
    messagesContainer.innerHTML = `
        <div class="welcome-message">
            <h2>Welcome! 👋</h2>
            <p>Ask questions about your codebase architecture:</p>
            <ul>
                <li>"What functions are available?"</li>
                <li>"What services are used?"</li>
                <li>"What dependencies does this use?"</li>
                <li>"What functions use Firebase?"</li>
                <li>"What would break if I rename getAdminStorage?"</li>
            </ul>
            <p><strong>Select a repository above to get started.</strong></p>
        </div>
    `;
    chatHistory = [];
}

