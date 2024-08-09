use std::{cmp, collections::HashMap, io::{Read, Write}, sync::Mutex};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use lazy_static::lazy_static;
use log::info;
use screeps::{Position, RoomCoordinate, RoomName, RoomXY};

use crate::traits::room::RoomNameExtensions;

pub mod compressed_matrix;

pub fn compress_room_name(name: RoomName) -> String {
    name.to_string()
}

pub fn compress_pos(pos: Position) -> String {
    format!("{}:{}:{}", pos.x(), pos.y(), pos.room_name())
}

pub fn decompress_pos(pos: String) -> Position {
    let parts: Vec<&str> = pos.split(':').collect();
    Position::new(RoomCoordinate::new(parts[0].parse().unwrap()).unwrap(), RoomCoordinate::new(parts[1].parse().unwrap()).unwrap(), RoomName::new(parts[2]).unwrap())
}

pub fn compress_pos_list(list: Vec<Position>) -> String {
    let mut result = String::new();
    for pos in list {
        result.push_str(&compress_pos(pos));
        result.push(',');
    }
    result
}

pub fn encode_pos_list(list: Vec<Position>) -> String {
    let mut encoded_pos = compress_pos_list(list);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(encoded_pos.as_bytes());
    if let Ok(compressed) = encoder.finish() {
        let compressed_string = base65536::encode(&compressed, None);

        info!("  [COMPRESSION] Compressed {} bytes to {} bytes", encoded_pos.len(), compressed_string.len());

        compressed_string
    } else {
        info!("  [COMPRESSION] Failed to compress!");

        encoded_pos
    }
}

pub fn decode_pos_list(pos: String) -> Vec<Position> {
    if let Ok(decoded_string) = base65536::decode(&pos, true) {
        let mut encoder = GzDecoder::new(&decoded_string[..]);
        let mut decoded_string = String::new();

        if let Ok(_) = encoder.read_to_string(&mut decoded_string) {
            let mut result = Vec::new();
            let parts: Vec<&str> = decoded_string.split(',').collect();
            for part in parts {
                if part == "" {
                    continue;
                }

                result.push(decompress_pos(part.to_string()));
            }
            return result;
        }
    }

    Vec::new()
}