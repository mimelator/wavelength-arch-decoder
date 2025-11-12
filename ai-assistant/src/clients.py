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

    async def get_graph(self, repo_id: str) -> Dict[str, Any]:
        """Get knowledge graph for repository"""
        response = await self.client.get(
            f"{self.base_url}/api/v1/repositories/{repo_id}/graph"
        )
        response.raise_for_status()
        return response.json()

    async def health_check(self) -> Dict[str, Any]:
        """Check if Architecture Decoder service is available"""
        try:
            response = await self.client.get(
                f"{self.base_url}/health",
                timeout=5.0
            )
            response.raise_for_status()
            return {
                "available": True,
                "status": "ok",
                "response": response.json()
            }
        except httpx.TimeoutException:
            return {
                "available": False,
                "status": "timeout",
                "error": "Connection timeout"
            }
        except httpx.ConnectError:
            return {
                "available": False,
                "status": "connection_error",
                "error": "Cannot connect to Architecture Decoder service"
            }
        except Exception as e:
            return {
                "available": False,
                "status": "error",
                "error": str(e)
            }

    async def close(self):
        """Close HTTP client"""
        await self.client.aclose()

