use reed_solomon_erasure::galois_8;
use reed_solomon_erasure::ReedSolomon;
use std::env;
use std::fs::File;
use std::io::{Read, Write};

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

fn main() {
    // // Read the input data from the input.txt file and save the encoded shards
    // encode_and_save("input.txt").expect("Unable to encode and save the input data");

    // // Read the shards from disk and restore the original data
    // let data_shards = 8;
    // let parity_shards = 4;
    // let original_data_len = std::fs::metadata("input.txt").expect("Unable to read input.txt metadata").len() as usize;
    // let restored_data = restore_data(data_shards, parity_shards, original_data_len).unwrap();
    // println!("Restored data: {:?}", String::from_utf8_lossy(&restored_data));


    // Check if the input file name is provided
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return;
    }

    // Read the input data from the specified file and save the encoded shards
    encode_and_save(&args[1]).expect("Unable to encode and save the input data");

    // Read the shards from disk and restore the original data
    let data_shards = 8;
    let parity_shards = 4;
    let original_data_len = std::fs::metadata(&args[1]).expect("Unable to read input file metadata").len() as usize;
    let restored_data = restore_data(data_shards, parity_shards, original_data_len).unwrap();
    println!("Restored data: {:?}", String::from_utf8_lossy(&restored_data));
}

fn encode_and_save(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the input data from the input.txt file
    let mut file = File::open(input_file)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

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
        let filename = format!("shard_{}.bin", i);
        let mut file = File::create(&filename)?;
        file.write_all(shard)?;
        println!("Saved shard {} to disk as {}.", i, filename);
    }

    Ok(())
}

fn restore_data(
    data_shards: usize,
    parity_shards: usize,
    original_data_len: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let total_shards = data_shards + parity_shards;

    // Reconstruct the Reed-Solomon instance
    let rs = ReedSolomon::<galois_8::Field>::new(data_shards, parity_shards)?;

    // Read the shards from disk
    let mut shards: Vec<Box<[u8]>> = Vec::with_capacity(total_shards);
    let mut shard_present: Vec<bool> = vec![false; total_shards];
    for i in 0..total_shards {
        let filename = format!("shard_{}.bin", i);
        match File::open(&filename) {
            Ok(mut file) => {
                let mut shard = Vec::new();
                file.read_to_end(&mut shard)?;
                shards.push(shard.into_boxed_slice());
                shard_present[i] = true;
            }
            Err(_) => {
                eprintln!("Failed to read shard {} from disk. Assuming it is missing.", i);
                shards.push(Vec::new().into_boxed_slice());
            }
        }
    }

    // Zip the shards and shard_present vectors
    let mut shards_with_presence: Vec<(Box<[u8]>, bool)> = shards.into_iter().zip(shard_present.into_iter()).collect();

    // Restore the original data using Reed-Solomon decoding
    rs.reconstruct(&mut shards_with_presence)?;

    // Remove padding and return the original data
    let mut restored_data = Vec::new();
    for chunk in shards_with_presence[..data_shards].iter().flat_map(|(shard, _)| shard.iter()) {
        restored_data.push(*chunk);
        if restored_data.len() == original_data_len {
            break;
        }
    }

    Ok(restored_data)
}
