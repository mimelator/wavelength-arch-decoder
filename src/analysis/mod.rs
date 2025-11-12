pub mod dependencies;
pub mod graph;
pub mod code_structure;
pub mod code_relationships;
pub mod tool_detector;

pub use dependencies::{DependencyExtractor, PackageDependency, PackageManager, DependencyManifest};
pub use graph::{DependencyGraph, DependencyNode, VersionConflict, DependencyStatistics};
pub use code_structure::{CodeAnalyzer, CodeStructure, CodeElement, CodeElementType, CodeCall};
pub use code_relationships::{CodeRelationshipDetector, CodeRelationship, RelationshipTargetType};
pub use tool_detector::{ToolDetector, DetectedTool, ToolType, ToolCategory, ToolScript};

