//! Threads and a thread list which includes scheduling

use crate::{consts::THREAD_NUMBER, println, thread, util, Registers};
use core::arch::asm;

/// The idle thread function
#[inline(always)]
pub fn idle() {
    loop {
        // Docs: "Emits a machine instruction to signal the processor that it is running in a busy-wait spin-loop ("spin lock").
        // Upon receiving the spin-loop signal the processor can optimize its behavior by, for example,
        // saving power or switching hyper-threads."
        // Unfortunately however, this yields an undefined instruction exception
        // core::hint::spin_loop();
        util::nop()
    }
}

pub static mut THREADS: ThreadList = ThreadList {
    array: [
        None, None, None, None, None, None, None, None, 
        None, None, None, None, None, None, None, None,
    ],
    curr_thread: 0,
};

/// A threads State. To get whether a thread in ready state is actually running,
/// check whether its id matches the ThreadLists curr_thread
#[derive(Debug)]
pub enum State {
    Ready,
    Sleeping(u32),
    WaitingForChar,
}

/// A Thread-ID. Is always also an index into the ThreadList array
pub type ID = usize;
type ThreadArray = [Option<Thread>; crate::THREAD_NUMBER];

#[derive(Debug)]
pub struct Thread {
    pub id: ID,
    pub state: State,
    pub regs: Registers,
    pub psr: u32,
    next_thread: Option<ID>,
}

/// Gets the global ThreadList
#[inline(always)]
pub fn get_threads() -> &'static mut ThreadList {
    unsafe { &mut THREADS }
}

#[derive(Debug)]
pub struct ThreadList {
    pub array: ThreadArray,
    pub curr_thread: ID,
}

// TODO: make ThreadList iterable over links
impl ThreadList {
    /// Initializes the ThreadList so it can be used safely.
    /// This mostly means initializing the idle thread.
    #[inline(always)]
    pub fn init(&mut self) -> &mut Self {
        // The idle thread can run in SYS_MODE
        self.array[0] = Some(Thread {
            id: 0,
            state: State::Ready,
            regs: thread!(idle()),
            psr: crate::SYS_MODE,
            next_thread: None,
        });
        self
    }

    pub fn iter_from(&self, f: fn(&Thread) -> Option<ID>) -> ThreadListIter {
        let get_id = || {
            let _id = f(self.curr_thread())?;
            if _id >= self.array.len() {
                None
            } else {
                Some(_id)
            }
        };
        ThreadListIter {
            curr_thread: get_id(),
            thread_list: self,
        }
    }

