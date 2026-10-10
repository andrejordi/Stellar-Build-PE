#![cfg(test)]

use crate::{TicketGateContract, TicketGateContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};

const TICKET_PRICE: i128 = 10_000_000;

struct Fixture<'a> {
    env: &'a Env,
    organizer: Address,
    token: Address,
    gate: TicketGateContractClient<'a>,
}

fn create_fixture(env: &Env) -> Fixture<'_> {
    env.mock_all_auths();
    let organizer = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    let gate_id = env.register(
        TicketGateContract,
        (organizer.clone(), token.clone(), TICKET_PRICE),
    );
    Fixture {
        env,
        organizer,
        token,
        gate: TicketGateContractClient::new(env, &gate_id),
    }
}

fn create_funded_buyer(fixture: &Fixture<'_>) -> Address {
    let buyer = Address::generate(fixture.env);
    token::StellarAssetClient::new(fixture.env, &fixture.token).mint(&buyer, &TICKET_PRICE);
    buyer
}

fn balance_of(fixture: &Fixture<'_>, holder: &Address) -> i128 {
    token::Client::new(fixture.env, &fixture.token).balance(holder)
}

#[test]
fn buy_ticket_pays_the_organizer() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    let buyer = create_funded_buyer(&fixture);
    fixture.gate.buy_ticket(&buyer);
    assert_eq!(balance_of(&fixture, &buyer), 0);
    assert_eq!(balance_of(&fixture, &fixture.organizer), TICKET_PRICE);
}

#[test]
fn buy_ticket_registers_ownership_with_sequential_ids() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    let first_buyer = create_funded_buyer(&fixture);
    let second_buyer = create_funded_buyer(&fixture);
    assert_eq!(fixture.gate.buy_ticket(&first_buyer), 1);
    assert_eq!(fixture.gate.buy_ticket(&second_buyer), 2);
    assert_eq!(fixture.gate.owner_of(&1), Some(first_buyer));
    assert_eq!(fixture.gate.owner_of(&2), Some(second_buyer));
    assert_eq!(fixture.gate.tickets_sold(), 2);
}

#[test]
fn owner_of_unknown_ticket_is_none() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    assert_eq!(fixture.gate.owner_of(&1), None);
}

#[test]
#[should_panic]
fn buy_ticket_without_funds_fails() {
    let env = Env::default();
    let fixture = create_fixture(&env);
    let broke_buyer = Address::generate(&env);
    fixture.gate.buy_ticket(&broke_buyer);
}
