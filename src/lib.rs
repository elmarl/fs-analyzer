// lib.rs
use std::fs::{self, DirEntry};
use std::io::{self, Error};
use std::path::Path;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

/// Graph module to manage the relationship between files and directories.
pub mod graph {
    use crate::file_analyzer::Node;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    use std::path::PathBuf;

    /// Represents a data structure where each node knows its parent.
    pub struct Graph {
        nodes: Vec<Node>,
    }

    impl Graph {
        /// Creates a new empty Graph.
        pub fn new() -> Graph {
            Graph { nodes: Vec::new() }
        }

        /// Returns the number of elements in the graph.
        pub fn elements_len(&self) -> usize {
            self.nodes.len()
        }

        /// Retrieves the indices of all parent nodes for a given node index.
        pub fn get_parents(&self, mut index: usize) -> Vec<usize> {
            let mut parents = Vec::new();
            while let Some(parent) = self.nodes[index].parent {
                parents.push(parent);
                index = parent;
            }
            parents
        }

        /// Finds the top `num` largest files and returns their details.
        pub fn largest_file(&self, num: usize) -> Vec<(PathBuf, String, u64)> {
            let mut heap = BinaryHeap::with_capacity(num);
            for (idx, node) in self.nodes.iter().enumerate() {
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
                let node = &self.nodes[idx];
                let parents = self.get_parents(idx);
                let dir_path = parents
                    .iter()
                    .rev()
                    .fold(PathBuf::new(), |acc, &p| acc.join(&self.nodes[p].name));

                results.push((dir_path, node.name.clone(), size));
            }

            // Sort results in descending order of size
            results.sort_by(|a, b| b.2.cmp(&a.2));
            results
        }

        /// Adds a node to the graph and returns its index.
        pub fn push(&mut self, node: Node) -> usize {
            self.nodes.push(node);
            self.nodes.len() - 1
        }
    }
}

pub mod file_analyzer {
    use super::graph::Graph;
    use super::*;
    use std::path::PathBuf;

    /// Represents a file or directory node.
    #[derive(Debug)]
    pub struct Node {
        pub name: String,          // Name of the file or directory
        pub path: PathBuf,         // Path to the file or directory
        pub size: u64,             // Size in bytes
        pub is_dir: bool,          // Is this node a directory
        pub parent: Option<usize>, // Index of the parent node
    }

    impl Default for Node {
        fn default() -> Node {
            Node {
                name: String::new(),
                path: PathBuf::new(),
                size: 0,
                is_dir: false,
                parent: None,
            }
        }
    }

    /// Begins the file analysis process.
    pub fn start(root: PathBuf, num: usize) -> Result<(), Error> {
        // Initialize a thread-safe Graph using Arc and Mutex
        let graph = Arc::new(Mutex::new(Graph::new()));
        let root_node = Node {
            path: root.clone(),
            is_dir: true,
            name: root
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.to_string_lossy().into_owned()),
            parent: None,
            ..Default::default()
        };
        let root_index = {
            let mut graph_lock = graph.lock().unwrap();
            graph_lock.push(root_node)
        };

        get_nodes(&root, root_index, Arc::clone(&graph))?;

        // Acquire a lock to access the graph for listing largest files
        let graph_lock = graph.lock().unwrap();
        println!("Total elements: {}", graph_lock.elements_len(),);

        let largest_files = graph_lock.largest_file(num);
        drop(graph_lock);

        for (path, name, size) in largest_files {
            println!("Size: {} bytes | Path: {}/{}", size, path.display(), name);
        }

        Ok(())
    }

    /// Creates and returns a `Node` from a directory entry.
    fn create_node(dir: &DirEntry, parent: Option<usize>) -> Result<Node, Error> {
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
            parent,
            ..Default::default()
        })
    }

    /// Recursively scans the directory and populates the graph with nodes.
    fn get_nodes(dir: &Path, parent_index: usize, graph: Arc<Mutex<Graph>>) -> io::Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries: Vec<DirEntry> = match fs::read_dir(dir) {
            Ok(read_dir) => read_dir.filter_map(|e| e.ok()).collect(),
            Err(e) => {
                eprintln!("Failed to read directory {:?}: {}", dir, e);
                return Err(e);
            }
        };

        // Process entries in parallel
        entries.par_iter().try_for_each(|entry| -> io::Result<()> {
            let path = entry.path();
            let node = match create_node(entry, Some(parent_index)) {
                Ok(n) => n,
                Err(_) => return Ok(()), // Skip entries that failed to create a node
            };

            let current_index = {
                let mut graph_lock = graph.lock().unwrap();
                graph_lock.push(node)
            };

            if path.is_dir() {
                // Recursively scan subdirectories
                get_nodes(&path, current_index, Arc::clone(&graph))?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
