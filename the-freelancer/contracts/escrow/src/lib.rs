#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
pub enum DataKey {
    Client,
    Worker,
    Token,
    Amount,
    Funded,
}

fn read_address(env: &Env, key: &DataKey) -> Address {
    env.storage().instance().get(key).unwrap()
}

fn read_amount(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::Amount).unwrap()
}

fn read_is_funded(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Funded).unwrap_or(false)
}

fn write_is_funded(env: &Env, is_funded: bool) {
    env.storage().instance().set(&DataKey::Funded, &is_funded);
}

fn require_funded(env: &Env) {
    assert!(read_is_funded(env), "el contrato no tiene fondos en custodia");
}

fn require_not_funded(env: &Env) {
    assert!(!read_is_funded(env), "el contrato ya tiene fondos en custodia");
}

fn transfer_from_custody(env: &Env, recipient: &Address) {
    let token_address = read_address(env, &DataKey::Token);
    token::Client::new(env, &token_address).transfer(
        &env.current_contract_address(),
        recipient,
        &read_amount(env),
    );
    write_is_funded(env, false);
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor(env: Env, client: Address, worker: Address, token: Address, amount: i128) {
        assert!(amount > 0, "el monto debe ser positivo");
        let storage = env.storage().instance();
        storage.set(&DataKey::Client, &client);
        storage.set(&DataKey::Worker, &worker);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::Amount, &amount);
        storage.set(&DataKey::Funded, &false);
    }

    pub fn deposit(env: Env) {
        let client = read_address(&env, &DataKey::Client);
        client.require_auth();
        require_not_funded(&env);
        let token_address = read_address(&env, &DataKey::Token);
        token::Client::new(&env, &token_address).transfer(
            &client,
            &env.current_contract_address(),
            &read_amount(&env),
        );
        write_is_funded(&env, true);
    }

    pub fn release(env: Env) {
        read_address(&env, &DataKey::Client).require_auth();
        require_funded(&env);
        transfer_from_custody(&env, &read_address(&env, &DataKey::Worker));
    }

    pub fn cancel(env: Env) {
        read_address(&env, &DataKey::Worker).require_auth();
        require_funded(&env);
        transfer_from_custody(&env, &read_address(&env, &DataKey::Client));
    }
}

mod test;
