use std::{
    collections::{BTreeMap, HashMap},
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

    pub fn constructor_name(&self) -> String {
        let lower = self.mnemonic().replace(".", "");
        let mut chars = lower.chars();
        chars.next().unwrap().to_uppercase().chain(chars).collect()
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

fn instruction_type(operands: &Yaml, encodings: &[Encoding<String>]) {
    println!("#[derive(Debug, PartialEq, Eq, Copy, Clone)]");
    println!("pub enum Instruction {{");
    let mut unknown_added = false;
    for encoding in encodings {
        let format = InstructionFormat(&encoding.payload);
        let constructor_name = format.constructor_name();
        if constructor_name == "Unknown" {
            if unknown_added {
                continue;
            }
            unknown_added = true;
        }

        print!("    {constructor_name}");
        let operand_count = format.operands().count();
        if operand_count > 0 {
            print!("(");
        }
        let mut comma = "";
        for operand in format.operands() {
            print!("{comma}{}", operands[operand]["type"].as_str().unwrap());
            comma = ", ";
        }
        if operand_count > 0 {
            print!(")");
        }
        println!(",");
    }
    println!("}}");
}

fn instruction_decoder(operands: &Yaml, decision_tree: &DecisionTree<String>) {
    fn go(indent: usize, decision_tree: &DecisionTree<String>) {
        match decision_tree {
            DecisionTree::Leaf(format) => {
                let format = InstructionFormat(format);
                print!("Instruction::{}", format.constructor_name());
                let operand_count = format.operands().count();
                if operand_count > 0 {
                    print!("(");
                }
                let mut comma = "";
                for operand in format.operands() {
                    print!("{comma}{operand}()");
                    comma = ", ";
                }
                if operand_count > 0 {
                    print!(")");
                }
                print!(",");
            }
            DecisionTree::Match { bits, nodes } => {
                println!("match data.bits({}..{}) {{", bits.start, bits.end,);
                let bits_len = (bits.end - bits.start) as usize;
                for (value, node) in nodes {
                    let value = format!("{value:032b}");
                    print!(
                        "{}0b{} => ",
                        "    ".repeat(indent + 1),
                        &value[value.len() - bits_len..]
                    );
                    go(indent + 1, node);
                    println!();
                }
                print!("{}_ => ", "    ".repeat(indent + 1),);
                if nodes.len() == 1 << bits_len {
                    println!("unreachable!(),");
                } else {
                    println!("panic!(\"Unhandled instruction: {{:#034b}}\", data),");
                }
                print!("{}}}", "    ".repeat(indent));
            }
        }
    }

    println!("impl Instruction {{");
    println!("    pub fn decode(data: u32) -> Self {{");
    for (operand_name, operand) in operands.as_hash().unwrap() {
        println!(
            "        let {} = || {};",
            operand_name.as_str().unwrap(),
            operand["decode"].as_str().unwrap().replace("{}", "data"),
        );
    }
    print!("        ");
    go(2, decision_tree);

    println!("\n    }}");
    println!("}}");
}

fn display_impl(encodings: &[Encoding<String>]) {
    println!("impl Display for Instruction {{");
    println!("    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {{");
    println!("        match self {{");
    let mut unknown_added = false;
    for encoding in encodings {
        let format = InstructionFormat(&encoding.payload);
        let constructor_name = format.constructor_name();
        if constructor_name == "Unknown" {
            if unknown_added {
                continue;
            }
            unknown_added = true;
        }
        print!("            Instruction::{constructor_name}");
        let operand_count = format.operands().count();
        if operand_count > 0 {
            print!("(");
        }
        let mut comma = "";
        for operand in format.operands() {
            print!("{comma}{operand}");
            comma = ", ";
        }
        if operand_count > 0 {
            print!(")");
        }
        println!(" => write!(f, \"{}\"),", format.0);
    }
    println!("        }}");
    println!("    }}");
    println!("}}");
}

fn predicates<'a>(instructions: impl IntoIterator<Item = (&'a Yaml, &'a Yaml)>) {
    let mut predicate_opcodes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, instruction) in instructions {
        if let Some(predicates) = instruction["predicates"].as_vec() {
            let format = InstructionFormat(instruction["format"].as_str().unwrap());
            for predicate in predicates {
                let predicate = predicate.as_str().unwrap();
                predicate_opcodes
                    .entry(predicate.to_string())
                    .or_default()
                    .push(format.constructor_name());
            }
        }
    }

    if predicate_opcodes.is_empty() {
        return;
    }
    println!("impl Instruction {{");
    let mut newline = false;
    for (predicate, opcodes) in predicate_opcodes {
        if newline {
            println!();
        }
        println!("    pub fn {predicate}(self) -> bool {{");
        print!("        matches!(self, ");
        let mut bar = "";
        for opcode in opcodes {
            print!("{bar}Instruction::{opcode}(..)");
            bar = " | ";
        }
        println!(")");
        println!("    }}");
        newline = true;
    }
    println!("}}");
}

