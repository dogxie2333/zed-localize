use crate::catalog::{Entry, Plan, check_spans, hash_file};
use anyhow::{Context, Result, bail, ensure};
use std::{
    ffi::c_void,
    mem::{size_of, zeroed},
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    ptr::null,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        },
        Memory::{
            MEM_COMMIT, MEM_IMAGE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ,
            PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_READWRITE, VirtualProtectEx,
            VirtualQueryEx,
        },
        ProcessStatus::K32GetMappedFileNameW,
        SystemInformation::{GetSystemInfo, SYSTEM_INFO},
        Threading::{
            CREATE_SUSPENDED, CreateProcessW, OpenProcess, PROCESS_INFORMATION,
            PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
            QueryFullProcessImageNameW, ResumeThread, STARTUPINFOW, TerminateProcess,
        },
    },
};

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub struct Process {
    handle: Handle,
    thread: Option<Handle>,
    pub pid: u32,
    owned: bool,
    resumed: bool,
    job: Option<Handle>,
}

fn win_error(operation: &str) -> anyhow::Error {
    anyhow::Error::new(std::io::Error::last_os_error()).context(operation.to_string())
}

fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}

fn quoted(value: &str) -> String {
    let mut result = String::from("\"");
    let mut slashes = 0;
    for character in value.chars() {
        if character == '\\' {
            slashes += 1;
            continue;
        }
        if character == '"' {
            result.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
        } else {
            result.extend(std::iter::repeat_n('\\', slashes));
        }
        slashes = 0;
        result.push(character);
    }
    result.extend(std::iter::repeat_n('\\', slashes * 2));
    result.push('"');
    result
}

