/// A register struct
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
struct Thread {
    id: ID,
    state: State,
    regs: Registers,
    next_thread: Option<ID>,
}

struct ThreadList {
    array: [Option<Thread>;16],
    curr_thread: Option<ID>,
}

impl ThreadList {
    /// Add a thread to the thread_list. Returns a Result that contains the threads id. 
    pub fn create_thread(&mut self) -> Result<ID, &'static str> {
        // TODO: take a function with arguments and pass it to the thread
        let last_thread = self.array.iter().filter_map(|&option| option).find(|thread| thread.next_thread.is_none()).unwrap();
        let id = last_thread.id+1;
        if id >= self.array.len() {
            return Err("Can't create new thread. The list of threads is full.");
        }
        let new_thread = Thread{
            id,
            state: State::Ready,
            regs: Registers::empty(),
            next_thread: None
        };
        self.array[id] = Some(new_thread);
        self.curr_thread = Some(id);
        Ok(id)
    }

    pub fn get_curr_thread(&mut self) -> Option<&Thread> {
        if let Some(thread_id) = self.curr_thread {
            self.array.get(thread_id).unwrap().as_ref()
        } else{
            None
        }
    }

    pub fn schedule_next(&mut self) {
        if let Some(thread) = self.get_curr_thread() {
            if thread.next_thread.is_some() {
                self.curr_thread = thread.next_thread;
            } else {
                self.curr_thread = self.array.iter().find()
            }
        }
    }
}

#[link_section = ".kernel.thread_array"]
static mut THREADS: ThreadList = ThreadList {
    array: [None;16],
    curr_thread: None
};