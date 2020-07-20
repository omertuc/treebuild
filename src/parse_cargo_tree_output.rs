use itertools::Itertools;
use std::rc::Rc;

#[derive(Debug)]
pub struct TreeNode {
    pub name: String,
    pub children: Vec<Rc<TreeNode>>,
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
            name: package.to_string(),
        });
    }
    result
}

fn tree(flat: Vec<FlatEntry>) -> Rc<TreeNode> {
    let root = &flat[0];
    let candidates = &flat[1..];

    Rc::<_>::new(TreeNode {
        name: root.name.to_string(),
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