impl Process {
    pub fn spawn_suspended(exe: &Path, arguments: &[String]) -> Result<Self> {
        let executable = exe.canonicalize()?;
        let application = wide(executable.as_os_str());
        let cwd = wide(
            executable
                .parent()
                .context("No executable parent")?
                .as_os_str(),
        );
        let command = std::iter::once(executable.to_string_lossy().into_owned())
            .chain(arguments.iter().cloned())
            .map(|value| quoted(&value))
            .collect::<Vec<_>>()
            .join(" ");
        let mut command: Vec<u16> = command.encode_utf16().chain(Some(0)).collect();
        let mut startup: STARTUPINFOW = unsafe { zeroed() };
        startup.cb = size_of::<STARTUPINFOW>() as u32;
        let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
        // No inherited handles: launching from a captured terminal cannot keep
        // that terminal's output pipes alive after this tool exits.
        let ok = unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                0,
                CREATE_SUSPENDED,
                null(),
                cwd.as_ptr(),
                &startup,
                &mut info,
            )
        };
        if ok == 0 {
            return Err(win_error("CreateProcessW"));
        }
        let mut process = Self {
            handle: Handle(info.hProcess),
            thread: Some(Handle(info.hThread)),
            pid: info.dwProcessId,
            owned: true,
            resumed: false,
            job: None,
        };
        let job = unsafe { CreateJobObjectW(null(), null()) };
        if job.is_null() {
            return Err(win_error("CreateJobObjectW"));
        }
        process.job = Some(Handle(job));
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(win_error("SetInformationJobObject"));
        }
        if unsafe { AssignProcessToJobObject(job, process.handle.0) } == 0 {
            return Err(win_error("AssignProcessToJobObject"));
        }
        Ok(process)
    }

    pub fn open(pid: u32) -> Result<Self> {
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION
                    | PROCESS_VM_READ
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_OPERATION,
                0,
                pid,
            )
        };
        if handle.is_null() {
            return Err(win_error("OpenProcess"));
        }
        Ok(Self {
            handle: Handle(handle),
            thread: None,
            pid,
            owned: false,
            resumed: true,
            job: None,
        })
    }

    pub fn image_path(&self) -> Result<PathBuf> {
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        if unsafe { QueryFullProcessImageNameW(self.handle.0, 0, buffer.as_mut_ptr(), &mut length) }
            == 0
        {
            return Err(win_error("QueryFullProcessImageNameW"));
        }
        Ok(PathBuf::from(String::from_utf16(
            &buffer[..length as usize],
        )?))
    }

    pub fn verify_identity(&self, exe: &Path) -> Result<()> {
        let actual = self
            .image_path()?
            .canonicalize()?
            .to_string_lossy()
            .to_lowercase();
        let expected = exe.canonicalize()?.to_string_lossy().to_lowercase();
        ensure!(
            actual == expected,
            "PID is not running the requested executable"
        );
        Ok(())
    }

    pub fn image_base(&self) -> Result<usize> {
        let filename = self
            .image_path()?
            .file_name()
            .context("No image name")?
            .to_string_lossy()
            .to_lowercase();
        // The loader's module list may not be initialized in a new suspended
        // process. Query image mappings through the documented memory API.
        let mut address = 0usize;
        while address < 0x0000_8000_0000_0000 {
            let mut info: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };
            if unsafe {
                VirtualQueryEx(
                    self.handle.0,
                    address as *const c_void,
                    &mut info,
                    size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            } == 0
            {
                break;
            }
            if info.State == MEM_COMMIT && info.Type == MEM_IMAGE {
                let mut mapped = vec![0u16; 32768];
                let length = unsafe {
                    K32GetMappedFileNameW(
                        self.handle.0,
                        info.AllocationBase,
                        mapped.as_mut_ptr(),
                        mapped.len() as u32,
                    )
                };
                if length > 0 {
                    let name = String::from_utf16_lossy(&mapped[..length as usize]).to_lowercase();
                    if name.ends_with(&format!("\\{filename}")) {
                        return Ok(info.AllocationBase as usize);
                    }
                }
            }
            let next = (info.BaseAddress as usize)
                .checked_add(info.RegionSize)
                .context("Memory range overflow")?;
            if next <= address {
                break;
            }
            address = next;
        }
        bail!("Main executable mapping not found")
    }

    pub fn read(&self, address: usize, length: usize) -> Result<Vec<u8>> {
        let mut value = vec![0u8; length];
        let mut read = 0usize;
        if unsafe {
            ReadProcessMemory(
                self.handle.0,
                address as *const c_void,
                value.as_mut_ptr().cast(),
                length,
                &mut read,
            )
        } == 0
        {
            return Err(win_error("ReadProcessMemory"));
        }
        ensure!(read == length, "Incomplete memory read");
        Ok(value)
    }

    fn write(&self, base: usize, address: usize, value: &[u8]) -> Result<()> {
        ensure!(!value.is_empty(), "Empty write");
        let mut system: SYSTEM_INFO = unsafe { zeroed() };
        unsafe {
            GetSystemInfo(&mut system);
        }
        let page_size = system.dwPageSize as usize;
        let first = address / page_size * page_size;
        let last = address
            .checked_add(value.len() - 1)
            .context("Memory write overflow")?
            / page_size
            * page_size;
        let mut pages = Vec::new();
        for page in (first..=last).step_by(page_size) {
            let mut region: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };
            ensure!(
                unsafe {
                    VirtualQueryEx(
                        self.handle.0,
                        page as *const c_void,
                        &mut region,
                        size_of::<MEMORY_BASIC_INFORMATION>(),
                    )
                } != 0,
                "Unmapped patch page"
            );
            ensure!(
                region.State == MEM_COMMIT
                    && region.Type == MEM_IMAGE
                    && region.AllocationBase as usize == base,
                "Patch page is not part of the main image"
            );
            ensure!(
                region.Protect
                    & (PAGE_EXECUTE
                        | PAGE_EXECUTE_READ
                        | PAGE_EXECUTE_READWRITE
                        | PAGE_EXECUTE_WRITECOPY)
                    == 0,
                "Refusing executable memory"
            );
            pages.push((page, region.Protect));
        }
        let operation = (|| -> Result<()> {
            for &(page, _) in &pages {
                let mut previous = 0;
                if unsafe {
                    VirtualProtectEx(
                        self.handle.0,
                        page as *const c_void,
                        page_size,
                        PAGE_READWRITE,
                        &mut previous,
                    )
                } == 0
                {
                    return Err(win_error("VirtualProtectEx"));
                }
            }
            let mut written = 0;
            if unsafe {
                WriteProcessMemory(
                    self.handle.0,
                    address as *const c_void,
                    value.as_ptr().cast(),
                    value.len(),
                    &mut written,
                )
            } == 0
            {
                return Err(win_error("WriteProcessMemory"));
            }
            ensure!(written == value.len(), "Incomplete memory write");
            Ok(())
        })();
        let mut protection_errors = 0;
        for (page, protection) in pages.into_iter().rev() {
            let mut previous = 0;
            if unsafe {
                VirtualProtectEx(
                    self.handle.0,
                    page as *const c_void,
                    page_size,
                    protection,
                    &mut previous,
                )
            } == 0
            {
                protection_errors += 1;
            }
        }
        operation?;
        ensure!(
            protection_errors == 0,
            "Failed to restore {protection_errors} page protections"
        );
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        let thread = self.thread.as_ref().context("Not a spawned process")?;
        if unsafe { ResumeThread(thread.0) } == u32::MAX {
            return Err(win_error("ResumeThread"));
        }
        self.resumed = true;
        if let Some(job) = &self.job {
            let limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
            // Once startup is committed, Zed must survive the launcher's exit.
            // Before this point the kernel ends the paused child even if the
            // launcher is forcibly killed and Rust Drop cannot execute.
            if unsafe {
                SetInformationJobObject(
                    job.0,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            } == 0
            {
                return Err(win_error("Release startup job protection"));
            }
        }
        Ok(())
    }

    pub fn terminate_owned(&self) -> Result<()> {
        ensure!(
            self.owned,
            "Cannot end a process owned by another application"
        );
        if unsafe { TerminateProcess(self.handle.0, 0) } == 0 {
            return Err(win_error("TerminateProcess"));
        }
        Ok(())
    }

    pub fn terminate_owned_child(&self, pid: u32, exe: &Path) -> Result<()> {
        self.terminate_owned_descendant(pid, self.pid, exe)
    }
    pub fn terminate_owned_descendant(
        &self,
        pid: u32,
        expected_parent: u32,
        exe: &Path,
    ) -> Result<()> {
        use windows_sys::Win32::{
            Foundation::INVALID_HANDLE_VALUE,
            System::{
                Diagnostics::ToolHelp::{
                    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
                    TH32CS_SNAPPROCESS,
                },
                Threading::PROCESS_TERMINATE,
            },
        };
        ensure!(
            self.owned,
            "Only our isolated test parent may clean up its child"
        );
        let child = Self::open(pid)?;
        child.verify_identity(exe)?;
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        ensure!(
            snapshot != INVALID_HANDLE_VALUE,
            "Could not verify child parent"
        );
        let snapshot = Handle(snapshot);
        let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        let mut ok = unsafe { Process32FirstW(snapshot.0, &mut entry) };
        while ok != 0 {
            if entry.th32ProcessID == pid {
                found = entry.th32ParentProcessID == expected_parent;
                break;
            }
            ok = unsafe { Process32NextW(snapshot.0, &mut entry) };
        }
        ensure!(found, "PID was not created by our isolated test parent");
        let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
        ensure!(!handle.is_null(), "Could not open own test child");
        let handle = Handle(handle);
        ensure!(
            unsafe { TerminateProcess(handle.0, 0) } != 0,
            "Could not clean own test child"
        );
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.owned && !self.resumed {
            unsafe {
                TerminateProcess(self.handle.0, 0);
            }
        }
    }
}

