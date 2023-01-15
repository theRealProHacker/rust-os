use super::Registers;

const USER_MEM_SIZE: usize = 0x2_000_000 - 5 * 64 * 1024 - core::mem::size_of::<ThreadList>();
const THREAD_NUMBER: usize = 16;
const USER_STACK_SIZE: usize = USER_MEM_SIZE / 2 / THREAD_NUMBER;

#[link_section = ".kernel.thread_array"]
pub static mut THREADS: ThreadList = ThreadList {
    array: [None; THREAD_NUMBER],
    curr_thread: None,
};

#[link_section = ".user_mem"]
pub static USER_MEM: [u32; 0] = [0; 0];

type ID = usize;

#[derive(Copy, Clone)]
enum State {
    Running,
    Ready,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Thread {
    id: ID,
    state: State,
    pub regs: Registers,
    next_thread: Option<ID>,
}

pub struct ThreadList {
    array: [Option<Thread>; 16],
    curr_thread: Option<ID>,
}

impl ThreadList {
    /// Add a thread to the thread_list. Returns a Result that contains the threads id.
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
        // Whether we should instantly run the thread
        let run_thread = self.get_curr_thread().is_none();
        regs.sp = (USER_MEM.as_ptr() as usize + USER_STACK_SIZE * (id + 1)) as u32;
        let new_thread = Thread {
            id,
            state: if run_thread {
                State::Running
            } else {
                State::Ready
            },
            regs,
            next_thread: None,
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
            None => None,
        }
    }

    pub fn schedule_next(&mut self) -> Result<ID, &'static str> {
        if let Some(thread) = self.get_curr_thread() {
            if thread.next_thread.is_some() {
                self.curr_thread = thread.next_thread;
            } else {
                let (new_thread, _) = self
                    .array
                    .iter()
                    .enumerate()
                    .find(|(_, x)| x.is_some())
                    .unwrap();
                self.curr_thread = Some(new_thread);
            }
            Ok(self.curr_thread.unwrap())
        } else {
            Err("No thread could be scheduled because there are no threads ready")
        }
    }
}
