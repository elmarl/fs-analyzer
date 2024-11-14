use std::fs::{self, DirEntry};
use std::io::{self, Error};
use std::path::Path;

/// Graph module to manage the relationship between files and directories.
pub mod graph {
    use crate::file_analyzer::Node;
    use std::{cmp::Reverse, collections::BinaryHeap, path::PathBuf};

    /// Represents a relationship between parent and child nodes.
    struct Relation {
        parent: usize,
        children: Vec<usize>,
    }

    /// Data structure where elements are all in a single vector.
    /// Relations are defined in a separate vector, similar to a graph.
    pub struct Graph {
        elements: Vec<Node>,
        relations: Vec<Relation>,
    }

    impl Graph {
        /// Creates a new empty Graph.
        pub fn new() -> Graph {
            Graph {
                elements: Vec::new(),
                relations: Vec::new(),
            }
        }

        /// Returns the number of elements in the graph.
        pub fn elements_len(&self) -> usize {
            self.elements.len()
        }

        /// Returns the number of relations in the graph.
        pub fn relations_len(&self) -> usize {
            self.relations.len()
        }

        /// Retrieves the indices of all parent nodes for a given node index.
        pub fn get_parents(&self, mut index: usize) -> Vec<usize> {
            let mut parents = Vec::new();
            while let Some(relation) = self.relations.iter().find(|r| r.children.contains(&index)) {
                parents.push(relation.parent);
                index = relation.parent;
            }
            parents
        }

        /// Finds the top `num` largest files and returns their details.
        pub fn largest_file(&self, num: usize) -> Vec<(String, String, u64)> {
            let mut heap = BinaryHeap::with_capacity(num);
            for (idx, node) in self.elements.iter().enumerate() {
                if !node.is_dir {
                    if heap.len() < num {
                        heap.push(Reverse((node.size, idx)));
                    } else if node.size > heap.peek().unwrap().0 .0 {
                        heap.pop();
                        heap.push(Reverse((node.size, idx)));
                    }
                }
            }

            let mut results = Vec::with_capacity(heap.len());
            while let Some(Reverse((size, idx))) = heap.pop() {
                let node = &self.elements[idx];
                let parents = self.get_parents(idx);
                let dir_path = parents
                    .iter()
                    .rev()
                    .map(|&p| &self.elements[p].name)
                    .collect::<PathBuf>()
                    .display()
                    .to_string();

                results.push((dir_path, node.name.clone(), size));
            }

            // Sort results in descending order of size
            results.sort_by(|a, b| b.2.cmp(&a.2));
            results
        }

        /// Adds a node to the graph and establishes its parent-child relationship.
        pub fn push(&mut self, node: Node, parent_index: Option<usize>) -> usize {
            self.elements.push(node);
            let new_index = self.elements.len() - 1;

            if let Some(parent) = parent_index {
                if let Some(rel) = self.relations.iter_mut().find(|r| r.parent == parent) {
                    rel.children.push(new_index);
                } else {
                    self.relations.push(Relation {
                        parent,
                        children: vec![new_index],
                    });
                }
            }

            new_index
        }
    }
}

pub mod file_analyzer {
    use super::graph::Graph;
    use super::*;
    use std::path::PathBuf;

    /// Category of a file based on its type.
    pub enum Category {
        Media(String),
        Game,
        Text,
        None,
    }

    /// Represents a file or directory node.
    pub struct Node {
        pub name: String,  // Name of the file or directory
        pub path: PathBuf, // Path to the file or directory
        pub size: u64,     // Size in bytes
        pub is_dir: bool,  // Is this node a directory
    }

    impl Default for Node {
        fn default() -> Node {
            Node {
                name: String::new(),
                path: PathBuf::new(),
                size: 0,
                is_dir: false,
            }
        }
    }

    /// Begins the file analysis process.
    pub fn start(root: PathBuf, num: usize) -> Result<(), Error> {
        let mut graph = Graph::new();
        let root_node = Node {
            path: root.clone(),
            is_dir: true,
            name: root
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.to_string_lossy().into_owned()),
            ..Default::default()
        };
        let root_index = graph.push(root_node, None);

        get_nodes(&root, root_index, &mut graph)?;
        println!(
            "Total elements: {}, Total relations: {}",
            graph.elements_len(),
            graph.relations_len()
        );

        let largest_files = graph.largest_file(num);
        for (path, name, size) in largest_files {
            println!("Size: {} bytes | Path: {}/{}", size, path, name);
        }

        Ok(())
    }

    /// Creates and returns a `Node` from a directory entry.
    pub fn create_node(dir: &DirEntry) -> Result<Node, Error> {
        let path = dir.path();
        let metadata = match path.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                eprintln!("Failed to get metadata for {:?}: {}", path, e);
                return Err(e);
            }
        };
        let size = if metadata.is_file() {
            metadata.len()
        } else {
            0
        };
        let name = dir
            .file_name()
            .into_string()
            .unwrap_or_else(|_| "Invalid UTF-8".to_string());
        let is_dir = metadata.is_dir();

        Ok(Node {
            name,
            path: path.clone(),
            size,
            is_dir,
            ..Default::default()
        })
    }

    /// Recursively scans the directory and populates the graph with nodes.
    pub fn get_nodes(dir: &Path, parent_index: usize, graph: &mut Graph) -> io::Result<()> {
        if dir.is_dir() {
            for entry_result in fs::read_dir(dir)? {
                let entry = match entry_result {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Failed to read directory entry: {}", e);
                        continue;
                    }
                };

                let path = entry.path();
                let node = match create_node(&entry) {
                    Ok(n) => n,
                    Err(_) => continue, // Skip entries that failed to create a node
                };

                let current_index = graph.push(node, Some(parent_index));

                if path.is_dir() {
                    // Recursively scan subdirectories
                    get_nodes(&path, current_index, graph)?;
                }
            }
        }
        Ok(())
    }
}
