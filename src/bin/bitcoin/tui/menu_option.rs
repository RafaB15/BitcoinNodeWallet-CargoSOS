use super::error_tui::ErrorTUI;

use std::{
    convert::{From, TryFrom},
    fmt::Display,
};

#[derive(Debug, Clone, Copy)]
pub enum MenuOption {
    CreateAccount,
    ChangeAccount,
    RemoveAccount,
    SendTransaction,
    ShowAccounts,
    ShowBalance,
    LastTransactions,
    Exit,
}

const CREATE_ACCOUNT: char = '1';
const CHANGE_ACCOUNT: char = '2';
const REMOVE_ACCOUNT: char = '3';
const SEND_TRANSACTION: char = '4';
const SHOW_ACCOUNTS: char = '5';
const SHOW_BALANCE: char = '6';
const LAST_TRANSACTIONS: char = '7';
const EXIT: char = '8';

impl Display for MenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuOption::CreateAccount => write!(f, "Create account"),
            MenuOption::ChangeAccount => write!(f, "Change account"),
            MenuOption::RemoveAccount => write!(f, "Remove account"),
            MenuOption::SendTransaction => write!(f, "Send transaction"),
            MenuOption::ShowAccounts => write!(f, "Show accounts"),
            MenuOption::ShowBalance => write!(f, "Show balance"),
            MenuOption::LastTransactions => write!(f, "Last transactions"),
            MenuOption::Exit => write!(f, "Exit"),
        }
    }
}

impl From<MenuOption> for char {
    fn from(value: MenuOption) -> Self {
        match value {
            MenuOption::CreateAccount => CREATE_ACCOUNT,
            MenuOption::ChangeAccount => CHANGE_ACCOUNT,
            MenuOption::RemoveAccount => REMOVE_ACCOUNT,
            MenuOption::SendTransaction => SEND_TRANSACTION,
            MenuOption::ShowAccounts => SHOW_ACCOUNTS,
            MenuOption::ShowBalance => SHOW_BALANCE,
            MenuOption::LastTransactions => LAST_TRANSACTIONS,
            MenuOption::Exit => EXIT,
        }
    }
}

impl TryFrom<&str> for MenuOption {
    type Error = ErrorTUI;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value: char = match value.chars().next() {
            Some(value) => value,
            _ => return Err(ErrorTUI::InvalidMenuOption),
        };

        match value {
            CREATE_ACCOUNT => Ok(MenuOption::CreateAccount),
            CHANGE_ACCOUNT => Ok(MenuOption::ChangeAccount),
            SEND_TRANSACTION => Ok(MenuOption::SendTransaction),
            SHOW_ACCOUNTS => Ok(MenuOption::ShowAccounts),
            SHOW_BALANCE => Ok(MenuOption::ShowBalance),
            LAST_TRANSACTIONS => Ok(MenuOption::LastTransactions),
            EXIT => Ok(MenuOption::Exit),
            _ => Err(ErrorTUI::InvalidMenuOption),
        }
    }
}
