
-- REGION: RangeCheck Region
def selector_0_0: Prop := c.Selector 0 0 = 1
--Annotation: value
def advice_0_0: Prop := c.Advice 0 0 = 5
-- REGION: RangeCheck Region
------GATES-------
  S0 * (((((((((A0@0 * (1 - A0@0)) * (0x2 - A0@0)) * (0x3 - A0@0)) * (0x4 - A0@0)) * (0x5 - A0@0)) * (0x6 - A0@0)) * (0x7 - A0@0)) * (0x8 - A0@0)) * (0x9 - A0@0))
def gate_0: Prop := ∀ row : ℕ,   c.Selector 0 row * (((((((((c.Advice  0 (row) * (1 - c.Advice  0 (row))) * (0x2 - c.Advice  0 (row))) * (0x3 - c.Advice  0 (row))) * (0x4 - c.Advice  0 (row))) * (0x5 - c.Advice  0 (row))) * (0x6 - c.Advice  0 (row))) * (0x7 - c.Advice  0 (row))) * (0x8 - c.Advice  0 (row))) * (0x9 - c.Advice  0 (row))) = 0
