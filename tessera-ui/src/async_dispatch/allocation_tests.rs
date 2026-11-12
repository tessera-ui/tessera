use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

struct CountingAllocator;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = COUNTING.try_with(|counting| {
            if counting.get() {
                let _ = ALLOCATIONS.try_with(|total| total.set(total.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}

fn count_allocations(run: impl FnOnce()) -> usize {
    ALLOCATIONS.with(|total| total.set(0));
    COUNTING.with(|counting| counting.set(true));
    run();
    COUNTING.with(|counting| counting.set(false));
    ALLOCATIONS.with(Cell::get)
}

#[test]
fn frame_waits_resume_every_frame_without_allocating() {
    let session = crate::RuntimeSession::new();
    session.run(|| {
        let frames = Rc::new(Cell::new(0_u32));
        let counter = Rc::clone(&frames);
        let task = super::spawn_local(async move {
            loop {
                super::next_frame().await;
                counter.set(counter.get() + 1);
            }
        });
        let origin = Instant::now();
        let mut frame = 0_u64;
        let mut tick = || {
            frame += 1;
            crate::runtime::begin_frame_clock(origin + Duration::from_millis(frame * 16));
            crate::runtime::tick_frame_clock();
        };
        for _ in 0..4 {
            tick();
        }
        let warmed = frames.get();
        assert!(warmed >= 2, "the task resumes once per frame");
        let allocations = count_allocations(|| {
            for _ in 0..100 {
                tick();
            }
        });
        assert_eq!(frames.get(), warmed + 100);
        assert_eq!(allocations, 0, "steady frame waits must not allocate");
        drop(task);
        tick();
        assert!(
            !super::has_frame_waiters(),
            "a cancelled task stops waiting"
        );
    });
}

struct Ping {
    waker: Arc<Mutex<Option<Waker>>>,
    pending: bool,
}

impl Future for Ping {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.pending {
            self.pending = false;
            return Poll::Ready(());
        }
        self.pending = true;
        let mut slot = self.waker.lock().expect("waker slot");
        match slot.as_mut() {
            Some(waker) => waker.clone_from(context.waker()),
            None => *slot = Some(context.waker().clone()),
        }
        Poll::Pending
    }
}

#[test]
fn woken_tasks_poll_without_allocating() {
    let session = crate::RuntimeSession::new();
    session.run(|| {
        let waker = Arc::new(Mutex::new(None::<Waker>));
        let polls = Rc::new(Cell::new(0_u32));
        let (slot, counter) = (Arc::clone(&waker), Rc::clone(&polls));
        let _task = super::spawn_local(async move {
            loop {
                Ping {
                    waker: Arc::clone(&slot),
                    pending: false,
                }
                .await;
                counter.set(counter.get() + 1);
            }
        });
        let wake = || {
            waker
                .lock()
                .expect("waker slot")
                .as_ref()
                .expect("the task registered its waker")
                .wake_by_ref();
            crate::dispatcher::flush();
        };
        crate::dispatcher::flush();
        for _ in 0..4 {
            wake();
        }
        let warmed = polls.get();
        let allocations = count_allocations(|| {
            for _ in 0..100 {
                wake();
            }
        });
        assert_eq!(polls.get(), warmed + 100);
        assert_eq!(
            allocations, 0,
            "waking and polling a task must not allocate"
        );
    });
}
