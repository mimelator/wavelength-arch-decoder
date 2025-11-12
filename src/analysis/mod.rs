pub mod dependencies;
pub mod graph;
pub mod code_structure;
pub mod code_relationships;
pub mod tool_detector;
pub mod documentation;
pub mod test_detector;
pub mod utils;

pub use dependencies::{DependencyExtractor, PackageDependency, PackageManager, DependencyManifest};
pub use graph::{DependencyGraph, DependencyNode, VersionConflict, DependencyStatistics};
pub use code_structure::{CodeAnalyzer, CodeStructure, CodeElement, CodeElementType, CodeCall};
pub use code_relationships::{CodeRelationshipDetector, CodeRelationship, RelationshipTargetType};
pub use tool_detector::{ToolDetector, DetectedTool, ToolType, ToolCategory, ToolScript};
pub use documentation::{DocumentationIndexer, DocumentationFile, DocumentationType};
pub use test_detector::{TestDetector, DetectedTest, TestFramework};

