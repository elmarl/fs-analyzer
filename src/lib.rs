use std::ffi::OsString;
use std::fs::{self, DirEntry};
use std::io::{self, Error};
use std::path::Path;
use std::time::Instant;
// use dialoguer::{
//     Select,
//     theme::ColorfulTheme
// };
pub mod graph {
    use crate::file_analyzer::Node;
    use std::{fmt::Write};

    pub struct Graph {
        elements: Vec<Node>,
        relations: Vec<Relation>,
    }

    struct Relation {
        parent: usize,
        children: Vec<usize>,
    }
    impl Graph {
        pub fn new() -> Graph {
            Graph {
                elements: Vec::new(),
                relations: Vec::new(),
            }
        }
        pub fn elements_len(&self) -> usize {
            self.elements.len()
        }
        pub fn relations_len(&self) -> usize {
            self.relations.len()
        }
        pub fn print_rel(&self, mut len: usize) -> Result<(), std::fmt::Error> {
            let mut s = String::new();
            if self.relations.len() < len {
                len = self.relations.len();
            }
            for i in 0..len {
                write!(&mut s, "{}", i)?;
            }
            println!("{}", s);
            Ok(())
        }
        pub fn get_parents(&self, mut index: usize) -> Vec<usize> {
            let mut v: Vec<usize> = Vec::new();
            loop {
                let relation = self.relations.iter().find(|f| f.children.contains(&index));
                match relation {
                    Some(r) => {
                        v.push(r.parent);
                        index = r.parent;
                    }
                    None => break,
                }
            }
            v
        }

        #[inline(always)]
        pub fn get_smallest_index(index_value_map: &Vec<(usize, u64)>) -> usize {
            let mut min_i = 0;
            let mut min_value = index_value_map[min_i].1;
            for (i, &item) in index_value_map.iter().enumerate().skip(1) {
                if item.1 < min_value {
                    min_i = i;
                    min_value = item.1;
                }
            }
            min_i
        }
        ///
        /// Find the largest file and return its size, name and parent folder
        /// num - number of files to show, sorted by size descending
        pub fn largest_file(&self, mut num: u8) -> Vec<(String, String, u64)> {
            if num > self.elements.len() as u8 {
                num = self.elements.len() as u8;
            }
            let mut index_value_map: Vec<(usize, u64)> = Vec::new();
            for _ in 0..num {
                index_value_map.push((0,0));
            }
            for f in self.elements.iter().enumerate() {
                let min_index = Self::get_smallest_index(&index_value_map);
                if f.1.size > index_value_map[min_index].1 {
                    index_value_map[min_index].0 = f.0;
                    index_value_map[min_index].1 = f.1.size;
                }
            }
            if index_value_map.len() == 0 {
                panic!("Couldn't sort files.")
            }
            let mut results: Vec<(String, String, u64)> = Vec::new();
            for item in index_value_map {
                let n = self.elements.get(item.0).unwrap();
                let st = n.name.clone();
                let mut dir_path: Vec<String> = Vec::new();
                let parents = self.get_parents(item.0);
                for p in parents {
                    dir_path.push(self.elements.get(p).unwrap().name.clone());
                }
                let dir_path: String = dir_path
                .iter()
                .rev()
                .map(|s| {
                    let mut s = s.clone();
                    s.push('/');
                    s
                })
                .collect();
                results.push((dir_path, st, item.1));
            }
            results.sort_by_key(|f| f.2);
            results.reverse();
            results
        }

        pub fn push(&mut self, n: Node, i: Option<usize>) -> usize {
            self.elements.push(n);
            let new_ind: usize = self.elements.len() - 1;
            if let Some(i) = i {
                let a = self.relations.iter_mut().find(|v| v.parent == i);
                match a {
                    Some(a) => {
                        a.children.push(new_ind);
                    }
                    None => self.relations.push(Relation {
                        parent: i,
                        children: vec![new_ind],
                    }),
                };
            };
            new_ind
        }
    }
}

pub mod file_analyzer {

    use super::graph::Graph;
    use super::*;

    /// Category of file
    pub enum Category {
        Media(String),
        Game(),
        Text(),
        None(),
    }

    /// File descriptior
    pub struct Node {
        pub name: String,             // name of file
        pub path: std::ffi::OsString, // path to file (platform independent)
        pub size: u64,                // size in bytes of file
        system: bool,                 // system file or similar, not safe to delete
        category: Category,           // category of file
        is_dir: bool,                 // is current node a dir
    }

    impl Default for Node {
        fn default() -> Node {
            Node {
                name: String::from(""),
                path: OsString::from(""),
                size: 0,
                system: false,
                category: Category::None(),
                is_dir: false,
            }
        }
    }

    pub fn start(root: OsString, num: u8) -> Result<(), Error> {
        let mut db = Graph::new();
        let node = Node {
            path: OsString::from(&root),
            is_dir: true,
            ..Default::default()
        };
        db.push(node, None);
        let now = Instant::now();
        get_nodes(Path::new(&root), 0, &mut db)?;
        let elapsed = now.elapsed();
        println!("Gathered: {:.2?}", elapsed);
        println!(
            "Length of data and relations: {}, {}",
            db.elements_len(),
            db.relations_len()
        );
        let now = Instant::now();
        let largest_files = db.largest_file(num);
        for i in largest_files {
            println!("{:#?}", i);
        }
        let elapsed = now.elapsed();
        println!("Largest: {:.2?}", elapsed);

        // todo
        // let items = vec!["Item 1", "item 2"];
        // let selection = Select::with_theme(&ColorfulTheme::default())
        //     .items(&items)
        //     .default(0)
        //     .interact_opt()?;

        // match selection {
        //     Some(index) => println!("User selected item : {}", items[index]),
        //     None => println!("User did not select anything")
        // }
        Ok(())
    }

    pub fn create_node(dir: &DirEntry) -> Result<Node, Error> {
        let path = dir.path();
        let size = path.metadata().unwrap().len();
        let name = dir.file_name().into_string().unwrap();
        let is_dir = path.is_dir();
        Ok(Node {
            name,
            size: size,
            // path: OsString::from(&path),
            is_dir,
            ..Default::default()
        })
    }

    pub fn get_nodes(dir: &Path, parent_index: usize, db: &mut Graph) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;

                let path = entry.path();
                let n = create_node(&entry)?;
                let child_index = db.push(n, Some(parent_index));
                if path.is_dir() {
                    get_nodes(&path, child_index, db)?;
                } else {
                    get_nodes(&path, parent_index, db)?;
                }
            }
        }
        Ok(())
    }
}
