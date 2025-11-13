const API_BASE_URL = window.location.origin;

let currentRepositoryId = null;
let chatHistory = [];

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    loadRepositories();
    setupEventListeners();
    checkDecoderHealth();
    // Check health every 30 seconds
    setInterval(checkDecoderHealth, 30000);
});

async function loadRepositories() {
    try {
        // Use proxy endpoint on same origin to avoid CORS issues
        console.log('Loading repositories from:', API_BASE_URL);
        const response = await fetch(`${API_BASE_URL}/api/v1/repositories`);
        
        if (!response.ok) {
            // Try to get error details from response
            let errorData;
            try {
                errorData = await response.json();
            } catch (e) {
                errorData = { detail: `HTTP error! status: ${response.status}` };
            }
            const error = new Error(errorData.detail?.error || errorData.detail || `HTTP error! status: ${response.status}`);
            error.response = response;
            error.errorData = errorData;
            throw error;
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

async function checkDecoderHealth() {
    try {
        const response = await fetch(`${API_BASE_URL}/health`);
        const health = await response.json();
        
        const statusIndicator = document.getElementById('decoder-status');
        const statusDot = statusIndicator.querySelector('.status-dot');
        const statusText = statusIndicator.querySelector('.status-text');
        const chatMessages = document.getElementById('chat-messages');
        const existingWarning = chatMessages.querySelector('.decoder-warning');
        
        if (health.decoder_available) {
            statusIndicator.className = 'status-indicator status-available';
            statusDot.textContent = '🟢';
            statusText.textContent = 'Architecture Decoder Online';
            statusIndicator.title = 'Architecture Decoder service is available';
            
            // Remove warning if service is now available
            if (existingWarning) {
                existingWarning.remove();
            }
        } else {
            statusIndicator.className = 'status-indicator status-unavailable';
            statusDot.textContent = '🔴';
            statusText.textContent = 'Architecture Decoder Offline';
            
            const diagnostics = health.decoder_diagnostics || {};
            const decoderUrl = health.decoder_url || 'http://localhost:8080';
            const port = diagnostics.port || '8080';
            const errorMsg = health.decoder_error || health.decoder_status;
            
            statusIndicator.title = `Architecture Decoder unavailable: ${errorMsg}\nTrying to connect to: ${decoderUrl}`;
            
            // Show detailed warning message in chat if service is unavailable
            if (!existingWarning) {
                const warning = document.createElement('div');
                warning.className = 'decoder-warning system-message';
                
                let diagnosticHTML = `
                    <strong>⚠️ Architecture Decoder Connection Error</strong>
                    <div style="margin-top: 0.5rem;">
                        <strong>Error:</strong> ${errorMsg || 'Unknown error'}<br>
                        <strong>Connection URL:</strong> <code>${decoderUrl}</code><br>
                        <strong>Port:</strong> ${port}<br>
                `;
                
                if (diagnostics.suggestion) {
                    diagnosticHTML += `<div style="margin-top: 0.5rem; padding: 0.5rem; background: rgba(255, 193, 7, 0.1); border-left: 3px solid #ffc107; border-radius: 4px;">
                        <strong>💡 Suggestion:</strong> ${diagnostics.suggestion}
                    </div>`;
                }
                
                if (diagnostics.troubleshooting && diagnostics.troubleshooting.length > 0) {
                    diagnosticHTML += `<div style="margin-top: 0.5rem;">
                        <strong>🔧 Troubleshooting Steps:</strong>
                        <ul style="margin: 0.25rem 0; padding-left: 1.5rem;">
                            ${diagnostics.troubleshooting.map(step => `<li style="margin: 0.25rem 0;">${step}</li>`).join('')}
                        </ul>
                    </div>`;
                }
                
                if (diagnostics.config_help) {
                    diagnosticHTML += `<div style="margin-top: 0.5rem; padding: 0.5rem; background: rgba(33, 150, 243, 0.1); border-left: 3px solid #2196f3; border-radius: 4px;">
                        <strong>⚙️ Configuration:</strong><br>
                        <div style="margin-top: 0.25rem;">
                            To change the connection URL, edit the <code>${diagnostics.config_file || '.env'}</code> file in the <code>ai-assistant/</code> directory.
                        </div>
                        <div style="margin-top: 0.5rem;">
                            <strong>Current setting:</strong><br>
                            <code style="display: block; margin-top: 0.25rem; padding: 0.25rem; background: rgba(0,0,0,0.05); border-radius: 3px;">${diagnostics.config_example || `ARCHITECTURE_DECODER_URL=${decoderUrl}`}</code>
                        </div>
                        <small style="display: block; margin-top: 0.5rem; color: #666;">
                            After updating the file, restart the AI Assistant for changes to take effect.
                        </small>
                    </div>`;
                }
                
                diagnosticHTML += `</div>`;
                
                warning.innerHTML = diagnosticHTML;
                chatMessages.insertBefore(warning, chatMessages.firstChild);
            } else {
                // Update existing warning with latest error and diagnostics
                const errorText = existingWarning.querySelector('small') || existingWarning.querySelector('.error-text');
                if (errorText) {
                    errorText.textContent = `Error: ${errorMsg}`;
                }
            }
        }
    } catch (error) {
        console.error('Failed to check decoder health:', error);
        const statusIndicator = document.getElementById('decoder-status');
        const statusDot = statusIndicator.querySelector('.status-dot');
        const statusText = statusIndicator.querySelector('.status-text');
        
        statusIndicator.className = 'status-indicator status-error';
        statusDot.textContent = '🟡';
        statusText.textContent = 'Status Unknown';
        statusIndicator.title = `Failed to check status: ${error.message}`;
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
            // Try to get error details from response
            let errorData;
            try {
                errorData = await response.json();
            } catch (e) {
                errorData = { detail: `HTTP error! status: ${response.status}` };
            }
            const error = new Error(errorData.detail?.error || errorData.detail || `HTTP error! status: ${response.status}`);
            error.response = response;
            error.errorData = errorData;
            throw error;
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
        
        // Check if it's a connection error with diagnostic info
        let errorMessage = `Error: ${error.message}`;
        let diagnosticInfo = null;
        
        // Check if we have error data with diagnostics
        if (error.errorData && error.errorData.detail) {
            if (typeof error.errorData.detail === 'object' && error.errorData.detail.diagnostics) {
                diagnosticInfo = error.errorData.detail.diagnostics;
                errorMessage = error.errorData.detail.error || error.message;
            } else if (typeof error.errorData.detail === 'object' && error.errorData.detail.error) {
                errorMessage = error.errorData.detail.error;
                diagnosticInfo = error.errorData.detail.diagnostics;
            } else if (typeof error.errorData.detail === 'string') {
                errorMessage = error.errorData.detail;
            }
        }
        
        // Show error message with diagnostics if available
        if (diagnosticInfo) {
            let diagnosticHTML = `<strong>❌ Connection Error</strong><br>${errorMessage}`;
            
            if (diagnosticInfo.suggestion) {
                diagnosticHTML += `<div style="margin-top: 0.5rem; padding: 0.5rem; background: rgba(255, 193, 7, 0.1); border-left: 3px solid #ffc107; border-radius: 4px;">
                    <strong>💡 Suggestion:</strong> ${diagnosticInfo.suggestion}
                </div>`;
            }
            
            if (diagnosticInfo.troubleshooting && diagnosticInfo.troubleshooting.length > 0) {
                diagnosticHTML += `<div style="margin-top: 0.5rem;">
                    <strong>🔧 Troubleshooting:</strong>
                    <ul style="margin: 0.25rem 0; padding-left: 1.5rem;">
                        ${diagnosticInfo.troubleshooting.map(step => `<li style="margin: 0.25rem 0;">${step}</li>`).join('')}
                    </ul>
                </div>`;
            }
            
            if (diagnosticInfo.config_example) {
                diagnosticHTML += `<div style="margin-top: 0.5rem; padding: 0.5rem; background: rgba(33, 150, 243, 0.1); border-left: 3px solid #2196f3; border-radius: 4px;">
                    <strong>⚙️ Configuration:</strong><br>
                    <div style="margin-top: 0.25rem;">
                        To change the connection URL, edit the <code>${diagnosticInfo.config_file || '.env'}</code> file in the <code>ai-assistant/</code> directory.
                    </div>
                    <div style="margin-top: 0.5rem;">
                        <strong>Example setting:</strong><br>
                        <code style="display: block; margin-top: 0.25rem; padding: 0.25rem; background: rgba(0,0,0,0.05); border-radius: 3px;">${diagnosticInfo.config_example}</code>
                    </div>
                    <small style="display: block; margin-top: 0.5rem; color: #666;">
                        After updating the file, restart the AI Assistant for changes to take effect.
                    </small>
                </div>`;
            }
            
            addMessage('assistant', diagnosticHTML, false, true);
        } else {
            addMessage('assistant', errorMessage, false, true);
        }
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

