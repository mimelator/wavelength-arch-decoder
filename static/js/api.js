// API Client Library
class WavelengthAPI {
    constructor(baseURL = '/api/v1') {
        this.baseURL = baseURL;
        this.apiKey = localStorage.getItem('wavelength_api_key') || '';
    }

    setApiKey(key) {
        this.apiKey = key;
        localStorage.setItem('wavelength_api_key', key);
    }

    async request(endpoint, options = {}) {
        const url = `${this.baseURL}${endpoint}`;
        const headers = {
            'Content-Type': 'application/json',
            ...options.headers,
        };

        if (this.apiKey) {
            headers['Authorization'] = `Bearer ${this.apiKey}`;
        }

        try {
            const response = await fetch(url, {
                ...options,
                headers,
            });

            if (!response.ok) {
                const error = await response.json().catch(() => ({ error: response.statusText }));
                
                // If unauthorized (401), clear invalid API key
                if (response.status === 401) {
                    console.warn('API key invalid or expired, clearing from storage');
                    this.setApiKey('');
                    localStorage.removeItem('wavelength_api_key');
                    
                    // Show user-friendly error
                    throw new Error('Your API key is invalid or expired. Please login again.');
                }
                
                throw new Error(error.error || `HTTP ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            console.error('API request failed:', error);
            throw error;
        }
    }

    // Auth
    async register(email, password) {
        const result = await this.request('/auth/register', {
            method: 'POST',
            body: JSON.stringify({ email, password }),
        });
        if (result.api_key) {
            this.setApiKey(result.api_key);
        }
        return result;
    }

    async login(email, password) {
        const result = await this.request('/auth/login', {
            method: 'POST',
            body: JSON.stringify({ email, password }),
        });
        if (result.api_key) {
            this.setApiKey(result.api_key);
        }
        return result;
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
}

// Global API instance
const api = new WavelengthAPI();

