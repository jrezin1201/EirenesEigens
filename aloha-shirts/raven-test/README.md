# raven-test

**Testing Framework for RavensOne**

A complete testing solution for RavensOne applications with assertions, mocking, snapshots, and coverage reporting.

## 🚀 Quick Start

### Installation

```bash
raven pkg add raven-test
```

### Basic Usage

```raven
import { describe, it, expect } from "raven-test"

describe("Calculator", () => {
    it("should add two numbers", () => {
        let result = 2 + 2;
        expect(result).toBe(4);
    });

    it("should multiply numbers", () => {
        let result = 3 * 4;
        expect(result).toBe(12);
    });
});
```

## 📦 Features

- ✅ **Familiar API** - Jest/Mocha-like syntax (describe, it, expect)
- ✅ **Rich Assertions** - Comprehensive expect matchers
- ✅ **Mocking & Spying** - Mock functions, spies, and stubs
- ✅ **Snapshot Testing** - Capture and compare snapshots
- ✅ **Code Coverage** - Track line, function, and branch coverage
- ✅ **Async Support** - Test promises and async functions
- ✅ **Hooks** - beforeEach, afterEach, beforeAll, afterAll
- ✅ **Timeouts** - Configurable test timeouts
- ✅ **Watch Mode** - Auto-rerun tests on file changes

## 🎯 Writing Tests

### Test Structure

```raven
import { describe, it, beforeEach, afterEach } from "raven-test"

describe("User Service", () => {
    let user = null;

    beforeEach(() => {
        user = { name: "John", age: 30 };
    });

    afterEach(() => {
        user = null;
    });

    it("should create a user", () => {
        expect(user).toBeDefined();
        expect(user.name).toBe("John");
    });

    it("should update user age", () => {
        user.age = 31;
        expect(user.age).toBe(31);
    });
});
```

### Nested Suites

```raven
describe("Authentication", () => {
    describe("Login", () => {
        it("should accept valid credentials", () => {
            // test
        });

        it("should reject invalid credentials", () => {
            // test
        });
    });

    describe("Logout", () => {
        it("should clear session", () => {
            // test
        });
    });
});
```

### Skip and Only

```raven
// Skip a test
it.skip("should do something eventually", () => {
    // Not implemented yet
});

// Only run specific tests
it.only("should run only this test", () => {
    expect(true).toBe(true);
});
```

## ✅ Assertions

### Equality

```raven
expect(value).toBe(expected);           // Strict equality (===)
expect(value).toEqual(expected);        // Deep equality
expect(value).not.toBe(unexpected);     // Negation
```

### Truthiness

```raven
expect(value).toBeTruthy();
expect(value).toBeFalsy();
expect(value).toBeNull();
expect(value).toBeUndefined();
expect(value).toBeDefined();
```

### Numbers

```raven
expect(value).toBeGreaterThan(5);
expect(value).toBeGreaterThanOrEqual(10);
expect(value).toBeLessThan(20);
expect(value).toBeLessThanOrEqual(15);
expect(3.14159).toBeCloseTo(3.14, 2);  // Precision: 2 decimals
```

### Strings

```raven
expect("hello world").toContain("world");
expect("email@example.com").toMatch(/\w+@\w+\.\w+/);
expect("hello").toStartWith("hel");
expect("world").toEndWith("rld");
```

### Arrays & Objects

```raven
expect([1, 2, 3]).toHaveLength(3);
expect([1, 2, 3]).toContain(2);
expect({ name: "John" }).toHaveProperty("name");
expect({ age: 30 }).toHaveProperty("age", 30);
```

### Functions

```raven
expect(() => {
    throw new Error("Oops!");
}).toThrow();

expect(() => {
    throw new Error("Invalid input");
}).toThrow("Invalid");

expect(() => {
    throw new Error("Error 404");
}).toThrow(/Error \d+/);
```

## 🎭 Mocking

### Mock Functions

```raven
import { mock, expect } from "raven-test"

let mockFn = mock();

// Call the mock
mockFn("hello");
mockFn("world");

// Assertions
expect(mockFn).toHaveBeenCalled();
expect(mockFn).toHaveBeenCalledTimes(2);
expect(mockFn).toHaveBeenCalledWith("hello");
```

### Mock Return Values

```raven
let mockFn = mock();

// Return value
mockFn.mockReturnValue(42);
expect(mockFn()).toBe(42);

// Return different values
mockFn.mockReturnValueOnce(1)
      .mockReturnValueOnce(2)
      .mockReturnValue(3);

expect(mockFn()).toBe(1);
expect(mockFn()).toBe(2);
expect(mockFn()).toBe(3);
expect(mockFn()).toBe(3);
```

### Mock Implementation

```raven
let mockFn = mock();

mockFn.mockImplementation((x, y) => {
    return x + y;
});

expect(mockFn(2, 3)).toBe(5);
```

### Async Mocks

```raven
let mockFn = mock();

// Resolve
mockFn.mockResolvedValue("success");
mockFn().then(value => {
    expect(value).toBe("success");
});

// Reject
mockFn.mockRejectedValue(new Error("failed"));
mockFn().catch(error => {
    expect(error.message).toBe("failed");
});
```

