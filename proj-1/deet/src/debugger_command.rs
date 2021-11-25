pub enum DebuggerCommand {
    Quit,
    Run(Vec<String>),
    ContinueRun,
    BackTrace,
    Break(String),
}

pub fn parse_address(addr: &str) -> Option<usize> {
    let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
        &addr[2..]
    } else {
        &addr
    };
    usize::from_str_radix(addr_without_0x, 16).ok()
}


impl DebuggerCommand {

    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            },
            "c" | "cont" | "continue" => Some(DebuggerCommand::ContinueRun),
            "bt" | "back" | "backtrace" => Some(DebuggerCommand::BackTrace),
            "b" | "break" => { Some(DebuggerCommand::Break(tokens[1].to_string()))

                // let args=tokens[1];
                // if let Some(_address)=parse_address(&args[1..]){
                //     Some(DebuggerCommand::Break(_address))
                // }else{
                //     None
                // }
            } 
            // Default case:
            _ => None,
        }
    }

}
