use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StakePosition {
    pub staker: String,
    pub validator: String,
    pub amount: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EconomyState {
    pub treasury_account: String,
    pub balances: BTreeMap<String, u128>,
    pub stakes: Vec<StakePosition>,
}

impl Default for EconomyState {
    fn default() -> Self {
        let treasury_account = "AOXC_TREASURY".to_string();
        let mut balances = BTreeMap::new();
        balances.insert(treasury_account.clone(), 0);

        Self {
            treasury_account,
            balances,
            stakes: Vec::new(),
        }
    }
}

impl EconomyState {
    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }

        let bytes = fs::read(path).map_err(|e| format!("failed to read state: {e}"))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("failed to decode state: {e}"))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create state dir: {e}"))?;
        }

        let bytes =
            serde_json::to_vec_pretty(self).map_err(|e| format!("failed to encode state: {e}"))?;
        fs::write(path, bytes).map_err(|e| format!("failed to write state: {e}"))
    }

    pub fn ensure_account(&mut self, account: &str) {
        self.balances.entry(account.to_string()).or_insert(0);
    }

    pub fn mint_to_treasury(&mut self, amount: u128) {
        let balance = self
            .balances
            .entry(self.treasury_account.clone())
            .or_insert(0);
        *balance = balance.saturating_add(amount);
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u128) -> Result<(), String> {
        if amount == 0 {
            return Err("amount must be > 0".to_string());
        }

        self.ensure_account(from);
        self.ensure_account(to);

        let from_balance = self.balances.get_mut(from).expect("account exists");
        if *from_balance < amount {
            return Err(format!("insufficient balance in {from}"));
        }
        *from_balance -= amount;

        let to_balance = self.balances.get_mut(to).expect("account exists");
        *to_balance = to_balance.saturating_add(amount);
        Ok(())
    }

    pub fn delegate(&mut self, staker: &str, validator: &str, amount: u128) -> Result<(), String> {
        if amount == 0 {
            return Err("stake amount must be > 0".to_string());
        }

        self.ensure_account(staker);
        let staker_balance = self.balances.get_mut(staker).expect("account exists");
        if *staker_balance < amount {
            return Err(format!("insufficient balance in {staker}"));
        }
        *staker_balance -= amount;

        if let Some(position) = self
            .stakes
            .iter_mut()
            .find(|p| p.staker == staker && p.validator == validator)
        {
            position.amount = position.amount.saturating_add(amount);
        } else {
            self.stakes.push(StakePosition {
                staker: staker.to_string(),
                validator: validator.to_string(),
                amount,
            });
        }

        Ok(())
    }

    pub fn undelegate(
        &mut self,
        staker: &str,
        validator: &str,
        amount: u128,
    ) -> Result<(), String> {
        if amount == 0 {
            return Err("unstake amount must be > 0".to_string());
        }

        let Some(position) = self
            .stakes
            .iter_mut()
            .find(|p| p.staker == staker && p.validator == validator)
        else {
            return Err("stake position not found".to_string());
        };

        if position.amount < amount {
            return Err("stake position too small".to_string());
        }

        position.amount -= amount;
        self.stakes.retain(|p| p.amount > 0);
        self.ensure_account(staker);
        let staker_balance = self.balances.get_mut(staker).expect("account exists");
        *staker_balance = staker_balance.saturating_add(amount);
        Ok(())
    }

    pub fn total_staked(&self) -> u128 {
        self.stakes.iter().map(|s| s.amount).sum()
    }

    pub fn treasury_balance(&self) -> u128 {
        self.balances
            .get(&self.treasury_account)
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::EconomyState;

    #[test]
    fn delegation_roundtrip_works() {
        let mut state = EconomyState::default();
        state.mint_to_treasury(1_000);
        state
            .transfer("AOXC_TREASURY", "alice", 300)
            .expect("treasury transfer should work");

        state
            .delegate("alice", "validator-1", 200)
            .expect("delegate should work");
        assert_eq!(state.total_staked(), 200);

        state
            .undelegate("alice", "validator-1", 50)
            .expect("undelegate should work");
        assert_eq!(state.total_staked(), 150);
    }
}
