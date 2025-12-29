use core::cell::SyncUnsafeCell;
use core::arch::asm;

pub static SCHEDULER: SyncUnsafeCell<Scheduler> = SyncUnsafeCell::new(Scheduler::new());

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TaskContext {
    // Callee-saved registers
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rsp: u64,

    // Instruction pointer
    rip: u64
}

impl TaskContext {
    pub const fn new() -> Self {
        TaskContext {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rsp: 0,
            rip: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Dead
}

#[derive(Copy, Clone)]
pub struct Task {
    pub id: usize,
    pub context: TaskContext,
    pub stack_pointer: u64,
    pub stack_base: u64,
    pub state: TaskState,
    pub priority: u8
}

impl Task {
    pub fn new(
        id: usize,
        entry_point: fn() -> !,
        stack_base: u64,
        stack_size: usize,
    ) -> Self 
    {
        let stack_top = stack_base + stack_size as u64;

        // Set up initial stack frame
        let mut context = TaskContext::new();
        context.rip = entry_point as u64;

        unsafe {
            let stack_ptr = stack_top as *mut u64;

            // Dummy return address (tasks never return)
            *stack_ptr.offset(-1) = 0xDEADEEF
        }

        // one byte to account dummy return address
        context.rsp = (stack_top - 8) as u64;

        Task {
            id,
            context,
            stack_pointer: stack_top,
            stack_base,
            state: TaskState::Ready,
            priority: 1
        }
    }
}

const MAX_TASKS: usize = 32;

pub struct Scheduler {
    pub tasks: [Option<Task>; MAX_TASKS],
    current_task: usize,
    next_task_id: usize,
    first_empty_slot: usize,
    kernel_context: TaskContext
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            tasks: [None; MAX_TASKS],
            current_task: 0,
            next_task_id: 0,
            first_empty_slot: 0,
            kernel_context: TaskContext::new()
        }
    }

    pub fn return_to_kernel(&mut self) {
        let current_idx = self.current_task;
        
        unsafe {
            if let Some(current_task) = &mut self.tasks[current_idx] {
                current_task.state = TaskState::Ready;
                
                // This jumps back to kernel_dispatcher loop
                switch_context(&mut current_task.context, &self.kernel_context);
            }
        }
    }

    // kernel main loop - handles interrupts and schedules tasks
    pub fn kernel_dispatcher(&mut self) -> ! {
        // Initialize kernel context to return here

        self.kernel_context.rip = kernel_loop as *const () as u64;
        self.kernel_context.rsp = 0; // will be set on first return

        kernel_loop();

        fn kernel_loop() -> ! {
            loop {
                let scheduler = unsafe { &mut *SCHEDULER.get() };

                // Schedule next task
                if let Some(next_idx) = scheduler.schedule() {
                    scheduler.current_task = next_idx;

                    if let Some(next_task) = &mut scheduler.tasks[next_idx] {
                        next_task.state = TaskState::Running;

                        unsafe {
                            switch_context(&mut scheduler.kernel_context, &next_task.context);
                        }
                        // Returns here when task yields or is preempted
                    }
                } else {
                    // No tasks ready, halt until interrupt
                    unsafe {
                        asm!("hlt", options(nomem, nostack, preserves_flags));
                    }
                }
            }
        }
        
    }

    pub fn add_task(&mut self, entry_point: fn() -> !, stack: u64, stack_size: usize) -> Option<usize> {
        for i in self.first_empty_slot..MAX_TASKS {
            if self.tasks[i].is_none() {
                return self.allocate_task(i, entry_point, stack, stack_size);
            }
        }

        for i in MAX_TASKS..self.first_empty_slot {
            if self.tasks[i].is_none() {
                return self.allocate_task(i, entry_point, stack, stack_size);
            }
        }

        None
    }

    fn allocate_task(&mut self, position: usize, entry_point: fn() -> !, stack: u64, stack_size: usize) -> Option<usize> {
        let task = Task::new(self.next_task_id, entry_point, stack, stack_size);
        self.next_task_id += 1;
        let id = task.id;
        self.tasks[position] = Some(task);
        self.first_empty_slot = (position + 1) % MAX_TASKS;

        Some(id)
    }

    pub fn start(&mut self) {
        if let Some(task) = &mut self.tasks[self.current_task] {
            task.state = TaskState::Running;
        }
    }

    pub fn schedule(&mut self) -> Option<usize> {
        let start = self.current_task;
        let mut next = (start + 1) % MAX_TASKS;

        loop {
            if let Some(task) = &self.tasks[next]
                && (task.state == TaskState::Ready || task.state == TaskState::Running) {
                
                    return Some(next)
            }

            next = (next + 1) % MAX_TASKS;
            if next == start {
                break;
            }
        }

        None
    }

    pub fn switch_task(&mut self) -> bool {
        if let Some(next_idx) = self.schedule() {
            if next_idx == self.current_task {
                return false; // No switch needed
            }

            // Mark current as ready (if it was running)
            if let Some(current) = &mut self.tasks[self.current_task]
                && current.state == TaskState::Running {

                    current.state = TaskState::Ready;
            }

            // Mark next as running
            if let Some(next) = &mut self.tasks[next_idx] {
                next.state = TaskState::Running;
            }

            let old_idx = self.current_task;
            self.current_task = next_idx;

            // Perform context switch
            //
            unsafe {
                let tasks_ptr = self.tasks.as_mut_ptr();
                
                if let (Some(old_task), Some(new_task)) = 
                    (&mut *tasks_ptr.add(old_idx), &mut *tasks_ptr.add(next_idx)) {
                    switch_context(&mut old_task.context, &new_task.context);
                }
            }

            true
        } else {
            false
        }
    }

    pub fn get_current_task(&self) -> Option<&Task> {
        self.tasks[self.current_task].as_ref()
    }

    pub fn kill_current_task(&mut self) {
        if let Some(task) = &mut self.tasks[self.current_task] {
            task.state = TaskState::Dead;
        }

        self.switch_task();
    }
}


unsafe extern "C" {
    fn switch_context(old: *mut TaskContext, new: *const TaskContext);
}
