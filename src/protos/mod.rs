//! ```
//! use glx::protos::read_blobs;
//! use std::fs::File;
//!
//! assert!(read_blobs(File::open("pbf/massachusetts-latest.osm.pbf").unwrap()).next().is_some());
//! ```
//! https://wiki.openstreetmap.org/wiki/PBF_Format
//! Protobuf lives here: https://github.com/scrosby/OSM-binary
//! File format protobuf: https://github.com/scrosby/OSM-binary/blob/master/src/fileformat.proto
//! OSM format protobuf: https://github.com/scrosby/OSM-binary/blob/master/src/osmformat.proto
use crate::protos::fileformat::{Blob, BlobHeader};
use crate::protos::osmformat::{DenseNodes, HeaderBlock, Node, PrimitiveBlock, Way};
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use protobuf::Message;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Read;
use std::io::{ErrorKind, Write};

// Using the OSM protobuf definitions https://github.com/scrosby/OSM-binary, in the src dir
// These are the modules generated from those files
pub mod fileformat;
pub mod osmformat;

#[derive(Debug)]
pub enum FileBlock {
    Header(HeaderBlock),
    Primitive(PrimitiveBlock),
}

#[derive(Debug)]
pub struct BlobData {
    header: BlobHeader,
    blob: Blob,
}

impl BlobData {
    /// From the wiki docs:
    /// NOTE (12/2010): No encoder currently supports lzma or bzip2. To simplify decoder
    /// implementations, bzip2 has been deprecated and LZMA has become relegated to a proposed
    /// extension.
    ///
    /// In order to robustly detect illegal or corrupt files, I limit the maximum size of BlobHeader
    /// and Blob messages. The length of the BlobHeader should be less than 32 KiB (32*1024 bytes)
    /// and must be less than 64 KiB. The uncompressed length of a Blob should be less than 16 MiB
    /// (16*1024*1024 bytes) and must be less than 32 MiB.
    fn deserialize_self_as<M: Message>(&self) -> M {
        if self.blob.has_raw() {
            protobuf::parse_from_bytes(self.blob.get_raw()).unwrap()
        } else if self.blob.has_zlib_data() {
            assert!(self.blob.get_raw_size() < 32 * 1024 * 1024);
            let read = std::io::Cursor::new(self.blob.get_zlib_data());
            let mut decoder = ZlibDecoder::new(read);
            let result = protobuf::parse_from_reader(&mut decoder).unwrap();
            assert_eq!(
                decoder.total_out(),
                u64::try_from(self.blob.get_raw_size()).unwrap()
            );
            result
        } else {
            unimplemented!();
        }
    }

    /// From the wiki:
    /// Parsers should ignore and skip fileblock types that they do not recognize.
    pub fn deserialize(&self) -> FileBlock {
        match self.header.get_field_type() {
            "OSMHeader" => FileBlock::Header(self.deserialize_self_as()),
            "OSMData" => {
                let primitive: PrimitiveBlock = self.deserialize_self_as();
                assert_eq!(primitive.get_lat_offset(), 0, "TODO support lat/lon offset");
                assert_eq!(primitive.get_lon_offset(), 0, "TODO support lat/lon offset");
                FileBlock::Primitive(primitive)
            }
            _ => unimplemented!("TODO skip unrecognized fileblock types"),
        }
    }

    fn serialize_as<M: Message>(field_type: String, message: &M) -> Self {
        let mut buffer = Vec::new();
        let raw_size: i32 = {
            let mut encoder = ZlibEncoder::new(&mut buffer, flate2::Compression::default());
            message.write_to_writer(&mut encoder);
            i32::try_from(encoder.total_in()).unwrap()
        };

        Self {
            header: {
                let mut blob_header = BlobHeader::new();
                blob_header.set_field_type(field_type);
                blob_header
            },
            blob: {
                let mut blob = Blob::new();
                blob.set_raw_size(raw_size);
                blob.set_zlib_data(buffer);
                blob
            },
        }
    }

    pub fn serialize(file_block: &FileBlock) -> Self {
        match file_block {
            FileBlock::Header(header) => Self::serialize_as(String::from("OSMHeader"), header),
            FileBlock::Primitive(primitive) => {
                Self::serialize_as(String::from("OSMData"), primitive)
            }
        }
    }
}

pub trait ReadOsmPbf: Read {
    fn read_osm_pbf_blob(&mut self) -> Option<BlobData>;
}

