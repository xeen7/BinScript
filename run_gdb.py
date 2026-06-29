import subprocess
import os

with open("gdb_script", "w") as f:
    f.write("run\nbt\nquit\n")

proc = subprocess.run(["gdb", "-batch", "-x", "gdb_script", "./tests/test_raii_rethrow_bin"], capture_output=True, text=True)
print(proc.stdout)
