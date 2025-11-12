from typing import Dict, Any

class PromptTemplates:
    """Templates for AI prompts"""

    def build_query_prompt(
        self, query: str, context: Dict[str, Any], intent: Dict[str, Any]
    ) -> str:
        """Build prompt for general query"""
        repo = context.get("repository", {})
        sources = context.get("sources", [])

        prompt = f"""You are an AI assistant helping developers understand their codebase architecture.

Repository: {repo.get('name', 'Unknown')}
Language: {repo.get('language', 'Unknown')}

User Query: {query}

Based on the following codebase analysis, answer the user's question:

"""

        # Add code elements
        code_elements = [s for s in sources if s.get("type") == "code_element"]
        if code_elements:
            prompt += "\n## Available Functions/Code Elements:\n\n"
            for elem in code_elements:
                prompt += f"- **{elem.get('name')}**\n"
                if elem.get('signature'):
                    prompt += f"  Signature: `{elem['signature']}`\n"
                if elem.get('file_path'):
                    prompt += f"  Location: `{elem['file_path']}`\n"
                if elem.get('line'):
                    prompt += f"  Line: {elem['line']}\n"
                if elem.get('language'):
                    prompt += f"  Language: {elem['language']}\n"
                if elem.get('relationships'):
                    rel_count = len(elem['relationships'])
                    prompt += f"  Relationships: {rel_count} found\n"
                    # Show first few relationships
                    for rel in elem['relationships'][:3]:
                        prompt += f"    - {rel.get('relationship_type', 'unknown')}: {rel.get('target_name', 'unknown')}\n"
                prompt += "\n"

        # Add services
        services = [s for s in sources if s.get("type") == "service"]
        if services:
            prompt += "\n## Services Used:\n\n"
            for svc in services:
                prompt += f"- **{svc.get('name')}**"
                if svc.get('provider'):
                    prompt += f" ({svc.get('provider')})"
                if svc.get('service_type'):
                    prompt += f" - {svc.get('service_type')}"
                if svc.get('file_path'):
                    prompt += f"\n  Found in: `{svc['file_path']}`"
                prompt += "\n"
            prompt += "\n"

        # Add dependencies
        dependencies = [s for s in sources if s.get("type") == "dependency"]
        if dependencies:
            prompt += "\n## Dependencies:\n\n"
            for dep in dependencies:
                prompt += f"- **{dep.get('name')}**"
                if dep.get('version'):
                    prompt += f" (v{dep.get('version')})"
                if dep.get('package_manager'):
                    prompt += f" [{dep.get('package_manager')}]"
                prompt += "\n"
            prompt += "\n"

        # Add tools
        tools = [s for s in sources if s.get("type") == "tool"]
        if tools:
            prompt += "\n## Tools:\n\n"
            for tool in tools:
                prompt += f"- **{tool.get('name')}**"
                if tool.get('tool_type'):
                    prompt += f" ({tool.get('tool_type')})"
                if tool.get('config_file'):
                    prompt += f"\n  Config: `{tool['config_file']}`"
                prompt += "\n"
            prompt += "\n"

        # Add related entities
        related = context.get("related", {})
        if related.get("services") or related.get("dependencies"):
            prompt += "\n## Related Entities:\n\n"
            if related.get("services"):
                prompt += f"Related Services: {', '.join(related['services'][:5])}\n"
            if related.get("dependencies"):
                prompt += f"Related Dependencies: {', '.join(related['dependencies'][:5])}\n"
            prompt += "\n"

        prompt += """
Please provide a clear, concise answer to the user's question. Include:
- Specific function/entity names and locations
- How elements relate to each other
- Usage examples if available
- Any important warnings or considerations

Be thorough but concise."""

        return prompt

    def build_refactoring_prompt(
        self, impact: Dict[str, Any], proposed_changes: str
    ) -> str:
        """Build prompt for refactoring analysis"""
        prompt = f"""You are an AI assistant analyzing refactoring impact.

Proposed Changes: {proposed_changes}

## Impact Analysis:

**Affected Functions**: {len(impact.get('affected_functions', []))}
**Affected Services**: {len(impact.get('affected_services', []))}
**Affected Dependencies**: {len(impact.get('affected_dependencies', []))}

### Affected Functions:
"""

        for func in impact.get('affected_functions', [])[:10]:
            prompt += f"- **{func.get('name')}**\n"
            if func.get('file_path'):
                prompt += f"  Location: `{func['file_path']}`\n"
            prompt += f"  Relationship: {func.get('relationship', 'unknown')}\n"
            prompt += f"  Risk: {func.get('risk', 'unknown')}\n\n"

        if impact.get('call_chains'):
            prompt += f"\n### Call Chains:\n"
            for chain in impact.get('call_chains', [])[:5]:
                path = chain.get('path', [])
                if path:
                    prompt += f"- {' → '.join(path)}\n"

        prompt += """
Based on this impact analysis, provide:
1. Risk assessment (low/medium/high) with reasoning
2. Step-by-step refactoring recommendations
3. Testing considerations
4. Potential breaking changes
5. Migration strategy

Be specific about what could break and why."""

        return prompt

