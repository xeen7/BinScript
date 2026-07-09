import sys
import re

rcs = {}
decs = {}
incs = {}

with open('run.log', 'r') as f:
    for line in f:
        line = line.strip()
        m_alloc_arr = re.search(r'Allocated Shared Array: (0x[0-9a-f]+)', line)
        if m_alloc_arr:
            ptr = m_alloc_arr.group(1)
            rcs[ptr] = 1
            incs[ptr] = 0
            decs[ptr] = 0
            continue

        m_alloc_obj = re.search(r'Allocated Shared Object: (0x[0-9a-f]+)', line)
        if m_alloc_obj:
            ptr = m_alloc_obj.group(1)
            rcs[ptr] = 1
            incs[ptr] = 0
            decs[ptr] = 0
            continue
            
        m_inc = re.search(r'circ_inc: (0x[0-9a-f]+)', line)
        if m_inc:
            ptr = m_inc.group(1)
            if ptr not in rcs: rcs[ptr] = 1
            rcs[ptr] += 1
            incs[ptr] = incs.get(ptr, 0) + 1
            continue
            
        m_dec = re.search(r'circ_dec: (0x[0-9a-f]+)', line)
        if m_dec:
            ptr = m_dec.group(1)
            if ptr not in rcs: rcs[ptr] = 1
            rcs[ptr] -= 1
            decs[ptr] = decs.get(ptr, 0) + 1
            continue

print("Objects tracking:")
for ptr, rc in rcs.items():
    print(f"{ptr}: final_rc={rc}, incs={incs.get(ptr, 0)}, decs={decs.get(ptr, 0)}")

print(f"\nTotal allocs tracked: {len(rcs)}")
