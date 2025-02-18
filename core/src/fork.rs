// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//-
// Copyright 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Write as _;
use std::net::TcpListener;
use std::net::TcpStream;
use std::panic;
use std::process;
use std::process::Child;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;
use std::process::Termination;

use crate::cmdline;
use crate::error::Result;


const OCCURS_ENV: &str = "TEST_FORK_OCCURS";
const OCCURS_TERM_LENGTH: usize = 17; /* ':' plus 16 hexits */


/// Simulate a process fork.
///
/// Since this is not a true process fork, the calling code must be structured
/// to ensure that the child process, upon starting from the same entry point,
/// also reaches this same `fork()` call. Recursive forks are supported; the
/// child branch is taken from all child processes of the fork even if it is
/// not directly the child of a particular branch. However, encountering the
/// same fork point more than once in a single execution sequence of a child
/// process is not (e.g., putting this call in a recursive function) and
/// results in unspecified behaviour.
///
/// `fork_id` is a unique identifier identifying this particular fork location.
/// This *must* be stable across processes of the same executable; pointers are
/// not suitable stable, and string constants may not be suitably unique. The
/// [`fork_id!()`] macro is the recommended way to supply this
/// parameter.
///
/// `test_name` must exactly match the full path of the test function being
/// run.
///
/// If `test` panics, the child process exits with a failure code immediately
/// rather than let the panic propagate out of the `fork()` call.
///
/// ## Panics
///
/// Panics if the environment indicates that there are already at least 16
/// levels of fork nesting.
///
/// Panics if `std::env::current_exe()` fails determine the path to the current
/// executable.
///
/// Panics if any argument to the current process is not valid UTF-8.
pub fn fork<F, T>(fork_id: &str, test_name: &str, test: F) -> Result<()>
where
    // NB: We use `Fn` here, because `FnMut` and `FnOnce` would allow
    //     for modification of captured variables, but that will not
    //     work across process boundaries.
    F: Fn() -> T,
    T: Termination,
{
    fn supervise_child(child: &mut Child) {
        let status = child.wait().expect("failed to wait for child");
        assert!(
            status.success(),
            "child exited unsuccessfully with {}",
            status
        );
    }

    fn no_configure_child(_child: &mut Command) {}

    fork_int(
        test_name,
        fork_id,
        no_configure_child,
        supervise_child,
        test,
    )
}

/// Simulate a process fork.
///
/// This function is similar to [`fork`], except that it allows for data
/// exchange with the child process.
pub fn fork_in_out<F, T>(fork_id: &str, test_name: &str, test: F, data: &mut [u8]) -> Result<()>
where
    F: Fn(&mut [u8]) -> T,
    T: Termination,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind TCP socket");
    let addr = listener.local_addr().unwrap();
    let data_len = data.len();

    fork_int(
        test_name,
        fork_id,
        |cmd| {
            cmd.env(fork_id, addr.to_string());
        },
        |child| {
            let (mut stream, _addr) = listener
                .accept()
                .expect("failed to listen for child connection");
            let () = stream
                .write_all(data)
                .expect("failed to send data to child");
            let () = stream
                .read_exact(data)
                .expect("failed to receive data from child");
            let status = child.wait().expect("failed to wait for child");
            assert!(
                status.success(),
                "child exited unsuccessfully with {}",
                status
            );
        },
        || {
            let addr = env::var(fork_id).unwrap_or_else(|err| {
                panic!("failed to retrieve {fork_id} environment variable: {err}")
            });
            let mut stream =
                TcpStream::connect(addr).expect("failed to establish connection with parent");

            let mut data = Vec::with_capacity(data_len);
            // SAFETY: The `Vec` contains `data_len` `u8` values, which
            //         are valid for any bit pattern, so we can safely
            //         adjust the length.
            let () = unsafe { data.set_len(data_len) };

            let () = stream
                .read_exact(&mut data)
                .expect("failed to receive data from parent");
            let status = test(&mut data);
            let () = stream
                .write_all(&data)
                .expect("failed to send data to parent");
            status
        },
    )
}

