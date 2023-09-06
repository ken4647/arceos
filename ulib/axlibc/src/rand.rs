//! Random number generator.

use core::sync::atomic::{AtomicU64, Ordering::SeqCst};

static SEED: AtomicU64 = AtomicU64::new(0xa2ce_a2ce);

// Returns a 32-bit unsigned pseudo random interger based on LCG.
fn random_lcg() -> u32 {
    let new_seed = (SEED.load(SeqCst).wrapping_mul(25214903917) + 11)%(1<<48);
    SEED.store(new_seed, SeqCst);
    new_seed as u32
}


// Checking if the CPU core is compatible with hardware random number instructions.
#[cfg(feature = "random-hw")]
fn has_rdrand() -> bool{
    #[cfg(target_arch = "x86_64")]{
        let mut ecx:u32;
        unsafe{
            core::arch::asm!(
                "mov eax, 1",
                "cpuid",
                out("ecx") ecx
            )        
        }
        ecx&(1<<30)!=0
    }
    #[cfg(target_arch = "aarch64")]{
        let mut ID_AA64ISAR0_EL1:u64;
        unsafe{
            core::arch::asm!(
                "mrs {},ID_AA64ISAR0_EL1",
                out(reg) ID_AA64ISAR0_EL1
            )    
        }
        ID_AA64ISAR0_EL1&(0b1111<<60) == 0b0001<<60
    }    
    #[cfg(target_arch = "riscv64")]{
        false
    }
}

// Implement hardware random instruction for different arch
#[cfg(feature = "random-hw")]
fn random_hw() -> u32 {
    #[cfg(target_arch = "x86_64")]{
        let mut rand:u32;
        unsafe{
            core::arch::asm!{
                "rdrand {0:e}",
                out(reg) rand
            }        
        }
        rand as u32    
    }
    #[cfg(target_arch = "aarch64")]{
        let mut rand:u64; 
        unsafe{
            core::arch::asm!{
                "mrs {}, s3_3_c2_c4_0", // s3_3_c2_c4_0 is register `rndr`
                out(reg) rand
            }        
        }
        rand as u32
    }#[cfg(target_arch = "riscv64")]{
        panic!("riscv64 has no rdrand instructions")
    }    
}

/// Sets the seed for the random number generator implemented by LCG.
#[no_mangle]
pub unsafe extern "C" fn ax_srand(_seed: u32) {
    #[cfg(feature = "random-hw")]{
        info!("hardware instruction doesn't need seed");
    }
    #[cfg(not(feature = "random-hw"))]{
       SEED.store(_seed as u64, SeqCst); 
    }
    
}

// generate random in u32
#[no_mangle]
pub unsafe extern "C" fn ax_rand_u32() -> u32 {
    #[cfg(feature = "random-hw")]{
        match has_rdrand() {
            true => {
                random_hw()
            }
            false => {
                warn!("The cpu doesn't support `rdrand` or `rndr` instruction, returning LCG instead");
                random_lcg()
            }
        }
    }
    #[cfg(not(feature = "random-hw"))]{
        random_lcg()
    }
}
