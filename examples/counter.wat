;; RavensOne Counter Example (compiled to WASM)
;; This demonstrates what a simple .raven file would compile to

(module
  ;; Import environment functions
  (import "env" "log" (func $log (param i32 i32)))
  (import "env" "createElement" (func $createElement (param i32 i32) (result i32)))
  (import "env" "setInnerHTML" (func $setInnerHTML (param i32 i32 i32)))
  (import "env" "mountElement" (func $mountElement (param i32 i32 i32)))

  ;; Memory for strings and data
  (memory (export "memory") 1)

  ;; String constants
  (data (i32.const 0) "Counter initialized")
  (data (i32.const 100) "div")
  (data (i32.const 104) "<h1>Count: </h1><button>Increment</button>")
  (data (i32.const 200) "#app")

  ;; Global state (counter value)
  (global $count (mut i32) (i32.const 0))

  ;; Initialize function
  (func (export "init")
    ;; Log initialization
    (call $log (i32.const 0) (i32.const 19))
  )

  ;; Get current count
  (func (export "getCount") (result i32)
    global.get $count
  )

  ;; Increment counter
  (func (export "increment") (result i32)
    ;; Increment global
    (global.set $count
      (i32.add (global.get $count) (i32.const 1))
    )

    ;; Return new value
    global.get $count
  )

  ;; Decrement counter
  (func (export "decrement") (result i32)
    ;; Decrement global
    (global.set $count
      (i32.sub (global.get $count) (i32.const 1))
    )

    ;; Return new value
    global.get $count
  )

  ;; Reset counter
  (func (export "reset")
    (global.set $count (i32.const 0))
  )

  ;; Add two numbers (example)
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    (i32.add (local.get $a) (local.get $b))
  )

  ;; Multiply two numbers (example)
  (func (export "multiply") (param $a i32) (param $b i32) (result i32)
    (i32.mul (local.get $a) (local.get $b))
  )

  ;; Fibonacci (recursive example)
  (func (export "fibonacci") (param $n i32) (result i32)
    (if (result i32) (i32.lt_u (local.get $n) (i32.const 2))
      (then (local.get $n))
      (else
        (i32.add
          (call 7 (i32.sub (local.get $n) (i32.const 1)))
          (call 7 (i32.sub (local.get $n) (i32.const 2)))
        )
      )
    )
  )
)
