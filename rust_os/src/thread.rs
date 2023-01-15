/// A register struct
#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Registers {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub sp: u32,
    pub lr: u32,
    pub pc: u32
}

impl Registers {
    /// Generates empty registers initialized at 0
    pub fn empty() -> Registers {
        unsafe { core::mem::transmute([0;16])}
    }

    pub fn from_regs() -> Registers {
        // IDEA: just push everything to the stack and then get the array from the current stackpointer upwards and cast it as Registers
        Registers::empty()
    }
}

type ID = usize;

#[derive(Copy, Clone)]
enum State {
    Running,
    Ready,
}

#[derive(Copy, Clone)]
pub struct Thread {
    id: ID,
    state: State,
    pub regs: Registers,
    next_thread: Option<ID>,
}

pub struct ThreadList {
    array: [Option<Thread>;16],
    curr_thread: Option<ID>,
}

impl ThreadList {
    /// Add a thread to the thread_list. Returns a Result that contains the threads id. 
    pub fn create_thread(&mut self, regs: Registers) -> Result<ID, &'static str> {
        let id = match self.array.iter().enumerate().find(|(_,thread)| thread.is_none()) {
            Some((id,_)) => id,
            None => return Err("Can't create new thread. The list of threads is full.")
        };
        // Whether we should instantly run the thread
        let run_thread = self.get_curr_thread().is_none();
        let new_thread = Thread{
            id,
            state: if run_thread {State::Running} else {State::Ready},
            regs,
            next_thread: None
        };
        self.array[id] = Some(new_thread);
        if run_thread {
            self.curr_thread = Some(id);
        }
        Ok(id)
    }

    pub fn get_curr_thread(&mut self) -> Option<&mut Thread> {
        match self.curr_thread {
            Some(thread_id) => self.array.get_mut(thread_id).unwrap().as_mut(),
            None => None
        }
    }

    pub fn schedule_next(&mut self) -> Result<ID, &'static str> {
        if let Some(thread) = self.get_curr_thread() {
            if thread.next_thread.is_some() {
                self.curr_thread = thread.next_thread;
            } else {
                let (new_thread,_) = self.array.iter().enumerate().find(|(_,x)| x.is_some()).unwrap();
                self.curr_thread = Some(new_thread);
            }
            Ok(self.curr_thread.unwrap())
        } else {
            Err("No thread could be scheduled because there are no threads ready")
        }
    }
}



#[link_section = ".kernel.thread_array"]
pub static mut THREADS: ThreadList = ThreadList {
    array: [None;16],
    curr_thread: None
};