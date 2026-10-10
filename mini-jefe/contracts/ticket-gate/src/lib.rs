#![no_std]

use soroban_sdk::{contract, contractevent, contractimpl, contracttype, token, Address, Env};

const INSTANCE_TTL_THRESHOLD: u32 = 50_000;
const INSTANCE_TTL_EXTEND_TO: u32 = 100_000;
const TICKET_TTL_THRESHOLD: u32 = 100_000;
const TICKET_TTL_EXTEND_TO: u32 = 200_000;

#[contracttype]
pub enum DataKey {
    Organizer,
    Token,
    Price,
    TicketCount,
    TicketOwner(u32),
}

#[contractevent]
pub struct TicketBought {
    #[topic]
    pub buyer: Address,
    pub ticket_id: u32,
    pub price: i128,
}

fn read_organizer(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Organizer).unwrap()
}

fn read_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

fn read_price(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::Price).unwrap()
}

fn read_ticket_count(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::TicketCount).unwrap_or(0)
}

fn write_ticket_count(env: &Env, ticket_count: u32) {
    env.storage().instance().set(&DataKey::TicketCount, &ticket_count);
}

fn read_ticket_owner(env: &Env, ticket_id: u32) -> Option<Address> {
    env.storage().persistent().get(&DataKey::TicketOwner(ticket_id))
}

fn write_ticket_owner(env: &Env, ticket_id: u32, owner: &Address) {
    let key = DataKey::TicketOwner(ticket_id);
    env.storage().persistent().set(&key, owner);
    env.storage()
        .persistent()
        .extend_ttl(&key, TICKET_TTL_THRESHOLD, TICKET_TTL_EXTEND_TO);
}

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

fn collect_payment(env: &Env, buyer: &Address) {
    token::Client::new(env, &read_token(env)).transfer(
        buyer,
        &read_organizer(env),
        &read_price(env),
    );
}

#[contract]
pub struct TicketGateContract;

#[contractimpl]
impl TicketGateContract {
    pub fn __constructor(env: Env, organizer: Address, token: Address, price: i128) {
        assert!(price > 0, "el precio debe ser positivo");
        let storage = env.storage().instance();
        storage.set(&DataKey::Organizer, &organizer);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::Price, &price);
    }

    pub fn buy_ticket(env: Env, buyer: Address) -> u32 {
        buyer.require_auth();
        collect_payment(&env, &buyer);
        let ticket_id = read_ticket_count(&env) + 1;
        write_ticket_owner(&env, ticket_id, &buyer);
        write_ticket_count(&env, ticket_id);
        extend_instance_ttl(&env);
        TicketBought {
            buyer,
            ticket_id,
            price: read_price(&env),
        }
        .publish(&env);
        ticket_id
    }

    pub fn owner_of(env: Env, ticket_id: u32) -> Option<Address> {
        read_ticket_owner(&env, ticket_id)
    }

    pub fn tickets_sold(env: Env) -> u32 {
        read_ticket_count(&env)
    }

    pub fn ticket_price(env: Env) -> i128 {
        read_price(&env)
    }
}

mod test;
