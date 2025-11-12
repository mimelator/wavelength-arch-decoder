from typing import Dict, Any, List
from src.clients import ArchitectureDecoderClient
from src.query_parser import QueryIntent

class ContextBuilder:
    """Build context from Architecture Decoder API"""

    def __init__(self, client: ArchitectureDecoderClient):
        self.client = client

    async def build(
        self,
        repository_id: str,
        intent: Dict[str, Any],
        max_results: int = 10,
        include_graph: bool = False
    ) -> Dict[str, Any]:
        """Build context based on query intent"""
        context = {
            "repository_id": repository_id,
            "sources": [],
            "related": {}
        }

        intent_type = intent.get("intent")
        topics = intent.get("topics", [])
        entities = intent.get("entities", [])

        # Get repository info
        try:
            repo_info = await self.client.get_repository(repository_id)
            context["repository"] = repo_info
        except Exception as e:
            print(f"Warning: Could not get repository info: {e}")
            context["repository"] = {"id": repository_id, "name": "Unknown"}

        # Build context based on intent
        if intent_type == QueryIntent.FIND_FUNCTIONS:
            context.update(await self._build_function_context(
                repository_id, topics, entities, max_results
            ))
        elif intent_type == QueryIntent.LIST_SERVICES:
            context.update(await self._build_service_context(
                repository_id, topics, max_results
            ))
        elif intent_type == QueryIntent.FIND_DEPENDENCIES:
            context.update(await self._build_dependency_context(
                repository_id, topics, max_results
            ))
        elif intent_type == QueryIntent.TOOL_DISCOVERY:
            context.update(await self._build_tool_context(
                repository_id, max_results
            ))
        elif intent_type == QueryIntent.FIND_TESTS:
            context.update(await self._build_test_context(
                repository_id, topics, max_results
            ))
        elif intent_type == QueryIntent.FIND_DOCUMENTATION:
            context.update(await self._build_documentation_context(
                repository_id, topics, max_results
            ))
        else:
            # General query - get all relevant data
            context.update(await self._build_general_context(
                repository_id, topics, entities, max_results
            ))

        # Include graph if requested
        if include_graph:
            try:
                context["graph"] = await self.client.get_graph(repository_id)
            except Exception as e:
                print(f"Warning: Could not get graph: {e}")
                context["graph"] = {}

        return context

    async def _build_function_context(
        self, repo_id: str, topics: List[str], entities: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for function discovery"""
        try:
            # Get all code elements
            code_elements = await self.client.get_code_elements(repo_id)
        except Exception as e:
            print(f"Warning: Could not get code elements: {e}")
            return {"sources": [], "related": {}}

        # Filter out common query words that aren't actual function names
        common_words = {"functions", "function", "available", "list", "show", "what", "which", "are", "is"}
        filtered_entities = [e for e in entities if e.lower() not in common_words]

        # Filter by topics/entities
        filtered = []
        for element in code_elements:
            # Check for function (case-insensitive)
            element_type = element.get("element_type", "").lower()
            if element_type != "function":
                continue

            # Skip functions from dependency directories (venv, node_modules, etc.)
            file_path = element.get("file_path", "")
            if not file_path:
                continue
            file_path_lower = file_path.lower()
            if any(dep_dir in file_path_lower for dep_dir in ["venv/", "node_modules/", ".venv/", "__pycache__/", "site-packages/"]):
                continue

            name = element.get("name", "").lower()

            # Check if matches topics or entities
            matches = False
            if topics:
                # If we have topics (like "firebase"), match against those
                matches = any(topic.lower() in name or topic.lower() in file_path_lower
                             for topic in topics)
            elif filtered_entities:
                # If we have specific entity names (not common words), match those
                matches = any(entity.lower() in name for entity in filtered_entities)
            else:
                # No specific topics or entities - match all functions
                matches = True

            if matches:
                filtered.append(element)

        # Get relationships for filtered functions
        sources = []
        for element in filtered[:max_results]:
            try:
                relationships = await self.client.get_code_relationships(
                    repo_id, code_element_id=element.get("id")
                )
            except:
                relationships = []

            sources.append({
                "type": "code_element",
                "id": element.get("id"),
                "name": element.get("name"),
                "signature": element.get("signature"),
                "file_path": element.get("file_path"),
                "line": element.get("line"),
                "language": element.get("language"),
                "relationships": relationships
            })

        # Get related services and dependencies
        related = await self._get_related_entities(repo_id, sources)

        return {
            "sources": sources,
            "related": related
        }

    async def _build_service_context(
        self, repo_id: str, topics: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for service discovery"""
        try:
            services = await self.client.get_services(repo_id)
        except Exception as e:
            print(f"Warning: Could not get services: {e}")
            return {"sources": [], "related": {}}

        # Filter by topics
        if topics:
            filtered = [
                s for s in services
                if any(topic.lower() in s.get("name", "").lower()
                      or topic.lower() in s.get("provider", "").lower()
                      for topic in topics)
            ]
        else:
            filtered = services

        sources = [
            {
                "type": "service",
                "id": s.get("id"),
                "name": s.get("name"),
                "provider": s.get("provider"),
                "service_type": s.get("service_type"),
                "file_path": s.get("file_path")
            }
            for s in filtered[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_dependency_context(
        self, repo_id: str, topics: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for dependency discovery"""
        try:
            dependencies = await self.client.get_dependencies(repo_id)
        except Exception as e:
            print(f"Warning: Could not get dependencies: {e}")
            return {"sources": [], "related": {}}

        # Filter by topics
        if topics:
            filtered = [
                d for d in dependencies
                if any(topic.lower() in d.get("name", "").lower()
                      for topic in topics)
            ]
        else:
            filtered = dependencies

        sources = [
            {
                "type": "dependency",
                "id": d.get("id"),
                "name": d.get("name"),
                "version": d.get("version"),
                "package_manager": d.get("package_manager")
            }
            for d in filtered[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_tool_context(
        self, repo_id: str, max_results: int
    ) -> Dict[str, Any]:
        """Build context for tool discovery"""
        try:
            tools = await self.client.get_tools(repo_id)
        except Exception as e:
            print(f"Warning: Could not get tools: {e}")
            return {"sources": [], "related": {}}

        sources = [
            {
                "type": "tool",
                "id": t.get("id"),
                "name": t.get("name"),
                "tool_type": t.get("tool_type"),
                "config_file": t.get("config_file")
            }
            for t in tools[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_test_context(
        self, repo_id: str, topics: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for test discovery"""
        try:
            tests = await self.client.get_tests(repo_id)
        except Exception as e:
            print(f"Warning: Could not get tests: {e}")
            return {"sources": [], "related": {}}

        # Filter by topics
        if topics:
            filtered = [
                t for t in tests
                if any(topic.lower() in t.get("name", "").lower()
                      or topic.lower() in t.get("test_framework", "").lower()
                      or topic.lower() in t.get("file_path", "").lower()
                      for topic in topics)
            ]
        else:
            filtered = tests

        sources = [
            {
                "type": "test",
                "id": t.get("id"),
                "name": t.get("name"),
                "test_framework": t.get("test_framework"),
                "test_type": t.get("test_type"),
                "file_path": t.get("file_path"),
                "line_number": t.get("line_number"),
                "language": t.get("language"),
                "suite_name": t.get("suite_name"),
                "signature": t.get("signature"),
            }
            for t in filtered[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_documentation_context(
        self, repo_id: str, topics: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build context for documentation discovery"""
        try:
            docs = await self.client.get_documentation(repo_id)
        except Exception as e:
            print(f"Warning: Could not get documentation: {e}")
            return {"sources": [], "related": {}}

        # Filter by topics
        if topics:
            filtered = [
                d for d in docs
                if any(topic.lower() in d.get("file_name", "").lower()
                      or topic.lower() in d.get("title", "").lower()
                      or topic.lower() in d.get("description", "").lower()
                      or topic.lower() in d.get("file_path", "").lower()
                      for topic in topics)
            ]
        else:
            filtered = docs

        sources = [
            {
                "type": "documentation",
                "id": d.get("id"),
                "file_name": d.get("file_name"),
                "file_path": d.get("file_path"),
                "doc_type": d.get("doc_type"),
                "title": d.get("title"),
                "description": d.get("description"),
                "word_count": d.get("word_count"),
                "line_count": d.get("line_count"),
                "has_code_examples": d.get("has_code_examples"),
                "has_api_references": d.get("has_api_references"),
                "has_diagrams": d.get("has_diagrams"),
                "content_preview": d.get("content_preview", "")[:500],  # Limit preview length
            }
            for d in filtered[:max_results]
        ]

        return {
            "sources": sources,
            "related": {}
        }

    async def _build_general_context(
        self, repo_id: str, topics: List[str], entities: List[str], max_results: int
    ) -> Dict[str, Any]:
        """Build general context for any query"""
        sources = []

        # Get a mix of code elements, services, dependencies, tests, documentation
        try:
            code_elements = await self.client.get_code_elements(repo_id)
            services = await self.client.get_services(repo_id)
            dependencies = await self.client.get_dependencies(repo_id)
            # Try to get tests and docs, but don't fail if they're not available
            try:
                tests = await self.client.get_tests(repo_id)
            except:
                tests = []
            try:
                docs = await self.client.get_documentation(repo_id)
            except:
                docs = []
        except Exception as e:
            print(f"Warning: Could not get general context: {e}")
            return {"sources": [], "related": {}}

        # Add relevant code elements
        for element in code_elements[:max_results//3]:
            if topics or entities:
                name = element.get("name", "").lower()
                if any(topic.lower() in name for topic in topics) or \
                   any(entity.lower() in name for entity in entities):
                    sources.append({
                        "type": "code_element",
                        "id": element.get("id"),
                        "name": element.get("name"),
                        "file_path": element.get("file_path")
                    })
            else:
                sources.append({
                    "type": "code_element",
                    "id": element.get("id"),
                    "name": element.get("name"),
                    "file_path": element.get("file_path")
                })

        # Add relevant services
        for service in services[:max_results//3]:
            sources.append({
                "type": "service",
                "id": service.get("id"),
                "name": service.get("name"),
                "provider": service.get("provider")
            })

        # Add relevant dependencies
        for dep in dependencies[:max_results//3]:
            sources.append({
                "type": "dependency",
                "id": dep.get("id"),
                "name": dep.get("name"),
                "version": dep.get("version")
            })

        # Add relevant tests if available
        if tests:
            for test in tests[:max_results//5]:
                if topics or entities:
                    name = test.get("name", "").lower()
                    if any(topic.lower() in name for topic in topics) or \
                       any(entity.lower() in name for entity in entities):
                        sources.append({
                            "type": "test",
                            "id": test.get("id"),
                            "name": test.get("name"),
                            "test_framework": test.get("test_framework"),
                            "file_path": test.get("file_path")
                        })
                else:
                    sources.append({
                        "type": "test",
                        "id": test.get("id"),
                        "name": test.get("name"),
                        "test_framework": test.get("test_framework"),
                        "file_path": test.get("file_path")
                    })

        # Add relevant documentation if available
        if docs:
            for doc in docs[:max_results//5]:
                if topics or entities:
                    name = doc.get("file_name", "").lower()
                    title = doc.get("title", "").lower() if doc.get("title") else ""
                    if any(topic.lower() in name or topic.lower() in title for topic in topics) or \
                       any(entity.lower() in name or entity.lower() in title for entity in entities):
                        sources.append({
                            "type": "documentation",
                            "id": doc.get("id"),
                            "file_name": doc.get("file_name"),
                            "file_path": doc.get("file_path"),
                            "title": doc.get("title")
                        })
                else:
                    sources.append({
                        "type": "documentation",
                        "id": doc.get("id"),
                        "file_name": doc.get("file_name"),
                        "file_path": doc.get("file_path"),
                        "title": doc.get("title")
                    })

        return {
            "sources": sources,
            "related": {}
        }

    async def _get_related_entities(
        self, repo_id: str, sources: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Get related services and dependencies for code elements"""
        related_services = set()
        related_dependencies = set()

        for source in sources:
            if source.get("relationships"):
                for rel in source["relationships"]:
                    if rel.get("target_type") == "service":
                        related_services.add(rel.get("target_name"))
                    elif rel.get("target_type") == "dependency":
                        related_dependencies.add(rel.get("target_name"))

        return {
            "services": list(related_services),
            "dependencies": list(related_dependencies)
        }

