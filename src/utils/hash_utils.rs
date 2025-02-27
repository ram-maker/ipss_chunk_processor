use sha2::{Digest, Sha256};

pub struct Hasher;
impl Hasher {
    pub fn pair(string1:&String,string2: &String) -> String {
        let mut hasher = Sha256::new();
        hasher.update(string1);
        hasher.update(string2);
        let result=hasher.finalize();
        hex::encode(result)
    }

}
