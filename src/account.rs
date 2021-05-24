use crate::{
    transaction::{AmountTransaction, Transaction},
    Money,
};
use num_traits::Zero;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug)]
pub struct ClientAccount {
    pub id: u16,
    pub available: Money,
    pub held: Money,
    pub locked: bool,
    deposit_transactions: HashMap<u32, (AmountTransaction, TransactionState)>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransactionState {
    Valid,
    Disputed,
    ChargedBack,
}

impl ClientAccount {
    pub fn new(id: u16) -> Self {
        ClientAccount {
            id,
            available: Money::zero(),
            held: Money::zero(),
            locked: false,
            deposit_transactions: HashMap::new(),
        }
    }

    pub fn try_apply(&mut self, transaction: Transaction) -> Result<(), TransactionApplyError> {
        use TransactionApplyError::*;

        if self.locked {
            return Err(ClientLocked(transaction));
        }
        match transaction {
            Transaction::Deposit(deposit) => {
                self.available += deposit.amount;
                self.deposit_transactions
                    .insert(deposit.tx_data.tx, (deposit, TransactionState::Valid));
            }
            Transaction::Withdrawal(ref withdrawal) => {
                if withdrawal.amount <= self.available {
                    self.available -= withdrawal.amount;
                } else {
                    return Err(NegativeBalance(transaction, self.available));
                }
            }
            Transaction::Dispute(ref dispute) => {
                self.ammend_tx(
                    dispute.tx,
                    transaction.action_description().into(),
                    TransactionState::Valid,
                    TransactionState::Disputed,
                    |client, amount| {
                        client.available -= amount;
                        client.held += amount;
                    },
                )?;
            }
            Transaction::Resolve(ref resolve) => {
                self.ammend_tx(
                    resolve.tx,
                    transaction.action_description().into(),
                    TransactionState::Disputed,
                    TransactionState::Valid,
                    |client, amount| {
                        client.available += amount;
                        client.held -= amount;
                    },
                )?;
            }
            Transaction::ChargeBack(ref chargeback) => {
                self.ammend_tx(
                    chargeback.tx,
                    transaction.action_description().into(),
                    TransactionState::Disputed,
                    TransactionState::ChargedBack,
                    |client, amount| {
                        client.held -= amount;
                        client.locked = true;
                    },
                )?;
            }
        };
        Ok(())
    }

    fn ammend_tx<F>(
        &mut self,
        tx_to_ammend: u32,
        ammendment_type: &str,
        expected_state: TransactionState,
        final_state: TransactionState,
        ammendment: F,
    ) -> Result<(), TransactionApplyError>
    where
        F: Fn(&mut Self, Money),
    {
        let client_id = self.id;
        let amount = self.find_and_ammend_tx(
            tx_to_ammend,
            ammendment_type,
            client_id,
            expected_state,
            final_state,
        )?;
        ammendment(self, amount);
        Ok(())
    }

    fn find_and_ammend_tx(
        &mut self,
        tx_to_ammend: u32,
        ammendment_type: &str,
        client_id: u16,
        expected_state: TransactionState,
        final_state: TransactionState,
    ) -> Result<Money, TransactionApplyError> {
        let (target_tx, state) = {
            self.deposit_transactions
                .get_mut(&tx_to_ammend)
                .ok_or_else(|| TransactionApplyError::MissingTransaction {
                    attemped_action: ammendment_type.into(),
                    client: client_id,
                    tx: tx_to_ammend,
                })?
        };
        if expected_state == *state {
            *state = final_state;
            Ok(target_tx.amount)
        } else {
            Err(TransactionApplyError::InvalidTransactionState {
                attemped_action: ammendment_type.into(),
                client: client_id,
                tx: tx_to_ammend,
                current_state: *state,
                expected_state,
            })
        }
    }
}

#[derive(Error, Debug)]
pub enum TransactionApplyError {
    #[error("negative balance not allowed, tx <{0:?}>, current available balance: {1}")]
    NegativeBalance(Transaction, Money),

    #[error("locked clients can't process transactions {0:?}")]
    ClientLocked(Transaction),

    #[error("attemped {attemped_action} on missing transaction: client {client} doesn't have transaction {tx}")]
    MissingTransaction {
        attemped_action: String,
        client: u16,
        tx: u32,
    },

    #[error("invalid transaction state for {attemped_action}: transaction {tx} for client {client} state is {current_state:?} but should be {expected_state:?}")]
    InvalidTransactionState {
        attemped_action: String,
        client: u16,
        tx: u32,
        current_state: TransactionState,
        expected_state: TransactionState,
    },
}
