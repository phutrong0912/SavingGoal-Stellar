
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, String,
};

// Lưu trữ thông tin mục tiêu tiết kiệm của một người dùng.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavingGoal {
    pub name: String,
    pub target: i128,
    pub saved: i128,
    pub withdrawn: bool,
    pub created_at: u64,
}

// Khóa lưu trữ dữ liệu theo địa chỉ người dùng.
#[contracttype]
pub enum DataKey {
    Goal(Address),
}

// Các lỗi có thể xảy ra trong smart contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    GoalAlreadyOpen = 1,
    InvalidName = 2,
    InvalidTarget = 3,
    GoalNotFound = 4,
    InvalidDeposit = 5,
    GoalAlreadyWithdrawn = 6,
    TargetNotReached = 7,
    AmountOverflow = 8,
}

#[contract]
pub struct SavingGoalContract;

#[contractimpl]
impl SavingGoalContract {
    // Tạo mục tiêu tiết kiệm mới cho người dùng.
    pub fn create_goal(
        env: Env,
        user: Address,
        name: String,
        target: i128,
    ) -> Result<(), Error> {
        user.require_auth();

        if name.len() == 0 {
            return Err(Error::InvalidName);
        }

        if target <= 0 {
            return Err(Error::InvalidTarget);
        }

        let key = DataKey::Goal(user.clone());

        if let Some(existing) =
            env.storage().persistent().get::<DataKey, SavingGoal>(&key)
        {
            if !existing.withdrawn {
                return Err(Error::GoalAlreadyOpen);
            }
        }

        let goal = SavingGoal {
            name,
            target,
            saved: 0,
            withdrawn: false,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&key, &goal);

        Ok(())
    }

    // Nạp thêm tiền vào mục tiêu tiết kiệm.
    pub fn deposit(
        env: Env,
        user: Address,
        amount: i128,
    ) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidDeposit);
        }

        let key = DataKey::Goal(user.clone());

        let mut goal: SavingGoal =
            match env.storage().persistent().get(&key) {
                Some(value) => value,
                None => return Err(Error::GoalNotFound),
            };

        if goal.withdrawn {
            return Err(Error::GoalAlreadyWithdrawn);
        }

        goal.saved = match goal.saved.checked_add(amount) {
            Some(value) => value,
            None => return Err(Error::AmountOverflow),
        };

        env.storage().persistent().set(&key, &goal);

        Ok(())
    }

    // Rút toàn bộ số tiền đã tiết kiệm khi đạt mục tiêu.
    pub fn withdraw(
        env: Env,
        user: Address,
    ) -> Result<i128, Error> {
        user.require_auth();

        let key = DataKey::Goal(user.clone());

        let mut goal: SavingGoal =
            match env.storage().persistent().get(&key) {
                Some(value) => value,
                None => return Err(Error::GoalNotFound),
            };

        if goal.withdrawn {
            return Err(Error::GoalAlreadyWithdrawn);
        }

        if goal.saved < goal.target {
            return Err(Error::TargetNotReached);
        }

        let amount = goal.saved;

        goal.withdrawn = true;

        env.storage().persistent().set(&key, &goal);

        Ok(amount)
    }

    // Cho phép mọi người xem thông tin mục tiêu tiết kiệm.
    pub fn get_goal(
        env: Env,
        user: Address,
    ) -> Result<SavingGoal, Error> {
        let key = DataKey::Goal(user);

        match env.storage().persistent().get(&key) {
            Some(goal) => Ok(goal),
            None => Err(Error::GoalNotFound),
        }
    }
}
