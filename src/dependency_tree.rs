use std::collections::{hash_set, HashMap, HashSet};
use std::path::Path;
use std::process::Command;

pub(crate) struct DependencyTree {
    root: String,
    nodes: HashMap<String, TreeNode>,
}

struct TreeNode {
    name: String,
    children: HashSet<String>,
}

impl DependencyTree {
    pub fn new(path: &Path) -> DependencyTree {
        let output = Command::new("cargo")
            .arg("tree")
            .arg("-e=no-dev")
            .arg("--prefix")
            .arg("depth")
            .arg("--no-dedupe")
            .current_dir(path)
            .output()
            .expect("Cargo tree failed");

        let output = String::from_utf8_lossy(&output.stdout);

        let mut map = HashMap::new();
        let mut stack = Vec::<String>::new();

        for line in output.lines() {
            let start = line.find(|c: char| !c.is_ascii_digit()).unwrap();
            let (depth, line) = line.split_at(start);
            let depth = depth.parse::<usize>().unwrap();

            let dep = crate_name_from_package_id(line);

            while depth < stack.len() {
                stack.pop();
            }

            if stack.contains(&dep) {
                continue;
            }

            map.entry(dep.clone()).or_insert_with(|| TreeNode {
                name: dep.clone(),
                children: HashSet::new(),
            });

            if let Some(parent) = stack.last() {
                map.entry(parent.clone()).and_modify(|parent| {
                    parent.children.insert(dep.clone());
                });
            }

            stack.push(dep);
        }

        stack.truncate(1);
        assert!(!stack.is_empty());

        DependencyTree {
            root: stack.pop().unwrap(),
            nodes: map,
        }
    }

    pub fn get(&self, name: &str) -> Option<Dependency> {
        if let Some(node) = self.nodes.get(name) {
            Some(Dependency {
                map: &self.nodes,
                node,
            })
        } else {
            None
        }
    }

    pub fn root(&self) -> Dependency {
        Dependency {
            map: &self.nodes,
            node: &self.nodes[&self.root],
        }
    }
}

#[derive(Clone)]
pub struct Dependency<'a> {
    map: &'a HashMap<String, TreeNode>,
    node: &'a TreeNode,
}

impl<'a> Dependency<'a> {
    pub fn name(&self) -> &str {
        &self.node.name
    }

    pub fn children_count(&self) -> usize {
        self.node.children.len()
    }
}

impl<'a> IntoIterator for Dependency<'a> {
    type IntoIter = DependencyIterator<'a>;
    type Item = <DependencyIterator<'a> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        DependencyIterator::new(self)
    }
}

pub struct DependencyIterator<'a> {
    node: Dependency<'a>,
    iter: hash_set::Iter<'a, String>,
    index: usize,
}

impl<'a> DependencyIterator<'a> {
    fn new(node: Dependency) -> DependencyIterator {
        let cloned = node.clone();
        DependencyIterator {
            node,
            iter: cloned.node.children.iter(),
            index: 0,
        }
    }

    pub fn name(&self) -> &str {
        self.node.name()
    }

    pub fn index(&self) -> Option<usize> {
        if self.index == 0 {
            None
        } else {
            Some(self.index - 1)
        }
    }

    pub fn len(&self) -> usize {
        self.node.children_count()
    }
}

impl<'a> Iterator for DependencyIterator<'a> {
    type Item = Dependency<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dep) = self.iter.next() {
            self.index += 1;
            Some(Dependency {
                map: self.node.map,
                node: &self.node.map[dep],
            })
        } else {
            None
        }
    }
}

pub(crate) fn crate_name_from_package_id(pkg_id: &str) -> String {
    let stop = pkg_id.find(" (").unwrap_or(pkg_id.len());
    let split = pkg_id[..stop].trim().rsplitn(2, " ").collect::<Vec<_>>();
    let start = if split[0].starts_with("v") { 1 } else { 0 };
    format!("{} {}", split[1].replace("_", "-"), &split[0][start..])
}
