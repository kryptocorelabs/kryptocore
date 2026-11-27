"""Run the existing Core allocation test inside a Windows process-memory job."""
import argparse
import ctypes
import subprocess
import sys
from ctypes import wintypes
from pathlib import Path

if sys.platform != "win32":
    print("SKIP Windows process-memory test requires Windows.")
    raise SystemExit(0)


class BasicLimit(ctypes.Structure):
    _fields_ = [
        ("process_time", ctypes.c_longlong), ("job_time", ctypes.c_longlong),
        ("flags", wintypes.DWORD), ("minimum", ctypes.c_size_t),
        ("maximum", ctypes.c_size_t), ("active", wintypes.DWORD),
        ("affinity", ctypes.c_size_t), ("priority", wintypes.DWORD),
        ("scheduling", wintypes.DWORD),
    ]


class IoCounters(ctypes.Structure):
    _fields_ = [(name, ctypes.c_ulonglong) for name in (
        "reads", "writes", "others", "read_bytes", "write_bytes", "other_bytes")]


class ExtendedLimit(ctypes.Structure):
    _fields_ = [("basic", BasicLimit), ("io", IoCounters),
                ("process_memory", ctypes.c_size_t), ("job_memory", ctypes.c_size_t),
                ("peak_process", ctypes.c_size_t), ("peak_job", ctypes.c_size_t)]


class Startup(ctypes.Structure):
    _fields_ = [("size", wintypes.DWORD), ("reserved", wintypes.LPWSTR),
                ("desktop", wintypes.LPWSTR), ("title", wintypes.LPWSTR),
                ("x", wintypes.DWORD), ("y", wintypes.DWORD),
                ("width", wintypes.DWORD), ("height", wintypes.DWORD),
                ("chars_x", wintypes.DWORD), ("chars_y", wintypes.DWORD),
                ("fill", wintypes.DWORD), ("flags", wintypes.DWORD),
                ("show", wintypes.WORD), ("reserved_size", wintypes.WORD),
                ("reserved_bytes", ctypes.POINTER(ctypes.c_byte)),
                ("stdin", wintypes.HANDLE), ("stdout", wintypes.HANDLE),
                ("stderr", wintypes.HANDLE)]


class ProcessInfo(ctypes.Structure):
    _fields_ = [("process", wintypes.HANDLE), ("thread", wintypes.HANDLE),
                ("process_id", wintypes.DWORD), ("thread_id", wintypes.DWORD)]


kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
kernel.CreateJobObjectW.restype = wintypes.HANDLE
kernel.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
kernel.CreateProcessW.argtypes = [wintypes.LPCWSTR, wintypes.LPWSTR, ctypes.c_void_p,
                                ctypes.c_void_p, wintypes.BOOL, wintypes.DWORD,
                                ctypes.c_void_p, wintypes.LPCWSTR, ctypes.POINTER(Startup),
                                ctypes.POINTER(ProcessInfo)]
kernel.ResumeThread.argtypes = [wintypes.HANDLE]
kernel.ResumeThread.restype = wintypes.DWORD
kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
kernel.WaitForSingleObject.restype = wintypes.DWORD
kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
kernel.TerminateProcess.argtypes = [wintypes.HANDLE, wintypes.UINT]
kernel.CloseHandle.argtypes = [wintypes.HANDLE]


def checked(result):
    if not result:
        raise ctypes.WinError(ctypes.get_last_error())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path, help="Compiled argon2_memory test executable")
    binary = parser.parse_args().binary.resolve(strict=True)
    test = "argon2id_reports_reservation_failure"
    listing = subprocess.run(
        [str(binary), "--list", "--ignored"], check=True, capture_output=True,
        text=True, timeout=30,
    )
    if test + ": test" not in listing.stdout.splitlines():
        raise RuntimeError("The supplied executable does not contain the ignored allocation test.")

    job = checked(kernel.CreateJobObjectW(None, None))
    info = ProcessInfo()
    try:
        limits = ExtendedLimit()
        limits.basic.flags = 0x100 | 0x2000  # PROCESS_MEMORY and KILL_ON_JOB_CLOSE
        limits.process_memory = 192 * 1024 * 1024
        checked(kernel.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)))
        command = ctypes.create_unicode_buffer(subprocess.list2cmdline([
            str(binary), "--ignored", "--exact", test, "--nocapture",
        ]))
        startup = Startup()
        startup.size = ctypes.sizeof(startup)
        # Apply the memory limit before the child runs any test instructions.
        checked(kernel.CreateProcessW(
            None, command, None, None, False, 0x4 | 0x08000000,  # SUSPENDED and NO_WINDOW
            None, None, ctypes.byref(startup), ctypes.byref(info),
        ))
        checked(kernel.AssignProcessToJobObject(job, info.process))
        if kernel.ResumeThread(info.thread) == 0xFFFFFFFF:
            raise ctypes.WinError(ctypes.get_last_error())
        result = kernel.WaitForSingleObject(info.process, 30_000)
        if result == 0xFFFFFFFF:
            raise ctypes.WinError(ctypes.get_last_error())
        if result != 0:
            raise RuntimeError("The isolated allocation-failure test did not finish.")
        status = wintypes.DWORD()
        checked(kernel.GetExitCodeProcess(info.process, ctypes.byref(status)))
        if status.value:
            raise RuntimeError(f"Core allocation test failed with exit code {status.value}.")
        print(f"PASS {test} under a Windows 192 MiB process memory limit")
    finally:
        if info.process:
            kernel.TerminateProcess(info.process, 1)
        for handle in (info.thread, info.process, job):
            if handle:
                kernel.CloseHandle(handle)


if __name__ == "__main__":
    main()
