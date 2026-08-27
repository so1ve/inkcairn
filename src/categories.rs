use std::collections::BTreeMap;

use crate::content::CategoryPath;
use crate::render::RenderedPost;

pub struct Category {
    pub path: Vec<CategoryPath>,
    pub posts: Vec<usize>,
    pub children: Vec<Self>,
}

struct Node {
    path: CategoryPath,
    posts: Vec<usize>,
    children: BTreeMap<String, Self>,
}

pub fn collect(posts: &[RenderedPost]) -> Vec<Category> {
    let mut roots = BTreeMap::<String, Node>::new();

    for (index, post) in posts.iter().enumerate() {
        let mut children = &mut roots;

        for (depth, category) in post.path.categories.iter().enumerate() {
            let node = children
                .entry(category.label.clone())
                .or_insert_with(|| Node {
                    path: category.clone(),
                    posts: Vec::new(),
                    children: BTreeMap::new(),
                });
            if depth + 1 == post.path.categories.len() {
                node.posts.push(index);
            }
            children = &mut node.children;
        }
    }

    build(roots, &[])
}

fn build(nodes: BTreeMap<String, Node>, parents: &[CategoryPath]) -> Vec<Category> {
    nodes
        .into_values()
        .map(|node| {
            let mut path = parents.to_vec();
            path.push(node.path);
            let children = build(node.children, &path);

            Category {
                path,
                posts: node.posts,
                children,
            }
        })
        .collect()
}
