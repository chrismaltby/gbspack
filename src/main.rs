use clap::{value_t, values_t, App, Arg};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
struct Bank {
  objects: Vec<ObjectData>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
struct ObjectData {
  size: u32,
  original_bank: u32,
  filename: String,
  stem: String,
  contents: String,
}

const BANK_SIZE: u32 = 16384;

fn main() -> std::io::Result<()> {
  let matches = App::new("GBStudio Pack")
    .version("1.0.0")
    .author("Chris Maltby. <chris.maltby@gmail.com>")
    .about("Packs object files created by GB Studio data into banks")
    .arg(
      Arg::with_name("offset")
        .short("b")
        .long("bank")
        .value_name("NN")
        .help("Sets the first bank to use (default 6)")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("output_path")
        .short("o")
        .long("output")
        .help("Set the output path (default updates in-place")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("ext")
        .short("e")
        .long("ext")
        .help("Replace the file extension for output files")
        .takes_value(true),
    )    
    .arg(
      Arg::with_name("INPUT")
        .help("Sets the input .o files to use")
        .required(true)
        .multiple(true)
        .index(1),
    )
    .arg(
      Arg::with_name("verbose")
        .short("v")
        .help("Sets the level of verbosity"),
    )
    .get_matches();

  let verbose = matches.is_present("verbose");
  let bank_offset = value_t!(matches.value_of("offset"), u32).unwrap_or(6);
  let input_files = values_t!(matches.values_of("INPUT"), String).unwrap();
  let output_path = value_t!(matches.value_of("output_path"), String).unwrap_or(("").to_string());
  let ext = value_t!(matches.value_of("ext"), String).unwrap_or(("o").to_string());

  if verbose {
    println!("Starting at bank={}", bank_offset);
    println!("Processing {} files", input_files.len());
    println!("Using extension .{}", ext);
    if output_path.len() > 0 {
      println!("Output path={}", output_path);
    }
  }

  // Convert input files to Vec<ObjectData>
  let mut objects = Vec::new();
  for filename in input_files {
    if verbose {
      println!("Processing file: {}", filename);
    }
    let object = to_object_data(&filename)?;
    if verbose {
      println!("Size was: {}", object.size);
    }    
    objects.push(object);
  }

  // Pack object data into banks
  let packed = pack_object_data(objects);

  // Write packed files back to disk
  let mut bank_no = bank_offset;
  for bin in packed {
    if verbose {
      println!("Bank={}", bank_no);
    }
    for object in bin.objects.iter() {
      let output_filename = if output_path.len() > 0 {
        // Store output in dir specified by output_path
        let path = Path::new(&output_path);
        let new_path = path.join(format!("{}.{}", object.stem, ext));
        new_path.to_str().unwrap().to_owned()
      } else {
        // Replace object file in-place
        let original_path = Path::new(&object.filename);
        let new_path = original_path.parent().unwrap().join(format!("{}.{}", object.stem, ext));
        new_path.to_str().unwrap().to_owned()
      };

      if verbose {
        println!("Writing file {}", output_filename);
      }
      let mut file = File::create(output_filename)?;
      let new_contents = set_bank(&object.contents, &object.stem, object.original_bank, bank_no);

      match file.write_all(new_contents.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", object.filename, why),
        Ok(_) => {}
      }
    }
    bank_no += 1;
  }

  if verbose {
    println!("Done");
  }

  Ok(())
}

/// Read an object file into a struct containing the information required
/// to pack the data into banks
fn to_object_data(filename: &String) -> std::io::Result<ObjectData> {
  let path = Path::new(filename);
  let stem = path.file_stem().unwrap().to_str().unwrap();
  let mut file = File::open(path)?;
  let mut contents = String::new();
  let mut size: u32 = 0;
  let mut original_bank: u32 = 0;

  file.read_to_string(&mut contents)?;

  for line in contents.lines() {
    if line.contains("A _CODE_") {
      let (parsed_size, parsed_bank) = parse_size(line.to_string());
      size = parsed_size;
      original_bank = parsed_bank;
      break;
    }
  }

  if size == 0 {
    panic!("Data size couldn't be calculated. Is initial bank set to CODE_255?")
  }

  Ok(ObjectData {
    filename: filename.to_string(),
    stem: stem.to_string(),
    contents: contents.to_string(),
    size,
    original_bank,
  })
}

/// Parse the size line from an object file to get the size as an integer
fn parse_size(line: String) -> (u32, u32) {
  let split = line.split(" ").collect::<Vec<&str>>();
  let bank_split = split[1].split("_").collect::<Vec<&str>>();

  let size = u32::from_str_radix(split[3], 16).unwrap();
  let bank = bank_split[2].parse::<u32>().unwrap();
  (size, bank)
}

/// Update an object file's contents replacing the bank references with
/// the specified bank number
fn set_bank(object_string: &String, stem: &String, original_bank: u32, bank_no: u32) -> String {
  let find_code = format!("CODE_{}", original_bank);
  let replace_code = format!("CODE_{}", bank_no);
  let find_def = format!("__bank_{} Def{:06X}", stem, original_bank);
  let replace_def = format!("__bank_{} Def{:06X}", stem, bank_no);
  object_string
    .replace(&find_code, &replace_code)
    .replace(&find_def, &replace_def)
}

/// Pack an vector of object data into a vector of banks
/// using a first fit algorithm after sorting the input data
/// by descending size
fn pack_object_data(mut objects: Vec<ObjectData>) -> Vec<Bank> {
  let mut banks = Vec::new();
  banks.push(Bank { objects: vec![] });

  // Sort objects by descending size
  objects.sort_by(|a, b| a.cmp(&b));

  if BANK_SIZE < objects.iter().max().unwrap().size {
    panic!("Object file too large to fit in bank.");
  }

  while !objects.is_empty() {
    let mut stored = false;
    let object = objects.pop().unwrap();

    // Find first fit in existing banks
    for bank in &mut banks {
      // Calculate current size of bank
      let res: u32 = bank.objects.iter().fold(0, |a, b| a + b.size);

      // If can fit store it here
      if (res + object.size) <= BANK_SIZE {
        bank.objects.push(object.clone());
        stored = true;
        break;
      }
    }

    // No room in existing banks, create a new bank
    if !stored {
      let mut new_bank = Bank { objects: vec![] };
      new_bank.objects.push(object.clone());
      banks.push(new_bank);
    }
  }

  banks
}
