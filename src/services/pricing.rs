use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct PricingComponents {
    pub ex_showroom: Decimal,
    pub insurance: Decimal,
    pub registration: Decimal,
    pub accessories_total: Decimal,
    pub handling_charges: Decimal,
    pub discount: Decimal,
    pub subsidy_amount: Decimal,
}

impl PricingComponents {
    /// Computes the exact on-road price:
    /// ex_showroom + insurance + registration + accessories + handling - discount - subsidy
    pub fn calculate_on_road_total(&self) -> Decimal {
        let additions = self.ex_showroom
            + self.insurance
            + self.registration
            + self.accessories_total
            + self.handling_charges;

        let deductions = self.discount + self.subsidy_amount;

        if additions > deductions {
            additions - deductions
        } else {
            Decimal::ZERO
        }
    }
}

/// Computes remaining balance due: total_amount - total_payments
pub fn calculate_balance_due(total_amount: Decimal, total_payments: Decimal) -> Decimal {
    if total_amount > total_payments {
        total_amount - total_payments
    } else {
        Decimal::ZERO
    }
}
