range: 5
Menor!

-- REGION: Assign value for poly range check
def selector_0_0: Prop := c.Selector 0 0 = 1
--Annotation: assign value
def advice_0_0: Prop := c.Advice 0 0 = 6
-- REGION: Assign value for poly range check
range: 10
Maior igual

-- REGION: Assign value for lookup range check
def selector_1_1: Prop := c.Selector 1 1 = 1
--Annotation: assign value
def advice_0_1: Prop := c.Advice 0 1 = 2
-- REGION: Assign value for lookup range check

-- REGION: load range check table
--Annotation: assign cell
def fixed_0: Prop := c.Fixed 0 0 = 0
--Annotation: assign cell
def fixed_1: Prop := c.Fixed 0 1 = 1
--Annotation: assign cell
def fixed_2: Prop := c.Fixed 0 2 = 2
--Annotation: assign cell
def fixed_3: Prop := c.Fixed 0 3 = 3
--Annotation: assign cell
def fixed_4: Prop := c.Fixed 0 4 = 4
--Annotation: assign cell
def fixed_5: Prop := c.Fixed 0 5 = 5
--Annotation: assign cell
def fixed_6: Prop := c.Fixed 0 6 = 6
--Annotation: assign cell
def fixed_7: Prop := c.Fixed 0 7 = 7
--Annotation: assign cell
def fixed_8: Prop := c.Fixed 0 8 = 8
--Annotation: assign cell
def fixed_9: Prop := c.Fixed 0 9 = 9
--Annotation: assign cell
def fixed_10: Prop := c.Fixed 0 10 = 10
--Annotation: assign cell
def fixed_11: Prop := c.Fixed 0 11 = 11
--Annotation: assign cell
def fixed_12: Prop := c.Fixed 0 12 = 12
--Annotation: assign cell
def fixed_13: Prop := c.Fixed 0 13 = 13
--Annotation: assign cell
def fixed_14: Prop := c.Fixed 0 14 = 14
--Annotation: assign cell
def fixed_15: Prop := c.Fixed 0 15 = 15
--Annotation: assign cell
def fixed_16: Prop := c.Fixed 0 16 = 16
--Annotation: assign cell
def fixed_17: Prop := c.Fixed 0 17 = 17
--Annotation: assign cell
def fixed_18: Prop := c.Fixed 0 18 = 18
--Annotation: assign cell
def fixed_19: Prop := c.Fixed 0 19 = 19
-- REGION: load range check table
------GATES-------
def gate_0: Prop := ∀ row : ℕ,   c.Selector 0 row * ((((((((c.Advice  0 (row) * (0 - c.Advice  0 (row))) * (1 - c.Advice  0 (row))) * (0x2 - c.Advice  0 (row))) * (0x3 - c.Advice  0 (row))) * (0x4 - c.Advice  0 (row))) * (0x5 - c.Advice  0 (row))) * (0x6 - c.Advice  0 (row))) * (0x7 - c.Advice  0 (row))) = 0
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
]
