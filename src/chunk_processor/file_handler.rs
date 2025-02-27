use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::slice::Chunks;
use std::sync::{Arc, Mutex};
use std::thread;
use log::{error, debug};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use derive_builder::Builder;

#[derive(Default,Clone)]
pub struct FileHandler<T> {
    file_path:T,
    file_size:u64,
    chunks:Vec<usize>,
}

impl<T:AsRef<Path>+Clone+Default+Send + 'static+Sync> FileHandler<T>{
    pub fn new(file_path:T)->Self
    {
        Self{
            file_path:file_path.clone(),
            ..Default::default()
        }
    }

    pub fn process_chunks(& mut self)->Result<(),std::io::Error>{
        let file_content=Arc::new(self.get_file()?);
        self.populate_fields(Arc::clone(&file_content))?;
        Ok(())
    }

    pub(crate) fn get_file(&self)->Result<File,std::io::Error>{
        let file = File::open(self.file_path.clone());
        if let Err(file_error) = file {
            error!("File Error: {:?}", file_error);
            return Err(file_error);
        }
        file
    }

    fn populate_fields(& mut self,file:Arc<File>) ->Result<(),std::io::Error> {
        self.file_size=file.metadata()?.len();
        debug!("File Size: {}", self.file_size);
        self.populate_chunk();
        Ok(())
    }

    fn populate_chunk(&mut self){
        let chunk_size=(self.file_size/10) as usize;
        let remainder_byte=(self.file_size%10) as usize;
        let mut chunks=vec![chunk_size;10];
        if remainder_byte > 0 {
            chunks.push(remainder_byte);
        }
        self.chunks=chunks;
        debug!("Vec chunk:{:?} ",self.chunks);
    }

    fn receive_chunks(&self,receiver: Receiver<String>)->Vec<String>{
        let mut chunks=Vec::<String>::new();
        receiver.iter().for_each(|chunk|{chunks.push(chunk.clone());});
        chunks

    }

    pub(crate) fn get_chunk(&self)->&Vec<usize>{
        &self.chunks
    }

}