### Spies

```raven
import { spy } from "raven-test"

let obj = {
    method: (x) => x * 2
};

let spyFn = spy(obj, "method");

// Call still works
let result = obj.method(5);
expect(result).toBe(10);

// But we can assert on it
expect(spyFn).toHaveBeenCalledWith(5);
```

### Stubs

```raven
import { stub } from "raven-test"

// Stub with no implementation
let stubFn = stub();
stubFn.mockReturnValue("stubbed");

expect(stubFn()).toBe("stubbed");
```

## 📸 Snapshot Testing

```raven
import { snapshot } from "raven-test"

describe("Component rendering", () => {
    it("should match snapshot", () => {
        let component = <MyComponent title="Hello" />;
        snapshot("MyComponent", component);
    });
});
```

Update snapshots:

```bash
raven test --updateSnapshots
```

## 📊 Code Coverage

Enable coverage in your test config:

```raven
import { TestRunner } from "raven-test"

let runner = TestRunner::new({
    coverage: true,
    verbose: true
});

runner.run().then((results) => {
    runner.print_results(results);
    printCoverageReport();
});
```

## ⏱️ Async Testing

### Promises

```raven
it("should resolve promise", () => {
    return fetchData().then(data => {
        expect(data).toBeDefined();
    });
});

it("should reject promise", () => {
    return fetchData().catch(error => {
        expect(error).toBeDefined();
    });
});
```

### Async/Await

```raven
it("should fetch user data", async () => {
    let user = await getUser(123);
    expect(user.name).toBe("John");
});
```

### Timeouts

```raven
it("should complete quickly", () => {
    // test
}, 1000); // 1 second timeout

it("should handle slow operations", async () => {
    await slowOperation();
}, 10000); // 10 second timeout
```

## 🔧 Hooks

```raven
describe("Database operations", () => {
    let db = null;

    beforeAll(() => {
        // Run once before all tests
        db = connectDatabase();
    });

    afterAll(() => {
        // Run once after all tests
        db.close();
    });

    beforeEach(() => {
        // Run before each test
        db.clear();
    });

    afterEach(() => {
        // Run after each test
        db.resetMocks();
    });

    it("should insert record", () => {
        db.insert({ name: "John" });
        expect(db.count()).toBe(1);
    });
});
```

## 🏃 Running Tests

### CLI

```bash
# Run all tests
raven test

# Run specific file
raven test src/utils.test.raven

# Watch mode
raven test --watch

# Coverage
raven test --coverage

# Verbose output
raven test --verbose

# Bail on first failure
raven test --bail
```

### Programmatic

```raven
import { TestRunner, getRootSuite } from "raven-test"

let runner = TestRunner::new({
    bail: false,
    verbose: true,
    timeout: 5000,
    coverage: true
});

runner.run().then((results) => {
    runner.print_results(results);

    if results.failed > 0 {
        process.exit(1);
    }
});
```

## 📋 Test Configuration

```raven
type TestConfig = {
    bail: Bool,          // Stop on first failure
    verbose: Bool,       // Detailed output
    timeout: Int,        // Global timeout (ms)
    parallel: Bool,      // Run tests in parallel
    coverage: Bool,      // Collect coverage
    watch: Bool,         // Watch mode
    pattern: String      // Test file pattern
}
```

## 🎨 Example: Testing a Component

```raven
import { describe, it, expect, mock } from "raven-test"
import { Button } from "raven-ui"

describe("Button component", () => {
    it("should render with text", () => {
        let button = <Button>Click me</Button>;
        expect(button.textContent).toBe("Click me");
    });

    it("should call onClick handler", () => {
        let onClick = mock();
        let button = <Button onclick={onClick}>Click</Button>;

        button.click();

        expect(onClick).toHaveBeenCalled();
        expect(onClick).toHaveBeenCalledTimes(1);
    });

    it("should be disabled", () => {
        let button = <Button disabled={true}>Disabled</Button>;
        expect(button.disabled).toBe(true);
    });
});
```

## 🎨 Example: Testing HTTP Requests

```raven
import { describe, it, expect, mock } from "raven-test"
import { get } from "raven-http"

describe("API requests", () => {
    it("should fetch users", async () => {
        let response = await get("/api/users");

        expect(response.status).toBe(200);
        expect(response.data).toHaveLength(10);
    });

    it("should handle errors", async () => {
        try {
            await get("/api/invalid");
        } catch (error) {
            expect(error.status).toBe(404);
        }
    });
});
```

## 📄 License

MIT License

## 🤝 Contributing

Contributions welcome! Please see the main RavensOne repository for contribution guidelines.

## 🔗 Links

- **Repository**: https://github.com/jrezin1201/RavensOne
- **Documentation**: https://ravensone.dev/docs/packages/raven-test
- **Registry**: https://registry.ravensone.dev/packages/raven-test
- **Issues**: https://github.com/jrezin1201/RavensOne/issues

---

**Made with ❤️ for the RavensOne community**
