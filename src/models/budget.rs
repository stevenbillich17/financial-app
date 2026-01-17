use rust_decimal::Decimal;

#[derive(Debug)]
pub struct CategoryBudget {
    pub id: i32,
    pub category: String,
    pub amount: Decimal,
}
