use super::Registers;

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

type ID = usize;
type PSR = u32;

#[derive(Copy, Clone, Debug)]
pub enum State {
    Running,
    Ready,
    Sleeping(u32),
}

#[derive(Copy, Clone, Debug)]
pub struct Thread {
    pub id: ID,
    pub state: State,
    pub regs: Registers,
    pub psr: PSR,
    next_thread: Option<ID>,
}

pub struct ThreadList {
    array: [Option<Thread>; THREAD_NUMBER],
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
        let is_idle_thread = self.curr_thread == 0 && self.get_thread(0).is_none();
        regs.sp = (&USER_MEM as *const () as usize + USER_STACK_SIZE * (id + 1)) as u32;
        regs.lr = super::util::exit as u32;
        let new_thread = Thread {
            id,
            state: if is_idle_thread {
                State::Running
            } else {
                State::Ready
            },
            psr: 0x1F, // User Mode
            regs,
            next_thread: None,
        };
        self.array[id] = Some(new_thread);
        if is_idle_thread {
            self.curr_thread = id;
        } else {
            self.get_thread(id - 1).unwrap().next_thread = Some(id);
        }
        Ok(id)
    }

    pub fn get_curr_thread(&mut self) -> &mut Thread {
        self.get_thread(self.curr_thread).unwrap()
    }

    pub fn schedule_next(&mut self) -> ID {
        // Wir haben immer einen current_thread
        let thread = self.get_curr_thread();
        if let Some(next_thread) = thread.next_thread {
            self.curr_thread = next_thread;
        } else {
            match self.idle_thread().next_thread {
                Some(thread) => self.curr_thread = thread,
                None => self.curr_thread = 0,
            }
        }
        self.curr_thread
    }

    pub fn get_thread(&mut self, id: ID) -> Option<&mut Thread> {
        match self.array.get_mut(id) {
            Some(element) => element.as_mut(),
            None => None,
        }
    }

    pub fn idle_thread(&mut self) -> &Thread {
        self.get_thread(0).unwrap()
    }

    pub fn end_thread(&mut self, id: ID) {
        if id == 0 {
            return;
        }
        self.schedule_next();
        self.array[id] = None;
    }
}

pub fn get_threads() -> &'static mut ThreadList {
    unsafe { &mut THREADS }
}

pub fn save_state(regs: &mut Registers) {
    let threads = get_threads();
}
