use crate::Money;

#[derive(Debug)]
pub enum Transaction {
    Deposit(AmountTransaction),
    Withdrawal(AmountTransaction),
    Dispute(TransactionData),
    Resolve(TransactionData),
    ChargeBack(TransactionData),
}

#[derive(Debug)]
pub struct TransactionData {
    pub client: u16,
    pub tx: u32,
}

#[derive(Debug)]
pub struct AmountTransaction {
    pub tx_data: TransactionData,
    pub amount: Money,
}

impl Transaction {
    pub fn action_description(&self) -> &'static str {
        match self {
            Transaction::Deposit(_) => "deposit",
            Transaction::Withdrawal(_) => "withdrawal",
            Transaction::Dispute(_) => "dispute",
            Transaction::Resolve(_) => "resolve",
            Transaction::ChargeBack(_) => "chargeback",
        }
    }
}
