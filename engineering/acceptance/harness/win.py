"""Windows calls the M5 checks need and the standard library does not give: suspending and resuming a
process by its id, a process's creation time and environment block, the process tree, and a job object
around a started process, as a host such as Claude Code holds its MCP server (SES-08).

Everything goes by process id; nothing here looks a process up by its name.
"""

import ctypes
import json
import subprocess
from ctypes import wintypes
from datetime import datetime, timedelta, timezone

k32 = ctypes.WinDLL("kernel32", use_last_error=True)
ntdll = ctypes.WinDLL("ntdll")

PROCESS_TERMINATE = 0x0001
PROCESS_VM_READ = 0x0010
PROCESS_SET_QUOTA = 0x0100
PROCESS_SUSPEND_RESUME = 0x0800
PROCESS_QUERY_INFORMATION = 0x0400
PROCESS_QUERY_LIMITED_INFORMATION = 0x1000

k32.OpenProcess.restype = wintypes.HANDLE
k32.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
k32.CloseHandle.argtypes = [wintypes.HANDLE]
k32.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
k32.ReadProcessMemory.argtypes = [wintypes.HANDLE, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t,
                                  ctypes.POINTER(ctypes.c_size_t)]
k32.CreateJobObjectW.restype = wintypes.HANDLE
k32.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
k32.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
k32.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
ntdll.NtSuspendProcess.argtypes = [wintypes.HANDLE]
ntdll.NtResumeProcess.argtypes = [wintypes.HANDLE]
ntdll.NtQueryInformationProcess.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.ULONG,
                                            ctypes.POINTER(wintypes.ULONG)]


def _open(pid, access):
    h = k32.OpenProcess(access, False, pid)
    if not h:
        raise OSError(f"OpenProcess({pid}) failed with error {ctypes.get_last_error()}")
    return h


def suspend(pid):
    h = _open(pid, PROCESS_SUSPEND_RESUME)
    try:
        status = ntdll.NtSuspendProcess(h)
        assert status == 0, f"NtSuspendProcess({pid}) returned {status:#x}"
    finally:
        k32.CloseHandle(h)


def resume(pid):
    h = _open(pid, PROCESS_SUSPEND_RESUME)
    try:
        status = ntdll.NtResumeProcess(h)
        assert status == 0, f"NtResumeProcess({pid}) returned {status:#x}"
    finally:
        k32.CloseHandle(h)


def resume_handle(handle):
    status = ntdll.NtResumeProcess(handle)
    assert status == 0, f"NtResumeProcess returned {status:#x}"


def created_at(pid):
    """The process's creation time, UTC."""
    h = _open(pid, PROCESS_QUERY_LIMITED_INFORMATION)
    try:
        times = [wintypes.FILETIME() for _ in range(4)]
        assert k32.GetProcessTimes(h, *[ctypes.byref(t) for t in times]), f"GetProcessTimes({pid}) failed"
        ticks = (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime
        return datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)
    finally:
        k32.CloseHandle(h)


def environment(pid):
    """The environment block of a 64-bit process of this user, as one string with NUL separators.

    Read from its PEB: ProcessParameters at PEB+0x20, Environment at +0x80 and EnvironmentSize at +0x3F0
    of RTL_USER_PROCESS_PARAMETERS on Windows 10 and 11.
    """
    h = _open(pid, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ)
    try:
        pbi = (ctypes.c_void_p * 6)()
        status = ntdll.NtQueryInformationProcess(h, 0, ctypes.byref(pbi), ctypes.sizeof(pbi), None)
        assert status == 0, f"NtQueryInformationProcess({pid}) returned {status:#x}"
        peb = pbi[1]

        def read(addr, size):
            buf = ctypes.create_string_buffer(size)
            n = ctypes.c_size_t()
            assert k32.ReadProcessMemory(h, ctypes.c_void_p(addr), buf, size, ctypes.byref(n)), \
                f"ReadProcessMemory of process {pid} failed with error {ctypes.get_last_error()}"
            return buf.raw[:n.value]

        params = int.from_bytes(read(peb + 0x20, 8), "little")
        env = int.from_bytes(read(params + 0x80, 8), "little")
        size = int.from_bytes(read(params + 0x3F0, 8), "little")
        return read(env, min(size, 1 << 20)).decode("utf-16-le", "replace")
    finally:
        k32.CloseHandle(h)


