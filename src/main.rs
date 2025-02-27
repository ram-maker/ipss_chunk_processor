use log::debug;
use chunk_ipfss::chunk_processor::file_handler::{FileHandler};
use chunk_ipfss::ipfss_processor::ipfss_processor::IpfsChain;
use chunk_ipfss::merkle_processor::merkle_processor::Block;
use chunk_ipfss::merkle_processor::merkle_proof::MerkleProof;

#[tokio::main]
async  fn main() -> Result<(), Box<dyn std::error::Error>>{
    env_logger::init();
    let mut file_handler=FileHandler::new("uploads/test_file");
    file_handler.process_chunks()?;
    let mut ipfs_chain=IpfsChain::new(file_handler);
    ipfs_chain.handle().await?;
    let mut block=Block::new(ipfs_chain.get_hashes());
    block.build_merkle_tree()?;
    let proof=MerkleProof::new(block,2).generate_merkle_proof()?;
    let verify=proof.verify_merkle_proof(&proof)?;
    debug!("proof:{:?}",verify);
    Ok(())
}
