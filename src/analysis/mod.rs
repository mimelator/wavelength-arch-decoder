pub mod dependencies;
pub mod graph;
pub mod code_structure;

pub use dependencies::{DependencyExtractor, PackageDependency, PackageManager, DependencyManifest};
pub use graph::{DependencyGraph, DependencyNode, VersionConflict, DependencyStatistics};
pub use code_structure::{CodeAnalyzer, CodeStructure, CodeElement, CodeElementType, CodeCall};

