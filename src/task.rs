use std::env;
use std::cmp::{min, max};
use bitvec::prelude::*;
use rand::{rngs::StdRng, Rng};
use rand_seeder::Seeder;
use anyhow::{anyhow, Result};

use crate::util::mod_inv;

type MemoryLayout = Vec<(u64, u64)>;
type MemoryLayoutIO = (MemoryLayout, MemoryLayout);

const ECC_MOD: u64 = (1u64 << 16) - 17;

#[derive(Debug, Copy, Clone)]
pub enum Task {
    ZeroXor,
    OneAdd1,
    TwoAdd16,
    TwoASub16,
    TwoBSub16,
    ThreeMul16,
    FourAdd16Mod,
    FourASub16Mod,
    FiveMul16Mod,
    FiveAInv16Mod,
    SixPointAdd,
    SevenPointMul,
    EightSha256,
}

impl Task {
    pub fn from_str(task_name: &str) -> Result<Self> {
        match task_name {
            "0" => Ok(Self::ZeroXor),
            "1" => Ok(Self::OneAdd1),
            "2" => Ok(Self::TwoAdd16),
            "2a" => Ok(Self::TwoASub16),
            "3" => Ok(Self::ThreeMul16),
            "4" => Ok(Self::FourAdd16Mod),
            "4a" => Ok(Self::FourASub16Mod),
            "5" => Ok(Self::FiveMul16Mod),
            "5a" => Ok(Self::FiveAInv16Mod),
            "6" => Ok(Self::SixPointAdd),
            "7" => Ok(Self::SevenPointMul),
            "8" => Ok(Self::EightSha256),
            _ => Err(anyhow!("Unknown task number {}", task_name))
        }
    }

