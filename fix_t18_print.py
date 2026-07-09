with open("tests/examples/t18_more_daily_use_cases.ts", "r") as f:
    code = f.read()

code = code.replace("  print(\"activeTodos = \" + activeTodos);", "  console.log(\"activeTodos = \" + activeTodos);")
code = code.replace("  print(\"activeTodos.length = \" + activeTodos.length);", "  console.log(\"activeTodos.length = \" + activeTodos.length);")
code = code.replace("  print(\"this.todos = \" + manager.getTodos(null));", "  console.log(\"this.todos = \" + manager.getTodos(null));")

with open("tests/examples/t18_more_daily_use_cases.ts", "w") as f:
    f.write(code)
