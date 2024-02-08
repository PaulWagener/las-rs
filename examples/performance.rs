//! Calculates the minimum X position

extern crate las;

use las::{raw, Point, Read, Reader};
use laz::LazVlr;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::time::Instant;

/// This program shows the performance difference in LAZ files between the current Reader
/// and an alternative reader using the parallel feature of `laz-rs`. Both programs calculate
/// the maximum X value from the point data.
fn main() {
    let path = std::env::args()
        .skip(1)
        .next()
        .expect("Must provide a path to a las file");

    const RUNS: i32 = 5;

    println!("Original reader");
    for run in 1..=RUNS {
        let start = Instant::now();

        let mut reader = Reader::from_path(&path).unwrap();
        let max_x = reader
            .points()
            .map(|p| p.unwrap().x)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        if run == 1 {
            println!("max_x: {max_x}");
        }

        println!("Run {run}/{RUNS}: {:.2?}", start.elapsed());
    }

    println!();

    // Calculate
    println!("LAZ parallel reader");
    for run in 1..=RUNS {
        let start = Instant::now();

        // Read header using the original API
        let header = Reader::from_path(&path).unwrap().header().clone();

        // Create a byte reader directly from the file
        let mut reader = BufReader::new(File::open(&path).unwrap());
        reader
            .seek(SeekFrom::Start(
                header.clone().into_raw().unwrap().offset_to_point_data as u64,
            ))
            .unwrap();

        // Decompress the whole file into a big buffer
        let mut buffer =
            vec![0_u8; header.point_format().len() as usize * header.number_of_points() as usize];

        let lazvlr = header
            .vlrs()
            .iter()
            .find(|vlr| vlr.user_id == LazVlr::USER_ID && vlr.record_id == LazVlr::RECORD_ID)
            .unwrap();

        laz::ParLasZipDecompressor::new(reader, LazVlr::from_buffer(&lazvlr.data).unwrap())
            .unwrap()
            .decompress_many(&mut buffer)
            .unwrap();

        // Decode points and find max X value
        let max_x = buffer
            .chunks_exact(header.point_format().len() as usize)
            .map(|point| {
                let point = Point::new(
                    raw::point::Point::read_from(point, header.point_format()).unwrap(),
                    header.transforms(),
                );
                point.x
            })
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();

        if run == 1 {
            println!("max_x: {}", max_x);
        }
        println!("Run {run}/{RUNS}: {:.2?}", start.elapsed());
    }
}
