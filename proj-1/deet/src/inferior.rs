use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::Child;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::io;
use crate::dwarf_data::{DwarfData, /*Error as DwarfError*/};
use std::mem::size_of;
use std::collections::HashMap;


pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
}

#[derive(Clone)]
pub struct BreakPoint{
    pub address:usize,
    pub orig_byte:u8,
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>,break_list:&mut HashMap<usize,BreakPoint>) -> Option<Inferior> {
        // TODO: implement me!
        // println!("new inter {:?} {:?}",target,args);
        let mut child_no_spawn_args= Command::new(target);
        child_no_spawn_args.args(args);
        unsafe{
            child_no_spawn_args.pre_exec(child_traceme);
        }

        let child_spawn=child_no_spawn_args.spawn().ok()?;
        let mut _ret=Inferior{
            child:child_spawn,
        };
 
        let status=_ret.wait(None).ok()?;
        match status{
            Status::Stopped(sign,_)=>{
                match sign{
                    signal::Signal::SIGTRAP => {
                        let bp = break_list.clone();
                        for addr in bp.keys() {
                            match _ret.write_byte(*addr, 0xcc as u8).ok() {
                                Some(orig_instr) => { break_list.insert(*addr, BreakPoint{ address: *addr, orig_byte: orig_instr }); },
                                None => {println!("Write Memory Error:inferior write byte on invalid address 0x{:x}",*addr);},
                            }
                        }          
                        return Some(_ret);}
                    _=>{return None;}
                };
            }
            _=>{return None;}
        }
    }
    pub fn print_backtrace(&self,debug_data:&DwarfData) -> Result<(), nix::Error>{
        if let Ok(_r)= ptrace::getregs(self.getpid()){

            let mut instruction_ptr=_r.rip;
            let mut base_ptr=_r.rbp;
            loop{
                // println!("%rip register: {:#x}", _rip);
                if let Some(filename_line) = debug_data.get_line_from_addr(instruction_ptr as usize){
                    if let Some(function_name)=debug_data.get_function_from_addr(instruction_ptr as usize){
                        println!("{}    ({}:{}) ",function_name,filename_line.file,filename_line.number);
                        if function_name==String::from("main"){
                            break;
                        }
                        instruction_ptr=ptrace::read(self.pid(), (base_ptr+8) as ptrace::AddressType)? as u64;
                        base_ptr = ptrace::read(self.pid(), base_ptr as ptrace::AddressType)? as u64;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn getpid(&self) -> Pid{
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    pub fn wakeup(&mut self,break_list:&HashMap<usize,BreakPoint>) -> Result<Status, nix::Error>{
        if let Ok(mut _r)= ptrace::getregs(self.getpid()){
            let instruction_ptr=_r.rip as usize;
            if let Some(breakpoint)= break_list.get(&(instruction_ptr-1)){
                self.write_byte(breakpoint.address, breakpoint.orig_byte).ok();
                _r.rip=(_r.rip-1) as u64;
                ptrace::setregs(self.pid(), _r).ok();
                ptrace::step(self.pid(),None).ok();
                
                match self.wait(None).ok().unwrap() {
                    Status::Exited(exit_code) => return Ok(Status::Exited(exit_code)),
                    Status::Signaled(signal) => return Ok(Status::Signaled(signal)),
                    Status::Stopped(_, _) => {
                        self.write_byte(instruction_ptr - 1, 0xcc as u8).ok();
                    }
                }
            }

        }
        ptrace::cont(self.getpid(), None).ok();
        self.wait(None)
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    pub fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }
}
