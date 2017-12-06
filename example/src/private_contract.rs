extern crate ethabi;
#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;
extern crate ethereum_types as types;
extern crate solaris;
extern crate rustc_hex;
extern crate rand;
extern crate secp256k1;
extern crate tiny_keccak;

fn main() {
	solaris::main(include_bytes!("../res/PrivateContract_sol_PrivateContract.abi"));
}

#[cfg(test)]
mod tests {
	use super::*;
	use rustc_hex::{FromHex, ToHex};
	use secp256k1::key::{SecretKey, PublicKey};

	use_contract!(private_contract, "PrivateContract", "res/PrivateContract_sol_PrivateContract.abi");

	pub struct Validator {
		address: types::Address,
		private: SecretKey,
		public: PublicKey,
	}

	pub struct Secp256k1Parts {
		v: [u8; 32], // Uint
		r: [u8; 32], // Bytes
		s: [u8; 32], // Bytes
	}

	pub fn secp256k1_signature_parts(secret_key: &SecretKey, message_hash: &[u8]) -> Secp256k1Parts {
		use secp256k1::{Message, Secp256k1};
		use solaris::sol;

		let secp = Secp256k1::new();
		let sp_msg = &Message::from_slice(message_hash).unwrap();
		let (recid, sigdata) = secp
			.sign_recoverable(sp_msg, secret_key)
			.unwrap()
			.serialize_compact(&secp);

		let mut s = [0u8; 32];
		let mut r = [0u8; 32];

		s.copy_from_slice(&sigdata[0..32]);
		r.copy_from_slice(&sigdata[32..64]);
		let v: [u8; 32] = sol::raw::uint(recid.to_i32() as u64); 

		Secp256k1Parts{v: v, r: r, s: s}
	}

	pub fn create_validator() -> Validator { 
		use rand::os::OsRng;
		use secp256k1::Secp256k1;
		use tiny_keccak::keccak256;

		let secp = Secp256k1::new();
		let mut rng = OsRng::new().unwrap();

		let vprv = SecretKey::new(&secp, &mut rng);
		let vpub = PublicKey::from_secret_key(&secp, &vprv).unwrap();

		let mut vaddr = [0u8; 20];
		vaddr.copy_from_slice(
			&keccak256(vpub.serialize_vec(&secp, false).as_slice())[12..]
			);

		Validator {
			address: vaddr.into(),
			private: vprv, 
			public: vpub,
		}
	}

	fn setup() -> (solaris::evm::Evm, private_contract::PrivateContract, Vec<Validator>) {

		let contract = private_contract::PrivateContract::default();
		let code = include_str!("../res/PrivateContract_sol_PrivateContract.bin");
	
		// PrivateContract initialization arguments
		let init_code = vec![];
		let init_state = vec![];
	
		let mut evm = solaris::evm();

		let mut vals: Vec<Validator> = Vec::new();

		for i in 0..3 {
			vals.push(create_validator());
		}

		let owner = 3.into();
		let _address = evm.with_sender(owner).deploy(
			&contract.constructor(
				code.from_hex().unwrap(),
				vec![vals[0].address, vals[1].address, vals[2].address], 
				init_code, 
				init_state,
				)
			);
	
		(evm, contract, vals)
	}
	
	#[test]
	fn it_should_have_inited() {
		let (_evm, _contract, _validators) = setup();
	}

	#[test]
	fn it_should_allow_state_change_if_all_the_signatures_are_ok() {
		let (mut evm, contract, validators) = setup();
		let pcon = contract.functions();

		assert_eq!(pcon.state().call(&mut evm).unwrap().to_hex(), "", "Initial State should be empty");

		let new_state = "ffaabb55ffaabb55ffaabb55ffaabb55ffaabb55ffaabb55ffaabb55ffaabb55".from_hex().unwrap();

		let mut parts: Vec<Secp256k1Parts> = Vec::new();
		for v in validators {
			parts.push(
				secp256k1_signature_parts(&v.private, new_state.as_slice())
			);
		}

		let mut vs: Vec<[u8; 32]> = Vec::new();
		let mut rs: Vec<[u8; 32]> = Vec::new();
		let mut ss: Vec<[u8; 32]> = Vec::new();
		for p in parts {
			vs.push(p.v);
			rs.push(p.r);
			ss.push(p.s);
		}

		pcon.set_state().transact(
			new_state.as_slice(),
			vs,
			rs,
			ss,
			&mut evm)
		.unwrap();

		let actual_state = pcon.state().call(&mut evm).unwrap().to_hex();
		assert_eq!(new_state.to_hex(), actual_state, "Initial State should be {}, got {}", new_state.to_hex(), actual_state);
	}
}
