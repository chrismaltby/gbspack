extern crate gbspack;

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_parse_area_size() {
    let input = "A _CODE_3 size 8 flags 0 addr 0".to_owned();
    let expected_output = gbspack::ObjectBankData { size: 8, bank: 3 };
    assert_eq!(gbspack::parse_size(&input), expected_output);
  }

  #[test]
  fn test_parse_area_size_hex() {
    let input = "A _CODE_15 size ff flags 0 addr 0".to_owned();
    let expected_output = gbspack::ObjectBankData {
      size: 255,
      bank: 15,
    };
    assert_eq!(gbspack::parse_size(&input), expected_output);
  }

  #[test]
  fn test_parse_areas() {
    let input = "XL3
H 2 areas 5 global symbols
S b_wait_frames Ref000000
S .__.ABS. Def000000
S _wait_frames Ref000000
S ___bank_SCRIPT_3 Def0000FF
A _CODE size 0 flags 0 addr 0
A _CODE_5 size 5 flags 0 addr 0
A _CODE_255 size 55 flags 0 addr 0
S _SCRIPT_3 Def000000"
      .to_owned();
    let expected_output = vec![
      gbspack::ObjectBankData { size: 5, bank: 5 },
      gbspack::ObjectBankData {
        size: 85,
        bank: 255,
      },
    ];
    let output = gbspack::parse_sizes(&input);
    assert_eq!(output.len(), 2);
    assert_eq!(output, expected_output);
  }

  #[test]
  fn test_pack_areas() {
    let input = vec![
      gbspack::ObjectData {
        filename: "a.o".to_string(),
        contents: "hello world".to_string(),
        banks: vec![
          gbspack::ObjectBankData { size: 5, bank: 1 },
          gbspack::ObjectBankData {
            size: 16380,
            bank: 255,
          },
        ],
      },
      gbspack::ObjectData {
        filename: "b.o".to_string(),
        contents: "second file".to_string(),
        banks: vec![
          gbspack::ObjectBankData { size: 15, bank: 2 },
          gbspack::ObjectBankData {
            size: 500,
            bank: 255,
          },
          gbspack::ObjectBankData {
            size: 40,
            bank: 255,
          },
        ],
      },
    ];
    let output = gbspack::pack_object_data(input, 255, 0, true);
    assert_eq!(output[0].filename, "a.o");
    assert_eq!(output[1].filename, "b.o");
    assert_eq!(output[0].replacements[0].from, 1);
    assert_eq!(output[0].replacements[0].to, 1);
    assert_eq!(output[0].replacements[1].from, 255);
    assert_eq!(output[0].replacements[1].to, 3);
    assert_eq!(output[1].replacements[0].from, 255);
    assert_eq!(output[1].replacements[0].to, 1);
    assert_eq!(output[1].replacements[1].from, 255);
    assert_eq!( output[1].replacements[1].to, 1);
    assert_eq!(output[1].replacements[2].from, 2);
    assert_eq!(output[1].replacements[2].to, 2);
  }


  #[test]
  fn test_pack_areas_mbc1() {
    let input = vec![
      gbspack::ObjectData {
        filename: "a.o".to_string(),
        contents: "hello world".to_string(),
        banks: vec![
          gbspack::ObjectBankData { size: 5, bank: 1 },
          gbspack::ObjectBankData {
            size: 16380,
            bank: 255,
          },
        ],
      },
      gbspack::ObjectData {
        filename: "b.o".to_string(),
        contents: "second file".to_string(),
        banks: vec![
          gbspack::ObjectBankData { size: 15, bank: 2 },
          gbspack::ObjectBankData {
            size: 16380,
            bank: 255,
          },
          gbspack::ObjectBankData {
            size: 16380,
            bank: 255,
          },
        ],
      },
    ];
    let output = gbspack::pack_object_data(input, 255, 31, true);
    assert_eq!(output[0].filename, "a.o");
    assert_eq!(output[1].filename, "b.o");
    assert_eq!(output[0].replacements[0].from, 1);
    assert_eq!(output[0].replacements[0].to, 1);
    assert_eq!(output[0].replacements[1].from, 255);
    assert_eq!(output[0].replacements[1].to, 31);
    assert_eq!(output[1].replacements[0].from, 2);
    assert_eq!(output[1].replacements[0].to, 2);
    assert_eq!(output[1].replacements[1].from, 255);
    assert_eq!( output[1].replacements[1].to, 33);
    assert_eq!(output[1].replacements[2].from, 255);
    assert_eq!(output[1].replacements[2].to, 34);
  }


  #[test]
  fn test_replace_one_bank() {
    let input = "XL3
H 2 areas 5 global symbols
S b_wait_frames Ref000000
S .__.ABS. Def000000
S _wait_frames Ref000000
S ___bank_SCRIPT_3 Def0000FF
A _CODE size 0 flags 0 addr 0
A _CODE_5 size 5 flags 0 addr 0
A _CODE_255 size 55 flags 0 addr 0
S _SCRIPT_3 Def000000"
      .to_owned();

    let expected_output = "XL3
H 2 areas 5 global symbols
S b_wait_frames Ref000000
S .__.ABS. Def000000
S _wait_frames Ref000000
S ___bank_SCRIPT_3 Def00000F
A _CODE size 0 flags 0 addr 0
A _CODE_5 size 5 flags 0 addr 0
A _CODE_15 size 55 flags 0 addr 0
S _SCRIPT_3 Def000000"
      .to_owned();

    assert_eq!(gbspack::replace_bank(&input, 255, 15), expected_output);
  }

  #[test]
  fn test_replace_multiple_banks() {
    let input = "XL3
H 2 areas 5 global symbols
S b_wait_frames Ref000000
S .__.ABS. Def000000
S _wait_frames Ref000000
S ___bank_SCRIPT_3 Def0000FF
A _CODE size 0 flags 0 addr 0
A _CODE_5 size 5 flags 0 addr 0
A _CODE_255 size 55 flags 0 addr 0
S _SCRIPT_3 Def000000"
      .to_owned();

    let expected_output = "XL3
H 2 areas 5 global symbols
S b_wait_frames Ref000000
S .__.ABS. Def000000
S _wait_frames Ref000000
S ___bank_SCRIPT_3 Def00000F
A _CODE size 0 flags 0 addr 0
A _CODE_14 size 5 flags 0 addr 0
A _CODE_15 size 55 flags 0 addr 0
S _SCRIPT_3 Def000000"
      .to_owned();

    assert_eq!(
      gbspack::replace_bank(&gbspack::replace_bank(&input, 5, 14), 255, 15),
      expected_output
    );
  }

  #[test]
  fn test_replace_all_banks() {
    let input = "XL3
H 2 areas 5 global symbols
S b_wait_frames Ref000000
S .__.ABS. Def000000
S _wait_frames Ref000000
S ___bank_SCRIPT_3 Def0000FF
S ___bank_SCRIPT_4 Def000005
A _CODE size 0 flags 0 addr 0
A _CODE_5 size 5 flags 0 addr 0
A _CODE_255 size 55 flags 0 addr 0
S _SCRIPT_3 Def000000"
      .to_owned();

    let expected_output = "XL3
H 2 areas 5 global symbols
S b_wait_frames Ref000000
S .__.ABS. Def000000
S _wait_frames Ref000000
S ___bank_SCRIPT_3 Def000007
S ___bank_SCRIPT_4 Def000006
A _CODE size 0 flags 0 addr 0
A _CODE_6 size 5 flags 0 addr 0
A _CODE_7 size 55 flags 0 addr 0
S _SCRIPT_3 Def000000"
      .to_owned();

    let patch = vec![
      gbspack::BankReplacement { from: 5, to: 6 },
      gbspack::BankReplacement { from: 255, to: 7 },
    ];

    assert_eq!(gbspack::replace_all_banks(&input, patch), expected_output);
  }

  #[test]
  fn test_cart_size() {
    assert_eq!(gbspack::to_cart_size(5), 8);
    assert_eq!(gbspack::to_cart_size(6), 8);
    assert_eq!(gbspack::to_cart_size(7), 8);
    assert_eq!(gbspack::to_cart_size(8), 16);
    assert_eq!(gbspack::to_cart_size(31), 32);
    assert_eq!(gbspack::to_cart_size(32), 64);
    assert_eq!(gbspack::to_cart_size(33), 64);
  }

  #[test]
  fn test_output_filename() {
    assert_eq!(gbspack::to_output_filename("/a/b/c.o", "", "o"), "/a/b/c.o");
    assert_eq!(gbspack::to_output_filename("/a/b/c.o", "", "rel"), "/a/b/c.rel");
    assert_eq!(gbspack::to_output_filename("/a/b/c.o", "/d/e", "o"), "/d/e/c.o");
    assert_eq!(gbspack::to_output_filename("/a/b/c.o", "/d/e", "rel"), "/d/e/c.rel");

  }

}
