"""Windows Job Object glue, matching the existing Rust startup safety policy."""
import ctypes as c
from ctypes import wintypes as w

class Basic(c.Structure):
    _fields_=[('process_time',c.c_int64),('job_time',c.c_int64),('flags',w.DWORD),
        ('minimum',c.c_size_t),('maximum',c.c_size_t),('active',w.DWORD),
        ('affinity',c.c_size_t),('priority',w.DWORD),('scheduling',w.DWORD)]
class Extended(c.Structure):
    _fields_=[('basic',Basic),('io',c.c_uint64*6),('process_memory',c.c_size_t),
        ('job_memory',c.c_size_t),('peak_process',c.c_size_t),('peak_job',c.c_size_t)]

class PendingLaunchGuard:
    def __init__(self,pid):
        self.api=c.WinDLL('kernel32',use_last_error=True)
        self.api.CreateJobObjectW.argtypes=[c.c_void_p,w.LPCWSTR]
        self.api.CreateJobObjectW.restype=w.HANDLE
        self.api.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD]
        self.api.OpenProcess.restype=w.HANDLE
        self.api.CloseHandle.argtypes=[w.HANDLE]
        self.api.AssignProcessToJobObject.argtypes=[w.HANDLE,w.HANDLE]
        self.api.SetInformationJobObject.argtypes=[w.HANDLE,c.c_int,c.c_void_p,w.DWORD]
        self.handle=self.api.CreateJobObjectW(None,None)
        if not self.handle:raise c.WinError(c.get_last_error())
        try:
            self.set_flags(0x2000) # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            process=self.api.OpenProcess(0x0100|0x0001,False,pid) # SET_QUOTA | TERMINATE
            if not process:raise c.WinError(c.get_last_error())
            try:
                if not self.api.AssignProcessToJobObject(self.handle,process):raise c.WinError(c.get_last_error())
            finally:self.api.CloseHandle(process)
        except Exception:
            self.close()
            raise
    def set_flags(self,flags):
        limits=Extended();limits.basic.flags=flags
        if not self.api.SetInformationJobObject(self.handle,9,c.byref(limits),c.sizeof(limits)):
            raise c.WinError(c.get_last_error())
    def release(self):
        self.set_flags(0)
        self.close()
    def close(self):
        if self.handle:
            self.api.CloseHandle(self.handle)
            self.handle=None
