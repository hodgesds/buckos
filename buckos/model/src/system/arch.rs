use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Arch {
    Alpha,
    Amd64,
    Arm,
    Arm64,
    Hppa,
    Ia64,
    Loong,
    M68k,
    Mips,
    Ppc,
    Ppc64,
    Riscv,
    S390,
    Sparc,
    X86,
}