    pub fn iter<'a>(&'a self) -> ThreadListIter<'a> {
        ThreadListIter {
            curr_thread: Some(self.curr_thread),
            thread_list: self,
        }
    }

    pub fn iter_mut(&mut self) -> ThreadListIterMut {
        ThreadListIterMut {
            curr_thread: Some(self.curr_thread),
            ptr: self.array.as_mut_ptr(),
        }
    }

    /// Add a thread to the ThreadList. Returns a Result that contains the threads id.
    /// The Registers pc and arguments need to be initialized beforehand.
    pub fn create_thread(&mut self, mut regs: Registers) -> Result<ID, &'static str> {
        // We go through all threads up to the semi last one
        // If the next thread position is free then we set the hosting thread
        // (the thread directly before the thread to be created)
        let id: ID = 0;
        // Rust makes me want to do pointer arithmetic again
        let pointer = self.array.as_ptr();
        for thread in self.iter_mut() {
            let next_id = thread.id + 1;
            if next_id >= THREAD_NUMBER {
                return Err("Couldn't create new thread. Thread array is full");
            }
            if unsafe { &*(pointer.offset(next_id as isize)) }
                .as_ref()
                .is_none()
            {
                regs.sp = (crate::USER_MEM - crate::USER_STACK_SIZE * next_id) as u32;
                regs.lr = util::exit as u32; // Should jump back to exit
                let new_thread = Thread {
                    id: next_id,
                    state: State::Ready,
                    psr: crate::USR_MODE,
                    regs,
                    next_thread: thread.next_thread,
                };
                thread.next_thread = Some(next_id);
                self.array[next_id] = Some(new_thread);
                return Ok(next_id);
            }
        }
        if id == 0 {
            panic!("Couldn't find any thread in ThreadList");
        } else {
            loop {}
        }
    }

    #[inline(always)]
    fn _schedule_next(&self) -> ID {
        fn can_schedule(t: &&Thread) -> bool {
            match t.state {
                State::Ready => t.id != 0,
                _ => false,
            }
        }
        // First look into the slice after curr_thread
        if let Some(thread) = self.iter_from(|t| t.next_thread).find(can_schedule) {
            return thread.id;
        }
        // Now look through the whole list
        if let Some(thread) = self.iter().find(can_schedule) {
            return thread.id;
        }
        // else return the idle thread
        0
    }

    /// Schedules the next thread to run
    pub fn schedule_next(&mut self) -> ID {
        let id = self._schedule_next();
        println!("Scheduled thread {:#?}", self.get_thread(id));
        self.curr_thread = id;
        id
    }

    /// Get a reference to the current thread
    #[inline(always)]
    pub fn curr_thread(&self) -> &Thread {
        self.get_thread(self.curr_thread).unwrap()
    }

    /// Get a mutable reference to the current thread
    #[inline(always)]
    pub fn curr_mut_thread(&mut self) -> &mut Thread {
        self.get_mut_thread(self.curr_thread).unwrap()
    }

    /// Get the thread with the given ID
    #[inline(always)]
    pub fn get_thread(&self, id: ID) -> Option<&Thread> {
        match self.array.get(id) {
            Some(element) => element.as_ref(),
            None => None,
        }
    }

    /// Get a mutable reference to the thread with the given ID
    #[inline(always)]
    fn get_mut_thread(&mut self, id: ID) -> Option<&mut Thread> {
        match self.array.get_mut(id) {
            Some(element) => element.as_mut(),
            None => None,
        }
    }

    /// Ends the thread with the given id if possible (the idle thread can never be deleted)
    #[inline(always)]
    pub fn end_thread(&mut self, id: ID) -> Option<()> {
        if id == 0 {
            panic!("Tried to end idle thread")
        }
        let next_thread = self.get_thread(id)?.next_thread;
        // Now we can delete the current thread
        self.array[id] = None;
        // The thread that points towards the deleted thread
        let thread_before = self
            .iter_mut()
            .find(|t| t.next_thread == Some(id))
            .expect("Couldn't find a thread that points towards the deleted thread");
        thread_before.next_thread = next_thread;
        let thread_before_id = thread_before.id;
        if self.curr_thread == id {
            self.curr_thread = thread_before_id;
        }
        Some(())
    }

    /// Saves the context from the given regs and the spsr to the current threads regs
    #[inline(always)]
    pub fn save_state(&mut self, regs: &Registers) {
        if self.curr_thread == 0 {
            // We don't save any state into the idle thread
            return;
        }
        let thread = self.curr_mut_thread();
        thread.regs = regs.clone();
        crate::get_psr!(a = spsr);
        thread.psr = a;
    }

    /// Writes the current threads context run into regs and the spsr
    #[inline(always)]
    pub fn put_state(&mut self, regs: &mut Registers) {
        let thread = self.curr_thread();
        regs.clone_from(&thread.regs);
        let psr = thread.psr;
        crate::set_psr!(spsr = psr);
    }
}

pub struct ThreadListIter<'a> {
    curr_thread: Option<ID>,
    thread_list: &'a ThreadList,
}

impl<'a> Iterator for ThreadListIter<'a> {
    type Item = &'a Thread;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.curr_thread?;
        let thread = self.thread_list.get_thread(id)?;
        self.curr_thread = thread.next_thread;
        Some(thread)
    }
}

pub struct ThreadListIterMut {
    curr_thread: Option<ID>,
    ptr: *mut Option<Thread>,
}

impl Iterator for ThreadListIterMut {
    type Item = &'static mut Thread;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.curr_thread?;
        let thread = unsafe { (*self.ptr.offset(id as isize)).as_mut() }?;
        self.curr_thread = thread.next_thread;
        Some(thread)
    }
}
