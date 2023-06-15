use super::error_tui::ErrorTUI;

use std::{
    convert::{From, TryFrom},
    fmt::Display,
};

#[derive(Debug, Clone, Copy)]
pub enum MenuOption {
    CreateAccount,
    ChangeAccount,
    SendTransaction,
    ShowBalance,
    ShowAccounts,
    Exit,
}

const CREATE_ACCOUNT: char = '1';
const CHANGE_ACCOUNT: char = '2';
const SEND_TRANSACTION: char = '3';
const SHOW_BALANCE: char = '4';
const SHOW_ACCOUNTS: char = '5';
const EXIT: char = '6';

impl Display for MenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuOption::CreateAccount => write!(f, "Create account"),
            MenuOption::ChangeAccount => write!(f, "Change account"),
            MenuOption::SendTransaction => write!(f, "Send transaction"),
            MenuOption::ShowBalance => write!(f, "Show balance"),
            MenuOption::ShowAccounts => write!(f, "Show accounts"),
            MenuOption::Exit => write!(f, "Exit"),
        }
    }
}

impl From<MenuOption> for char {
    fn from(value: MenuOption) -> Self {
        match value {
            MenuOption::CreateAccount => CREATE_ACCOUNT,
            MenuOption::ChangeAccount => CHANGE_ACCOUNT,
            MenuOption::SendTransaction => SEND_TRANSACTION,
            MenuOption::ShowBalance => SHOW_BALANCE,
            MenuOption::ShowAccounts => SHOW_ACCOUNTS,
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
            SHOW_BALANCE => Ok(MenuOption::ShowBalance),
            SHOW_ACCOUNTS => Ok(MenuOption::ShowAccounts),
            EXIT => Ok(MenuOption::Exit),
            _ => Err(ErrorTUI::InvalidMenuOption),
        }
    }
}
