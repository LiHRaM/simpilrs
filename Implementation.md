# Implementation

The simpIL grammar is roughly defined on page 2 of the paper. By roughly, I mean that it is missing a definitive list of binary and unary operators, referring instead to _typical_ operators.

Thankfully, there are examples of these operators throughout the paper, but I want to leave the original definition up for reference.

## simpIL grammar (Table I)

```
program ::= stmt*

stmt s ::= var := exp
         | store(exp, exp)
         | goto exp 
         | assert exp 
         | if exp then goto exp else goto exp

exp e ::= load(exp) 
        | exp binop exp 
        | unop exp 
        | var 
        | get_input(src) 
        | v

binop ::= typical binary operators
unop  ::= typical unary operators
value v ::= 32-bit unsigned integer
```

Along with the grammar, we see the "meta-syntactic variables used in the execution context".

## simpIL Execution Context Variables

**Context** | **Meaning**
----------- | ----------------------------------------------------------
Sigma       | Maps a statement number to a statement.
µ           | Maps a memory address to the current value at that address
Delta       | Maps a variable name to its value
pc          | The program counter
i           | The next instruction

Ideally, these should map 1:1 to the internal representation of the interpreter in [interpreter.rs](src/interpreter.rs)

## Grammar Extensions

Since the language is defined ad-hoc in several ways, I've decided to gather the operators, defining their operation in natural language.

**Binary** | **Unary** | **Definition**
---------- | --------- | ----------------------------
`+`        |           | Add `left` to `right`
`-`        |           | Subtract `right` from `left`
`/`        |           | Divide `left` by `right`
`*`        |           | Multiply `left` by `right`
`=`        |           | Compare `left` and `right`.
