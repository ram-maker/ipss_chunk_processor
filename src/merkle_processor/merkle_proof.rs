use std::error::Error;
use crate::merkle_processor::merkle_processor::{Block, MerkleNode};
use crate::utils::hash_utils::Hasher;

#[derive(Debug, Default,Clone)]
pub struct MerkleProof{
    chunk_hash: String,
    proof_hashes: Vec<(String, bool)>,
    block: Block,
    target_index: usize,
    root_hash: String,
}
impl MerkleProof {
    pub fn new(block: Block,target_index:usize) -> Self {
        Self{
            block,
            target_index,
            ..Default::default()
        }
    }

    pub fn generate_merkle_proof(&self) -> Result<MerkleProof, Box<dyn Error>> {

        if let None = self.block.get_merkle_root(){
            return Err("merkle root is empty".into());
        };
        let chunk_hash=self.block.get_hashes()[self.target_index.clone()].clone();
        let root_hash = self.block.get_merkle_root().clone().unwrap();
        let merkle_tree=self.block.get_merkle_tree().clone().unwrap();
        let mut proof_hashes=Vec::new();
        self.collect_proof_hashes(&merkle_tree,self.target_index, &mut proof_hashes)?;
        Ok(MerkleProof{
            chunk_hash,
            proof_hashes,
            root_hash,
            target_index: self.target_index.clone(),
            block: self.block.clone(),
        })
    }

    fn collect_proof_hashes(
        &self,
        node: &MerkleNode,
        target_index: usize,
        proof: &mut Vec<(String, bool)>,
    ) -> Result<bool, Box<dyn Error>> {
        match node {
            MerkleNode::Leaf(hash) => {
                let is_target = target_index == 0 || hash == &self.block.get_hashes()[target_index];
                Ok(is_target)
            }
            MerkleNode::Branch(_, left, right) => {
                self.traverse_and_collect_proofs(left,right, target_index, proof)
            }
        }
    }

    fn traverse_and_collect_proofs(&self,left:&Box<MerkleNode>, right:&Box<MerkleNode>,target_index:usize,proof: &mut Vec<(String, bool)>)-> Result<bool, Box<dyn Error>> {
        let found_in_left = self.collect_proof_hashes(left, target_index, proof)?;
        if found_in_left {
            match &**right {
                MerkleNode::Leaf(hash) => {
                    proof.push((hash.clone(), true));
                }
                MerkleNode::Branch(hash, _, _) => {
                    proof.push((hash.clone(), true));
                }
            }
            Ok(true)
        } else {
            let found_in_right = self.collect_proof_hashes(right, target_index, proof)?;
            if found_in_right {
                match &**left {
                    MerkleNode::Leaf(hash) => {
                        proof.push((hash.clone(), false));
                    }
                    MerkleNode::Branch(hash, _, _) => {
                        proof.push((hash.clone(), false));
                    }
                }
            }
            Ok(found_in_right)
        }
    }

     pub fn verify_merkle_proof(&self, proof: &MerkleProof) -> Result<bool, Box<dyn Error>> {
        let mut current_hash = proof.chunk_hash.clone();
        for (hash, is_right) in &proof.proof_hashes {
            if *is_right {
                current_hash = Hasher::pair(&current_hash, hash);
            } else {
                current_hash = Hasher::pair(hash, &current_hash);
            }
        }
        Ok(current_hash == proof.root_hash)
    }
}