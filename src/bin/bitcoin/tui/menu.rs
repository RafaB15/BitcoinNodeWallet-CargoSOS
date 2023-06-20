use super::{error_tui::ErrorTUI, menu_option::MenuOption};

use cargosos_bitcoin::logs::logger_sender::LoggerSender;

use std::io::stdin;

fn print_menu() {
    let options: &[MenuOption] = &[
        MenuOption::CreateAccount,
        MenuOption::ChangeAccount,
        MenuOption::RemoveAccount,
        MenuOption::SendTransaction,
        MenuOption::ShowAccounts,
        MenuOption::ShowBalance,
        MenuOption::LastTransactions,
        MenuOption::Exit,
    ];

    for option in options {
        let option_id: char = (*option).into();
        println!("{option} [{option_id}]");
    }
}

pub fn select_option(logger: LoggerSender) -> Result<MenuOption, ErrorTUI> {
    println!("Select an option:");
    print_menu();

    let mut option: String = String::new();
    if stdin().read_line(&mut option).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        let _: MenuOption = match MenuOption::try_from(option.trim()) {
            Ok(result) => return Ok(result),
            Err(error) => {
                let _ =
                    logger.log_wallet(format!("Put an invalid option, with error: {:?}", error));

                option.clear();
                println!("Error, please enter a valid option:");
                print_menu();
                if stdin().read_line(&mut option).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }
                continue;
            }
        };
    }
}