pub fn verify(process: &Process, base: usize, entries: &[Entry], translated: bool) -> Result<()> {
    for entry in entries {
        let expected = hex::decode(if translated {
            &entry.after
        } else {
            &entry.before
        })?;
        ensure!(
            process.read(base + entry.rva, expected.len())? == expected,
            "Memory verification failed at RVA {}",
            entry.rva
        );
    }
    Ok(())
}

pub fn apply(
    process: &Process,
    base: usize,
    plan: &Plan,
    fail_after: Option<usize>,
) -> Result<usize> {
    ensure!(
        process.owned && !process.resumed,
        "Apply requires this tool's newly suspended process"
    );
    ensure!(
        plan.supported && !plan.entries.is_empty(),
        "Unsupported or empty patch plan"
    );
    check_spans(&plan.section, &plan.entries)?;
    ensure!(
        hash_file(Path::new(&plan.exe))? == plan.sha256,
        "Executable changed before patching"
    );
    process.verify_identity(Path::new(&plan.exe))?;
    verify(process, base, &plan.entries, false)?;
    let mut changed = 0;
    let operation = (|| -> Result<()> {
        for entry in &plan.entries {
            changed += 1;
            process.write(base, base + entry.rva, &hex::decode(&entry.after)?)?;
            if fail_after == Some(changed) {
                bail!("Injected rollback test failure");
            }
        }
        verify(process, base, &plan.entries, true)
    })();
    if let Err(error) = operation {
        // This transaction targets a newly suspended process; no user thread
        // can concurrently modify these static data candidates.
        let mut rollback_errors = 0;
        for entry in plan.entries[..changed].iter().rev() {
            if process
                .write(base, base + entry.rva, &hex::decode(&entry.before)?)
                .is_err()
            {
                rollback_errors += 1;
            }
        }
        ensure!(
            rollback_errors == 0,
            "Patch failed and rollback had {rollback_errors} errors"
        );
        return Err(error);
    }
    Ok(changed)
}

pub fn restore(process: &Process, base: usize, plan: &Plan) -> Result<usize> {
    ensure!(plan.supported, "Unsupported executable");
    check_spans(&plan.section, &plan.entries)?;
    process.verify_identity(Path::new(&plan.exe))?;
    let mut changed = Vec::new();
    for entry in &plan.entries {
        let before = hex::decode(&entry.before)?;
        let after = hex::decode(&entry.after)?;
        let current = process.read(base + entry.rva, before.len())?;
        if current == after {
            changed.push(entry);
        } else {
            ensure!(current == before, "Unknown bytes; refusing restore");
        }
    }
    for entry in changed.iter().rev() {
        process.write(base, base + entry.rva, &hex::decode(&entry.before)?)?;
    }
    verify(process, base, &plan.entries, false)?;
    Ok(changed.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_argument_quoting() {
        assert_eq!(quoted("plain"), "\"plain\"");
        assert_eq!(quoted("a\"b"), "\"a\\\"b\"");
        assert_eq!(quoted("C:\\folder\\"), "\"C:\\folder\\\\\"");
    }
}
