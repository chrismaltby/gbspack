use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Bank {
    pub objects: Vec<(usize, ObjectBankData)>,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub struct ObjectBankData {
    pub size: u32,
    pub bank: u32,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct ObjectData {
    pub banks: Vec<ObjectBankData>,
    pub filename: String,
    pub contents: String,
}

#[derive(Debug)]
pub struct BankReplacement {
    pub from: u32,
    pub to: u32,
}

#[derive(Debug)]
pub struct ObjectPatch {
    pub filename: String,
    pub contents: String,
    pub replacements: Vec<BankReplacement>,
}

const BANK_SIZE: u32 = 16384;

/// Read an object file into a struct containing the information required
/// to pack the data into banks
pub fn to_object_data(filename: &String) -> std::io::Result<ObjectData> {
    let path = Path::new(filename);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let banks = parse_sizes(&contents);

    Ok(ObjectData {
        filename: filename.to_string(),
        contents: contents.to_string(),
        banks,
    })
}

pub fn parse_sizes(contents: &String) -> Vec<ObjectBankData> {
    let mut banks = Vec::new();
    for line in contents.lines() {
        if line.contains("A _CODE_") {
            let parsed_size = parse_size(&line.to_string());
            banks.push(parsed_size);
        }
    }
    banks
}

/// Parse the size line from an object file to get the size as an integer
pub fn parse_size(line: &String) -> ObjectBankData {
    let split = line.split(" ").collect::<Vec<&str>>();
    let bank_split = split[1].split("_").collect::<Vec<&str>>();
    let size = u32::from_str_radix(split[3], 16).unwrap();
    let bank = bank_split[2].parse::<u32>().unwrap();
    ObjectBankData { size, bank }
}

/// Update an object file's contents replacing the bank references with
/// the specified bank number
pub fn replace_bank(object_string: &String, original_bank: u32, bank_no: u32) -> String {
    let mut new_string = object_string.clone();
    // Find banked functions
    for line in object_string.lines() {
        if line.starts_with("S b_") {
            let split = line[4..].split(" ").collect::<Vec<&str>>();
            let fn_name = split[0];
            let fn_def = format!("S _{}", fn_name);
            // If symbol has pair
            if new_string.contains(&fn_def) {
                let find_banked_fn_def = format!("b_{} Def{:06X}", fn_name, original_bank);
                let replace_banked_fn_def = format!("b_{} Def{:06X}", fn_name, bank_no);
                new_string = new_string.replace(&find_banked_fn_def, &replace_banked_fn_def);
            }
        }
    }
    let find_code = format!("CODE_{}", original_bank);
    let replace_code = format!("CODE_{}", bank_no);
    let replaced_string = new_string.replace(&find_code, &replace_code);
    let re = Regex::new(&format!("__bank_(?P<s>[^ ]*) Def{:06X}", original_bank)).unwrap();
    let result = re.replace_all(&replaced_string, "__bank_$s Def00000F");
    result.to_string()
}

/// Pack an vector of object data into a vector of banks
/// using a first fit algorithm after sorting the input data
/// by descending size
pub fn pack_object_data(
    objects: Vec<ObjectData>,
    filter: u32,
    bank_offset: u32,
    mbc1: bool,
) -> Vec<ObjectPatch> {
    let mut banks = Vec::new();

    let mut areas: Vec<(usize, ObjectBankData)> = objects
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, x)| x.banks.into_iter().map(move |y| (i, y)))
        .flatten()
        .collect();

    // Sort objects by descending size
    areas.sort_by(|a, b| b.1.size.cmp(&a.1.size));

    let max_size = areas.iter().map(|a| a.1.size).max().unwrap();

    if BANK_SIZE < max_size {
        panic!("Object file too large to fit in bank.");
    }

    // Add the extra banks first
    let arr = vec![Bank { objects: vec![] }; bank_offset as usize];
    banks.extend_from_slice(&arr);

    // Pack fixed areas
    if filter != 0 {
        for area in areas.iter() {
            if area.1.bank != filter {
                let size_diff: i32 = (area.1.bank as i32) - (banks.len() as i32);
                if size_diff > 0 {
                    // Add the extra banks first
                    let arr = vec![Bank { objects: vec![] }; size_diff as usize];
                    banks.extend_from_slice(&arr);
                }
                banks[(area.1.bank - 1) as usize].objects.push(area.clone());
            }
        }
    }

    // Check fixed areas are within max size
    let max_fixed_area_size = banks
        .iter()
        .map(|b| b.objects.iter().fold(0, |a, b| a + b.1.size))
        .max()
        .unwrap_or(0);

    if BANK_SIZE < max_fixed_area_size {
        panic!("Bank overflow");
    }

    // Pack unfixed areas
    for area in areas.iter() {
        if filter == 0 || area.1.bank == filter {
            let mut stored = false;

            // Find first fit in existing banks
            let mut bank_no = 0;
            for bank in &mut banks {
                bank_no += 1;

                // Skip until at bank_offset
                if bank_no < bank_offset {
                    continue;
                }

                // Calculate current size of bank
                let res: u32 = bank.objects.iter().fold(0, |a, b| a + b.1.size);

                // If can fit store it here
                if (res + area.1.size) <= BANK_SIZE {
                    bank.objects.push(area.clone());
                    stored = true;
                    break;
                }
            }
            // No room in existing banks, create a new bank
            if !stored {
                let mut new_bank = Bank { objects: vec![] };
                new_bank.objects.push(area.clone());
                banks.push(new_bank);
            }
        }
    }

    // Convert packed data into object patch
    let patch = objects
        .into_iter()
        .enumerate()
        .map(|(i, x)| ObjectPatch {
            filename: x.filename,
            contents: x.contents,
            replacements: get_bank_replacements(i, &banks, mbc1),
        })
        .collect();

    patch
}

fn get_bank_replacements(index: usize, packed: &Vec<Bank>, mbc1: bool) -> Vec<BankReplacement> {
    let mut replacements: Vec<BankReplacement> = vec![];

    // Write packed files back to disk
    let mut bank_no = 1;
    for bin in packed {
        for object in bin.objects.iter() {
            if mbc1 {
                if bank_no == 0x20 || bank_no == 0x40 || bank_no == 0x60 {
                    bank_no += 1;
                }
            }
            if object.0 == index {
                replacements.push(BankReplacement {
                    from: object.1.bank,
                    to: bank_no
                })
            }
        }
        bank_no += 1;
    }

    replacements
}