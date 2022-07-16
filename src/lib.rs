use std::ffi::OsString;
use std::fs::{self, DirEntry};
use std::io::{self, Error};
use std::path::Path;

pub mod file_analyzer {

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
        name: String,             // name of file
        path: std::ffi::OsString, // path to file (platform independent)
        size: usize,              // size in bytes of file
        system: bool,             // system file or similar, not safe to delete
        category: Category,       // category of file
        is_dir: bool,             // is current node a dir
        // parent: Option<Box<Node>>, // parent node
        children: Vec<Node>, //
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
                children: Vec::new(),
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

    impl Crud for Node {
        fn add_child(&mut self, node: Node) {
            self.children.push(node);
        }
    }

    pub fn start(root: OsString) -> Result<(), Error> {
        let s = root.to_os_string();
        let mut node = Node {
            path: OsString::from(&root),
            is_dir: true,
            ..Default::default()
        };
        let mut nlist = NodeList {
            root_path: s,
            root: node,
        };
        let node = &mut nlist.root;
        get_nodes(Path::new(&root), node)?;
        print_list(&nlist);
        Ok(())
    }

    pub fn print_list(nl: &NodeList) {
        let r = &nl.root;
        for c in &r.children {
            match c.is_dir {
                true => {
                    println!("dir: {}", c.name);
                    println!("-----");
                },
                false => println!("{}", c.name),
            }
            println!("{}", c.size);
        }
        for c in &r.children {
            print_nodes(c);
        }
    }

    pub fn print_nodes(n: &Node) {
        match n.is_dir {
            true => {
                println!("dir: {}", n.name);
                println!("-----");
            }
            false => println!("{}", n.name),
        }
        for c in &n.children {
            print_nodes(c);
        }
    }

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
            size: size as usize,
            path: OsString::from(&path),
            is_dir,
            ..Default::default()
        });
    }

    pub fn get_nodes(dir: &Path, parent: &mut Node) -> io::Result<()> {
        // std::mem::size_of::Node;
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;

                let path = entry.path();
                let n = create_node(&entry)?;
                parent.children.push(n);
                let node = parent.children.last_mut().unwrap();
                if path.is_dir() {
                    get_nodes(&path, node)?;
                } else {
                    get_nodes(&path, parent)?;
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
