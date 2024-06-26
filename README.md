# Seph

A job scheduler for a single machine.

Built because I needed a better way to run multiple gpu-using experiments one after the other.

- Just running in a terminal: Have to be at the PC when the jobs finish.
- A script that runs all the experiments: Can't edit (add, cancel, reorder) the set of experiments after starting.

```bash
seph run "echo \$PWD > pwd"
cat pwd

seph run "echo \$PWD"
seph output <job_id>
```

## Design

The design is a wrapper around a (not-yet priority) queue.
A background worker pops a job off the queue when it finished running the current job.
`seph run` is append.

## Install from source

```bash
git clone https://github.com/marktuddenham/seph.git
cd seph

cargo build --release
sudo mv ./target/release/seph-daemon /usr/bin/
sudo chown root:root /usr/bin/seph-daemon
```

To install the terminal tool for just this user.
```bash
cargo install --bin seph --path cmd
```

or for all users
```bash
sudo mv ./target/release/seph /usr/bin
```

## Running the daemon

You can setup the daemon as a `systemd` service.

In `/etc/systemd/system/seph.service`

```systemd
[Unit]
Description=Seph Daemon

[Service]
ExecStart=/usr/bin/seph-daemon

[Install]
WantedBy=multi-user.target
```

> [!CAUTION]
> Running the Seph daemon as root allows *any* user to run *any* command as *any* other user, including root -- without sudo.

## Feature list

- [x] Add jobs
- [x] Get output of jobs
- [x] Get streamed output of jobs (eqiv of `tail -f log`)
- [ ] Cancelling jobs
- [ ] Listing running/ran jobs
- [ ] Schedule a job to run multiple times
    - [ ] Abort options for the remaining jobs in a muti-run job if one errors
- [ ] Reordering jobs
- [x] Capture user's environment
- [ ] Request resources, e.g. number of GPUs
- [ ] Add priorities to jobs

## Security

There's probably a bunch of security holes, especially if you run the daemon as root.
