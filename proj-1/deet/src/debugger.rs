use crate::debugger_command::{DebuggerCommand,parse_address};
use crate::inferior::{Inferior,Status,BreakPoint};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::dwarf_data::{DwarfData, Error as DwarfError};
use std::collections::HashMap;


pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data :DwarfData,
    break_list:HashMap<usize,BreakPoint>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        debug_data.print();

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data:debug_data,
            break_list:HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            // println!{"run loop"};
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if let Some(child)=self.inferior.as_mut(){      // kill any existing inferiors
                        if let Ok(_)= child.kill(){
                            println!("kill ok when re-run");
                        }else{
                            println!("kill failed when re-run");
                        }
                        child.wait(None).ok();
                    }

                    if let Some(inferior) = Inferior::new(&self.target, &args,&mut self.break_list) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        let child=self.inferior.as_mut().unwrap();
                        match child.wakeup(&self.break_list){
                            Ok(status)=> match status{
                                Status::Stopped(sig,instruction_ptr)=> {
                                    println!("Child stopped by signal {}",sig);
                                    if let Some(filename_line) = self.debug_data.get_line_from_addr(instruction_ptr as usize){
                                        if let Some(function_name)=self.debug_data.get_function_from_addr(instruction_ptr as usize){
                                            println!("Stopped   {}    ({}:{}) ",function_name,filename_line.file,filename_line.number);
                                        }
                                    }
                                },
                                Status::Exited(code)=> {println!("Child exited (status {})",code);},
                                Status::Signaled(sig) => {println!("Child signaled signal {}",sig);},
                            },
                            Err(e)=>{
                                println!("err {}",e);
                            },
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                },
                DebuggerCommand::Quit => {
                    if let Some(child)=self.inferior.as_mut(){
                        if let Ok(_)= child.kill(){
                            println!("kill exist child when quit");
                        }
                        child.wait(None).ok();
                    }
                    println!("quit");
                    return;
                },
                DebuggerCommand::ContinueRun => {
                    if let Some(child)=self.inferior.as_mut(){
                        match child.wakeup(&self.break_list){
                            Ok(status)=> match status{
                                Status::Stopped(sig,instruction_ptr)=> {
                                    println!("Child stopped signal {}",sig);
                                    if let Some(filename_line) = self.debug_data.get_line_from_addr(instruction_ptr as usize){
                                        if let Some(function_name)=self.debug_data.get_function_from_addr(instruction_ptr as usize){
                                            println!("Stopped   {}    ({}:{}) ",function_name,filename_line.file,filename_line.number);
                                        }
                                    }
                                },
                                Status::Exited(code)=> {println!("Continue Child exited (status {})",code);},
                                Status::Signaled(sig) => {println!("Continue Child signaled signal {}",sig);},
                            },
                            Err(e)=>{
                                println!("err {}",e);
                            },
                        };
                    }else{
                        println!("no inferior to continue!");
                    }
                },
                DebuggerCommand::BackTrace=>{
                    if let Some(child)=self.inferior.as_mut(){
                        match child.print_backtrace(&self.debug_data){
                            Ok(_)=>{},
                            Err(_)=>{},
                        };
                    }else{
                        println!("no inferior when backtrace!");
                    }
                },
                DebuggerCommand::Break(args)=>{
                    let mut total_address=0;
                    if &args[0..1]=="*"{
                        if let Some(_address)=parse_address(&args[1..]){
                            total_address=_address;
                        }else{
                            println!("Invalid Address");
                        }   
                    }else if let Some(_line_number)=args.parse::<usize>().ok(){
                        if let Some(_address)=self.debug_data.get_addr_for_line(None,_line_number){
                            println!("line number address {} {} ",_line_number,_address);
                            total_address=_address;
                        }
                    }else{
                        if let Some(_address)=self.debug_data.get_addr_for_function(None, &args){
                            total_address=_address;
                            println!("function number address {} {} ",args,_address);
                        }else{
                            println!("Invalid Address");
                        }
                    }

                    println!("0x{:x}",total_address);
                    if let Some(child)=self.inferior.as_mut(){ // inferior stopped ,insert directly
                        if let Some(orig_instr)=child.write_byte(total_address,0xcc as u8).ok(){
                            let break_list_len=self.break_list.len();
                            self.break_list.insert(total_address,BreakPoint{
                                address:total_address,
                                orig_byte:orig_instr,
                            });
                            println!("Set breakpoint {} at 0x{:x}",break_list_len,total_address);
                        }else{
                            println!("Write Memory Error:inferior write byte on invalid address 0x{:x}",total_address);
                        }
                    }else{  // inferior not run ,insert when inferior::new()
                        let break_list_len=self.break_list.len();
                        self.break_list.insert(total_address,BreakPoint{
                            address:total_address,
                            orig_byte:0,
                        });
                        println!("Set breakpoint {} at 0x{:x}",break_list_len,total_address);
                    }
                },
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        // println!("get_next_command");
        loop {
            // Print prompt and get next line of user input
            // println!("get_next_command loop");

            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    // println!("Eof");
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    // println!("???????????????{}",line);
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    println!("{:?}",tokens);
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        // println!("next command is {:?}",tokens);
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}

