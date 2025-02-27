use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use futures::stream::{self, StreamExt};
use ipfs_api::{IpfsApi, IpfsClient};
use log::{debug, error};
use crate::chunk_processor::file_handler::FileHandler;
static IPFS_CLIENT: OnceLock<IpfsClient> = OnceLock::new();
#[derive(Default)]
pub struct IpfsChain<T>{
    file_handler: FileHandler<T>,
    hashes:Vec<String>
}
impl <T:Default>IpfsChain<T>{
    pub fn new(file_handler: FileHandler<T>) -> Self{
        Self{
            file_handler,
            ..Default::default()
        }
    }
}
impl <T:AsRef<Path>+Clone+Default+Send + 'static+Sync> IpfsChain<T>{

    fn create_client(){
        IPFS_CLIENT.get_or_init(|| {IpfsClient::default()});
    }

    pub fn get_client() ->  Result<&'static IpfsClient, Box<dyn std::error::Error>> {
        IPFS_CLIENT.get().ok_or("Cant get IPFS connection".into())
    }
    pub async fn handle(&mut self)->Result<(),Box<dyn std::error::Error>>{
        let file=self.file_handler.get_file()?;
        Self::create_client();
        self.spawn_ipfs_worker(Arc::new(file)).await?;
        Ok(())
    }

    async fn spawn_ipfs_worker(&mut self,file:Arc<File>)->Result<(),Box<dyn std::error::Error>>{
        let mut start=0;
        let futures = self.file_handler.get_chunk().iter().enumerate().map(move |(idx, chunk)| {
            let chunk_size = *chunk;
            let current_start = start;
            start = start + chunk_size as u64;
            let file_clone=Arc::clone(&file);
            async move {
                match Self::upload_to_node(file_clone, current_start, chunk_size).await {
                    Ok(cid) => {
                        debug!("Chunk {}: Uploaded to IPFS with CID {}", idx, cid);
                        Ok(cid)
                    }
                    Err(e) => {
                        error!("Error processing chunk {} with IPFS: {:?}", idx, e);
                        Err(e)
                    }
                }
            }
        });
        let results = stream::iter(futures)
            .buffer_unordered(4) // Process up to 4 chunks concurrently
            .collect::<Vec<_>>()
            .await;
        let mut chunk_hashes = Vec::new();
        for result in results {
            match result {
                Ok(cid) => chunk_hashes.push(cid),
                Err(e) => return Err(e.into()),
            }
        }
        self.hashes=chunk_hashes;
        Ok(())
    }

    async fn upload_to_node(mut file:Arc<File>,file_byte_position:u64,chunk_byte:usize)->Result<String,Box<dyn std::error::Error>>{
        let ipfs_client= IpfsChain::<String>::get_client()?;
        file.seek(SeekFrom::Start(file_byte_position))?;
        let mut buffer = vec![0; chunk_byte];
        file.read_exact(&mut buffer)?;
        let cursor=Cursor::new(buffer);
        let cid = ipfs_client.add(cursor).await?;
        Ok(cid.hash)
    }

    pub fn get_hashes(&self)->&Vec<String>{&self.hashes}


}