use std::io::{self, prelude::*};

const MAX_INT: usize = 59049; // 2_222_222_222 + 1

#[derive(Debug)]
struct Interpreter {
    memory: [Word; 59049],
    reg: Registers,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            memory: [Word(0); 59049],
            reg: Registers {
                a: Word(0),
                c: Word(0),
                d: Word(0),
            },
        }
    }

    fn get_mut(&mut self, addr: Word) -> &mut Word {
        &mut self.memory[addr.0]
    }

    fn jmp(&mut self) {
        let d_v = *self.get_mut(self.reg.d);
        self.reg.c = d_v;
    }

    fn out(&mut self) {
        let a = self.reg.a;
        print!("{}", char::try_from(a.0 as u8 % 128).unwrap_or('\u{FFFD}'));
    }

    fn r#in(&mut self) {
        self.reg.a = Word(io::stdin().bytes().next().unwrap().unwrap() as u8 as usize);
    }

    fn init_mem(&mut self, s: &str) -> Result<usize, InvalidOpCode> {
        static OPCODES: [usize; 8] = [4, 5, 23, 39, 40, 62, 68, 81];
        let mut i = 0;
        for c in s.chars().filter(|c| !c.is_whitespace()) {
            if !OPCODES.contains(&((c as usize + i) % 94)) {
                return Err(InvalidOpCode {
                    op: (c as u8 + i as u8) % 94,
                });
            }
            self.memory[i] = Word(c as usize);
            i += 1;
        }
        for i in i..MAX_INT {
            self.memory[i] = self.memory[i - 2].crz(self.memory[i - 1]);
        }
        Ok(i)
    }

    fn rot(&mut self) {
        let w = self.get_mut(self.reg.d);
        *w = w.rot();
        self.reg.a = *w;
    }

    fn set_d(&mut self) {
        self.reg.d = *self.get_mut(self.reg.d);
    }

    fn execute(&mut self) {
        let mut cycles = 0;
        loop {
            let decoded = self.decode_instruction();
            match decoded {
                4 => self.jmp(),
                5 => self.out(),
                23 => self.r#in(),
                39 => self.rot(),
                40 => self.set_d(),
                62 => {
                    let a = self.reg.a;
                    let ptr = self.get_mut(self.reg.d);
                    let res = ptr.crz(a);
                    *ptr = res;
                    self.reg.a = res;
                }
                68 => {}
                81 => break,
                _ => {}
            }

            self.encrypt_and_advance_pc();

            //println!("Cycle {}", cycles);
            cycles += 1;
        }

        println!("Finished in {} cycles", cycles);
    }

    fn decode_instruction(&mut self) -> usize {
        let instruction = *self.get_mut(self.reg.c);
        let decoded = (instruction + self.reg.c).0 % 94;
        decoded
    }

    fn encrypt_and_advance_pc(&mut self) {
        let instruction = *self.get_mut(self.reg.c);
        *self.get_mut(self.reg.c) = Word(encrypt(instruction.0 % 94));
        self.reg.c.inc();
        self.reg.d.inc();
    }
}

#[derive(Debug)]
struct Registers {
    a: Word,
    c: Word,
    d: Word,
}

#[derive(Debug)]
struct InvalidOpCode {
    op: u8,
}

// calculates the crz operation for one trit
fn calc_crz(trit_a: usize, trit_b: usize) -> usize {
    match (trit_a, trit_b) {
        (0, 0) => 1,
        (1, 0) => 1,
        (2, 0) => 2,
        (0, 1) => 0,
        (1, 1) => 0,
        (2, 1) => 2,
        (0, 2) => 0,
        (1, 2) => 2,
        (2, 2) => 1,
        (_, _) => panic!("output out of range"),
    }
}

// one trinary word
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Word(usize);

impl std::ops::Add<Word> for Word {
    type Output = Word;
    fn add(self, rhs: Word) -> Self::Output {
        Word((self.0 + rhs.0) % MAX_INT)
    }
}

impl Word {
    fn inc(&mut self) {
        self.0 = (self.0 + 1) % MAX_INT
    }

    fn crz(self, other: Word) -> Word {
        let mut a = self.0;
        let mut b = other.0;
        let mut c = 0;

        for i in 0..10 {
            let last_trit_a = a % 3;
            let last_trit_b = b % 3;
            a /= 3;
            b /= 3;
            c += calc_crz(last_trit_a, last_trit_b) * 3usize.pow(i);
        }

        assert!(c <= MAX_INT);

        Word(c)
    }

    fn rot(mut self) -> Word {
        let trit = self.0 % 3;
        self.0 /= 3;
        self.0 += trit * 3_usize.pow(9);
        self
    }

    fn as_tri_str(&self) -> String {
        let mut a = self.0;
        let mut s_reversed = String::new();
        for _ in 0..10 {
            s_reversed.push_str(&format!("{}", a % 3));
            a /= 3;
        }

        s_reversed.chars().rev().collect()
    }

    fn from_str(s: &str) -> Result<Word, std::num::ParseIntError> {
        usize::from_str_radix(s, 3).map(Word)
    }
}

fn main() {
    let arg = match std::env::args().nth(1) {
        Some(arg) => arg,
        None => {
            println!("Error: no file specified");
            return;
        }
    };

    let mut f = match std::fs::File::open(arg) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open file: {}", e);
            return;
        }
    };
    let mut src = String::new();
    f.read_to_string(&mut src);

    let mut interpreter = Interpreter::new();
    let n = interpreter.init_mem(&src).unwrap();
    interpreter.execute();
}

fn encrypt(n: usize) -> usize {
    match n {
        0 => 57,
        1 => 109,
        2 => 60,
        3 => 46,
        4 => 84,
        5 => 86,
        6 => 97,
        7 => 99,
        8 => 96,
        9 => 117,
        10 => 89,
        11 => 42,
        12 => 77,
        13 => 75,
        14 => 39,
        15 => 88,
        16 => 126,
        17 => 120,
        18 => 68,
        19 => 108,
        20 => 125,
        21 => 82,
        22 => 69,
        23 => 111,
        24 => 107,
        25 => 78,
        26 => 58,
        27 => 35,
        28 => 63,
        29 => 71,
        30 => 34,
        31 => 105,
        32 => 64,
        33 => 53,
        34 => 122,
        35 => 93,
        36 => 38,
        37 => 103,
        38 => 113,
        39 => 116,
        40 => 121,
        41 => 102,
        42 => 114,
        43 => 36,
        44 => 40,
        45 => 119,
        46 => 101,
        47 => 52,
        48 => 123,
        49 => 87,
        50 => 80,
        51 => 41,
        52 => 72,
        53 => 45,
        54 => 90,
        55 => 110,
        56 => 44,
        57 => 91,
        58 => 37,
        59 => 92,
        60 => 51,
        61 => 100,
        62 => 76,
        63 => 43,
        64 => 81,
        65 => 59,
        66 => 62,
        67 => 85,
        68 => 33,
        69 => 112,
        70 => 74,
        71 => 83,
        72 => 55,
        73 => 50,
        74 => 70,
        75 => 104,
        76 => 79,
        77 => 65,
        78 => 49,
        79 => 67,
        80 => 66,
        81 => 54,
        82 => 118,
        83 => 94,
        84 => 61,
        85 => 73,
        86 => 95,
        87 => 48,
        88 => 47,
        89 => 56,
        90 => 124,
        91 => 106,
        92 => 115,
        93 => 98,
        _ => panic!("tried to encrypt something out of bounds"),
    }
}
