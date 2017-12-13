extern crate ethabi;
extern crate ethcore;
extern crate ethcore_bigint as bigint;
extern crate ethcore_bytes;
extern crate ethereum_types as types;
extern crate evm as ethcore_evm;
extern crate vm;

#[macro_use]
extern crate lazy_static;

mod trace;

pub mod evm;
pub mod sol;
pub mod unit;

lazy_static! {
    pub static ref FOUNDATION: ethcore::spec::Spec = ethcore::ethereum::new_foundation(&::std::env::temp_dir());
    pub static ref BYZANTIUM: ethcore::spec::Spec = ethcore::ethereum::new_byzantium_test();
}

pub fn main(_json_bytes: &[u8]) {
    println!("This might be a contract CLI in the future.");
}

pub fn evm() -> evm::Evm {
    evm::Evm::default()
}

pub fn evm_byzantium() -> evm::Evm {
    evm::Evm::new_byzantium()
}