    fn get_tc(self, tc_id: i8, rng: &mut StdRng) -> Result<MemoryLayoutIO> {
        let tc = match self {
            Task::ZeroXor => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    _ => (rng.gen::<u64>() & 0x01, rng.gen::<u64>() & 0x01),
                };
                let out = in_a ^ in_b;

                (vec![(in_a, 1), (in_b, 1)], vec![(out, 1)])
            }
            Task::OneAdd1 => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    _ => (rng.gen::<u64>() & 0x01, rng.gen::<u64>() & 0x01),
                };
                let out = in_a + in_b;

                (vec![(in_a, 1), (in_b, 1)], vec![(out, 2)])
            }
            Task::TwoAdd16 => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    4 => (0x00ff, 8),
                    5 => (0x0100, 8),
                    6 => (0xffff, 0),
                    7 => (0xffff, 1),
                    8 => (8, 0x00ff),
                    9 => (8, 0x0100),
                    10 => (0, 0xffff),
                    11 => (0x0001, 0xffff),
                    12 => (0xffff, 0xffff),
                    _ => (rng.gen::<u64>() & 0xffff, rng.gen::<u64>() & 0xffff),
                };
                let out = in_a + in_b;

                (vec![(in_a, 16), (in_b, 16)], vec![(out, 17)])
            }
            Task::TwoASub16 => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (1, 1),
                    3 => (0x0100, 1),
                    4 => (0x0100, 0x0100),
                    5 => (0xffff, 0),
                    6 => (0xffff, 1),
                    7 => (0xffff, 0x0100),
                    8 => (0xffff, 0xffff),
                    _ => {
                        let (tmp_a, tmp_b) = (rng.gen::<u64>() & 0xffff, rng.gen::<u64>() & 0xffff);
                        (max(tmp_a, tmp_b), min(tmp_a, tmp_b))
                    },
                };
                let out = (in_a + 0x10000 - in_b) & 0xffff;

                (vec![(in_a, 16), (in_b, 16)], vec![(out, 16)])
            }
            Task::TwoBSub16 => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    4 => (0x0100, 1),
                    5 => (0xffff, 0),
                    6 => (0xffff, 0x0100),
                    7 => (0xffff, 1),
                    8 => (1, 0x0100),
                    9 => (0, 0xffff),
                    10 => (0x0100, 0xffff),
                    11 => (1, 0xffff),
                    12 => (0xffff, 0xffff),
                    _ => (rng.gen::<u64>() & 0xffff, rng.gen::<u64>() & 0xffff),
                };
                let out = (in_a + 0x10000 - in_b) & 0xffff;

                (vec![(in_a, 16), (in_b, 16)], vec![(out, 16)])
            }
            Task::ThreeMul16 => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    4 => (0x0aa0, 0x0003),
                    5 => (0xffff, 0),
                    6 => (0xffff, 1),
                    7 => (0x0003, 0x0aa0),
                    8 => (0, 0xffff),
                    9 => (1, 0xffff),
                    10 => (0xffff, 0xffff),
                    _ => (rng.gen::<u64>() & 0xffff, rng.gen::<u64>() & 0xffff),
                };
                let out = in_a * in_b;

                (vec![(in_a, 16), (in_b, 16)], vec![(out, 32)])
            }
            Task::FourAdd16Mod => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    4 => (0x0100, 0x0080),
                    5 => (ECC_MOD-1, 0),
                    6 => (ECC_MOD-1, 1),
                    7 => (0x0080, 0x0100),
                    8 => (0, ECC_MOD-1),
                    9 => (1, ECC_MOD-1),
                    10 => (ECC_MOD-1, ECC_MOD-1),
                    _ => (rng.gen::<u64>() % ECC_MOD, rng.gen::<u64>() % ECC_MOD),
                };
                let out = (in_a + in_b) % ECC_MOD;

                (vec![(in_a, 16), (in_b, 16)], vec![(out, 16)])
            }
            Task::FourASub16Mod => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    4 => (0x0100, 0x0080),
                    5 => (ECC_MOD-1, 0),
                    6 => (ECC_MOD-1, 1),
                    7 => (0x0080, 0x0100),
                    8 => (0, ECC_MOD-1),
                    9 => (1, ECC_MOD-1),
                    10 => (ECC_MOD-1, ECC_MOD-1),
                    _ => (rng.gen::<u64>() % ECC_MOD, rng.gen::<u64>() % ECC_MOD),
                };
                let out = (in_a + ECC_MOD - in_b) % ECC_MOD;

                (vec![(in_a, 16), (in_b, 16)], vec![(out, 16)])
            }
            Task::FiveMul16Mod => {
                let (in_a, in_b) = match tc_id {
                    0 => (0, 0),
                    1 => (1, 0),
                    2 => (0, 1),
                    3 => (1, 1),
                    4 => (0x0aa0, 0x0003),
                    5 => (ECC_MOD-1, 0),
                    6 => (ECC_MOD-1, 1),
                    7 => (0x0003, 0x0aa0),
                    8 => (0, ECC_MOD-1),
                    9 => (1, ECC_MOD-1),
                    10 => (ECC_MOD-1, ECC_MOD-1),
                    _ => (rng.gen::<u64>() % ECC_MOD, rng.gen::<u64>() % ECC_MOD),
                };
                let out = (in_a * in_b) % ECC_MOD;

                (vec![(in_a, 16), (in_b, 16)], vec![(out, 16)])
            }
            Task::FiveAInv16Mod => {
                let in_a = match tc_id {
                    0 => 1,
                    1 => 2,
                    2 => 3,
                    3 => 4,
                    4 => mod_inv(2, ECC_MOD),
                    5 => mod_inv(3, ECC_MOD),
                    6 => mod_inv(4, ECC_MOD),
                    7 => ECC_MOD-2,
                    8 => ECC_MOD-1,
                    _ => 1 + (rng.gen::<u64>() % (ECC_MOD-1)),
                };
                let out = mod_inv(in_a, ECC_MOD);

                (vec![(in_a, 16)], vec![(out, 16)])
            }
            _ => {
                Err(anyhow!("Task {:?} not implemented", self))?;
                unreachable!();
            }
        };

        Ok(tc)
    }

    fn pack(spans: MemoryLayout) -> BitVec<u8> {
        let mut bv = bitvec![u8, Lsb0; 0; spans.iter().map(|x| (*x).1).sum::<u64>() as usize];

        let mut cur: usize = 0;
        for (value, width) in spans.iter() {
            for pos in 0..(*width as usize) {
                bv.set(pos + cur, ((value >> pos) & 1) == 1);
            }
            cur += *width as usize;
        }

        bv
    }

    pub fn load_tc(self, tc_id: i8) -> Result<(BitVec<u8>, BitVec<u8>)> {
        let mut rng: StdRng = Seeder::from(format!(
            "WPKPP/{}/{:?}/{}",
            env::var("WPKPP_SEED").unwrap_or("NOSEED".to_string()),
            self,
            tc_id
        ))
        .make_rng();

        let (input_layout, output_layout) = self.get_tc(tc_id, &mut rng)?;
        Ok((Self::pack(input_layout), Self::pack(output_layout)))
    }
}
