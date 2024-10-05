
-- REGION: load range-check table
--Annotation: num_bits
def fixed_0: Prop := c.Fixed 0 0 = 1
--Annotation: num_bits
def fixed_1: Prop := c.Fixed 0 1 = 2
--Annotation: num_bits
def fixed_2: Prop := c.Fixed 0 2 = 3
--Annotation: num_bits
def fixed_3: Prop := c.Fixed 0 3 = 4
--Annotation: num_bits
def fixed_4: Prop := c.Fixed 0 4 = 5
--Annotation: num_bits
def fixed_5: Prop := c.Fixed 0 5 = 6
--Annotation: num_bits
def fixed_6: Prop := c.Fixed 0 6 = 7
--Annotation: num_bits
def fixed_7: Prop := c.Fixed 0 7 = 8
--Annotation: num_bits
def fixed_8: Prop := c.Fixed 0 8 = 9
--Annotation: num_bits
def fixed_9: Prop := c.Fixed 0 9 = 10
--Annotation: num_bits
def fixed_10: Prop := c.Fixed 0 10 = 11
--Annotation: num_bits
def fixed_11: Prop := c.Fixed 0 11 = 12
--Annotation: num_bits
def fixed_12: Prop := c.Fixed 0 12 = 13
--Annotation: num_bits
def fixed_13: Prop := c.Fixed 0 13 = 14
--Annotation: num_bits
def fixed_14: Prop := c.Fixed 0 14 = 15
--Annotation: num_bits
def fixed_15: Prop := c.Fixed 0 15 = 16
--Annotation: num_bits
def fixed_16: Prop := c.Fixed 0 16 = 17
--Annotation: num_bits
def fixed_17: Prop := c.Fixed 0 17 = 18
--Annotation: num_bits
def fixed_18: Prop := c.Fixed 0 18 = 19
-- REGION: load range-check table

-- REGION: load range-check table
--Annotation: num_bits
def fixed_19: Prop := c.Fixed 1 0 = 2
--Annotation: num_bits
def fixed_20: Prop := c.Fixed 1 1 = 4
--Annotation: num_bits
def fixed_21: Prop := c.Fixed 1 2 = 6
--Annotation: num_bits
def fixed_22: Prop := c.Fixed 1 3 = 8
--Annotation: num_bits
def fixed_23: Prop := c.Fixed 1 4 = 10
--Annotation: num_bits
def fixed_24: Prop := c.Fixed 1 5 = 12
--Annotation: num_bits
def fixed_25: Prop := c.Fixed 1 6 = 14
--Annotation: num_bits
def fixed_26: Prop := c.Fixed 1 7 = 16
--Annotation: num_bits
def fixed_27: Prop := c.Fixed 1 8 = 18
--Annotation: num_bits
def fixed_28: Prop := c.Fixed 1 9 = 20
--Annotation: num_bits
def fixed_29: Prop := c.Fixed 1 10 = 22
--Annotation: num_bits
def fixed_30: Prop := c.Fixed 1 11 = 24
--Annotation: num_bits
def fixed_31: Prop := c.Fixed 1 12 = 26
--Annotation: num_bits
def fixed_32: Prop := c.Fixed 1 13 = 28
--Annotation: num_bits
def fixed_33: Prop := c.Fixed 1 14 = 30
--Annotation: num_bits
def fixed_34: Prop := c.Fixed 1 15 = 32
--Annotation: num_bits
def fixed_35: Prop := c.Fixed 1 16 = 34
--Annotation: num_bits
def fixed_36: Prop := c.Fixed 1 17 = 36
--Annotation: num_bits
def fixed_37: Prop := c.Fixed 1 18 = 38
-- REGION: load range-check table

-- REGION: Assign value for simple range check
def selector_0_0: Prop := c.Selector 0 0 = 1
--Annotation: value
def advice_0_0: Prop := c.Advice 0 0 = 15
-- REGION: Assign value for simple range check

-- REGION: Assign value for lookup range check
def selector_1_1: Prop := c.Selector 1 1 = 1
--Annotation: value
def advice_0_1: Prop := c.Advice 0 1 = 15
-- REGION: Assign value for lookup range check
------GATES-------
def gate_0: Prop := ∀ row : ℕ,   c.Selector 0 row * (((((((c.Advice  0 (row) * (1 - c.Advice  0 (row))) * (0x2 - c.Advice  0 (row))) * (0x3 - c.Advice  0 (row))) * (0x4 - c.Advice  0 (row))) * (0x5 - c.Advice  0 (row))) * (0x6 - c.Advice  0 (row))) * (0x7 - c.Advice  0 (row))) = 0
[
    Argument {
        input_expressions: [
            Product(
                Selector(
                    Selector(
                        1,
                        false,
                    ),
                ),
                Advice {
                    query_index: 0,
                    column_index: 0,
                    rotation: Rotation(
                        0,
                    ),
                },
            ),
        ],
        table_expressions: [
            Fixed {
                query_index: 0,
                column_index: 0,
                rotation: Rotation(
                    0,
                ),
            },
        ],
    },
    Argument {
        input_expressions: [
            Sum(
                Product(
                    Selector(
                        Selector(
                            1,
                            false,
                        ),
                    ),
                    Advice {
                        query_index: 0,
                        column_index: 0,
                        rotation: Rotation(
                            0,
                        ),
                    },
                ),
                Constant(
                    One,
                ),
            ),
        ],
        table_expressions: [
            Fixed {
                query_index: 1,
                column_index: 1,
                rotation: Rotation(
                    0,
                ),
            },
        ],
    },
]
