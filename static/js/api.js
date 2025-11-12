// API Client Library
class WavelengthAPI {
    constructor(baseURL = '/api/v1') {
        this.baseURL = baseURL;
    }

    async request(endpoint, options = {}) {
        const url = `${this.baseURL}${endpoint}`;
        const headers = {
            'Content-Type': 'application/json',
            ...options.headers,
        };

        try {
            const response = await fetch(url, {
                ...options,
                headers,
            });

            if (!response.ok) {
                const error = await response.json().catch(() => ({ error: response.statusText }));
                throw new Error(error.error || `HTTP ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            console.error('API request failed:', error);
            throw error;
        }
    }

    // Repositories
    async listRepositories() {
        return this.request('/repositories');
    }

    async getRepository(id) {
        return this.request(`/repositories/${id}`);
    }

    async createRepository(name, url, branch = 'main', authType = undefined, authValue = undefined) {
        const body = { name, url, branch };
        if (authType) {
            body.auth_type = authType;
        }
        if (authValue) {
            body.auth_value = authValue;
        }
        return this.request('/repositories', {
            method: 'POST',
            body: JSON.stringify(body),
        });
    }

    async analyzeRepository(id) {
        return this.request(`/repositories/${id}/analyze`, {
            method: 'POST',
            body: JSON.stringify({ repository_id: id }),
        });
    }

    async deleteRepository(id) {
        return this.request(`/repositories/${id}`, {
            method: 'DELETE',
        });
    }

    async getDependencies(repoId) {
        return this.request(`/repositories/${repoId}/dependencies`);
    }

    async getServices(repoId) {
        return this.request(`/repositories/${repoId}/services`);
    }

    async getGraph(repoId) {
        return this.request(`/repositories/${repoId}/graph`);
    }

    async getCodeElements(repoId) {
        return this.request(`/repositories/${repoId}/code/elements`);
    }

    async getSecurityEntities(repoId) {
        return this.request(`/repositories/${repoId}/security/entities`);
    }

    async getSecurityVulnerabilities(repoId) {
        return this.request(`/repositories/${repoId}/security/vulnerabilities`);
    }

    async getTools(repoId) {
        return this.request(`/repositories/${repoId}/tools`);
    }

    async getToolScripts(repoId, toolId) {
        return this.request(`/repositories/${repoId}/tools/${toolId}/scripts`);
    }

    // Jobs
    async createJob(repositoryId, jobType = 'analyze_repository') {
        return this.request('/jobs', {
            method: 'POST',
            body: JSON.stringify({
                repository_id: repositoryId,
                job_type: jobType,
            }),
        });
    }

    async getJobStatus(jobId) {
        return this.request(`/jobs/${jobId}`);
    }

    async listJobs(status = null) {
        const params = status ? `?status=${status}` : '';
        return this.request(`/jobs${params}`);
    }

    // Search
    async searchDependencies(query) {
        return this.request(`/dependencies/search?q=${encodeURIComponent(query)}`);
    }

    async searchServices(provider) {
        return this.request(`/services/search?provider=${encodeURIComponent(provider)}`);
    }

    async getEntityDetails(repoId, entityType, entityId) {
        return this.request(`/repositories/${repoId}/entities/${entityType}/${entityId}`);
    }

    async getVersion() {
        return this.request('/version');
    }

    async getAnalysisProgress(repoId) {
        return this.request(`/repositories/${repoId}/progress`);
    }

    async getReport(repoId) {
        // Report endpoint returns HTML, not JSON
        const url = `${this.baseURL}/repositories/${repoId}/report`;
        return fetch(url).then(response => {
            if (!response.ok) {
                return response.json().then(err => {
                    throw new Error(err.error || `HTTP ${response.status}`);
                });
            }
            return response.text(); // Return HTML as text
        });
    }

    async getDocumentation(repoId) {
        return this.request(`/repositories/${repoId}/documentation`);
    }
}

// Global API instance
const api = new WavelengthAPI();

