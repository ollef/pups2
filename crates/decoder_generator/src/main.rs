use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter},
    ops::Range,
};

use yaml_rust2::{Yaml, YamlLoader};

#[derive(Debug)]
struct Encoding<T> {
    bits: u32,
    mask: u32,
    payload: T,
}

impl<T: Display> Display for Encoding<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for bit in (0..32).rev() {
            if bit == 5 || bit == 10 || bit == 15 || bit == 20 || bit == 25 {
                write!(f, " ")?;
            }
            let mask = (self.mask >> bit) & 1;
            let bits = (self.bits >> bit) & 1;
            if mask == 0 {
                write!(f, ".")?;
            } else if bits == 0 {
                write!(f, "0")?;
            } else {
                write!(f, "1")?;
            }
        }
        write!(f, ": {}", self.payload)?;
        Ok(())
    }
}

#[derive(Debug)]
enum DecisionTree<T> {
    Leaf(T),
    Match {
        bits: Range<u32>,
        nodes: BTreeMap<u32, DecisionTree<T>>,
    },
}

struct InstructionFormat<'a>(&'a str);

impl InstructionFormat<'_> {
    pub fn mnemonic(&self) -> &str {
        if let Some(eq_index) = self.0.find('=') {
            self.0[eq_index + 1..]
                .split_ascii_whitespace()
                .next()
                .unwrap()
        } else {
            self.0.split_ascii_whitespace().next().unwrap()
        }
    }

    pub fn operands(&self) -> impl Iterator<Item = &str> {
        self.0.match_indices("{").map(|(i, _)| {
            let end = self.0[i..].find('}').expect("Missing closing brace");
            let braced = &self.0[i + 1..i + end];
            if let Some(colon) = braced.find(':') {
                &braced[..colon]
            } else {
                braced
            }
        })
    }
}

impl<T: Clone + Display> DecisionTree<T> {
    pub fn new(encodings: &[Encoding<T>]) -> Self {
        if encodings.is_empty() {
            panic!("Cannot create a decision tree with no encodings");
        }
        if encodings.len() == 1 && encodings[0].mask == 0 {
            return DecisionTree::Leaf(encodings[0].payload.clone());
        }

        let mut discriminant_mask: u32 = !0;
        for encoding in encodings.iter() {
            discriminant_mask &= encoding.mask;
        }
        if discriminant_mask == 0 {
            println!("No discriminating bits in encodings:");
            for encoding in encodings.iter() {
                println!("{}", encoding);
            }
            panic!("No discriminating bits in encodings");
        }

        let range_start = discriminant_mask.trailing_zeros();
        let range_end = range_start + (discriminant_mask >> range_start).trailing_ones();
        let discriminant_mask =
            !((1 << range_start) - 1) & (1u32.checked_shl(range_end).unwrap_or(0)).wrapping_sub(1);

        println!("Discriminant mask: {:#034b}", discriminant_mask);
        println!("Range start: {}", range_start);
        println!("Range end: {}", range_end);

        let mut nodes: BTreeMap<u32, Vec<Encoding<T>>> = BTreeMap::new();
        for encoding in encodings.iter() {
            let bits = (encoding.bits & discriminant_mask) >> range_start;
            nodes.entry(bits).or_default().push(Encoding {
                bits: encoding.bits,
                mask: encoding.mask & !discriminant_mask,
                payload: encoding.payload.clone(),
            });
        }

        let nodes = nodes
            .into_iter()
            .map(|(bits, encodings)| {
                let node = DecisionTree::new(&encodings);
                (bits, node)
            })
            .collect();
        DecisionTree::Match {
            bits: range_start..range_end,
            nodes,
        }
    }
}

fn opcode_visitor_trait(encodings: &[Encoding<String>]) {
    println!("pub trait OpcodeVisitor {{");
    println!("    type Output;");
    let mut unknown_added = false;
    for encoding in encodings {
        let format = InstructionFormat(&encoding.payload);
        let mnemonic = format.mnemonic();
        if mnemonic == "unknown" {
            if unknown_added {
                continue;
            }
            unknown_added = true;
        }

        println!("    fn {mnemonic}(self, instruction: Instruction) -> Self::Output;");
    }
    println!("}}");
}

fn instruction_visitor_trait(operands: &Yaml, encodings: &[Encoding<String>]) {
    println!("pub trait InstructionVisitor {{");
    println!("    type Output;");
    let mut unknown_added = false;
    for encoding in encodings {
        let format = InstructionFormat(&encoding.payload);
        let mnemonic = format.mnemonic();
        if mnemonic == "unknown" {
            if unknown_added {
                continue;
            }
            unknown_added = true;
        }

        print!("    fn {mnemonic}(self");
        for operand in format.operands() {
            let operand_type = operands[operand]["type"].as_str().unwrap();
            print!(", {operand}: {operand_type}");
        }

        println!(") -> Self::Output;");
    }
    println!("}}");
}

fn main() {
    let arg = std::env::args().nth(1).unwrap();
    let string = std::fs::read_to_string(arg).unwrap();
    let yaml = YamlLoader::load_from_str(&string).unwrap();
    // println!("{:#?}", yaml);
    let raw_instructions = &yaml[0]["instructions"].as_hash().unwrap();
    let mut encodings = Vec::with_capacity(raw_instructions.len());
    for (encoding_str, format) in raw_instructions.iter() {
        println!("{:#?}", encoding_str);
        println!("{:#?}", format);
        println!(
            "mnemonic: {}",
            InstructionFormat(format.as_str().unwrap()).mnemonic()
        );
        let mut encoding = Encoding {
            bits: 0,
            mask: 0,
            payload: format.as_str().unwrap().to_string(),
        };
        for char in encoding_str.as_str().unwrap().chars() {
            match char {
                ' ' => continue,
                '0' => {
                    encoding.bits <<= 1;
                    encoding.mask <<= 1;
                    encoding.bits |= 0;
                    encoding.mask |= 1;
                }
                '1' => {
                    encoding.bits <<= 1;
                    encoding.mask <<= 1;
                    encoding.bits |= 1;
                    encoding.mask |= 1;
                }
                '.' => {
                    encoding.bits <<= 1;
                    encoding.mask <<= 1;
                    encoding.bits |= 0;
                    encoding.mask |= 0;
                }
                _ => panic!("Invalid character in encoding: {}", char),
            }
        }
        println!("{}", encoding);
        encodings.push(encoding);
    }
    let decision_tree = DecisionTree::new(&encodings);
    println!("{:#?}", decision_tree);
    opcode_visitor_trait(&encodings);
    instruction_visitor_trait(&yaml[0]["operands"], &encodings);
}
