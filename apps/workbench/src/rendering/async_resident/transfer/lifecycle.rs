use anyhow::{Context, Result, bail};
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{INFINITE, WaitForSingleObject};

use super::AsyncTransfer;

impl AsyncTransfer {
    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        if let Some(value) = self.armed_gate.take() {
            unsafe { self.gate_fence.Signal(value) }
                .context("async shutdown gate signal failed")?;
        }
        let Some(value) = self
            .pending
            .as_ref()
            .map(|pending| pending.report.copy_fence)
        else {
            return Ok(());
        };
        if unsafe { self.copy_fence.GetCompletedValue() } >= value {
            return Ok(());
        }
        unsafe { self.copy_fence.SetEventOnCompletion(value, self.copy_event) }
            .context("async copy completion event failed")?;
        let wait = unsafe { WaitForSingleObject(self.copy_event, INFINITE) };
        if wait != WAIT_OBJECT_0 {
            bail!("async copy wait returned {wait:?}");
        }
        Ok(())
    }
}

impl Drop for AsyncTransfer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_idle();
            let _ = CloseHandle(self.copy_event);
        }
    }
}
