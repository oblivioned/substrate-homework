use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn test_create_revoke_transfer_claim() {
	new_test_ext().execute_with(|| {
		let claim = vec![1,2,3];

		// create
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		// transfer
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), 2, claim.clone()));

		// revoke
		assert_ok!(PoeModule::revoke_claim(Origin::signed(2), claim.clone()));
	});
}

#[test]
fn test_error_proof_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![1];

		// create
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		// exist
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	});
}

#[test]
fn test_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let not_exist_claim = vec![2];

		// case 1
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), not_exist_claim.clone()),
			Error::<Test>::ClaimNotExist
		);

		// case 2
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), 2, not_exist_claim.clone()),
			Error::<Test>::ClaimNotExist
		);

	});
}

#[test]
fn test_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![1];

		// create
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		// case 1 - transfer
		assert_noop!(PoeModule::transfer_claim(Origin::signed(2), 2, claim.clone()), Error::<Test>::NotClaimOwner);

		// case 2 - revoke
		assert_noop!(PoeModule::revoke_claim(Origin::signed(2), claim.clone()), Error::<Test>::NotClaimOwner);

	});
}

#[test]
fn test_invalid_claim() {
	new_test_ext().execute_with(|| {
		let long_claim = vec![1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1];

		// create
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), long_claim.clone()),
			Error::<Test>::InvalidClaim
		);
	});
}