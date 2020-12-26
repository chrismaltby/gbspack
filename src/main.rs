use clap::{value_t, values_t, App, Arg};
use std::fs::File;
use std::io::prelude::*;
use gbspacklib;

fn main() -> std::io::Result<()> {
  let matches = App::new("GBStudio Pack")
    .version("1.2.5")
    .author("Chris Maltby. <chris.maltby@gmail.com>")
    .about("Packs object files created by GB Studio data into banks")
    .arg(
      Arg::with_name("offset")
        .short("b")
        .long("bank")
        .value_name("NN")
        .help("Sets the first bank to use (default 1)")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("filter")
      .short("f")
      .long("filter")
      .value_name("NN")
      .help("Only repack files from specified bank (default repack all banks)")
      .takes_value(true),
    )
    .arg(
      Arg::with_name("mbc1")
        .long("mbc1")
        .help("Use MBC1 hardware (skip banks 0x20, 0x40 and 0x60)"),
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
      Arg::with_name("print_max")
        .short("p")
        .long("print-max")
        .help("Output the max bank number used"),
    )
    .arg(
      Arg::with_name("print_cart")
        .short("c")
        .long("print-cart")
        .help("Output the minimum cartridge size required"),
    )
    .arg(
      Arg::with_name("verbose")
        .short("v")
        .help("Sets the level of verbosity"),
    )
    .get_matches();

  let verbose = matches.is_present("verbose");
  let print_max = matches.is_present("print_max");
  let print_cart = matches.is_present("print_cart");
  let mbc1 = matches.is_present("mbc1");
  let bank_offset = value_t!(matches.value_of("offset"), u32).unwrap_or(1);
  let input_files = values_t!(matches.values_of("INPUT"), String).unwrap();
  let output_path = value_t!(matches.value_of("output_path"), String).unwrap_or(("").to_string());
  let ext = value_t!(matches.value_of("ext"), String).unwrap_or(("o").to_string());
  let filter = value_t!(matches.value_of("filter"), u32).unwrap_or(0);

  if verbose {
    println!("Starting at bank={}", bank_offset);
    println!("Processing {} files", input_files.len());
    println!("Using extension .{}", ext);
    if output_path.len() > 0 {
      println!("Output path={}", output_path);
    }
    if mbc1 {
      println!("Using MBC1 hardware");
    }
  }

  // Convert input files to Vec<ObjectData>
  let mut objects = Vec::new();
  for filename in input_files {
    if verbose {
      println!("Processing file: {}", filename);
    }
    let object = gbspacklib::to_object_data(&filename)?;
    objects.push(object);
  }

  // Pack object data into banks
  let packed = gbspacklib::pack_object_data(objects, filter, bank_offset, mbc1);

  let max_bank_no = gbspacklib::get_patch_max_bank(&packed);

  for patch in packed {
    let output_filename = gbspacklib::to_output_filename(&patch.filename, &output_path, &ext);
    if verbose {
      println!("Writing file {}", output_filename);
    }
    let new_contents = gbspacklib::replace_all_banks(&patch.contents, patch.replacements);
    let mut file = File::create(output_filename.clone())?;
    match file.write_all(new_contents.as_bytes()) {
      Err(why) => panic!("couldn't write to {}: {}", output_filename, why),
      Ok(_) => {}
    }    
  }

  if verbose {
    println!("Done");
  }

  if print_cart {
    println!("{}", gbspacklib::to_cart_size(max_bank_no));
  } else if print_max {
    println!("{}", max_bank_no);
  }

  Ok(())
}
