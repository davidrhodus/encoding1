use reed_solomon_erasure::galois_8;
use reed_solomon_erasure::ReedSolomon;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use serde_json::{Result as SerdeResult, Value};
use blake3::Hasher;
use std::collections::HashMap;


// fn main() {
//     // Define the input data
//     //let data = b"Hello, world!";

//     let mut file = File::open("input.txt").expect("Unable to open input.txt");
//     let mut data = Vec::new();
//     file.read_to_end(&mut data).expect("Unable to read input.txt");

//     // Configure the Reed-Solomon encoder
//     let data_shards = 8;
//     let parity_shards = 4;
//     let total_shards = data_shards + parity_shards;

//     let rs = ReedSolomon::<galois_8::Field>::new(data_shards, parity_shards).unwrap();

//     // Pad the input data to fit into the data shards
//     let shard_size = (data.len() + data_shards - 1) / data_shards;
//     let padded_data = {
//         let mut tmp = Vec::with_capacity(shard_size * data_shards);
//         tmp.extend_from_slice(&data);
//         tmp.resize(shard_size * data_shards, 0);
//         tmp
//     };

//     // Create and populate the shards
//     let mut shards: Vec<Vec<u8>> = vec![vec![0; shard_size]; total_shards];
//     for (i, chunk) in padded_data.chunks(shard_size).enumerate() {
//         shards[i].copy_from_slice(chunk);
//     }

//     // Encode the input data using Reed-Solomon
//     rs.encode(&mut shards).unwrap();

//     // Save each shard to an individual file
//     for (i, shard) in shards.iter().enumerate() {
//         let filename = format!("shard_{}.bin", i);
//         let mut file = File::create(&filename).unwrap();
//         file.write_all(shard).unwrap();
//         println!("Saved shard {} to disk as {}.", i, filename);
//     }

//     // Read the shards from disk and restore the original data
//     let restored_data = restore_data(data_shards, parity_shards, data.len()).unwrap();
//     println!("Restored data: {:?}", String::from_utf8_lossy(&restored_data));
// }

// fn main() {
//     // // Read the input data from the input.txt file and save the encoded shards
//     // encode_and_save("input.txt").expect("Unable to encode and save the input data");

//     // // Read the shards from disk and restore the original data
//     // let data_shards = 8;
//     // let parity_shards = 4;
//     // let original_data_len = std::fs::metadata("input.txt").expect("Unable to read input.txt metadata").len() as usize;
//     // let restored_data = restore_data(data_shards, parity_shards, original_data_len).unwrap();
//     // println!("Restored data: {:?}", String::from_utf8_lossy(&restored_data));


//     // Check if the input file name is provided
//     let args: Vec<String> = env::args().collect();
//     if args.len() < 2 {
//         eprintln!("Usage: {} <input_file>", args[0]);
//         return;
//     }

//     // Read the input data from the specified file and save the encoded shards
//     encode_and_save(&args[1]).expect("Unable to encode and save the input data");

//     // Read the shards from disk and restore the original data
//     let data_shards = 8;
//     let parity_shards = 4;
//     let original_data_len = std::fs::metadata(&args[1]).expect("Unable to read input file metadata").len() as usize;
//     let restored_data = restore_data(data_shards, parity_shards, original_data_len).unwrap();
//     println!("Restored data: {:?}", String::from_utf8_lossy(&restored_data));
// }

// Update the main function to use the new function when restoring data
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return;
    }

    encode_and_save(&args[1]).expect("Unable to encode and save the input data");

    let data_shards = 8;
    let parity_shards = 4;
    let original_data_len = std::fs::metadata(&args[1]).expect("Unable to read input file metadata").len() as usize;
    let hash_hex = get_hash_from_metadata(&args[1]).expect("Unable to get hash from metadata");
    let restored_data = restore_data(data_shards, parity_shards, original_data_len, &hash_hex).unwrap();
    println!("Restored data: {:?}", String::from_utf8_lossy(&restored_data));
}

// fn encode_and_save(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
//     // Read the input data from the input.txt file
//     let mut file = File::open(input_file)?;
//     let mut data = Vec::new();
//     file.read_to_end(&mut data)?;

//     // Configure the Reed-Solomon encoder
//     let data_shards = 8;
//     let parity_shards = 4;
//     let total_shards = data_shards + parity_shards;

//     let rs = ReedSolomon::<galois_8::Field>::new(data_shards, parity_shards)?;

//     // Pad the input data to fit into the data shards
//     let shard_size = (data.len() + data_shards - 1) / data_shards;
//     let padded_data = {
//         let mut tmp = Vec::with_capacity(shard_size * data_shards);
//         tmp.extend_from_slice(&data);
//         tmp.resize(shard_size * data_shards, 0);
//         tmp
//     };

//     // Create and populate the shards
//     let mut shards: Vec<Vec<u8>> = vec![vec![0; shard_size]; total_shards];
//     for (i, chunk) in padded_data.chunks(shard_size).enumerate() {
//         shards[i].copy_from_slice(chunk);
//     }

//     // Encode the input data using Reed-Solomon
//     rs.encode(&mut shards)?;

//     // Save each shard to an individual file
//     for (i, shard) in shards.iter().enumerate() {
//         let filename = format!("shard_{}.bin", i);
//         let mut file = File::create(&filename)?;
//         file.write_all(shard)?;
//         println!("Saved shard {} to disk as {}.", i, filename);
//     }

//     Ok(())
// }

