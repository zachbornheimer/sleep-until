src/main.rs: composition root; wires the system adapters into the product operation
src/app.rs: the product operation; one command line in, one exit status out
src/cli.rs: pure parsing of the command line into a request
src/interval.rs: pure parsing of GNU `NUMBER[SUFFIX]` intervals
src/time_of_day.rs: validated local time of day and its parser
src/wait.rs: the waiting workflow; observe the target, decide a plan, execute it
src/ports.rs: narrow capabilities the workflow needs (clock, calendar, sleeper)
src/system.rs: the only adapters; real clock, C-library calendar, thread sleeper
src/failure.rs: every way slp can fail, with its message