fn definitions_and_uses<'a>(
    operands: &Yaml,
    instructions: impl IntoIterator<Item = (&'a Yaml, &'a Yaml)>,
    encodings: &[Encoding<String>],
) {
    let mut uses: HashMap<String, Vec<String>> = HashMap::new();
    let mut defs: HashMap<String, Vec<String>> = HashMap::new();
    for encoding in encodings {
        let format = InstructionFormat(&encoding.payload);
        let mnemonic = format.mnemonic();
        let eq_index = encoding.payload.find('=');
        if let Some(eq_index) = eq_index {
            let def_format = InstructionFormat(&encoding.payload[..eq_index]);
            for def in def_format.operands() {
                defs.entry(mnemonic.to_string())
                    .or_default()
                    .push(def.to_string());
            }
        }
        let use_format = InstructionFormat(&encoding.payload[eq_index.unwrap_or_default()..]);
        for use_ in use_format.operands() {
            let type_ = operands[use_]["type"].as_str().unwrap();
            if matches!(type_, "u8" | "u16" | "u32" | "i8" | "i16" | "i32") {
                continue;
            }
            uses.entry(mnemonic.to_string())
                .or_default()
                .push(use_.to_string());
        }
    }

    for (_, instruction) in instructions {
        if let Some(instruction_uses) = instruction["uses"].as_vec() {
            let format = InstructionFormat(instruction["format"].as_str().unwrap());
            let mnemonic = format.mnemonic();
            for use_ in instruction_uses {
                uses.entry(mnemonic.to_string())
                    .or_default()
                    .push(use_.as_str().unwrap().to_string());
            }
        }
        if let Some(instruction_defs) = instruction["defs"].as_vec() {
            let format = InstructionFormat(instruction["format"].as_str().unwrap());
            let mnemonic = format.mnemonic();
            for def in instruction_defs {
                defs.entry(mnemonic.to_string())
                    .or_default()
                    .push(def.as_str().unwrap().to_string());
            }
        }
    }

    let max_uses = uses.values().map(|v| v.len()).max().unwrap_or(0);
    let max_defs = defs.values().map(|v| v.len()).max().unwrap_or(0);

    println!("impl Instruction {{");
    println!("    pub fn raw_definitions(self) -> [Option<Occurrence>; {max_defs}] {{");
    println!("        match self {{");
    let mut unknown_added = false;
    for encoding in encodings {
        let format = InstructionFormat(&encoding.payload);
        let defs = defs.get(format.mnemonic()).cloned().unwrap_or_default();
        let constructor_name = format.constructor_name();
        if constructor_name == "Unknown" {
            if unknown_added {
                continue;
            }
            unknown_added = true;
        }
        print!("            Instruction::{constructor_name}");
        let operand_count = format.operands().count();
        if operand_count > 0 {
            print!("(");
        }
        let mut comma = "";
        for operand in format.operands() {
            if defs.contains(&operand.to_string()) {
                print!("{comma}{operand}");
            } else {
                print!("{comma}_");
            }
            comma = ", ";
        }
        if operand_count > 0 {
            print!(")");
        }
        print!(" => [");
        let mut comma = "";
        for def in &defs {
            print!("{comma}Some(Occurrence::from({def}))");
            comma = ", ";
        }
        for _ in defs.len()..max_defs {
            print!("{comma}None");
            comma = ", ";
        }
        println!("],");
    }
    println!("        }}");
    println!("    }}");
    println!();
    println!("    pub fn raw_uses(self) -> [Option<Occurrence>; {max_uses}] {{");
    println!("        match self {{");
    let mut unknown_added = false;
    for encoding in encodings {
        let format = InstructionFormat(&encoding.payload);
        let uses = uses.get(format.mnemonic()).cloned().unwrap_or_default();
        let constructor_name = format.constructor_name();
        if constructor_name == "Unknown" {
            if unknown_added {
                continue;
            }
            unknown_added = true;
        }
        print!("            Instruction::{constructor_name}");
        let operand_count = format.operands().count();
        if operand_count > 0 {
            print!("(");
        }
        let mut comma = "";
        for operand in format.operands() {
            if uses.contains(&operand.to_string()) {
                print!("{comma}{operand}");
            } else {
                print!("{comma}_");
            }
            comma = ", ";
        }
        if operand_count > 0 {
            print!(")");
        }
        print!(" => [");
        let mut comma = "";
        for use_ in &uses {
            print!("{comma}Some(Occurrence::from({use_}))");
            comma = ", ";
        }
        for _ in uses.len()..max_uses {
            print!("{comma}None");
            comma = ", ";
        }
        println!("],");
    }
    println!("        }}");
    println!("    }}");
    println!("}}");
}

fn main() {
    let arg = std::env::args().nth(1).unwrap();
    let string = std::fs::read_to_string(arg).unwrap();
    let yaml = YamlLoader::load_from_str(&string).unwrap();
    let raw_instructions = yaml[0]["instructions"].as_hash().unwrap();
    let mut encodings = Vec::with_capacity(raw_instructions.len());
    for (encoding_str, format) in raw_instructions.iter() {
        let format = format
            .as_str()
            .unwrap_or_else(|| format["format"].as_str().unwrap());
        let mut encoding = Encoding {
            bits: 0,
            mask: 0,
            payload: format.to_string(),
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
        encodings.push(encoding);
    }
    let decision_tree = DecisionTree::new(&encodings);

    println!("{}", &yaml[0]["imports"].as_str().unwrap());
    println!();
    instruction_type(&yaml[0]["operands"], &encodings);
    println!();
    instruction_decoder(&yaml[0]["operands"], &decision_tree);
    println!();
    display_impl(&encodings);
    println!();
    predicates(raw_instructions);
    println!();
    definitions_and_uses(&yaml[0]["operands"], raw_instructions, &encodings);
}