fn encode_and_save(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the input data from the input file
    let mut file = File::open(input_file)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Compute the BLAKE3 hash of the input data
    let hash = {
        let mut hasher = Hasher::new();
        hasher.update(&data);
        hasher.finalize()
    };
    let hash_hex = hash.to_hex();

    let metadata_filename = "metadata.txt";
    let mut metadata: HashMap<String, String> = match File::open(&metadata_filename) {
        Ok(file) => {
            serde_json::from_reader(file).unwrap_or_else(|_| {
                eprintln!("Failed to read metadata file. Creating a new one.");
                HashMap::new()
            })
        }
        Err(_) => {
            eprintln!("Metadata file not found. Creating a new one.");
            HashMap::new()
        }
    };

    metadata.insert(input_file.to_string(), hash_hex.to_string());
    let metadata_file = File::create(metadata_filename)?;
    serde_json::to_writer_pretty(metadata_file, &metadata)?;

    // Configure the Reed-Solomon encoder
    let data_shards = 8;
    let parity_shards = 4;
    let total_shards = data_shards + parity_shards;

    let rs = ReedSolomon::<galois_8::Field>::new(data_shards, parity_shards)?;

    // Pad the input data to fit into the data shards
    let shard_size = (data.len() + data_shards - 1) / data_shards;
    let padded_data = {
        let mut tmp = Vec::with_capacity(shard_size * data_shards);
        tmp.extend_from_slice(&data);
        tmp.resize(shard_size * data_shards, 0);
        tmp
    };

    // Create and populate the shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; shard_size]; total_shards];
    for (i, chunk) in padded_data.chunks(shard_size).enumerate() {
        shards[i].copy_from_slice(chunk);
    }

    // Encode the input data using Reed-Solomon
    rs.encode(&mut shards)?;

    // Save each shard to an individual file
    for (i, shard) in shards.iter().enumerate() {
        let filename = format!("{}_shard_{}.bin", hash_hex, i);
        let mut file = File::create(&filename)?;
        file.write_all(shard)?;
        println!("Saved shard {} to disk as {}.", i, filename);
    }

    Ok(())
}

// fn restore_data(
//     data_shards: usize,
//     parity_shards: usize,
//     original_data_len: usize,
// ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
//     let total_shards = data_shards + parity_shards;

//     // Reconstruct the Reed-Solomon instance
//     let rs = ReedSolomon::<galois_8::Field>::new(data_shards, parity_shards)?;

//     // Read the shards from disk
//     let mut shards: Vec<Box<[u8]>> = Vec::with_capacity(total_shards);
//     let mut shard_present: Vec<bool> = vec![false; total_shards];
//     for i in 0..total_shards {
//         let filename = format!("shard_{}.bin", i);
//         match File::open(&filename) {
//             Ok(mut file) => {
//                 let mut shard = Vec::new();
//                 file.read_to_end(&mut shard)?;
//                 shards.push(shard.into_boxed_slice());
//                 shard_present[i] = true;
//             }
//             Err(_) => {
//                 eprintln!("Failed to read shard {} from disk. Assuming it is missing.", i);
//                 shards.push(Vec::new().into_boxed_slice());
//             }
//         }
//     }

//     // Zip the shards and shard_present vectors
//     let mut shards_with_presence: Vec<(Box<[u8]>, bool)> = shards.into_iter().zip(shard_present.into_iter()).collect();

//     // Restore the original data using Reed-Solomon decoding
//     rs.reconstruct(&mut shards_with_presence)?;

//     // Remove padding and return the original data
//     let mut restored_data = Vec::new();
//     for chunk in shards_with_presence[..data_shards].iter().flat_map(|(shard, _)| shard.iter()) {
//         restored_data.push(*chunk);
//         if restored_data.len() == original_data_len {
//             break;
//         }
//     }

//     Ok(restored_data)
// }

// Create a new function to get the hash from the metadata
fn get_hash_from_metadata(input_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let metadata_filename = "metadata.txt";
    let file = File::open(&metadata_filename)?;
    let metadata: HashMap<String, String> = serde_json::from_reader(file)?;
    
    metadata.get(input_file)
        .map(|hash| hash.to_string())
        .ok_or_else(|| format!("File {} not found in metadata.", input_file).into())
}

fn restore_data(data_shards: usize, parity_shards: usize, original_data_len: usize, hash_hex: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let total_shards = data_shards + parity_shards;
    let rs = ReedSolomon::<galois_8::Field>::new(data_shards, parity_shards)?;

    // Read the shard files from disk
    let mut shards: Vec<Option<Box<[u8]>>> = Vec::with_capacity(total_shards);
    let mut shard_present: Vec<bool> = vec![false; total_shards];

    for i in 0..total_shards {
        let filename = format!("{}_shard_{}.bin", hash_hex, i);

        match File::open(&filename) {
            Ok(mut file) => {
                let mut shard = Vec::new();
                file.read_to_end(&mut shard)?;
                shards.push(Some(shard.into_boxed_slice()));
                shard_present[i] = true;
            }
            Err(_) => {
                shards.push(None);
            }
        }
    }

    // Reconstruct the missing shards using Reed-Solomon
    //rs.reconstruct_shards(&mut shards, &shard_present)?;
    rs.reconstruct(&mut shards)?;


    // Combine the data shards to restore the original data
    let mut restored_data = Vec::with_capacity(original_data_len);
    for shard in shards.into_iter().take(data_shards) {
        if let Some(shard) = shard {
            restored_data.extend_from_slice(&shard);
        }
    }

    restored_data.truncate(original_data_len);
    Ok(restored_data)
}

