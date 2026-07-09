with open("tests/examples/t18_more_daily_use_cases.ts", "r") as f:
    lines = f.readlines()

with open("tests/examples/t18_more_daily_use_cases.ts", "w") as f:
    for line in lines:
        if "assertEqual(activeTodos.length, 2, \"Active todos count\");" in line:
            f.write('  print("activeTodos = " + activeTodos);\n')
            f.write('  print("activeTodos.length = " + activeTodos.length);\n')
            f.write('  print("this.todos = " + manager.getTodos(null));\n')
        f.write(line)
