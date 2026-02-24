# Tasks
The server needs an actual task system to support future tasks. Tasks will likely be a required feature if a calendar
is added, especially if a calendar hooks into a refactored reminder feature. More tasks will also be needed for
maintenance, like cleaning up old logs.

## Requirements
A "task" will have the following requirements
- If the task is recurring, this will need to be defined somewhere
- Hold arbitrary data
  - A task writing logs will have no overlap in data with one that handles calendar events
- Reports or stores task success/failure somewhere
  - This will also want to be cleaned up so tasks don't endlessly write data

Additionally, tasks will need to be managed by some sort of task manager or scheduler. This manager should be able to
receive a request to run a non-recurring task and must handle running recurring tasks at the correct time. The ability
for endpoints to begin a non-recurring task means that shared state will need to have a reference to this task
scheduler, so they can call its methods.

## Task
Each task can use its own struct. An example is the existing log creation task, which looks like
```rust
struct LogCreationTask {
    method: String,
    uri: String,
    user_id: String,
    date_time: i64,
}
```

## Method One - Dyn Trait
Shared behavior for tasks can exist in a `Task` trait, utilized as a trait object. A trait object (not just a trait)
will be necessary so the object can be passed across channels. This trait might look like
```rust
pub trait Task {
    fn run_task(& self, db_handle: &PatDatabase) -> Pin<Box<dyn Future<Output=Result<(), ()>>>>;
}
```

This trait can then be implemented on a struct;
```rust
impl Task for LogCreationTask {
  fn run_task(& self, db_handle: &PatDatabase) -> Pin<Box<dyn Future<Output=Result<(), ()>>>> {
    let doc: Document = (); // Format data for log generation
    let future = async move {
      match cloned_ref.insert_one::<Log>(doc).await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
      }
    };
    Box::pin(future)
  }
}
```

The `run_task` method looks strange because a dyn compatible function cannot be async, so we need to return
a boxed future which will be handled by the task manager. This boxed future must be pinned so the memory
does not move during polling.

Using trait objects, shared behavior for a task can be defined while still using arbitrary task data.

A channel can be created and stored in app state where data can be sent to/from a task management thread.

### Task Manger
The only current task works by spawning a tokio task (a "non blocking unit of execution") which endlessly loops.
This loop begins by waiting an interval, does some work, and then resets the loop. This instantiation can create a
task manager instead of a log creation task. The task manager can run every few seconds, check for work to do, perform
its work, and then go back to waiting. The current log creation task will become a recurring task that the manager
will run when it wakes, if the right amount of time has passed.

Communication with the task manager can be done through a channel, and the task manager can check for recurring tasks
by storing a task configuration struct. This config struct can be generated at app setup from deserializing a task
config file. The task manager will need to map tasks from this config to task methods to call, and keep an internal
clock to track the passage of time so it knows when to call a recurring task.

### Positives
The task/thread generation logic is short and simple with this route, and the code outside the task manager will be
simple and easy to read.

### Negatives
The task manager will potentially be a complicated struct and the dyn trait may cause problems. Dyn traits have special
restrictions and are less performant than a static approach, the `Pin<Box<dyn Trait>>` channel will also not impl `Send`
by default, and it will need to implement this to be a part of the app state. This is an unsafe trait.

This route will also make testing much more complicated. A testing API which advances time on the task manager opens
a large can of worms. If we advance time by a month to test recurring task A, how many times should A be run? What
about other recurring tasks, do they all get run?

## Method Two - Static Task Manager and Channels
Recurring tasks can exist in a series of Tokio tasks with channels allowing bidirectional communication. A task manager
can hold on to references allowing sending/receiving data to/from tasks. These tasks can use a Tokio `select!` macro
to run when either the recurrence interval passes or when manually triggered from some channel.

If manually triggered by a channel, the task will need to also return data on another channel. This will allow a test
which triggers the task to be able to `.await` on a future which allows it to know when the task is done running.

A task might look like
```rust
task::spawn(async move {
    let mut interval = time::interval(Duration::from_secs(5));
    interval.tick().await; // The first tick of an interval resolves instantly
    let mut should_respond = false;
    loop {
        // This select! allows the task to be run manually so we don't need to wait in a test
        tokio::select! {
            _ = interval.tick() => {}
            _ = task_receiver_channel.changed() => {
                // Reset the tick interval as the task was manually called some portion of time
                // into the wait
                interval.reset();
                should_respond = true;
            }
        }
        
        // Perform task work here

        // This is kind of hacky?
        if should_respond {
            let _ = task_response_sender_channel.send("Jobs done");
            should_respond = false;
        }
    }
});
```

And the task manager would look like:
```rust
pub struct TaskManager {
    log_creation_send_channel: Sender<&'static str>,
    log_creation_receive_channel: Receiver<&'static str>,
}

impl TaskManager {
    pub fn new(
        log_creation_send_channel: Sender<&'static str>,
        log_creation_receive_channel: Receiver<&'static str>,
    ) -> Self {
        Self {
            log_creation_send_channel,
            log_creation_receive_channel,
        }
    }
}

impl TaskManager {
    pub async fn run_logs_task(&mut self) {
        // Manually run the recurring task to generate logs and wait for it to finish
        let _ = self.log_creation_send_channel.send("run task");
        let _ = self.log_creation_receive_channel.changed().await;
    }
}
```

### Positives
This route is much simpler with its logic and more efficiently leverages Tokio. Testing recurring tasks becomes as easy
as calling some `task_manager.run_task().await`.

### Negatives
This route involves more boilerplate and will require creation of some task manager struct which needs to hold on to
a bunch of channels. Each recurring task will require storage of two channels (One to send a request to begin the task,
one to receive confirmation that the task finished). Maybe tasks can be stored in something like a hashmap?
