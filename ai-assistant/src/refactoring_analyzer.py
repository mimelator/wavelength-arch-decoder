from typing import Dict, Any, List
from src.clients import ArchitectureDecoderClient

class RefactoringAnalyzer:
    """Analyze refactoring impact"""

    def __init__(self, client: ArchitectureDecoderClient):
        self.client = client

    async def analyze(
        self,
        repository_id: str,
        target_elements: List[str],
        proposed_changes: str
    ) -> Dict[str, Any]:
        """Analyze refactoring impact"""
        impact = {
            "affected_functions": [],
            "affected_services": [],
            "affected_dependencies": [],
            "call_chains": [],
            "risk_level": "low",
            "recommendations": []
        }

        # For each target element, find what depends on it
        for element_id in target_elements:
            try:
                # Get element details
                relationships = await self.client.get_code_relationships(
                    repository_id, code_element_id=element_id
                )

                # Find callers (functions that call this)
                callers = await self._find_callers(repository_id, element_id)

                # Find callees (functions this calls)
                callees = await self._find_callees(repository_id, element_id, relationships)

                # Build call chains
                chains = await self._build_call_chains(
                    repository_id, element_id, callers
                )

                impact["affected_functions"].extend(callers)
                impact["call_chains"].extend(chains)
            except Exception as e:
                print(f"Warning: Error analyzing element {element_id}: {e}")

        # Assess risk
        impact["risk_level"] = self._assess_risk(impact)

        # Generate recommendations
        impact["recommendations"] = self._generate_recommendations(impact)

        return impact

    async def _find_callers(
        self, repo_id: str, element_id: str
    ) -> List[Dict[str, Any]]:
        """Find functions that call this element"""
        try:
            # Get code calls
            calls = await self.client.get_code_calls(repo_id)

            # Filter calls where target is our element
            callers = []
            for call in calls:
                if call.get("target_id") == element_id or call.get("target_element_id") == element_id:
                    callers.append({
                        "id": call.get("source_id") or call.get("source_element_id"),
                        "name": call.get("source_name") or call.get("source_function"),
                        "file_path": call.get("source_file_path") or call.get("source_file"),
                        "relationship": "calls",
                        "risk": "high"  # Direct call = high risk
                    })

            return callers
        except Exception as e:
            print(f"Warning: Could not find callers: {e}")
            return []

    async def _find_callees(
        self, repo_id: str, element_id: str, relationships: List[Dict[str, Any]]
    ) -> List[Dict[str, Any]]:
        """Find functions called by this element"""
        callees = []
        for rel in relationships:
            if rel.get("relationship_type") == "calls" or rel.get("target_type") == "code_element":
                callees.append({
                    "id": rel.get("target_id"),
                    "name": rel.get("target_name"),
                    "relationship": "called_by"
                })

        return callees

    async def _build_call_chains(
        self, repo_id: str, element_id: str, callers: List[Dict[str, Any]]
    ) -> List[Dict[str, Any]]:
        """Build call chains starting from callers"""
        chains = []

        for caller in callers[:5]:  # Limit to avoid explosion
            chain = await self._traverse_up(
                repo_id, caller.get("id"), [caller.get("name", "unknown")]
            )
            if chain:
                chains.append({
                    "path": chain,
                    "depth": len(chain)
                })

        return chains

    async def _traverse_up(
        self, repo_id: str, element_id: str, path: List[str]
    ) -> List[str]:
        """Traverse call graph upward"""
        if len(path) > 5:  # Limit depth
            return path

        # Get callers of this element
        callers = await self._find_callers(repo_id, element_id)

        if not callers:
            return path

        # Take first caller and continue
        next_caller = callers[0]
        return await self._traverse_up(
            repo_id, next_caller.get("id"), path + [next_caller.get("name", "unknown")]
        )

    def _assess_risk(self, impact: Dict[str, Any]) -> str:
        """Assess overall risk level"""
        affected_count = len(impact.get("affected_functions", []))
        chain_depth = max(
            [c.get("depth", 0) for c in impact.get("call_chains", [])],
            default=0
        )

        if affected_count > 10 or chain_depth > 4:
            return "high"
        elif affected_count > 5 or chain_depth > 2:
            return "medium"
        else:
            return "low"

    def _generate_recommendations(
        self, impact: Dict[str, Any]
    ) -> List[str]:
        """Generate refactoring recommendations"""
        recommendations = []

        affected_count = len(impact.get("affected_functions", []))

        if affected_count > 0:
            recommendations.append(
                f"Update {affected_count} affected function(s) before refactoring"
            )

        if impact.get("risk_level") == "high":
            recommendations.append(
                "Consider creating a wrapper function first"
            )
            recommendations.append(
                "Add deprecation warnings before removal"
            )

        recommendations.append("Run full test suite after changes")
        recommendations.append("Update documentation")

        return recommendations

