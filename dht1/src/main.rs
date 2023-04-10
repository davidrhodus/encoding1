use reed_solomon_erasure::galois_8;
use reed_solomon_erasure::ReedSolomon;
use serde_json;
use std::fs::File;
use std::io::Write;

fn main() {
    // Define the input data
    let data = b"Hello, world!";

    // Configure the Reed-Solomon encoder
    let data_shards = 8;
    let parity_shards = 4;
    let total_shards = data_shards + parity_shards;

    let rs = ReedSolomon::<galois_8::Field>::new(data_shards, parity_shards).unwrap();

    // Pad the input data to fit into the data shards
    let shard_size = (data.len() + data_shards - 1) / data_shards;
    let padded_data = {
        let mut tmp = Vec::with_capacity(shard_size * data_shards);
        tmp.extend_from_slice(data);
        tmp.resize(shard_size * data_shards, 0);
        tmp
    };

    // Create and populate the shards
    let mut shards: Vec<Vec<u8>> = vec![vec![0; shard_size]; total_shards];
    for (i, chunk) in padded_data.chunks(shard_size).enumerate() {
        shards[i].copy_from_slice(chunk);
    }

    // Encode the input data using Reed-Solomon
    rs.encode(&mut shards).unwrap();

    // Serialize the encoded data to JSON
    let serialized_data = serde_json::to_string(&shards).unwrap();

    // Save the serialized data to a file
    let mut file = File::create("encoded_data.json").unwrap();
    file.write_all(serialized_data.as_bytes()).unwrap();
    println!("Encoded data saved to disk.");
}
