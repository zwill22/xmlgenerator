use crate::error::XSDValidationError;
use libc::{STDERR_FILENO, c_int, close, dup, dup2, pipe};

pub(crate) struct WarningHandler {
    saved_stderr: Option<c_int>,
    pipe_fd: Option<[c_int; 2]>,
    pipe_read: usize,
    pipe_write: usize,
}

impl WarningHandler {
    pub(crate) fn new() -> WarningHandler {
        WarningHandler {
            saved_stderr: None,
            pipe_fd: None,
            pipe_read: 0,
            pipe_write: 1,
        }
    }

    pub(crate) fn redirect(&mut self) -> Result<(), XSDValidationError> {
        let saved_stderr = unsafe { dup(STDERR_FILENO) };

        if saved_stderr == -1 {
            return Err(XSDValidationError::OutputRedirectError(
                "cannot duplicate stderr".to_string(),
            ));
        }

        self.saved_stderr = Some(saved_stderr);

        let mut pipe_fd: [c_int; 2] = [-1; 2];

        if (unsafe { pipe(&mut pipe_fd[0]) }) == -1 {
            return Err(XSDValidationError::OutputRedirectError(
                "cannot create pipe".to_string(),
            ));
        }

        // redirect stderr to pipe/log_file
        if (unsafe { dup2(pipe_fd[self.pipe_write], STDERR_FILENO) }) == -1 {
            return Err(XSDValidationError::OutputRedirectError(
                "cannot redirect stderr to pipe".to_string(),
            ));
        }

        self.pipe_fd = Some(pipe_fd);

        Ok(())
    }

    fn restore_stderr(&mut self) {
        if let Some(val) = self.saved_stderr {
            unsafe {
                dup2(val, STDERR_FILENO);
                close(val);
            }
        }
    }

    fn close_pipe(&mut self) {
        if let Some(pipe) = self.pipe_fd {
            unsafe {
                close(pipe[self.pipe_read]);

                close(pipe[self.pipe_write]);
            }
        }
    }

    pub(crate) fn restore(&mut self) {
        self.restore_stderr();
        self.close_pipe();
    }
}

impl Drop for WarningHandler {
    fn drop(&mut self) {
        self.restore();
    }
}
