use dispel_core::references::save_file::SaveFile;
// use dispel_core::Extractor;
use std::{env, fs};

fn check(path: &str) {
    let original = fs::read(path).unwrap();
    println!("File: {}, size: {}", path, original.len());

    let save = SaveFile::parse(&original).unwrap();
    println!("  Journal entries: {}", save.journal.main.len());

    // let mut output = Vec::new();
    // SaveFile::to_writer(&[save], &mut output).unwrap();
    //
    // if original.len() == output.len() {
    //     let mismatches: Vec<usize> = (0..original.len())
    //         .filter(|&i| original[i] != output[i])
    //         .collect();
    //     if mismatches.is_empty() {
    //         println!("  ✓ ALL {} BYTES MATCH!", original.len());
    //     } else {
    //         println!("  ✗ {} mismatches (showing first 10):", mismatches.len());
    //         for &i in mismatches.iter().take(10) {
    //             println!(
    //                 "    byte {}: orig={:02x} out={:02x} (expected {})",
    //                 i,
    //                 original[i],
    //                 output[i],
    //                 if output[i] == original[i] {
    //                     "MATCH"
    //                 } else {
    //                     "DIFF"
    //                 }
    //             );
    //         }
    //     }
    // } else {
    //     println!(
    //         "  ✗ Size mismatch: original={} output={}",
    //         original.len(),
    //         output.len()
    //     );
    // }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        for path in &args[1..] {
            check(path);
            println!();
        }
    } else {
        check("nuno-0.sav");
        // check("0.sav");
        // check("2.sav");
    }
}
