use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TransactionType {
    Income,
    Expense
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    pub category: String,
}

impl Transaction {
    pub fn new(id: String, date: NaiveDate, description: String, amount: Decimal, transaction_type: TransactionType, category: String) -> Self {
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