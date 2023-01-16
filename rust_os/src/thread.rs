use crate::{println, set_psr, Registers};
use core::arch::asm;

const USER_MEM_SIZE: usize = 0x2_000_000 - 5 * 64 * 1024 - core::mem::size_of::<ThreadList>();
const THREAD_NUMBER: usize = 16;
const USER_STACK_SIZE: usize = USER_MEM_SIZE / 2 / THREAD_NUMBER;

#[link_section = ".kernel.thread_array"]
pub static mut THREADS: ThreadList = ThreadList {
    array: [None; THREAD_NUMBER],
    curr_thread: 0,
};

#[link_section = ".user_mem"]
pub static USER_MEM: () = ();

pub type ID = usize;
type PSR = u32;

#[derive(Copy, Clone, Debug)]
pub enum State {
    Running,
    Ready,
    Sleeping(u32),
    WaitingForChar,
}

#[derive(Copy, Clone, Debug)]
pub struct Thread {
    pub id: ID,
    pub state: State,
    pub regs: Registers,
    pub psr: PSR,
    // TODO: make ThreadList iterable over links
    next_thread: Option<ID>,
}

impl Thread {
    pub fn can_schedule(&self) -> bool {
        match self.state {
            State::Ready => self.id != 0,
            _ => false,
        }
    }
}

pub struct ThreadList {
    pub array: [Option<Thread>; THREAD_NUMBER],
    pub curr_thread: ID,
}

impl ThreadList {
    /// Add a thread to the ThreadList. Returns a Result that contains the threads id.
    /// The Registers pc and arguments need to be initialized beforehand
    pub fn create_thread(&mut self, mut regs: Registers) -> Result<ID, &'static str> {
        let id = match self
            .array
            .iter()
            .enumerate()
            .find(|(_, thread)| thread.is_none())
        {
            Some((id, _)) => id,
            None => return Err("Can't create new thread. The list of threads is full."),
        };
        regs.sp = (&USER_MEM as *const () as usize + USER_STACK_SIZE * (id + 1)) as u32;
        regs.lr = super::util::exit as u32;
        let new_thread = Thread {
            id,
            state: State::Ready,
            psr: 0x1F, // User Mode
            regs,
            next_thread: None,
        };
        self.array[id] = Some(new_thread);
        // newly created threads get a headstart -> could lead to exploitation
        self.set_curr_thread(id);
        Ok(id)
    }

    pub fn curr_thread(&self) -> &Thread {
        self.get_thread(self.curr_thread).unwrap()
    }

    pub fn curr_mut_thread(&mut self) -> &mut Thread {
        self.get_mut_thread(self.curr_thread).unwrap()
    }

    pub fn set_curr_thread(&mut self, id: ID) {
        if let Some(old_thread) = self.get_mut_thread(id) {
            old_thread.state = State::Ready
        }
        self.curr_thread = id;
        self.curr_mut_thread().state = State::Running
    }

    fn _schedule_next(&self) -> ID {
        // First look into the slice after the current_thread
        let start = self.curr_thread().next_thread;
        if let Some(_start) = start && let Some(thread) = self.array[_start..].iter().find_map(|v| match v.as_ref() {
            Some(thread) => if thread.can_schedule() {Some(thread)} else {None},
            None => None
        }) {
            return thread.id
        }
        // Now look through the whole list
        if let Some(thread) = self.array.iter().find_map(|v| match v.as_ref() {
            Some(thread) => {
                if thread.can_schedule() {
                    Some(thread)
                } else {
                    None
                }
            }
            None => None,
        }) {
            return thread.id;
        }
        // Else return the idle thread
        0
    }

    /// Schedules the next thread to run
    pub fn schedule_next(&mut self) -> ID {
        let id = self._schedule_next();
        self.set_curr_thread(id);
        id
    }

    pub fn get_thread(&self, id: ID) -> Option<&Thread> {
        match self.array.get(id) {
            Some(element) => element.as_ref(),
            None => None,
        }
    }

    pub fn get_mut_thread(&mut self, id: ID) -> Option<&mut Thread> {
        match self.array.get_mut(id) {
            Some(element) => element.as_mut(),
            None => None,
        }
    }

    pub fn end_thread(&mut self, id: ID) {
        if id == 0 {
            println!("Ignored request to delete idle thread");
            return;
        } else if self.curr_thread == id {
            self.schedule_next();
        }
        self.array[id] = None;
    }

    pub fn save_state(&mut self, regs: &mut Registers) {
        let thread = self.curr_mut_thread();
        thread.regs = regs.clone();
        crate::get_psr!(a = spsr);
        thread.psr = a;
    }

    pub fn put_state(&mut self, regs: &mut Registers) {
        let thread = self.curr_thread();
        regs.clone_from(&thread.regs);
        let psr = thread.psr;
        set_psr!(spsr = psr);
    }
}

pub fn get_threads() -> &'static mut ThreadList {
    unsafe { &mut THREADS }
}
