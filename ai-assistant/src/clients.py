import httpx
from typing import Dict, Any, List, Optional

class ArchitectureDecoderClient:
    """Client for Architecture Decoder API"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.client = httpx.AsyncClient(timeout=30.0)

    async def get_repository(self, repo_id: str) -> Dict[str, Any]:
        """Get repository details"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}"
        )
        response.raise_for_status()
        return response.json()

    async def get_code_elements(
        self, repo_id: str, element_type: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Get code elements for repository"""
        url = f"{self.base_url}/api/v1/repositories/{repo_id}/code/elements"
        if element_type:
            url += f"?type={element_type}"
        response = await self.client.get(url)
        response.raise_for_status()
        return response.json()

    async def get_code_relationships(
        self, repo_id: str, code_element_id: Optional[str] = None,
        target_type: Optional[str] = None, target_id: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Get code relationships"""
        url = f"{self.base_url}/api/v1/repositories/{repo_id}/code/relationships"
        params = {}
        if code_element_id:
            params["code_element_id"] = code_element_id
        if target_type:
            params["target_type"] = target_type
        if target_id:
            params["target_id"] = target_id

        response = await self.client.get(url, params=params)
        response.raise_for_status()
        return response.json()

    async def get_code_calls(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get code calls for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/code/calls"
        )
        response.raise_for_status()
        return response.json()

    async def get_services(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get services for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/services"
        )
        response.raise_for_status()
        return response.json()

    async def get_dependencies(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get dependencies for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/dependencies"
        )
        response.raise_for_status()
        return response.json()

    async def get_tools(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get tools for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/tools"
        )
        response.raise_for_status()
        return response.json()

    async def get_tests(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get tests for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/tests"
        )
        response.raise_for_status()
        return response.json()

    async def get_documentation(self, repo_id: str) -> List[Dict[str, Any]]:
        """Get documentation files for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/documentation"
        )
        response.raise_for_status()
        return response.json()

    async def get_graph(self, repo_id: str) -> Dict[str, Any]:
        """Get knowledge graph for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/graph"
        )
        response.raise_for_status()
        return response.json()

    async def health_check(self) -> Dict[str, Any]:
        """Check if Architecture Decoder service is available"""
        from urllib.parse import urlparse
        
        # Parse the base URL to extract connection details
        parsed_url = urlparse(self.base_url)
        host = parsed_url.hostname or "localhost"
        port = parsed_url.port or (443 if parsed_url.scheme == "https" else 80)
        scheme = parsed_url.scheme or "http"
        full_url = f"{scheme}://{host}:{port}"
        
        diagnostic_info = {
            "url": self.base_url,
            "host": host,
            "port": port,
            "scheme": scheme,
            "full_url": full_url,
            "config_env_var": "ARCHITECTURE_DECODER_URL",
            "config_file": ".env",
            "default_url": "http://localhost:8080"
        }
        
        try:
            response = await self.client.get(
                f"{self.base_url}/health",
                timeout=5.0
            )
            response.raise_for_status()
            return {
                "available": True,
                "status": "ok",
                "response": response.json(),
                "diagnostics": diagnostic_info
            }
        except httpx.TimeoutException as e:
            return {
                "available": False,
                "status": "timeout",
                "error": f"Connection timeout after 5 seconds",
                "diagnostics": {
                    **diagnostic_info,
                    "suggestion": f"The Architecture Decoder server at {full_url} did not respond in time. Check if the server is running and accessible.",
                    "troubleshooting": [
                        f"Verify the server is running: curl {full_url}/health",
                        f"Check if the port {port} is correct (default is 8080)",
                        f"Ensure firewall/network allows connections to {host}:{port}",
                        f"Check server logs for startup errors"
                    ]
                }
            }
        except httpx.ConnectError as e:
            error_msg = str(e)
            # Extract more details from the error
            is_refused = "refused" in error_msg.lower() or "connection refused" in error_msg.lower()
            is_unreachable = "unreachable" in error_msg.lower() or "name resolution" in error_msg.lower()
            
            suggestion = ""
            troubleshooting = []
            
            if is_refused:
                suggestion = f"Connection refused - the server at {host}:{port} is not accepting connections. The server may not be running or the port is incorrect."
                troubleshooting = [
                    f"Start the Architecture Decoder server: cargo run (or ./target/release/wavelength-arch-decoder)",
                    f"Verify the server is listening on port {port}: lsof -i :{port} (macOS/Linux) or netstat -an | findstr :{port} (Windows)",
                    f"Check if port {port} is correct. Default is 8080. Change with PORT environment variable.",
                    f"If your server uses a different port, update ARCHITECTURE_DECODER_URL in the .env file"
                ]
            elif is_unreachable:
                suggestion = f"Cannot reach {host}. Check network connectivity and hostname resolution."
                troubleshooting = [
                    f"Verify hostname '{host}' resolves correctly: ping {host}",
                    f"If using localhost, ensure you're connecting to the correct machine",
                    f"Check network/firewall settings",
                    f"For remote servers, verify VPN/network access"
                ]
            else:
                suggestion = f"Cannot connect to Architecture Decoder at {full_url}"
                troubleshooting = [
                    f"Verify the server is running: curl {full_url}/health",
                    f"Check your .env file - the ARCHITECTURE_DECODER_URL is currently set to: {self.base_url}",
                    f"If your server uses a different port, update ARCHITECTURE_DECODER_URL in the .env file",
                    f"Default URL is http://localhost:8080",
                    f"Check server logs for connection errors"
                ]
            
            return {
                "available": False,
                "status": "connection_error",
                "error": f"Cannot connect to Architecture Decoder service: {error_msg}",
                "diagnostics": {
                    **diagnostic_info,
                    "suggestion": suggestion,
                    "troubleshooting": troubleshooting,
                    "config_help": f"To change the connection URL, edit the ARCHITECTURE_DECODER_URL variable in {diagnostic_info['config_file']} file:",
                    "config_example": f"ARCHITECTURE_DECODER_URL={full_url}"
                }
            }
        except httpx.HTTPStatusError as e:
            return {
                "available": False,
                "status": "http_error",
                "error": f"HTTP {e.response.status_code}: {e.response.text[:200]}",
                "diagnostics": {
                    **diagnostic_info,
                    "suggestion": f"Server responded with error {e.response.status_code}. The server may be running but encountering issues.",
                    "troubleshooting": [
                        f"Check server logs for errors",
                        f"Verify the server is fully started (not still initializing)",
                        f"Test the health endpoint directly: curl {full_url}/health"
                    ]
                }
            }
        except Exception as e:
            return {
                "available": False,
                "status": "error",
                "error": str(e),
                "diagnostics": {
                    **diagnostic_info,
                    "suggestion": f"Unexpected error connecting to Architecture Decoder",
                    "troubleshooting": [
                        f"Check server logs",
                        f"Verify network connectivity",
                        f"Ensure ARCHITECTURE_DECODER_URL is correctly set in .env"
                    ]
                }
            }

    async def close(self):
        """Close HTTP client"""
        await self.client.aclose()

