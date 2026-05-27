function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

interface Todo {
  id: number;
  title: string;
  completed: boolean;
  priority: string;
}

class TaskManager {
  #todos: any[] = [];
  #nextId: number = 1;

  addTodo(title: string, priority: string) {
    const todo = {
      id: this.#nextId,
      title: title,
      completed: false,
      priority: priority
    };
    this.#todos.push(todo);
    this.#nextId = this.#nextId + 1;
    return todo.id;
  }

  toggleTodo(id: number) {
    for (let i = 0; i < this.#todos.length; i++) {
      if (this.#todos[i].id === id) {
        this.#todos[i].completed = !this.#todos[i].completed;
        return true;
      }
    }
    return false;
  }

  deleteTodo(id: number) {
    const originalLength = this.#todos.length;
    this.#todos = this.#todos.filter(todo => todo.id !== id);
    return this.#todos.length < originalLength;
  }

  getTodos(filterCompleted: any) {
    if (filterCompleted === null) {
      return this.#todos;
    }
    return this.#todos.filter(todo => todo.completed === filterCompleted);
  }

  getPriorityCount(priority: string) {
    return this.#todos.filter(todo => todo.priority === priority).length;
  }
}

function runTaskManagerTests() {
  const manager = new TaskManager();

  const id1 = manager.addTodo("Buy groceries", "high");
  const id2 = manager.addTodo("Clean the house", "medium");
  const id3 = manager.addTodo("Code TSDroid compiler", "high");

  assertEqual(id1, 1, "First todo id");
  assertEqual(id2, 2, "Second todo id");
  assertEqual(id3, 3, "Third todo id");

  assertEqual(manager.getPriorityCount("high"), 2, "High priority todo count");

  const toggleResult = manager.toggleTodo(id1);
  assertEqual(toggleResult, true, "Toggle todo success");

  const activeTodos = manager.getTodos(false);
  const completedTodos = manager.getTodos(true);
  assertEqual(activeTodos.length, 2, "Active todos count");
  assertEqual(completedTodos.length, 1, "Completed todos count");
  assertEqual(completedTodos[0].title, "Buy groceries", "Completed todo title");

  const deleteResult = manager.deleteTodo(id2);
  assertEqual(deleteResult, true, "Delete todo success");
  assertEqual(manager.getTodos(null).length, 2, "Total todos count after deletion");
}

function runSettingsMergeTests() {
  const defaultSettings: any = {
    theme: "light",
    notifications: {
      email: true,
      push: false
    },
    language: "en"
  };

  const userSettings: any = {
    theme: "dark",
    notifications: {
      push: true
    }
  };

  const mergedSettings: any = {
    ...defaultSettings,
    ...userSettings,
    notifications: {
      ...defaultSettings.notifications,
      ...userSettings.notifications
    }
  };

  assertEqual(mergedSettings.theme, "dark", "Overridden string setting");
  assertEqual(mergedSettings.language, "en", "Preserved default setting");
  assertEqual(mergedSettings.notifications.email, true, "Preserved nested default setting");
  assertEqual(mergedSettings.notifications.push, true, "Overridden nested custom setting");
}

function validateForm(formData: any): any {
  const errors: any = {};

  if (!formData.name || formData.name.length < 3) {
    errors.name = "Name must be at least 3 characters long";
  }

  if (formData.age === null || formData.age < 18) {
    errors.age = "You must be at least 18 years old";
  }

  if (!formData.email || !formData.email.includes("@")) {
    errors.email = "Invalid email format";
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors: errors
  };
}

function runFormValidatorTests() {
  const validData: any = {
    name: "John Doe",
    age: 25,
    email: "john@example.com"
  };

  const invalidData: any = {
    name: "Jo",
    age: 15,
    email: "invalid_email"
  };

  const validResult = validateForm(validData);
  assertEqual(validResult.valid, true, "Validation passes for valid data");

  const invalidResult = validateForm(invalidData);
  assertEqual(invalidResult.valid, false, "Validation fails for invalid data");
  assertEqual(invalidResult.errors.name, "Name must be at least 3 characters long", "Error for short name");
  assertEqual(invalidResult.errors.age, "You must be at least 18 years old", "Error for underage");
  assertEqual(invalidResult.errors.email, "Invalid email format", "Error for malformed email");
}

function truncate(str: string, maxLength: number) {
  if (str.length <= maxLength) {
    return str;
  }
  return str.substring(0, maxLength) + "...";
}

function runUtilityTests() {
  const longText = "TSDroid is an advanced TypeScript to JVM compiler";
  assertEqual(truncate(longText, 10), "TSDroid is...", "Truncate string exceeding limit");
  assertEqual(truncate("Short", 10), "Short", "Truncate string within limit");
}

function main() {
  console.log("=== RUNNING MORE DAILY USE CASES TEST SUITE ===");
  
  runTaskManagerTests();
  runSettingsMergeTests();
  runFormValidatorTests();
  runUtilityTests();

  console.log("=== ALL ADDITIONAL DAILY USE CASES PASSED SUCCESSFULLY ===");
}

main();
