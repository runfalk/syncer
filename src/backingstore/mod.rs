extern crate bincode;
extern crate libc;

mod blobstorage;
mod metadatadb;

use self::blobstorage::*;
use self::metadatadb::*;
pub use self::blobstorage::BlobHash;
use super::filesystem::FSEntry;

use self::bincode::{serialize, deserialize, Infinite};
use self::libc::c_int;
use std::sync::Mutex;
use std::fs;

pub struct BackingStore {
  blobs: BlobStorage,
  nodes: MetadataDB,
  node_counter: Mutex<u64>,
}

impl BackingStore {
  pub fn new(path: &str) -> Result<Self, c_int> {
    match fs::create_dir_all(&path) {
      Ok(_) => {},
      Err(_) => return Err(libc::EIO),
    }

    Ok(Self {
      blobs: BlobStorage::new(path),
      nodes: MetadataDB::new(path),
      node_counter: Mutex::new(0),
    })
  }

  pub fn blob_zero(size: usize) -> BlobHash {
    BlobStorage::zero(size)
  }

  pub fn add_blob(&self, data: &[u8]) -> Result<BlobHash, c_int> {
    self.blobs.add_blob(data)
  }

  pub fn create_node(&self, entry: FSEntry) -> Result<u64, c_int> {
    let node = {
      let mut counter = self.node_counter.lock().unwrap();
      *counter += 1;
      *counter
    };
    try!(self.save_node(node, &entry));
    Ok(node)
  }

  pub fn save_node(&self, node: u64, entry: &FSEntry) -> Result<(), c_int> {
    let encoded: Vec<u8> = serialize(entry, Infinite).unwrap();
    let hash = try!(self.blobs.add_blob(&encoded));
    try!(self.nodes.set(node, &hash));
    Ok(())
  }

  pub fn get_node(&self, node: u64) -> Result<FSEntry, c_int> {
    let hash = try!(self.nodes.get(node));
    let buffer = try!(self.blobs.read_all(&hash));
    Ok(deserialize(&buffer[..]).unwrap())
  }

  pub fn read(&self, hash: &BlobHash, offset: usize, bytes: usize) -> Result<Vec<u8>, c_int> {
    self.blobs.read(hash, offset, bytes)
  }

  pub fn write(&self, hash: &BlobHash, offset: usize, data: &[u8]) -> Result<BlobHash, c_int> {
    self.blobs.write(hash, offset, data)
  }
}