impl<R: Read + 'static> ReadOsmPbf for R {
    // can and should this be made async-friendly? probably not that important for a task that is
    // already so CPU-intensive.
    fn read_osm_pbf_blob(&mut self) -> Option<BlobData> {
        let block_header_len_bytes = match self.read_u32::<BigEndian>() {
            Ok(len) => len,
            Err(e) => {
                // TODO handle the situation where we read like 1 byte here
                if e.kind() == ErrorKind::UnexpectedEof {
                    return None;
                } else {
                    panic!();
                }
            }
        };

        assert!(block_header_len_bytes < 64 * 1024);

        let mut read_blob_header = self.take(block_header_len_bytes as u64);
        let header: BlobHeader = protobuf::parse_from_reader(&mut read_blob_header).unwrap();

        let mut read_blob = self.take(header.get_datasize() as u64);
        Some(BlobData {
            header,
            blob: protobuf::parse_from_reader(&mut read_blob).unwrap(),
        })
    }
}

pub trait WriteOsmPbf: Write {
    fn write_osm_pbf_blob(&mut self, blob_data: BlobData);
}

pub fn read_blobs<R: Read + 'static>(mut read: R) -> impl Iterator<Item = BlobData> {
    std::iter::from_fn(move || read.read_osm_pbf_blob())
}

pub fn write_blobs<I: Iterator<Item = BlobData>, W: Write + 'static>(blobs: I, mut write: W) {
    for blob in blobs {
        unimplemented!()
    }
}

pub fn iter_nodes(primitive_block: &PrimitiveBlock) -> impl Iterator<Item = &Node> {
    primitive_block
        .get_primitivegroup()
        .iter()
        .flat_map(|group| group.get_nodes().iter())
}

pub fn iter_dense_nodeses(primitive_block: &PrimitiveBlock) -> impl Iterator<Item = &DenseNodes> {
    primitive_block
        .get_primitivegroup()
        .iter()
        .map(|group| group.get_dense())
}

pub fn iter_node_ids(mut way: Way) -> impl Iterator<Item = i64> {
    let mut n = 0;
    way.take_refs().into_iter().map(move |dn| {
        n += dn;
        n
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct DenseNode {
    pub id: i64,
    pub lat: i64,
    pub lon: i64,
}

// Transform the column-oriented DenseNodes data structure into a row-oriented struct
pub fn as_vec_dense_nodes(dense_nodes: &DenseNodes) -> Vec<DenseNode> {
    let mut id_acc = 0;
    let mut lat_acc = 0;
    let mut lon_acc = 0;
    dense_nodes
        .get_id()
        .into_iter()
        .zip(dense_nodes.get_lat())
        .zip(dense_nodes.get_lon())
        .map(|((&id, &lat), &lon)| {
            id_acc += id;
            lat_acc += lat;
            lon_acc += lon;
            DenseNode {
                id: id_acc,
                lat: lat_acc,
                lon: lon_acc,
            }
        })
        .collect()
}

pub fn iter_ways(primitive_block: &PrimitiveBlock) -> impl Iterator<Item = &Way> {
    primitive_block
        .get_primitivegroup()
        .iter()
        .flat_map(|group| group.get_ways().iter())
}

pub struct MyWay {
    pub way: Way,
    pub tags: HashMap<String, String>,
}

pub fn into_vec_ways(mut primitive_block: PrimitiveBlock) -> Vec<MyWay> {
    let strings: Vec<String> = primitive_block
        .take_stringtable()
        .take_s()
        .into_iter()
        .map(|bytes| String::from_utf8(bytes).unwrap())
        .collect();
    primitive_block
        .take_primitivegroup()
        .into_iter()
        .flat_map(|mut group| {
            group.take_ways().into_iter().map(|way: Way| {
                let tags = way
                    .get_keys()
                    .iter()
                    .zip(way.get_vals())
                    .map(|(&key, &value)| {
                        (
                            strings[usize::try_from(key).unwrap()].clone(),
                            strings[usize::try_from(value).unwrap()].clone(),
                        )
                    })
                    .collect();
                MyWay { way, tags }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::fs::File;
    use std::io::Read;

    fn get_reader() -> impl Read {
        File::open("pbf/massachusetts-latest.osm.pbf").unwrap()
    }

    #[test]
    fn test_count_blobs() {
        assert!(read_blobs(get_reader()).count() > 0);
    }

    #[test]
    fn test_read_blocks() {
        let vec_blob: Vec<BlobData> = read_blobs(get_reader()).collect();

        assert!(
            vec_blob
                .par_iter()
                .map(|blob_data| blob_data.deserialize())
                .count()
                > 0
        );
    }

    #[test]
    fn test_count_ways() {
        let vec_blob: Vec<BlobData> = read_blobs(get_reader()).collect();

        assert!(
            dbg!(vec_blob
                .par_iter()
                .map(|blob_data| {
                    if let FileBlock::Primitive(primitive_block) = blob_data.deserialize() {
                        iter_ways(&primitive_block).count()
                    } else {
                        0
                    }
                })
                .sum::<usize>())
                > 0
        );
    }
}
