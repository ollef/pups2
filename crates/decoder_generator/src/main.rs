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

fn instruction_type(operands: &Yaml, encodings: &[Encoding<String>]) {
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
        let mut comma = "";
        if format.operands().next().is_some() {
            print!("(");
        }
        for operand in format.operands() {
            let operand_type = operands[operand]["type"].as_str().unwrap();
            print!("{comma}{operand_type}");
            comma = ", ";
        }
        if format.operands().next().is_some() {
            print!(")");
        }
        println!(",");
    }
    println!("}}");
}

#[derive(Debug, PartialEq, Eq)]
enum Opcode {
    Sll,
    Srl,
}

struct Instruction {
    opcode: Opcode,
    raw: u32,
}

trait Operands<const OPCODE: usize> {
    type Output;
    fn operands(self) -> Self::Output;
}

impl Operands<{ Opcode::Sll as _ }> for Instruction {
    type Output = (u16, u16);

    #[inline(always)]
    fn operands(self) -> Self::Output {
        assert!(self.opcode == Opcode::Sll);
        ((self.raw & 0xffff) as u16, (self.raw >> 16) as u16)
    }
}

impl Operands<{ Opcode::Srl as _ }> for Instruction {
    type Output = u8;

    #[inline(always)]
    fn operands(self) -> Self::Output {
        assert!(self.opcode == Opcode::Srl);
        (self.raw & 0xffff) as u8
    }
}

fn lol() {
    let instr = Instruction {
        opcode: Opcode::Sll,
        raw: 0,
    };
    let operands = <Instruction as Operands<{ Opcode::Srl as _ }>>::operands(instr);
}

// macro_rules! opcode {
//     ($variant:pat, $($args:pat), *) =>
//     {
//       instruction @ Instruction { opcode: $variant, .. } if let ($($args),*) = <Instruction as Operands<{ $variant as _ }>>::operands(instr)
//     }
// }

macro_rules! operands {
    (Sll, $instruction:expr) => {
        ($instruction.raw as u16, ($instruction.raw >> 16) as u16)
    };
    (Srl, $instruction:expr) => {
        $instruction.raw as u8
    };
}

macro_rules! match_instruction {
    ($scrutinee:expr,
        $(($variant:ident, $operands:pat) => $body:block ) *
    ) => {
        match $scrutinee {
            $(Instruction { opcode: Opcode::$variant, .. } => {
                let $operands = operands!($variant, $scrutinee);
                $body
            })*
        }
    };
}

fn lol2() {
    let instr = Instruction {
        opcode: Opcode::Sll,
        raw: 0,
    };
    let a = match_instruction!(instr,
        (Sll, (a, b)) => { 1234
        }
        (Srl, a) => {
            let b = a;
            println!("Srl: {a}");
            123
        }
    );
    println!("{a:?}");
    // match instr {
    //     opcode!(Opcode::Sll, a, b) => {
    //         let a = a;
    //         println!("Sll: {a}, {b}");
    //     }
    //     opcode!(Opcode::Srl, a, b) => {
    //         println!("Sll: {a}, {b}");
    //     }
    // }
}

fn decoder(operands: &Yaml, decision_tree: &DecisionTree<String>) {
    fn go(indent: usize, decision_tree: &DecisionTree<String>) {
        match decision_tree {
            DecisionTree::Leaf(format) => {
                let format = InstructionFormat(format);
                print!("Instruction::{}", format.constructor_name());
                let mut comma = "";
                if format.operands().next().is_some() {
                    print!("(");
                }
                for operand in format.operands() {
                    print!("{comma}{operand}()");
                    comma = ", ";
                }
                if format.operands().next().is_some() {
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

    println!("pub fn decode(data: u32) -> Instruction {{");
    for (operand, fields) in operands.as_hash().unwrap() {
        let operand = operand.as_str().unwrap();
        let decode = fields["decode"].as_str().unwrap();
        let decode = decode.replace("{}", "data");
        println!("    let {operand} = || {decode};");
    }

    print!("    ");
    go(1, decision_tree);

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
        let format = format
            .as_str()
            .unwrap_or_else(|| format["format"].as_str().unwrap());
        println!("{:#?}", format);
        println!("mnemonic: {}", InstructionFormat(format).mnemonic());
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
        println!("{}", encoding);
        encodings.push(encoding);
    }
    let decision_tree = DecisionTree::new(&encodings);
    println!("{:#?}", decision_tree);
    instruction_type(&yaml[0]["operands"], &encodings);
    decoder(&yaml[0]["operands"], &decision_tree);
    let instr = Instruction {
        opcode: Opcode::Sll,
        raw: 100,
    };
    let a = match_instruction!(instr, {
        (Sll, (a, b)) => {
            println!("Sll: {a}, {b}"); 223
    }
        (Srl, a) => {
            println!("Srl: {a}");
            123
        }
    });
    println!("{a:?}");
}
