#![cfg(test)]

use crate::{EscrowContract, EscrowContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};

const ESCROW_AMOUNT: i128 = 10_000_000;

struct Fixture<'a> {
    env: &'a Env,
    client: Address,
    worker: Address,
    token: Address,
    escrow: EscrowContractClient<'a>,
}

fn create_fixture(env: &Env) -> Fixture<'_> {
    env.mock_all_auths();
    let client = Address::generate(env);
    let worker = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    token::StellarAssetClient::new(env, &token).mint(&client, &ESCROW_AMOUNT);
    let escrow_id = env.register(
        EscrowContract,
        (client.clone(), worker.clone(), token.clone(), ESCROW_AMOUNT),
    );
    Fixture {
        env,
        client,
        worker,
        token,
        escrow: EscrowContractClient::new(env, &escrow_id),
    }
}

fn balance_of(fixture: &Fixture<'_>, holder: &Address) -> i128 {
    token::Client::new(fixture.env, &fixture.token).balance(holder)
}

#[test]
fn deposit_moves_funds_into_custody() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    fixture.escrow.deposit();
    assert_eq!(balance_of(&fixture, &fixture.client), 0);
    assert_eq!(balance_of(&fixture, &fixture.escrow.address), ESCROW_AMOUNT);
}

#[test]
fn release_pays_the_worker() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    fixture.escrow.deposit();
    fixture.escrow.release();
    assert_eq!(balance_of(&fixture, &fixture.worker), ESCROW_AMOUNT);
    assert_eq!(balance_of(&fixture, &fixture.escrow.address), 0);
}

#[test]
fn cancel_refunds_the_client() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    fixture.escrow.deposit();
    fixture.escrow.cancel();
    assert_eq!(balance_of(&fixture, &fixture.client), ESCROW_AMOUNT);
    assert_eq!(balance_of(&fixture, &fixture.worker), 0);
}

#[test]
#[should_panic(expected = "el contrato no tiene fondos en custodia")]
fn release_without_deposit_fails() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    fixture.escrow.release();
}

#[test]
#[should_panic(expected = "el contrato ya tiene fondos en custodia")]
fn deposit_twice_fails() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    fixture.escrow.deposit();
    fixture.escrow.deposit();
}

#[test]
#[should_panic(expected = "el contrato no tiene fondos en custodia")]
fn release_twice_fails() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    fixture.escrow.deposit();
    fixture.escrow.release();
    fixture.escrow.release();
}