pub(crate) fn fork_int<M, P, C, R, T>(
    test_name: &str,
    fork_id: &str,
    process_modifier: M,
    in_parent: P,
    in_child: C,
) -> Result<R>
where
    M: FnOnce(&mut process::Command),
    P: FnOnce(&mut Child) -> R,
    T: Termination,
    C: FnOnce() -> T,
{
    // Erase the generics so we don't instantiate the actual implementation for
    // every single test
    let mut return_value = None;
    let mut process_modifier = Some(process_modifier);
    let mut in_parent = Some(in_parent);
    let mut in_child = Some(in_child);

    fork_impl(
        test_name,
        fork_id,
        &mut |cmd| process_modifier.take().unwrap()(cmd),
        &mut |child| return_value = Some(in_parent.take().unwrap()(child)),
        &mut || in_child.take().unwrap()(),
    )
    .map(|_| return_value.unwrap())
}

fn fork_impl<T: Termination>(
    test_name: &str,
    fork_id: &str,
    process_modifier: &mut dyn FnMut(&mut process::Command),
    in_parent: &mut dyn FnMut(&mut Child),
    in_child: &mut dyn FnMut() -> T,
) -> Result<()> {
    let mut occurs = env::var(OCCURS_ENV).unwrap_or_else(|_| String::new());
    if occurs.contains(fork_id) {
        match panic::catch_unwind(panic::AssertUnwindSafe(in_child)) {
            Ok(test_result) => {
                let rc = if test_result.report() == ExitCode::SUCCESS {
                    0
                } else {
                    70
                };
                process::exit(rc)
            }
            // Assume that the default panic handler already printed something
            //
            // We don't use process::abort() since it produces core dumps on
            // some systems and isn't something more special than a normal
            // panic.
            Err(_) => process::exit(70 /* EX_SOFTWARE */),
        }
    } else {
        // Prevent misconfiguration creating a fork bomb
        if occurs.len() > 16 * OCCURS_TERM_LENGTH {
            panic!("test-fork: Not forking due to >=16 levels of recursion");
        }

        struct KillOnDrop(Child);
        impl Drop for KillOnDrop {
            fn drop(&mut self) {
                // Kill the child if it hasn't exited yet
                let _ = self.0.kill();

                // Copy the child's output to our own
                // Awkwardly, `print!()` and `println!()` are our only gateway
                // to putting things in the captured output. Generally test
                // output really is text, so work on that assumption and read
                // line-by-line, converting lossily into UTF-8 so we can
                // println!() it.

                fn drain(read: &mut dyn Read, stderr: bool) {
                    let mut buf = Vec::new();
                    let mut br = io::BufReader::new(read);
                    loop {
                        // We can't use read_line() or lines() since they break if
                        // there's any non-UTF-8 output at all. \n occurs at the
                        // end of the line endings on all major platforms, so we
                        // can just use that as a delimiter.
                        if br.read_until(b'\n', &mut buf).is_err() {
                            break;
                        }
                        if buf.is_empty() {
                            break;
                        }

                        // not println!() because we already have a line ending
                        // from above.
                        let s = String::from_utf8_lossy(&buf);
                        if stderr {
                            eprint!("{s}");
                        } else {
                            print!("{s}");
                        }
                        buf.clear();
                    }
                }

                if let Some(stdout) = self.0.stdout.as_mut() {
                    let () = drain(stdout, false);
                }

                if let Some(stderr) = self.0.stderr.as_mut() {
                    let () = drain(stderr, true);
                }
            }
        }

        occurs.push_str(fork_id);
        let mut command =
            process::Command::new(env::current_exe().expect("current_exe() failed, cannot fork"));
        command
            .args(cmdline::strip_cmdline(env::args())?)
            .args(cmdline::RUN_TEST_ARGS)
            .arg(test_name)
            .env(OCCURS_ENV, &occurs)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        process_modifier(&mut command);

        let mut child = command.spawn().map(KillOnDrop)?;
        let () = in_parent(&mut child.0);

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use std::thread;

    use crate::fork_id;


    fn sleep(ms: u64) {
        thread::sleep(::std::time::Duration::from_millis(ms));
    }

    fn capturing_output(cmd: &mut process::Command) {
        cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());
    }

    fn inherit_output(cmd: &mut process::Command) {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    fn wait_for_child_output(child: &mut Child) -> String {
        let mut output = String::new();
        child
            .stdout
            .as_mut()
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();
        assert!(child.wait().unwrap().success());
        output
    }

    fn wait_for_child(child: &mut Child) {
        assert!(child.wait().unwrap().success());
    }

    #[test]
    fn fork_basically_works() {
        let status = fork_int(
            "fork::test::fork_basically_works",
            fork_id!(),
            |_| (),
            |child| child.wait().unwrap(),
            || println!("hello from child"),
        )
        .unwrap();
        assert!(status.success());
    }

    #[test]
    fn child_output_captured_and_repeated() {
        let output = fork_int(
            "fork::test::child_output_captured_and_repeated",
            fork_id!(),
            capturing_output,
            wait_for_child_output,
            || {
                fork_int(
                    "fork::test::child_output_captured_and_repeated",
                    fork_id!(),
                    |_| (),
                    wait_for_child,
                    || println!("hello from child"),
                )
                .unwrap()
            },
        )
        .unwrap();
        assert!(output.contains("hello from child"));
    }

    #[test]
    fn child_killed_if_parent_exits_first() {
        let output = fork_int(
            "fork::test::child_killed_if_parent_exits_first",
            fork_id!(),
            capturing_output,
            wait_for_child_output,
            || {
                fork_int(
                    "fork::test::child_killed_if_parent_exits_first",
                    fork_id!(),
                    inherit_output,
                    |_| (),
                    || {
                        sleep(100);
                        println!("hello from child");
                    },
                )
                .unwrap()
            },
        )
        .unwrap();

        sleep(200);
        assert!(
            !output.contains("hello from child"),
            "Had unexpected output:\n{}",
            output
        );
    }

    #[test]
    fn child_killed_if_parent_panics_first() {
        let output = fork_int(
            "fork::test::child_killed_if_parent_panics_first",
            fork_id!(),
            capturing_output,
            wait_for_child_output,
            || {
                assert!(panic::catch_unwind(panic::AssertUnwindSafe(|| fork_int(
                    "fork::test::child_killed_if_parent_panics_first",
                    fork_id!(),
                    inherit_output,
                    |_| panic!("testing a panic, nothing to see here"),
                    || {
                        sleep(100);
                        println!("hello from child");
                    }
                )
                .unwrap()))
                .is_err());
            },
        )
        .unwrap();

        sleep(200);
        assert!(
            !output.contains("hello from child"),
            "Had unexpected output:\n{}",
            output
        );
    }

    #[test]
    fn child_aborted_if_panics() {
        let status = fork_int::<_, _, _, _, ()>(
            "fork::test::child_aborted_if_panics",
            fork_id!(),
            |_| (),
            |child| child.wait().unwrap(),
            || panic!("testing a panic, nothing to see here"),
        )
        .unwrap();
        assert_eq!(70, status.code().unwrap());
    }

    /// Check that we can exchange data with the child process.
    #[test]
    fn data_exchange() {
        let mut data = [1, 2, 3, 4, 5];

        let () = fork_in_out(
            fork_id!(),
            "fork::test::data_exchange",
            |data| {
                assert_eq!(data.len(), 5);
                let () = data.iter_mut().for_each(|x| *x += 1);
            },
            data.as_mut_slice(),
        )
        .unwrap();

        assert_eq!(data, [2, 3, 4, 5, 6]);
    }
}
