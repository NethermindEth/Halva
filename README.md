Please note this repository is heavily WIP. We have implemented fixed and instance assignments, copy constraints, and gates, are part way through implementing lookups, and are looking into extending the model to rule out edge cases, especially as gates approach the end of the table. We also will likely add a variable k to enforce a table length of 2^k.

A brief tutorial
---------------------
To extract a circuit into Lean4 one needs to write an extracting program first. It is a stand alone executable that depends on the circuit being extracted and the extraction library provided by this repository. It can be as simple as one `main.rs` file. Let us outline how one can write an extracting program.
1. Import the necessary dependencies:
```rust
// These are the extracting macro and the enum type for 
// specifying the extraction target.
use halo2_extr::{extract, extraction::Target};
// This is a symbolic field type.
use halo2_extr::field::TermField;
``` 
2. If we have a circuit of the form
```rust
struct MyCircuit<F: Field> {
    a: Value<F>,
    b: Value<F>,
    c: Value<F>,
}
```
where `a`, `b` and `c` are private inputs to the circuit we will assign them symbolic values
```rust
let a = TermField::from("a");
let b = TermField::from("b");
let c = TermField::from("c");
```
and instantiate the circuit
```rust
let circuit = MyCircuit {
    a: Value::known(a),
    b: Value::known(b),
    c: Value::known(c),
};
```
3. Finally, we generate the code that outputs Lean4 code. Use the `extract` macro, specifying your circuit type (`MyCircuit`), the target (`Target::AdviceGenerator`), and the circuit instance (`circuit`).
The `AdviceGenerator` target indicates that you want to extract the advice generation logic from the circuit (This might not work in the majority of cases!).
```rust
extract!(MyCircuit, Target::AdviceGenerator, circuit);
```

The full example can be found [here](examples/two-chip.rs).