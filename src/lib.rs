use std::ffi::OsString;
use std::fs::{self, DirEntry};
use std::io::{self, Error};
use std::path::Path;

pub mod graph {
    use crate::file_analyzer::Node;
    use std::{fmt::Write, io::Error, ffi::OsString};

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
        ///
        /// Find the largest file and return its size, name and parent folder
        pub fn largest_file(&self) -> (String, String, u64) {
            let mut s = 0u64;
            let mut st = String::new();
            let mut index: Option<usize> = None;
            for f in self.elements.iter().enumerate() {
                if f.1.size > s {
                    s = f.1.size;
                    st = f.1.name.clone();
                    index = Some(f.0);
                }
            }
            // let it = self.relations.iter().cycle();
            // let node_path = Vec::new();
            let mut dir_path: Vec<OsString> = Vec::new();
            loop {
                let relation = self.relations.iter().find(|f| f.children.contains(&index.unwrap()));
                match relation {
                    Some(r) => {
                        let parent_folder = self.elements.get(r.parent).unwrap().path.clone();
                        dir_path.push(parent_folder.clone());
                        index = Some(r.parent);
                    },
                    None => break
                }    
            };
            // let res = self.relations.iter().find(|f| f.children.contains(&index.unwrap())).unwrap();
            // let parent_index = res.parent;
            // let parent_folder = self.elements.get(parent_index).unwrap().name.clone();

            // with OsString
            let mut fstr: Vec<String> = Vec::new();
            for f in dir_path {
                let h = f.to_str().map(|s| s.to_string()).unwrap();
                fstr.push(h);
            }
            
            (fstr.join("/"), st, s)
        }

        pub fn push(&mut self, n: Node, i: Option<usize>) -> usize {
            self.elements.push(n);
            let new_ind: usize = self.elements.len() - 1;
            match i {
                Some(i) => {
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
                }
                None => (),
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
        pub name: String,         // name of file
        pub path: std::ffi::OsString, // path to file (platform independent)
        pub size: u64,            // size in bytes of file
        system: bool,             // system file or similar, not safe to delete
        category: Category,       // category of file
        is_dir: bool,             // is current node a dir
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

    pub struct NodeList {
        root: Node,
        root_path: OsString,
    }

    trait Crud {
        fn add_child(&mut self, node: Node);
    }

    // impl Crud for Node {
    //     fn add_child(&mut self, node: Node) {
    //         self.children.push(node);
    //     }
    // }

    pub fn start(root: OsString) -> Result<(), Error> {
        let mut db = Graph::new();
        let s = root.to_os_string();
        let node = Node {
            path: OsString::from(&root),
            is_dir: true,
            ..Default::default()
        };
        // let mut nlist = NodeList {
        //     root_path: s,
        //     root: node,
        // };
        db.push(node, None);
        // let node = &mut nlist.root;
        get_nodes(Path::new(&root), 0, &mut db)?;
        println!(
            "Length of data and relations: {}, {}",
            db.elements_len(),
            db.relations_len()
        );
        // db.print_rel(5).unwrap();
        println!("{:#?}", db.largest_file());
        Ok(())
    }

    // pub fn print_list(nl: &NodeList) {
    //     let r = &nl.root;
    //     for c in &r.children {
    //         match c.is_dir {
    //             true => {
    //                 println!("dir: {}", c.name);
    //                 println!("-----");
    //             }
    //             false => println!("{}", c.name),
    //         }
    //         println!("{}", c.size);
    //     }
    //     for c in &r.children {
    //         print_nodes(c);
    //     }
    // }

    // pub fn print_nodes(n: &Node) {
    //     match n.is_dir {
    //         true => {
    //             println!("dir: {}", n.name);
    //             println!("-----");
    //         }
    //         false => println!("{}", n.name),
    //     }
    //     for c in &n.children {
    //         print_nodes(c);
    //     }
    // }

    pub fn read_dir(path: &Path) -> Result<(), Error> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            println!("{}", path.to_str().unwrap());
        }
        Ok(())
    }

    pub fn create_node(dir: &DirEntry) -> Result<Node, Error> {
        let path = dir.path();
        let size = path.metadata().unwrap().len();
        let name = dir.file_name().into_string().unwrap();
        let is_dir = path.is_dir();
        return Ok(Node {
            name,
            size: size,
            path: OsString::from(&path),
            is_dir,
            ..Default::default()
        });
    }

    pub fn get_nodes(dir: &Path, parent_index: usize, db: &mut Graph) -> io::Result<()> {
        // std::mem::size_of::Node;
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

    pub fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
        // std::mem::size_of::Node;
        let s = std::mem::size_of::<Node>();
        println!("Size is: {s}");
        if dir.is_dir() {
            let mut x = 0;
            for entry in fs::read_dir(dir)? {
                let entry = entry?;

                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    let size = path.metadata().unwrap().len();
                    println!("{size}");
                    x += 1;
                    cb(&entry);
                }
            }
            println!("{x}");
        }
        Ok(())
    }
}
