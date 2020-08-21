use itertools::Itertools;
use md5;
use std::rc::Rc;
use crate::drawing::Color;

#[derive(Debug)]
pub struct TreeNode {
    pub name: String,
    pub children: Vec<Rc<TreeNode>>,
    pub color: Color
}

#[derive(Debug, Clone)]
struct FlatEntry {
    depth: usize,
    name: String,
}

fn parse(raw: String) -> Vec<FlatEntry> {
    let mut result = Vec::<FlatEntry>::new();
    for line in raw.lines() {
        let idx = line
            .chars()
            .take_while(|x| x.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .unwrap();

        let start = line.find(|c: char| !c.is_ascii_digit()).unwrap();
        let stop = line.find(|c: char| c.is_whitespace()).unwrap();

        let package = &line[start..stop];

        result.push(FlatEntry {
            depth: idx,
            name: package.replace("_", "-").to_string(),
        });
    }
    result
}

fn tree(flat: Vec<FlatEntry>) -> Rc<TreeNode> {
    let root = &flat[0];
    let candidates = &flat[1..];

    let name = root.name.to_string();

    let digest = md5::compute(name.clone().into_bytes());

    Rc::<_>::new(TreeNode {
        color: (digest[0], digest[1], digest[2]),
        name: name,
        children: candidates
            .iter()
            .take_while(|child| child.depth > root.depth)
            .enumerate()
            .filter_map(|(idx, child)| {
                if child.depth == root.depth + 1 {
                    Some(tree(candidates[idx..].to_vec()))
                } else {
                    None
                }
            })
            .sorted_by_key(|child| child.children.len())
            .collect::<Vec<_>>(),
    })
}

pub fn parse_tree(raw: String) -> Rc<TreeNode> {
    tree(parse(raw))
}
