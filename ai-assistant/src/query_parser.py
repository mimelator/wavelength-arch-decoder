import re
from typing import Dict, Any, List
from enum import Enum

class QueryIntent(Enum):
    FIND_FUNCTIONS = "find_functions"
    LIST_SERVICES = "list_services"
    FIND_DEPENDENCIES = "find_dependencies"
    REFACTORING_IMPACT = "refactoring_impact"
    TOOL_DISCOVERY = "tool_discovery"
    GENERAL = "general"

class QueryParser:
    """Parse natural language queries to identify intent"""

    INTENT_PATTERNS = {
        QueryIntent.FIND_FUNCTIONS: [
            r"what functions",
            r"show.*functions",
            r"list.*functions",
            r"functions.*available",
            r"how.*function",
            r"which functions",
        ],
        QueryIntent.LIST_SERVICES: [
            r"what services",
            r"which services",
            r"services.*used",
            r"show.*services",
            r"list.*services",
        ],
        QueryIntent.FIND_DEPENDENCIES: [
            r"what dependencies",
            r"which dependencies",
            r"dependencies.*used",
            r"list.*dependencies",
        ],
        QueryIntent.REFACTORING_IMPACT: [
            r"what would break",
            r"what.*impact",
            r"refactor",
            r"rename",
            r"remove",
            r"change",
            r"can i.*remove",
            r"can i.*rename",
            r"safe.*remove",
        ],
        QueryIntent.TOOL_DISCOVERY: [
            r"what tools",
            r"build tools",
            r"test.*tools",
            r"linter",
            r"which tools",
        ],
    }

    def parse(self, query: str) -> Dict[str, Any]:
        """Parse query and return intent and extracted information"""
        query_lower = query.lower()

        # Identify intent
        intent = QueryIntent.GENERAL
        for intent_type, patterns in self.INTENT_PATTERNS.items():
            for pattern in patterns:
                if re.search(pattern, query_lower):
                    intent = intent_type
                    break
            if intent != QueryIntent.GENERAL:
                break

        # Extract entities (function names, service names, etc.)
        entities = self._extract_entities(query)

        # Extract topics
        topics = self._extract_topics(query)

        return {
            "intent": intent,
            "entities": entities,
            "topics": topics,
            "original_query": query
        }

    def _extract_entities(self, query: str) -> List[str]:
        """Extract potential entity names from query"""
        entities = []

        # Look for quoted strings
        entities.extend(re.findall(r'"([^"]+)"', query))

        # Look for function-like names (camelCase, snake_case)
        entities.extend(re.findall(r'\b([a-z][a-zA-Z0-9_]*)\b', query))

        # Remove common words
        common_words = {"what", "which", "show", "list", "the", "are", "is", "a", "an", "to", "for", "with", "from"}
        entities = [e for e in entities if e.lower() not in common_words]

        return list(set(entities))

    def _extract_topics(self, query: str) -> List[str]:
        """Extract topics/keywords from query"""
        topics = []
        topic_keywords = [
            "authentication", "auth", "storage", "database", "api",
            "firebase", "aws", "stripe", "payment", "email", "notification",
            "user", "admin", "file", "upload", "download", "session",
            "token", "security", "encryption", "validation"
        ]

        query_lower = query.lower()
        for keyword in topic_keywords:
            if keyword in query_lower:
                topics.append(keyword)

        return topics

