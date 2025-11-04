use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug)]
pub enum TransactionType {
    Income,
    Expense
}

#[derive(Debug)]
pub struct Transaction {
    pub id: u64,
    pub date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    pub category: String,
}

impl Transaction {
    pub fn new(id: u64, date: NaiveDate, description: String, amount: Decimal, transaction_type: TransactionType, category: String) -> Self {
        Self {
            id,
            date,
            description,
            amount,
            transaction_type,
            category,
        }
    }
}