def processes():
    """{pid: (parent pid, image name, command line)} of every process, from Win32_Process."""
    out = subprocess.run(["powershell", "-NoProfile", "-Command",
                          "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name,CommandLine"
                          " | ConvertTo-Json -Compress"],
                         capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120).stdout
    return {p["ProcessId"]: (p["ParentProcessId"], p["Name"] or "", p["CommandLine"] or "") for p in json.loads(out)}


def descendants(pid, procs=None):
    """Every process whose parent chain leads to pid, children first in the order found."""
    procs = processes() if procs is None else procs
    found, frontier = [], [pid]
    while frontier:
        parent = frontier.pop()
        for child, (ppid, _, _) in procs.items():
            if ppid == parent and child not in found and child != pid:
                found.append(child)
                frontier.append(child)
    return found


def runner_processes(procs=None):
    """{pid: command line} of every `shakedown runner` process."""
    procs = processes() if procs is None else procs
    return {pid: cmd for pid, (_, name, cmd) in procs.items()
            if name.lower() == "shakedown.exe" and " runner" in cmd.lower()}


def terminate(pid):
    subprocess.run(["taskkill", "/PID", str(pid), "/F"], capture_output=True, timeout=30)


# ------------------------------------------------------------------ job objects

JOB_OBJECT_LIMIT_BREAKAWAY_OK = 0x0800
JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE = 0x2000
JobObjectExtendedLimitInformation = 9


class _Basic(ctypes.Structure):
    _fields_ = [("PerProcessUserTimeLimit", ctypes.c_int64), ("PerJobUserTimeLimit", ctypes.c_int64),
                ("LimitFlags", wintypes.DWORD), ("MinimumWorkingSetSize", ctypes.c_size_t),
                ("MaximumWorkingSetSize", ctypes.c_size_t), ("ActiveProcessLimit", wintypes.DWORD),
                ("Affinity", ctypes.c_size_t), ("PriorityClass", wintypes.DWORD),
                ("SchedulingClass", wintypes.DWORD)]


class _Io(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint64) for n in ("Read", "Write", "Other", "ReadT", "WriteT", "OtherT")]


class _Extended(ctypes.Structure):
    _fields_ = [("Basic", _Basic), ("Io", _Io), ("ProcessMemoryLimit", ctypes.c_size_t),
                ("JobMemoryLimit", ctypes.c_size_t), ("PeakProcessMemoryUsed", ctypes.c_size_t),
                ("PeakJobMemoryUsed", ctypes.c_size_t)]


class Job:
    """A job object with kill-on-close, as a host holds a stdio server; breakaway allowed or not."""

    def __init__(self, breakaway):
        self.handle = k32.CreateJobObjectW(None, None)
        assert self.handle, f"CreateJobObjectW failed with error {ctypes.get_last_error()}"
        info = _Extended()
        info.Basic.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | (JOB_OBJECT_LIMIT_BREAKAWAY_OK if breakaway else 0)
        assert k32.SetInformationJobObject(self.handle, JobObjectExtendedLimitInformation, ctypes.byref(info),
                                           ctypes.sizeof(info)), \
            f"SetInformationJobObject failed with error {ctypes.get_last_error()}"

    def assign(self, process_handle):
        assert k32.AssignProcessToJobObject(self.handle, process_handle), \
            f"AssignProcessToJobObject failed with error {ctypes.get_last_error()}"

    def close(self):
        """Closes the only handle, which ends every process in the job."""
        if self.handle:
            k32.CloseHandle(self.handle)
            self.handle = None
