use crate::utils::hash_utils::Hasher;
use derive_builder::Builder;
use std::error::Error;
use log::debug;

#[derive(Default, Debug, Clone, Builder)]
#[builder(default)]
pub struct Block {
    hashes: Vec<String>,
    merkle_root: Option<String>,
    merkle_tree: Option<MerkleNode>,
}
#[derive(Debug, Clone)]
pub enum MerkleNode {
    Leaf(String),
    Branch(String, Box<MerkleNode>, Box<MerkleNode>),
}

impl Block {
    pub fn new(hashes: &Vec<String>) -> Block {
        Self {
            hashes: hashes.clone(),
            ..Default::default()
        }
    }

    pub fn build_merkle_tree(&mut self) -> Result<(), Box<dyn Error>> {
        if self.hashes.is_empty() {
            return Err("Empty Hashes".into());
        }
        let nodes = self
            .hashes
            .iter()
            .map(|x| MerkleNode::Leaf(x.clone()))
            .collect::<Vec<MerkleNode>>();
        self.construct_tree(nodes);
        Ok(())
    }

    fn construct_tree(&mut self, mut nodes: Vec<MerkleNode>) {
        let mut count = 0;
        while nodes.len() > 1 {
            let mut level = Vec::new();
            for pair in nodes.chunks(2) {
                self.create_new_level(pair,&mut level);
                count += 1;
            }
            nodes=level;
        }
        self.set_merkle_root(&nodes);

    }
    fn create_new_level<'a>(&self, pair: & 'a [MerkleNode], level:& 'a mut Vec<MerkleNode>)->& 'a mut Vec<MerkleNode>{
        let mut hashed_string = String::new();
        if pair.len() == 2 {
            let left=&pair[0];
            let right=&pair[1];
            hashed_string = self.get_hash_from_pair(left, right);
            level.push(MerkleNode::Branch(
                hashed_string.clone(),
                Box::new(pair[0].clone()),
                Box::new(pair[1].clone()),
            ));
        } else {
            level.push(pair[0].clone());
        }
        level
    }
    fn get_hash_from_pair(&self, left: &MerkleNode,right:&MerkleNode) -> String {
        match (left, right) {
            (MerkleNode::Leaf(hash1), MerkleNode::Leaf(hash2)) => Hasher::pair(hash1, hash2),
            (
                MerkleNode::Branch(hash1, _, _),
                MerkleNode::Branch(hash2, _, _),
            ) => Hasher::pair(hash1, hash2),
            (MerkleNode::Leaf(hash1), MerkleNode::Branch(hash2, _, _)) => {
                Hasher::pair(hash1, hash2)
            }
            (MerkleNode::Branch(hash1, _, _), MerkleNode::Leaf(hash2)) => {
                Hasher::pair(hash1, hash2)
            }
        }
    }

    fn set_merkle_root(&mut self, merkle_node: &Vec<MerkleNode>) {
        if let Some(root_node) = merkle_node.first() {
            self.merkle_tree=Some(root_node.clone());
            match root_node {
                MerkleNode::Leaf(hash) => {
                    self.merkle_root = Some(hash.clone());
                },
                MerkleNode::Branch(hash, hash2, _) => {
                    self.merkle_root = Some(hash.clone());
                }
            }
        }
        debug!("Merkle root hash:{:?}",self.merkle_root);
    }

    pub fn get_merkle_root(&self) -> &Option<String> {
        &self.merkle_root
    }

    pub fn get_merkle_tree(&self) -> &Option<MerkleNode> {
        &self.merkle_tree
    }

    pub fn get_hashes(&self) -> &Vec<String> {
        &self.hashes
    }
}
