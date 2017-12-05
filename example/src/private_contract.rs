extern crate ethabi;
#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;
extern crate ethereum_types as types;
extern crate solaris;
extern crate rustc_hex;

use solaris::sol;
use ethabi::Token;

fn main() {
    solaris::main(include_bytes!("../res/PrivateContract_sol_PrivateContract.abi"));
}

#[cfg(test)]
mod tests {
	use super::*;
	use rustc_hex::{FromHex, ToHex};

	use_contract!(private_contract, "PrivateContract", "res/PrivateContract_sol_PrivateContract.abi");
	
	fn setup() -> (solaris::evm::Evm, private_contract::PrivateContract) {
	    let contract = private_contract::PrivateContract::default();
	    let code = include_str!("../res/PrivateContract_sol_PrivateContract.bin");
	
	    // PrivateContract initialization arguments
	    let validators = vec![sol::address(10), sol::address(20), sol::address(30)];
	    let init_code = vec![];
	    let init_state = vec![];
	
	    let mut evm = solaris::evm();
	
	    let owner = 3.into();
	    let _address = evm.with_sender(owner).deploy(
            &contract.constructor(code.from_hex().unwrap(),
                validators, 
                init_code, 
                init_state,
                )
            );
	
	    (evm, contract)
	}
	
	#[test]
	fn it_should_have_inited() {
	    let (_evm, _contract) = setup();
	}
}
