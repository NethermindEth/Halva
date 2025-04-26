A brief tutorial
---------------------
To extract a circuit into Lean4 one needs to write an extracting program first. It is a stand alone executable that depends on the circuit being extracted and the extraction library provided by this repository. It can be as simple as one `main.rs` file. Let us outline how one can write an extracting program.
1. Import the necessary dependencies:
```rust
use halo2_extr::{
    // The extractor that constructs the circuit and generates Lean
    extraction::ExtractingAssignment,
    // Our symbolic field type
    field::TermField
};
use halo2_proofs::{
    arithmetic::Field,
    circuit::Value,
    plonk::Circuit,
};
``` 
2. If we have a circuit of the form
```rust
struct MyCircuit<F: Field> {
    a: Value<F>,
    b: Value<F>,
    c: Value<F>,
}
impl <F: Field> Circuit <F> for MyCircuit<F> {
    ...
}
```
where `a`, `b` and `c` are private inputs to the circuit we will assign them symbolic values
```rust
let a = TermField::create_symbol("a");
let b = TermField::create_symbol("b");
let c = TermField::create_symbol("c");
```
and instantiate the circuit
```rust
let circuit = MyCircuit {
    a: Value::known(a.into()),
    b: Value::known(b.into()),
    c: Value::known(c.into()),
};
```
These symbolic values will show up in the Lean as members of the circuit structure named `sym_a`, `sym_b`, and `sym_c` respectively.

3. Finally, we run the extractor to generator Lean code.
```rust
ExtractingAssignment::run(
    &circuit,
    "Tutorial.MyCircuit", // The Lean namespace to create the circuit in
    &["a", "b", "c"] // The names of any symbolic values
).unwrap();
```
The code will be output to stdout, so you will likely want to redirect it into a file in a Lean project. At the end of the output you will find `meets_constraints`, a proposition which asserts that all of the constraints hold for a given instantiation of the circuit.

Several full examples can be found in the `examples` directory, and corresponding Lean proofs can be found in our repo [here](https://github.com/NethermindEth/halo2-fv).



Unsafe features
---------------------
Some methods in Halo2's Field type return something concrete depending on the value of the field element. Because TermField is symbolic, it is not generally possible to make claims about its concrete value during Rust execution. Oftentimes making sure you only run constraint generation code and not witness generation code will avoid this, however sometimes this is not enough. For such cases we have the following features: `unsafe-equality`, `unsafe-ord`, `unsafe-invert`. These should only be used if you are sure the following behaviour will be correct for your circuit:

Unsafe Equality
In general unsafe equality checks whether the two TermFields are exactly identical. Note that when the result of calculations on `TermField::Val` fit correctly in an i64 we evaluate them, but if they don't then the result is an unevaluated expression of the calculation. Additionally, the circuit's constraints may, for example, assert that a symbolic value x is always 0, but we won't know this in the Rust and so comparing x to 0 would produce false because they are not textually identical.

Unsafe Ord
For similar reasons to above we cannot order TermFields based on value. If you absolutely require `Ord` or `PartialOrd` but are okay with the ordering being undefined and potentially unpredictable, `unsafe-ord` will cause the `cmp` and `partial_cmp` methods to always return `Less`, as opposed to panicking if the feature is not enabled.

Unsafe Invert
The Field invert method returns a Choice depending on whether the input is 0. As described above, we cannot know definitively if a TermField is 0, so we default to always inverting the value. For totality, Lean's `inv` (which is what will be used for such expressions) returns 0 for an input of 0.