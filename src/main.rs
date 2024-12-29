use std::{env, fmt::{self, Display}, process::exit};

use regex::Regex;

fn main() {
    let mut args : Vec<String> = env::args().collect();    
    args = args.split_off(0);

    let mut modified_args = vec![];

    let non_number_regex : Regex = Regex::new(r"[^0-9a-f]").unwrap();

    for (i, arg) in args.iter_mut().enumerate() {
        if i == 0 { continue; }
        modified_args.push(non_number_regex.replace_all(arg, "").to_string());
    }

    let uuids = modified_args.iter().map( |str| {
        let res = u128::from_str_radix(str,16);
        if res.is_err() {
            dbg!(res.unwrap_err());
            exit(-1);
        }
        res.unwrap()
    }).map(|num| {
        UUID::fromu128(num)
    });

    for uuid in uuids {
        solve_uuid(uuid);
    }

}

fn solve_uuid(target : UUID) {
    println!("{target} target");

    let mut potential_seeds: Vec<i128> = vec![];

    let r = (target.high >> 32) as u32;

    println!("testing {:x}..{:x}", ((r as i128 ) << UNKNOWN_BITS), ((r as i128 ) << UNKNOWN_BITS) + (1 << UNKNOWN_BITS) - 1);

    for b in 0..(1 << UNKNOWN_BITS) {
        reverse(r as i128, b).inspect(|value| {
            potential_seeds.append(&mut value.to_vec());
        });
    }

    let mut best_digits = 0;
    let mut best_uuid : UUID = UUID::new(0, 0);
    let mut best_seed : i128 = -1;

    for seed in potential_seeds {
        if seed > i64::MAX as i128 || seed < i64::MIN as i128 {
            println!("Invalid Seed! {seed}")
        } else {
            let uuid = RNG { seed: seed.try_into().unwrap() }.random_uuid();
            //let first_int = RNG { seed: seed.try_into().unwrap() }.next(32);

            if uuid == target {
                println!("Seed {seed} WORKS!!!!!!!!!!!!!!!");
                break;
            } else {
                if uuid.digit_match_count(&target) > best_digits {
                    best_digits = uuid.digit_match_count(&target);
                    best_uuid = uuid;
                    best_seed = seed;
                }
            }
        }
    }

    println!("{best_uuid} was the closest match, with {best_digits} bits right at the start and seed {best_seed}");
    println!("seed as passed to setSeed is is {}",best_seed ^ MULTIPLIER)
}

struct RNG {
    seed : i64
}

impl RNG {
    fn new(seed : i64) -> RNG {
        let mut rng = RNG { seed: 0};
        rng.set_seed(seed);
        return rng;
    }

    fn set_seed(&mut self, seed : i64) {
        self.seed = (seed ^ MULTIPLIER as i64) & BIT_MASK as i64;

    }

    fn next(&mut self, bits : i32) -> i32 {
        let m = ((self.seed * MULTIPLIER as i64) + CONSTANT) & BIT_MASK as i64;
        self.seed = m;
        return (m >> (48 - bits)) as i32;
    }

    fn next_long(&mut self) -> i64 {
        let mut i:i64 = self.next(32) as i64;
        let j:i64 = self.next(32) as i64;
        i = i << 32;
        return i + j;
    }

    fn random_uuid(&mut self) -> UUID {
        let l = self.next_long() & -61441i64 | 16384i64;
        let m = self.next_long() & 4611686018427387903i64 | i64::MIN;

        return UUID::new(l,m);
    }
}

struct UUID {
    high : i64,
    low : i64
}

impl UUID {
    const fn fromu128(number : u128) -> UUID {
        UUID {
            high: (number >> 64) as i64,
            low: (number & ((1 << 64)-1)) as i64
        }
    }
    fn new(high : i64, low : i64) -> UUID {
        UUID {
            high: high,
            low: low
        }
    }
    fn digit_match_count(&self ,other : &Self) -> u32 {
        let matches = ! (other.high ^ self.high);
        matches.leading_ones()
    }
}

impl PartialEq for UUID {
    fn eq(&self, other: &Self) -> bool {
        self.high == other.high && self.low == other.low
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:x}{:x}",self.high,self.low))
    }
}

const MULTIPLIER : i128 = 25214903917;
const MODULO : i128 = 281474976710656;
const BIT_MASK : i128 = MODULO - 1;
const CONSTANT : i64 = 11;
const UNKNOWN_BITS : u32 = 16;

fn reverse(r : i128, b : i128) -> Option<Vec<i128>> {
    let v = (r << UNKNOWN_BITS) + b;
    let t = ((v - CONSTANT as i128)).rem_euclid(MODULO);

    inv(MULTIPLIER,t,MODULO)
}
// give x for
// ax = b mod n
fn inv(a : i128, b : i128, n : i128) -> Option<Vec<i128>> {
    let (gcd,inv_a,_) = egcd(a, n);

    if b % gcd != 0 {return None;}

    let k0 = (inv_a * (b / gcd)).rem_euclid(n);

    Some (
        (0..gcd).map(|i| {
            (k0 + (i * (n / gcd))).rem_euclid(n)
        }).collect()
    )
}

const fn egcd(a : i128, b : i128) -> (i128, i128, i128) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (r, s, t) = egcd(b, a % b);
        (r, t, s - t * (a / b))
    }
}